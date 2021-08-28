use crate::{mock::*, Error};
use bholdus_support::MultiCurrency;
use frame_support::{assert_noop, assert_ok, traits::Currency};
use sp_runtime::traits::BadOrigin;
use sp_std::prelude::*;

#[test]
fn register_resource_id_should_work() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        // Only admin can invoke this call
        assert_noop!(
            ChainBridgeTransfer::register_resource_id(
                Origin::signed(ALICE),
                BHO_RESOURCE_ID,
                BHO_CURRENCY
            ),
            BadOrigin
        );

        // Should register successfully
        assert_ok!(ChainBridgeTransfer::register_resource_id(
            Origin::root(),
            BHO_RESOURCE_ID,
            BHO_CURRENCY
        ));
        assert_eq!(
            ChainBridgeTransfer::currency_id_to_resource_id(BHO_CURRENCY),
            Some(BHO_RESOURCE_ID)
        );
        assert_eq!(
            ChainBridgeTransfer::resource_id_to_currency_id(BHO_RESOURCE_ID),
            Some(BHO_CURRENCY)
        );
        System::assert_last_event(
            crate::Event::ResourceIdRegistered(BHO_RESOURCE_ID, BHO_CURRENCY).into(),
        );

        // Register already registered resource id should throw
        assert_noop!(
            ChainBridgeTransfer::register_resource_id(
                Origin::root(),
                BHO_RESOURCE_ID,
                BHO_CURRENCY
            ),
            Error::<Runtime>::ResourceIdAlreadyRegistered
        );
    });
}

#[test]
fn unregister_resource_id_should_work() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        // Only admin can invoke this call
        assert_noop!(
            ChainBridgeTransfer::remove_resource_id(Origin::signed(ALICE), BHO_RESOURCE_ID),
            BadOrigin
        );

        // Unregister unregistered should throw error
        assert_noop!(
            ChainBridgeTransfer::remove_resource_id(Origin::root(), BHO_RESOURCE_ID),
            Error::<Runtime>::ResourceIdNotRegistered
        );

        assert_ok!(ChainBridgeTransfer::register_resource_id(
            Origin::root(),
            BHO_RESOURCE_ID,
            BHO_CURRENCY
        ));

        // Unregister registered resource id should succeed
        assert_ok!(ChainBridgeTransfer::remove_resource_id(
            Origin::root(),
            BHO_RESOURCE_ID
        ));
        assert_eq!(
            ChainBridgeTransfer::currency_id_to_resource_id(BHO_CURRENCY),
            None
        );
        assert_eq!(
            ChainBridgeTransfer::resource_id_to_currency_id(BHO_RESOURCE_ID),
            None
        );
        System::assert_last_event(
            crate::Event::ResourceIdUnregistered(BHO_RESOURCE_ID, BHO_CURRENCY).into(),
        );
    });
}

#[test]
fn transfer_to_bridge_should_work() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Transfer unregistered error should throw error
        assert_ok!(ChainBridge::whitelist(1));
        assert_noop!(
            ChainBridgeTransfer::transfer_to_bridge(
                Origin::signed(ALICE),
                BHO_CURRENCY,
                1,
                "0xNguyenVanA".as_bytes().to_vec(),
                100,
            ),
            Error::<Runtime>::ResourceIdNotRegistered
        );

        // Transfer to chain not whitelisted should throw error
        assert_noop!(
            ChainBridgeTransfer::transfer_to_bridge(
                Origin::signed(ALICE),
                BHO_CURRENCY,
                2,
                "0xNguyenVanA".as_bytes().to_vec(),
                100,
            ),
            Error::<Runtime>::InvalidDestChainId
        );

        // Transfer resource originated from bholdus should transfer instead of burn
        assert_ok!(ChainBridgeTransfer::register_resource_id(
            Origin::root(),
            BHO_RESOURCE_ID,
            BHO_CURRENCY
        ));
        assert_ok!(ChainBridgeTransfer::transfer_to_bridge(
            Origin::signed(ALICE),
            BHO_CURRENCY,
            1,
            "0xNguyenVanA".as_bytes().to_vec(),
            100,
        ),);
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BHO_CURRENCY, &ALICE),
            0
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(
                BHO_CURRENCY,
                &ChainBridge::account_id()
            ),
            100
        );

        // Transfer resource originated from foreign chain should burn user token
        assert_ok!(ChainBridgeTransfer::register_resource_id(
            Origin::root(),
            BNB_RESOURCE_ID,
            BNB_CURRENCY
        ));
        assert_ok!(ChainBridgeTransfer::transfer_to_bridge(
            Origin::signed(ALICE),
            BNB_CURRENCY,
            1,
            "0xNguyenVanA".as_bytes().to_vec(),
            100,
        ),);
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BNB_CURRENCY, &ALICE),
            0
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(
                BNB_CURRENCY,
                &ChainBridge::account_id()
            ),
            0
        );
    });
}

#[test]
fn transfer_from_bridge_should_work() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(ChainBridge::whitelist(1));
        // Transfer unregistered resource id should throw error;
        assert_noop!(
            ChainBridgeTransfer::transfer_from_bridge(
                Origin::signed(ChainBridge::account_id()),
                ALICE,
                100,
                BHO_RESOURCE_ID
            ),
            Error::<Runtime>::ResourceIdNotRegistered
        );

        assert_ok!(ChainBridgeTransfer::register_resource_id(
            Origin::root(),
            BHO_RESOURCE_ID,
            BHO_CURRENCY
        ));
        assert_ok!(ChainBridgeTransfer::transfer_to_bridge(
            Origin::signed(ALICE),
            BHO_CURRENCY,
            1,
            "0xNguyenVanA".as_bytes().to_vec(),
            100,
        ));

        assert_ok!(ChainBridgeTransfer::register_resource_id(
            Origin::root(),
            BNB_RESOURCE_ID,
            BNB_CURRENCY
        ));
        assert_ok!(ChainBridgeTransfer::transfer_to_bridge(
            Origin::signed(ALICE),
            BNB_CURRENCY,
            1,
            "0xNguyenVanA".as_bytes().to_vec(),
            100,
        ));

        // Transfer back to user with resource originated from bholdus should unlock tokens
        assert_ok!(ChainBridgeTransfer::transfer_from_bridge(
            Origin::signed(ChainBridge::account_id()),
            ALICE,
            100,
            BHO_RESOURCE_ID
        ));
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BHO_CURRENCY, &ALICE),
            100
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(
                BHO_CURRENCY,
                &ChainBridge::account_id()
            ),
            0
        );

        // Transfer back to user with resource originated from bholdus should mint tokens
        assert_ok!(ChainBridgeTransfer::transfer_from_bridge(
            Origin::signed(ChainBridge::account_id()),
            ALICE,
            100,
            BNB_RESOURCE_ID
        ));
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BHO_CURRENCY, &ALICE),
            100
        );
    });
}

#[test]
fn admin_transfer_from_bridge_should_work() {
    new_test_ext().execute_with(|| {});
}
