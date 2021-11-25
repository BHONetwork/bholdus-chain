//! Tests for Tokens pallet.

use super::*;
use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use sp_runtime::traits::BadOrigin;

type Blacklist = BTreeMap<Vec<u8>, Vec<u8>>;

fn test_blacklist(x: u8) -> Blacklist {
    let mut blacklist: Blacklist = BTreeMap::new();
    blacklist.insert(vec![x], vec![x]);
    blacklist.insert(vec![x + 1], vec![x + 1]);
    blacklist
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

        BholdusTokens::set_blacklist(Origin::root(), vec![5], vec![6]);
        BholdusTokens::set_blacklist(Origin::root(), vec![7], vec![8]);

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
fn create_and_mint_should_work() {
    new_test_ext().execute_with(|| {
        let name: Vec<u8> = vec![1];
        let symbol: Vec<u8> = vec![1];
        let decimals: u8 = 12;
        let beneficiary = BOB;
        let supply: Balance = 1000;
        let min_balance: Balance = 10;

        Balances::make_free_balance_be(&ALICE, 10);

        Balances::make_free_balance_be(&BOB, 100);

        Balances::make_free_balance_be(&EVE, 10);

        /*BholdusTokens::set_blacklist(Origin::root(), vec![0u8; 1], vec![0u8; 2]);
        assert_noop!(
            BholdusTokens::create_and_mint(
                Origin::signed(BOB),
                ALICE,
                vec![0u8; 1],
                vec![0u8; 2],
                decimals,
                beneficiary,
                supply,
                min_balance,
            ),
            Error::<Runtime>::AssetBlacklist
        );
        */

        assert_eq!(BholdusTokens::next_asset_id(), ASSET_ID);
        assert_ok!(BholdusTokens::create_and_mint(
            Origin::signed(BOB),
            ALICE,
            name,
            symbol,
            decimals,
            beneficiary,
            supply,
            min_balance,
        ));
        assert_eq!(BholdusTokens::next_asset_id(), 1);
        assert_eq!(BholdusTokens::total_balance(ASSET_ID, &BOB), 1000);
        assert_eq!(BholdusTokens::total_issuance(ASSET_ID), 1000);

        assert_ok!(BholdusTokens::mint(
            Origin::signed(ALICE),
            ASSET_ID,
            ALICE,
            500
        ));
        assert_eq!(BholdusTokens::total_balance(ASSET_ID, &ALICE), 500);
        assert_eq!(BholdusTokens::total_issuance(ASSET_ID), 1500);

        assert_ok!(BholdusTokens::mint(Origin::signed(ALICE), 0, EVE, 100));
        assert_eq!(BholdusTokens::total_balance(ASSET_ID, &EVE), 100);
        assert_eq!(BholdusTokens::total_issuance(ASSET_ID), 1600);

        assert_ok!(BholdusTokens::freeze(
            Origin::signed(ALICE),
            ASSET_ID,
            ALICE
        ));
        assert_noop!(
            BholdusTokens::transfer(Origin::signed(ALICE), ASSET_ID, EVE, 50),
            Error::<Runtime>::Frozen
        );
        assert_ok!(BholdusTokens::thaw(Origin::signed(ALICE), ASSET_ID, ALICE));
        assert_ok!(BholdusTokens::transfer(
            Origin::signed(ALICE),
            ASSET_ID,
            EVE,
            50
        ));
        assert_eq!(BholdusTokens::total_balance(ASSET_ID, &ALICE), 450);
        assert_eq!(BholdusTokens::total_balance(ASSET_ID, &EVE), 150);
        assert_eq!(BholdusTokens::total_issuance(ASSET_ID), 1600);

        assert_ok!(BholdusTokens::freeze_asset(Origin::signed(ALICE), ASSET_ID));
        let w = Asset::<Runtime>::get(ASSET_ID)
            .ok_or(Error::<Runtime>::Unknown)
            .unwrap();
        println!("Owner: {:?}", &w.owner);
        assert!(&w.is_frozen);

        assert_noop!(
            BholdusTokens::set_identity(Origin::signed(BOB), ASSET_ID, ten()),
            Error::<Runtime>::Frozen
        );

        assert_noop!(
            BholdusTokens::set_identity(Origin::signed(ALICE), ASSET_ID, ten()),
            Error::<Runtime>::NoPermission
        );

        assert_ok!(BholdusTokens::thaw_asset(Origin::signed(ALICE), ASSET_ID));
        let w1 = Asset::<Runtime>::get(ASSET_ID).ok_or(Error::<Runtime>::Unknown);
        assert!(!&w1.unwrap().is_frozen);

        assert_ok!(BholdusTokens::set_identity(
            Origin::signed(1),
            ASSET_ID,
            ten()
        ));

        assert_eq!(BholdusTokens::identity(ASSET_ID).unwrap().info, ten());
        assert!(!BholdusTokens::identity(ASSET_ID).unwrap().is_verifiable);
        assert_ok!(BholdusTokens::verify_asset(Origin::root(), ASSET_ID));
        assert_noop!(
            BholdusTokens::verify_asset(Origin::signed(BOB), ASSET_ID),
            BadOrigin
        );
        assert!(BholdusTokens::identity(ASSET_ID).unwrap().is_verifiable);
    })
}

#[test]
fn create_should_work() {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&1, 100);
        assert_eq!(BholdusTokens::next_asset_id(), ASSET_ID);
        assert_ok!(BholdusTokens::create(Origin::signed(1), 1, 1));
        assert_eq!(BholdusTokens::next_asset_id(), 1);
        assert_ok!(BholdusTokens::create(Origin::signed(1), 1, 1));
        assert_eq!(BholdusTokens::next_asset_id(), 2);
    })
}

#[test]
fn basic_minting_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(BholdusTokens::force_create(Origin::root(), 0, 1, true, 1));
        assert_ok!(BholdusTokens::mint(Origin::signed(1), 0, 1, 100));
        assert_eq!(BholdusTokens::total_balance(0, &1), 100);
        assert_eq!(BholdusTokens::total_issuance(0), 100);
        assert_ok!(BholdusTokens::mint(Origin::signed(1), 0, 1, 100));
        assert_eq!(BholdusTokens::total_issuance(0), 200);

        assert_ok!(BholdusTokens::mint(Origin::signed(1), 0, 2, 100));
        assert_eq!(BholdusTokens::total_balance(0, &2), 100);
    });
}

#[test]
fn transferring_frozen_asset_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(BholdusTokens::force_create(Origin::root(), 0, 1, true, 1));

        assert_ok!(BholdusTokens::mint(Origin::signed(1), 0, 1, 100));
        assert_eq!(BholdusTokens::total_balance(0, &1), 100);
        assert_ok!(BholdusTokens::freeze(Origin::signed(1), 0, 1));

        assert_noop!(
            BholdusTokens::transfer(Origin::signed(1), 0, 2, 50),
            Error::<Runtime>::Frozen
        );
        assert_ok!(BholdusTokens::thaw(Origin::signed(1), 0, 1));
        assert_ok!(BholdusTokens::transfer(Origin::signed(1), 0, 2, 50));
    })
}

#[test]
//#[allow(dead_code)]
fn verify_asset_frozen_asset_should_work() {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&1, 100);
        assert_ok!(BholdusTokens::create(Origin::signed(1), 1, 1));
        assert_ok!(BholdusTokens::freeze_asset(Origin::signed(1), ASSET_ID));

        let w = Asset::<Runtime>::get(ASSET_ID).ok_or(Error::<Runtime>::Unknown);
        assert!(&w.unwrap().is_frozen);

        assert_ok!(BholdusTokens::thaw_asset(Origin::signed(1), ASSET_ID));
        let w1 = Asset::<Runtime>::get(ASSET_ID).ok_or(Error::<Runtime>::Unknown);

        assert!(!&w1.unwrap().is_frozen);

        assert_ok!(BholdusTokens::set_identity(
            Origin::signed(1),
            ASSET_ID,
            ten()
        ));

        assert_eq!(BholdusTokens::identity(ASSET_ID).unwrap().info, ten());
        assert!(!BholdusTokens::identity(ASSET_ID).unwrap().is_verifiable);
        assert_ok!(BholdusTokens::verify_asset(Origin::root(), ASSET_ID));
        assert!(BholdusTokens::identity(ASSET_ID).unwrap().is_verifiable);
    })
}

#[test]
fn verify_asset_permission_should_not_work() {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&1, 100);
        assert_ok!(BholdusTokens::create(Origin::signed(1), 1, 1));

        /// Admin: 1
        /// Only allow admin of asset `verify_asset`
        ///
        /// Origin: 2
        /// Error: NoPermisson
        assert_noop!(
            BholdusTokens::verify_asset(Origin::signed(2), ASSET_ID),
            BadOrigin
        );
    })
}

#[test]
fn verify_asset_frozen_asset_should_not_work() {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&1, 100);
        Balances::make_free_balance_be(&2, 100);
        assert_ok!(BholdusTokens::create(Origin::signed(1), 1, 1));
        assert_ok!(BholdusTokens::create(Origin::signed(2), 2, 2));
        assert_noop!(
            BholdusTokens::verify_asset(Origin::root(), 1),
            Error::<Runtime>::Unknown
        );
        assert_ok!(BholdusTokens::set_identity(Origin::signed(2), 1, ten()));
        assert_ok!(BholdusTokens::freeze_asset(Origin::signed(2), 1));
        let w = Asset::<Runtime>::get(1).ok_or(Error::<Runtime>::Unknown);
        assert!(&w.unwrap().is_frozen);
        assert_noop!(
            BholdusTokens::verify_asset(Origin::root(), 1),
            Error::<Runtime>::Frozen
        );
    });
}

#[test]
fn set_identity_should_not_work() {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&1, 10);
        assert_ok!(BholdusTokens::create(Origin::signed(1), 1, 1));
        assert_ok!(BholdusTokens::create(Origin::signed(1), 1, 1));
        assert_ok!(BholdusTokens::create(Origin::signed(1), 1, 1));
        assert_ok!(BholdusTokens::freeze_asset(Origin::signed(1), 2));
        let w = Asset::<Runtime>::get(2).ok_or(Error::<Runtime>::Unknown);
        assert!(&w.unwrap().is_frozen);
        assert_noop!(
            BholdusTokens::set_identity(Origin::signed(1), 2, ten()),
            Error::<Runtime>::Frozen
        );
        assert_noop!(
            BholdusTokens::set_identity(Origin::signed(2), 2, ten()),
            Error::<Runtime>::NoPermission
        );
    });
}

#[test]
fn set_metadata_should_work() {
    new_test_ext().execute_with(|| {
        BholdusTokens::set_blacklist(Origin::root(), vec![0u8; 1], vec![0u8; 2]);

        assert_noop!(
            BholdusTokens::set_metadata(Origin::signed(1), 0, vec![0u8; 1], vec![0u8; 2], 12),
            Error::<Runtime>::AssetBlacklist
        );

        // Cannot add metadata to unknown asset
        assert_noop!(
            BholdusTokens::set_metadata(Origin::signed(1), 0, vec![0u8; 10], vec![0u8; 10], 12),
            Error::<Runtime>::Unknown,
        );

        assert_ok!(BholdusTokens::force_create(Origin::root(), 0, 1, true, 1));
        // Cannot add metadata to unowned asset
        assert_noop!(
            BholdusTokens::set_metadata(Origin::signed(2), 0, vec![0u8; 10], vec![0u8; 10], 12),
            Error::<Runtime>::NoPermission,
        );

        // Cannot add oversized metadata
        assert_noop!(
            BholdusTokens::set_metadata(Origin::signed(1), 0, vec![0u8; 100], vec![0u8; 10], 12),
            Error::<Runtime>::BadMetadata,
        );

        assert_noop!(
            BholdusTokens::set_metadata(Origin::signed(1), 0, vec![0u8; 10], vec![0u8; 100], 12),
            Error::<Runtime>::BadMetadata,
        );

        // Successfully add metadata and take deposit
        Balances::make_free_balance_be(&1, 30);
        assert_ok!(BholdusTokens::set_metadata(
            Origin::signed(1),
            0,
            vec![0u8; 10],
            vec![0u8; 10],
            12
        ));

        assert_eq!(Balances::free_balance(&1), 9); // ??

        // Clear Metadata
        assert!(Metadata::<Runtime>::contains_key(0));
        assert_noop!(
            BholdusTokens::clear_metadata(Origin::signed(2), 0),
            Error::<Runtime>::NoPermission
        );
        assert_noop!(
            BholdusTokens::clear_metadata(Origin::signed(1), 1),
            Error::<Runtime>::Unknown
        );
        assert_ok!(BholdusTokens::clear_metadata(Origin::signed(1), 0));
        assert!(!Metadata::<Runtime>::contains_key(0));
    });
}

//#[test]
#[allow(dead_code)]
fn transferring_to_frozen_account_should_work() {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&1, 100);
        assert_ok!(BholdusTokens::create(Origin::signed(1), 1, 1));
        assert_ok!(BholdusTokens::mint(Origin::signed(1), ASSET_ID, 1, 100));
        assert_ok!(BholdusTokens::mint(Origin::signed(1), ASSET_ID, 2, 100));
        assert_ok!(BholdusTokens::freeze(Origin::signed(1), ASSET_ID, 2));
        assert_ok!(BholdusTokens::transfer(Origin::signed(1), ASSET_ID, 2, 50));
        assert_eq!(BholdusTokens::total_balance(ASSET_ID, &2), 150);
    });
}

#[test]
//#[allow(dead_code)]
fn lifecycle_should_work() {
    new_test_ext().execute_with(|| {
        Balances::make_free_balance_be(&1, 100);
        assert_ok!(BholdusTokens::create(Origin::signed(1), 1, 1));
        assert_eq!(Balances::reserved_balance(&1), 1);
        assert!(Asset::<Runtime>::contains_key(ASSET_ID));

        assert_ok!(BholdusTokens::set_metadata(
            Origin::signed(1),
            ASSET_ID,
            vec![0],
            vec![0],
            12
        ));
        assert_eq!(Balances::reserved_balance(&1), 4);
        assert!(Metadata::<Runtime>::contains_key(ASSET_ID));

        Balances::make_free_balance_be(&10, 100);
        assert_ok!(BholdusTokens::mint(Origin::signed(1), ASSET_ID, 10, 100));
        Balances::make_free_balance_be(&20, 100);
        assert_ok!(BholdusTokens::mint(Origin::signed(1), ASSET_ID, 20, 100));
        assert_eq!(Account::<Runtime>::iter_prefix(ASSET_ID).count(), 2);

        let w = Asset::<Runtime>::get(ASSET_ID).unwrap().destroy_witness();
        assert_ok!(BholdusTokens::destroy(Origin::signed(1), ASSET_ID, w));
        assert_eq!(Balances::reserved_balance(&1), 0);

        assert!(!Asset::<Runtime>::contains_key(ASSET_ID));
        assert!(!Metadata::<Runtime>::contains_key(ASSET_ID));
        assert_eq!(Account::<Runtime>::iter_prefix(ASSET_ID).count(), 0);

        assert_ok!(BholdusTokens::create(Origin::signed(1), 1, 1));
        assert_eq!(Balances::reserved_balance(&1), 1);
        assert!(Asset::<Runtime>::contains_key(1));

        assert_ok!(BholdusTokens::set_metadata(
            Origin::signed(1),
            1,
            vec![0],
            vec![0],
            12
        ));
        assert_eq!(Balances::reserved_balance(&1), 4);
        assert!(Metadata::<Runtime>::contains_key(1));

        assert_ok!(BholdusTokens::mint(Origin::signed(1), 1, 10, 100));
        assert_ok!(BholdusTokens::mint(Origin::signed(1), 1, 20, 100));
        assert_eq!(Account::<Runtime>::iter_prefix(1).count(), 2);

        assert_ok!(BholdusTokens::set_identity(Origin::signed(1), 1, ten()));
        assert!(IdentityOf::<Runtime>::contains_key(1));
        assert_eq!(BholdusTokens::identity(1).unwrap().info, ten());
        assert!(!BholdusTokens::identity(1).unwrap().is_verifiable);
        assert_ok!(BholdusTokens::verify_asset(Origin::root(), 1));
        assert!(BholdusTokens::identity(1).unwrap().is_verifiable);

        let w = Asset::<Runtime>::get(1).unwrap().destroy_witness();
        assert_ok!(BholdusTokens::destroy(Origin::root(), 1, w));
        assert_eq!(Balances::reserved_balance(&1), 0);

        assert!(!Asset::<Runtime>::contains_key(1));
        assert!(!Metadata::<Runtime>::contains_key(1));
        assert!(!IdentityOf::<Runtime>::contains_key(1));
        assert_eq!(Account::<Runtime>::iter_prefix(1).count(), 0);
    });
}
