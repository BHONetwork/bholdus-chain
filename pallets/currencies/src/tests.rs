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
        assert_eq!(BholdusTokens::free_balance(X_TOKEN_ID, &ALICE), 100);
        assert_eq!(Currencies::free_balance(X_TOKEN_ID, &ALICE), 100);
        assert_ok!(Currencies::transfer(
            Some(ALICE).into(),
            BOB,
            X_TOKEN_ID,
            50
        ));
        assert_eq!(BholdusTokens::free_balance(X_TOKEN_ID, &ALICE), 50);
        assert_eq!(BholdusTokens::free_balance(X_TOKEN_ID, &BOB), 50);
        assert_eq!(BholdusTokens::total_balance(X_TOKEN_ID, &ALICE), 50);
        assert_eq!(BholdusTokens::total_balance(X_TOKEN_ID, &BOB), 50);
        assert_eq!(Currencies::free_balance(X_TOKEN_ID, &ALICE), 50);
        assert_eq!(Currencies::free_balance(X_TOKEN_ID, &BOB), 50);
        assert_ok!(BholdusTokens::mint(
            Origin::signed(ALICE),
            X_TOKEN_ID,
            BOB,
            100
        ));
        assert_eq!(BholdusTokens::total_issuance(X_TOKEN_ID), 200);

        // Test: transfer CurrencyId = 3
        PalletBalances::make_free_balance_be(&ALICE, 100);
        assert_ok!(BholdusTokens::create(Origin::signed(ALICE), ALICE, 1));
        assert_ok!(BholdusTokens::mint(Origin::signed(ALICE), 3, ALICE, 100));
        assert_ok!(Currencies::transfer(Some(ALICE).into(), BOB, 3, 50));
        assert_eq!(BholdusTokens::free_balance(3, &ALICE), 50);
        assert_eq!(BholdusTokens::free_balance(3, &BOB), 50);
        assert_eq!(BholdusTokens::total_issuance(3), 100);
    });
}
#[test]
fn multi_currency_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        // Test TOKEN_ID = 1
        // BOB: owner
        assert_ok!(BholdusTokens::freeze(Origin::signed(BOB), TOKEN_ID, BOB));
        // assert_ok!(BholdusTokens::freeze_asset(Origin::signed(ALICE), 1));
        assert_noop!(
            BholdusTokens::transfer(Origin::signed(BOB), TOKEN_ID, ALICE, 50),
            BholdusTokensError::<Runtime>::Frozen
        );
        assert_noop!(
            Currencies::transfer(Some(BOB).into(), ALICE, TOKEN_ID, 50),
            BholdusTokensError::<Runtime>::Frozen
        );
        assert_ok!(BholdusTokens::thaw(Origin::signed(BOB), TOKEN_ID, BOB));
        assert_ok!(Currencies::transfer(Some(BOB).into(), ALICE, TOKEN_ID, 50));
        assert_eq!(BholdusTokens::free_balance(TOKEN_ID, &ALICE), 50);
        assert_eq!(BholdusTokens::free_balance(TOKEN_ID, &BOB), 50);

        // X_TOKEN_ID = 2
        // ALICE: owner

        PalletBalances::make_free_balance_be(&ALICE, 100);
        assert_ok!(BholdusTokens::create(Origin::signed(ALICE), ALICE, 1));
        assert_ok!(BholdusTokens::mint(
            Origin::signed(ALICE),
            X_TOKEN_ID,
            ALICE,
            100
        ));
        assert_ok!(BholdusTokens::freeze_asset(
            Origin::signed(ALICE),
            X_TOKEN_ID
        ));
        assert_noop!(
            Currencies::transfer(Some(ALICE).into(), BOB, X_TOKEN_ID, 50),
            BholdusTokensError::<Runtime>::Frozen
        );

        assert_ok!(BholdusTokens::thaw_asset(Origin::signed(ALICE), 2));
        assert_ok!(Currencies::transfer(
            Some(ALICE).into(),
            BOB,
            X_TOKEN_ID,
            50
        ));
        assert_eq!(BholdusTokens::free_balance(X_TOKEN_ID, &ALICE), 150);
        assert_eq!(BholdusTokens::free_balance(X_TOKEN_ID, &BOB), 50);
    });
}

/* #[test]
fn native_currency_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(NativeCurrency::free_balance(&ALICE), 100);
        assert_ok!(Currencies::transfer_native_currency(
            Some(ALICE).into(),
            BOB,
            50
        ));

        assert_eq!(NativeCurrency::free_balance(&ALICE), 50);
        assert_eq!(NativeCurrency::free_balance(&BOB), 150);
        assert_eq!(Currencies::free_balance(0, &ALICE), 50);
    });
} */
