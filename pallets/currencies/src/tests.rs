#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

#[test]
fn multi_currency_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        PalletBalances::make_free_balance_be(&ALICE, 100);
        assert_ok!(BholdusTokens::create(Origin::signed(ALICE), 1, ALICE, 1));
        assert_ok!(BholdusTokens::mint(Origin::signed(ALICE), 1, ALICE, 100));

        assert_eq!(BholdusTokens::free_balance(1, &ALICE), 100);
        assert_eq!(Currencies::free_balance(1, &ALICE), 100);

        assert_ok!(Currencies::transfer(Some(ALICE).into(), BOB, 1, 50));
        assert_eq!(BholdusTokens::free_balance(1, &ALICE), 50);
        assert_eq!(BholdusTokens::free_balance(1, &BOB), 50);
        assert_eq!(BholdusTokens::total_balance(1, &ALICE), 50);
        assert_eq!(BholdusTokens::total_balance(1, &BOB), 50);
        assert_eq!(Currencies::free_balance(1, &ALICE), 50);
        assert_eq!(Currencies::free_balance(1, &BOB), 50);
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
