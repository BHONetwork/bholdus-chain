//! Tests for Tokens pallet.

use super::*;
use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use sp_runtime::traits::BadOrigin;
use sp_std::if_std;

type Blacklist = BTreeMap<Vec<u8>, Vec<u8>>;

fn test_blacklist(x: u8) -> Blacklist {
    let mut blacklist: Blacklist = BTreeMap::new();
    blacklist.insert(vec![x], vec![x]);
    blacklist.insert(vec![x + 1], vec![x + 1]);
    blacklist
}

#[test]
fn test_transfer_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        Balances::make_free_balance_be(&ALICE, 10);
        Balances::make_free_balance_be(&BOB, 1);
        assert_ok!(Balances::transfer(Origin::signed(ALICE).into(), BOB, 5));
        assert_eq!(Balances::free_balance(&ALICE), 5);
        assert_eq!(Balances::free_balance(&BOB), 6);
    })
}

#[test]
fn genesis_issuance_should_work() {
    ExtBuilder::default()
        .one_hundred_for_alice()
        .build()
        .execute_with(|| {
            assert_eq!(BholdusTokens::free_balance(BUSD, &ALICE), 100);
            assert_eq!(BholdusTokens::total_issuance(BUSD), 100);
            assert_eq!(BholdusTokens::total_balance(BUSD, &ALICE), 100);
            BholdusTokens::transfer(Origin::signed(ALICE), BUSD, BOB, 50);
            assert_eq!(BholdusTokens::total_issuance(BUSD), 100);
            assert_eq!(BholdusTokens::total_balance(BUSD, &ALICE), 50);
        })
}

#[test]
fn do_create_should_work() {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&BOB, 10);
        Balances::make_free_balance_be(&ALICE, 15);

        let name = vec![0u8; 10];
        let symbol: Vec<u8> = vec![1];
        let decimals: u8 = 12;
        let beneficiary = ALICE;
        assert_eq!(BholdusTokens::next_asset_id(), ASSET_ID);
        assert_ok!(BholdusTokens::do_create(
            &ALICE,
            name,
            symbol,
            decimals,
            &beneficiary,
        ));

        assert_ok!(BholdusTokens::do_mint(ASSET_ID, &beneficiary, 10, None,));
        assert_eq!(BholdusTokens::total_issuance(ASSET_ID), 10);

        let f = DebitFlags {
            keep_alive: false,
            best_effort: true,
        };

        assert_ok!(BholdusTokens::do_burn(ASSET_ID, &beneficiary, 10, None, f));
        assert_eq!(BholdusTokens::total_issuance(ASSET_ID), 0);
        assert_ok!(BholdusTokens::do_mint(ASSET_ID, &beneficiary, 150, None,));
        assert_ok!(BholdusTokens::transfer(
            Origin::signed(ALICE),
            ASSET_ID,
            BOB,
            50
        ));
        assert_eq!(BholdusTokens::free_balance(ASSET_ID, &ALICE), 100);
        assert_eq!(BholdusTokens::free_balance(ASSET_ID, &BOB), 50);
    });
}

#[test]
fn do_create_should_not_work() {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&BOB, 10);
        Balances::make_free_balance_be(&ALICE, 15);

        let name = vec![0u8; 10];
        let symbol: Vec<u8> = vec![1];
        let decimals: u8 = 12;
        let beneficiary = ALICE;
        assert_eq!(BholdusTokens::next_asset_id(), ASSET_ID);
        assert_ok!(BholdusTokens::do_create(
            &ALICE,
            name,
            symbol,
            decimals,
            &beneficiary,
        ));

        assert_ok!(BholdusTokens::freeze_asset(Origin::signed(ALICE), ASSET_ID));
        let w = Asset::<Runtime>::get(ASSET_ID)
            .ok_or(Error::<Runtime>::Unknown)
            .unwrap();
        assert!(&w.is_frozen);

        assert_noop!(
            BholdusTokens::set_identity(Origin::signed(ALICE), ASSET_ID, ten()),
            Error::<Runtime>::Frozen
        );

        assert_ok!(BholdusTokens::thaw_asset(Origin::signed(ALICE), ASSET_ID));
        let w1 = Asset::<Runtime>::get(ASSET_ID).ok_or(Error::<Runtime>::Unknown);
        assert!(!&w1.unwrap().is_frozen);
    })
}

#[test]
fn do_create_and_mint_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        Balances::make_free_balance_be(&ALICE, 15);

        let name = vec![0u8; 10];
        let symbol: Vec<u8> = vec![1];
        let decimals: u8 = 12;
        let beneficiary = ALICE;
        let supply: Balance = 1000;
        let min_balance: Balance = 10;

        assert_eq!(BholdusTokens::next_asset_id(), 1);

        assert_ok!(BholdusTokens::do_create_and_mint(
            &ALICE,
            &ALICE,
            name,
            symbol,
            decimals,
            &beneficiary,
            supply,
            min_balance,
        ));

        assert_eq!(BholdusTokens::total_issuance(1), supply);

        assert_ok!(BholdusTokens::do_mint(1, &beneficiary, 10, None,));
        assert_eq!(BholdusTokens::total_issuance(1), supply + 10);

        let f = DebitFlags {
            keep_alive: false,
            best_effort: true,
        };

        assert_ok!(BholdusTokens::do_burn(1, &beneficiary, 10, None, f));
        assert_eq!(BholdusTokens::total_issuance(1), supply);
    });
}

#[test]
fn blacklist_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(BholdusTokens::set_blacklist(
            Origin::root(),
            vec![1],
            vec![2]
        ));

        assert_eq!(
            AssetsBlacklist::<Runtime>::take().contains(&(vec![1], vec![2])),
            true
        );

        assert_ok!(BholdusTokens::set_blacklist(
            Origin::root(),
            vec![5],
            vec![6]
        ));
        assert_ok!(BholdusTokens::set_blacklist(
            Origin::root(),
            vec![7],
            vec![8]
        ));

        // assert_eq!(
        //     AssetsBlacklist::<Runtime>::take().contains(&(vec![5], vec![6])),
        //     true
        // );

        assert_eq!(
            AssetsBlacklist::<Runtime>::take().contains(&(vec![7], vec![8])),
            true
        );

        assert_eq!(
            AssetsBlacklist::<Runtime>::take().contains(&(vec![3], vec![4])),
            false
        );
    })
}

#[test]
fn passed_name() {
    new_test_ext().execute_with(|| {
        let s0 = String::from("BHO");
        let s1 = String::from("B  HO  ");

        let v: Vec<u8> = s1.into_bytes();
        let s = String::from_utf8(v).unwrap();
        let s_trim = s.replace(" ", "");

        assert_eq!(s_trim, s0);
        assert_eq!(s_trim.into_bytes(), s0.into_bytes());
    })
}

#[test]
fn invalid_symbol() {
    new_test_ext().execute_with(|| {
        let s0 = String::from("BHO");
        let s1 = String::from("B  HO  ");

        let v: Vec<u8> = s1.into_bytes();
        let s = String::from_utf8(v).unwrap();

        assert_eq!(s.into_bytes() != s0.into_bytes(), true);
    })
}
