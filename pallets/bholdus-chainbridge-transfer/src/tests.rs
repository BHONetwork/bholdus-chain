use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use sp_std::prelude::*;

#[test]
fn bridge_should_receive_token_and_release_token() {
    new_test_ext().execute_with(|| {
        assert_ok!(ChainBridge::whitelist_chain(Origin::root(), 1));
        assert_ok!(ChainBridgeTransfer::transfer_native_token(
            Origin::signed(USER_ID),
            "1".into(),
            100,
            1
        ));

        let bridge_account_id = ChainBridge::account_id();
        let bridge_balance = Balances::total_balance(&bridge_account_id);
        assert_eq!(bridge_balance, 100);

        ChainBridgeTransfer::release_native_token(Origin::signed(bridge_account_id), USER_ID, 100);
        let user_balance = Balances::total_balance(&USER_ID);
        assert_eq!(user_balance, 100);
    });
}
