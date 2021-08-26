#![cfg(test)]

use super::*;
use bholdus_tokens::Error as BholdusTokensError;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

#[test]
fn multi_currency_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        // Test: transfer CurrencyId = 1
        assert_eq!(BholdusTokens::free_balance(1, &ALICE), 100);
        assert_eq!(BholdusTokens::free_balance(1, &ALICE), 100);
        assert_eq!(Currencies::free_balance(1, &ALICE), 100);
        assert_ok!(Currencies::transfer(Some(ALICE).into(), BOB, 1, 50));
        assert_eq!(BholdusTokens::free_balance(1, &ALICE), 50);
        assert_eq!(BholdusTokens::free_balance(1, &BOB), 50);
        assert_eq!(BholdusTokens::total_balance(1, &ALICE), 50);
        assert_eq!(BholdusTokens::total_balance(1, &BOB), 50);
        assert_eq!(Currencies::free_balance(1, &ALICE), 50);
        assert_eq!(Currencies::free_balance(1, &BOB), 50);
        assert_ok!(BholdusTokens::mint(Origin::signed(ALICE), 1, BOB, 100));
        assert_eq!(BholdusTokens::total_issuance(1), 200);

        // Test: transfer CurrencyId = 2
        PalletBalances::make_free_balance_be(&ALICE, 100);
        assert_ok!(BholdusTokens::create(Origin::signed(ALICE), 2, ALICE, 1));
        assert_ok!(BholdusTokens::mint(Origin::signed(ALICE), 2, ALICE, 100));
        assert_ok!(Currencies::transfer(Some(ALICE).into(), BOB, 2, 50));
        assert_eq!(BholdusTokens::free_balance(2, &ALICE), 50);
        assert_eq!(BholdusTokens::free_balance(2, &BOB), 50);
        assert_eq!(BholdusTokens::total_issuance(2), 100);
    });
}
#[test]
fn multi_currency_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        // Test CurrencyId = 1
        assert_ok!(BholdusTokens::freeze(Origin::signed(ALICE), 1, ALICE));
        // assert_ok!(BholdusTokens::freeze_asset(Origin::signed(ALICE), 1));
        assert_noop!(
            BholdusTokens::transfer(Origin::signed(ALICE), 1, BOB, 50),
            BholdusTokensError::<Runtime>::Frozen
        );
        assert_noop!(
            Currencies::transfer(Some(ALICE).into(), BOB, 1, 50),
            BholdusTokensError::<Runtime>::Frozen
        );
        assert_ok!(BholdusTokens::thaw(Origin::signed(ALICE), 1, ALICE));
        assert_ok!(Currencies::transfer(Some(ALICE).into(), BOB, 1, 50));
        assert_eq!(BholdusTokens::free_balance(1, &ALICE), 50);
        assert_eq!(BholdusTokens::free_balance(1, &BOB), 50);

        // CurrencyId = 2

        PalletBalances::make_free_balance_be(&ALICE, 100);
        assert_ok!(BholdusTokens::create(Origin::signed(ALICE), 2, ALICE, 1));
        assert_ok!(BholdusTokens::mint(Origin::signed(ALICE), 2, ALICE, 100));
        assert_ok!(BholdusTokens::freeze_asset(Origin::signed(ALICE), 2));
        assert_noop!(
            Currencies::transfer(Some(ALICE).into(), BOB, 2, 50),
            BholdusTokensError::<Runtime>::Frozen
        );

        assert_ok!(BholdusTokens::thaw_asset(Origin::signed(ALICE), 2));
        assert_ok!(Currencies::transfer(Some(ALICE).into(), BOB, 2, 50));
        assert_eq!(BholdusTokens::free_balance(2, &ALICE), 50);
        assert_eq!(BholdusTokens::free_balance(2, &BOB), 50);
    });
}

#[test]
fn native_currency_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(NativeCurrency::free_balance(&ALICE), 100);
        assert_ok!(Currencies::transfer_native_currency(
            Some(ALICE).into(),
            BOB,
            50
        ));

        assert_eq!(NativeCurrency::free_balance(&ALICE), 50);
        assert_eq!(NativeCurrency::free_balance(&BOB), 50);
        assert_eq!(Currencies::free_balance(0, &ALICE), 50);
    });
}
