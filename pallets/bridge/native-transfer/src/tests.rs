use crate::{mock::*, Bytes, ChainId, OutboundTransferInfo, TransferId};
use bholdus_primitives::Balance;
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use sp_runtime::{traits::CheckedAdd, FixedPointNumber, FixedU128};

#[test]
fn force_register_relayer_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let relayer_id = 10;
        assert_noop!(
            BridgeNativeTransfer::force_register_relayer(Origin::signed(ALICE), relayer_id),
            BadOrigin
        );

        assert_ok!(BridgeNativeTransfer::force_register_relayer(
            Origin::root(),
            relayer_id
        ));

        assert_eq!(BridgeNativeTransfer::registered_relayers(relayer_id), true);
    });
}

#[test]
fn force_unregister_relayer_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let relayer_id = 10;
        assert_noop!(
            BridgeNativeTransfer::force_unregister_relayer(Origin::signed(ALICE), relayer_id),
            BadOrigin
        );

        assert_ok!(BridgeNativeTransfer::force_register_relayer(
            Origin::root(),
            relayer_id
        ));
        assert_eq!(BridgeNativeTransfer::registered_relayers(relayer_id), true);

        assert_ok!(BridgeNativeTransfer::force_unregister_relayer(
            Origin::root(),
            relayer_id
        ));
        assert_eq!(BridgeNativeTransfer::registered_relayers(relayer_id), false);
    });
}

#[test]
fn force_set_service_fee_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            BridgeNativeTransfer::force_set_service_fee(Origin::signed(ALICE), (4, 10)),
            BadOrigin
        );

        assert_ok!(BridgeNativeTransfer::force_set_service_fee(
            Origin::root(),
            (4, 10)
        ));
        assert_eq!(BridgeNativeTransfer::service_fee_rate(), (4, 10));
    });
}

#[test]
fn force_withdraw_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            BridgeNativeTransfer::force_withdraw(Origin::signed(ALICE), CHARLIE),
            BadOrigin
        );

        assert_ok!(Balances::transfer(
            Origin::signed(ALICE),
            BridgeNativeTransfer::pallet_account_id(),
            1_000
        ));
        assert_eq!(
            Balances::free_balance(BridgeNativeTransfer::pallet_account_id()),
            1_000
        );
        assert_ok!(BridgeNativeTransfer::force_withdraw(
            Origin::root(),
            CHARLIE
        ));
        assert_eq!(
            Balances::free_balance(BridgeNativeTransfer::pallet_account_id()),
            0
        );
        assert_eq!(Balances::free_balance(CHARLIE), 1_000);
    })
}

fn assert_initiate_transfer(
    from: &AccountId,
    to: Bytes,
    amount: u128,
    target_chain: ChainId,
    expected_transfer_id: TransferId,
) {
    let initial_from_balance = Balances::free_balance(from);
    let initial_pallet_balance = Balances::free_balance(BridgeNativeTransfer::pallet_account_id());

    assert_ok!(BridgeNativeTransfer::initiate_transfer(
        Origin::signed(from.clone()),
        to.clone(),
        amount,
        target_chain,
    ));

    assert_eq!(
        Balances::free_balance(from),
        initial_from_balance.checked_sub(amount).unwrap()
    );
    let service_fee_rate = BridgeNativeTransfer::service_fee_rate();
    let service_fee = FixedU128::checked_from_rational(service_fee_rate.0, service_fee_rate.1)
        .unwrap()
        .checked_mul_int(amount)
        .unwrap();
    let total_charge = amount.checked_add(service_fee).unwrap();

    assert_eq!(
        Balances::free_balance(BridgeNativeTransfer::pallet_account_id()),
        initial_pallet_balance.checked_add(total_charge).unwrap()
    );
    assert_eq!(
        BridgeNativeTransfer::outbound_transfers(expected_transfer_id).unwrap(),
        OutboundTransferInfo {
            amount,
            from: from.clone(),
            service_fee,
            target_chain,
            to: to.clone()
        }
    );
    assert_eq!(
        BridgeNativeTransfer::next_outbound_transfer_id(),
        expected_transfer_id.checked_add(1).unwrap()
    );

    System::assert_last_event(Event::BridgeNativeTransfer(
        crate::Event::OutboundTransferInitiated(
            expected_transfer_id,
            from.clone(),
            to.clone(),
            amount,
        ),
    ));
}

#[test]
fn force_register_chain_should_work() {
    let target_chain: u16 = 0;
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            BridgeNativeTransfer::force_register_chain(Origin::signed(ALICE), target_chain),
            BadOrigin
        );

        assert_eq!(BridgeNativeTransfer::registered_chains(target_chain), false);
        assert_ok!(BridgeNativeTransfer::force_register_chain(
            Origin::root(),
            target_chain
        ));
        assert_eq!(BridgeNativeTransfer::registered_chains(target_chain), true);
    });
}

#[test]
fn force_unregister_chain_should_work() {
    let target_chain: u16 = 0;
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            BridgeNativeTransfer::force_unregister_chain(Origin::signed(ALICE), target_chain),
            BadOrigin
        );

        assert_ok!(BridgeNativeTransfer::force_register_chain(
            Origin::root(),
            target_chain
        ));
        assert_eq!(BridgeNativeTransfer::registered_chains(target_chain), true);
        assert_ok!(BridgeNativeTransfer::force_unregister_chain(
            Origin::root(),
            target_chain
        ));
        assert_eq!(BridgeNativeTransfer::registered_chains(target_chain), false);
    });
}

#[test]
fn initiate_transfer_should_work() {
    let to = hex::decode("2e8688827CCb7B015552a9817ca3E9E3a08Ae596").unwrap();
    let transfer_amount = 1000_u128;
    let target_chain: ChainId = 1;
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        assert_noop!(
            BridgeNativeTransfer::initiate_transfer(
                Origin::signed(ALICE),
                to.clone(),
                1_000_000u128,
                target_chain
            ),
            crate::Error::<Runtime>::MustBeRegisteredChain
        );

        assert_ok!(BridgeNativeTransfer::force_register_chain(
            Origin::root(),
            target_chain
        ));

        assert_noop!(
            BridgeNativeTransfer::initiate_transfer(
                Origin::signed(ALICE),
                to.clone(),
                1,
                target_chain
            ),
            crate::Error::<Runtime>::MinimumDepositRequired
        );

        assert_noop!(
            BridgeNativeTransfer::initiate_transfer(
                Origin::signed(ALICE),
                to.clone(),
                1_000_000u128,
                target_chain
            ),
            pallet_balances::Error::<Runtime>::InsufficientBalance
        );

        assert_initiate_transfer(&ALICE, to.clone(), transfer_amount, target_chain, 0);
        assert_initiate_transfer(&BOB, to.clone(), transfer_amount, target_chain, 1);
    });
}

fn assert_confirm_transfer(relayer_id: AccountId, transfer_id: TransferId) {
    let relayer_initial_balance = Balances::free_balance(relayer_id);
    assert_ok!(BridgeNativeTransfer::confirm_transfer(
        Origin::signed(relayer_id),
        transfer_id
    ));

    assert_eq!(
        BridgeNativeTransfer::next_confirm_outbound_transfer_id(),
        transfer_id.checked_add(1).unwrap()
    );
    assert_eq!(
        Balances::free_balance(relayer_id),
        BridgeNativeTransfer::outbound_transfers(transfer_id)
            .unwrap()
            .service_fee
            .checked_add(relayer_initial_balance)
            .unwrap()
    );
}

#[test]
fn confirm_transfer_should_work() {
    let to = hex::decode("2e8688827CCb7B015552a9817ca3E9E3a08Ae596").unwrap();
    let transfer_amount = 1000_u128;
    let target_chain: ChainId = 1;
    let relayer_id: AccountId = 10;
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(BridgeNativeTransfer::force_register_chain(
            Origin::root(),
            target_chain
        ));

        assert_noop!(
            BridgeNativeTransfer::confirm_transfer(Origin::signed(relayer_id), 0),
            crate::Error::<Runtime>::MustBeRegisteredRelayer
        );

        assert_ok!(BridgeNativeTransfer::force_register_relayer(
            Origin::root(),
            relayer_id
        ));

        assert_initiate_transfer(&ALICE, to.clone(), transfer_amount, target_chain, 0);
        assert_initiate_transfer(&BOB, to.clone(), transfer_amount, target_chain, 1);

        assert_noop!(
            BridgeNativeTransfer::confirm_transfer(Origin::signed(relayer_id), 1),
            crate::Error::<Runtime>::UnexpectedOutboundTransferConfirmation
        );

        assert_confirm_transfer(relayer_id, 0);
        assert_confirm_transfer(relayer_id, 1);

        assert_noop!(
            BridgeNativeTransfer::confirm_transfer(Origin::signed(relayer_id), 1),
            crate::Error::<Runtime>::AllOutboundTransfersConfirmed
        );
    });
}

fn assert_release_tokens(
    relayer_id: AccountId,
    transfer_id: TransferId,
    from: Bytes,
    to: AccountId,
    amount: u128,
) {
    let initial_to_balance = Balances::free_balance(to);
    let initial_pallet_balance = Balances::free_balance(BridgeNativeTransfer::pallet_account_id());
    assert_ok!(BridgeNativeTransfer::release_tokens(
        Origin::signed(relayer_id),
        transfer_id,
        from.clone(),
        to,
        amount
    ));
    assert_eq!(
        Balances::free_balance(ALICE),
        initial_to_balance.checked_add(amount).unwrap()
    );
    assert_eq!(
        Balances::free_balance(BridgeNativeTransfer::pallet_account_id()),
        initial_pallet_balance.checked_sub(amount).unwrap()
    );
    assert_eq!(
        BridgeNativeTransfer::next_inbound_transfer_id(),
        transfer_id.checked_add(1).unwrap()
    );
    System::assert_last_event(Event::BridgeNativeTransfer(
        crate::Event::InboundTokenReleased(transfer_id, from.clone(), to, amount),
    ));
}

#[test]
fn release_tokens_should_work() {
    let target_chain: ChainId = 1;
    let relayer_id: AccountId = 10;
    let from = hex::decode("2e8688827CCb7B015552a9817ca3E9E3a08Ae596").unwrap();
    ExtBuilder::default()
        .with_balances(vec![(
            BridgeNativeTransfer::pallet_account_id(),
            100_000u128,
        )])
        .build()
        .execute_with(|| {
            System::set_block_number(1);

            assert_ok!(BridgeNativeTransfer::force_register_chain(
                Origin::root(),
                target_chain
            ));

            assert_noop!(
                BridgeNativeTransfer::release_tokens(
                    Origin::signed(ALICE),
                    0,
                    from.clone(),
                    ALICE,
                    1_000u128
                ),
                crate::Error::<Runtime>::MustBeRegisteredRelayer
            );

            assert_ok!(BridgeNativeTransfer::force_register_relayer(
                Origin::root(),
                relayer_id
            ));

            assert_noop!(
                BridgeNativeTransfer::release_tokens(
                    Origin::signed(relayer_id),
                    1,
                    from.clone(),
                    ALICE,
                    1_000u128
                ),
                crate::Error::<Runtime>::UnexpectedInboundTransfer
            );

            assert_release_tokens(relayer_id, 0, from.clone(), ALICE, 1_000u128);

            assert_noop!(
                BridgeNativeTransfer::release_tokens(
                    Origin::signed(relayer_id),
                    0,
                    from.clone(),
                    ALICE,
                    1_000u128
                ),
                crate::Error::<Runtime>::UnexpectedInboundTransfer
            );

            assert_noop!(
                BridgeNativeTransfer::release_tokens(
                    Origin::signed(relayer_id),
                    1,
                    from.clone(),
                    ALICE,
                    100_000u128
                ),
                pallet_balances::Error::<Runtime>::InsufficientBalance
            );
        });
}
