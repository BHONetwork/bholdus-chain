#![cfg(test)]

use super::*;
use frame_support::traits::Currency;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};

use bholdus_lib_nft::TokenInfo;
use bholdus_primitives::Balance;
use sp_runtime::{traits::BlakeTwo256, ArithmeticError};
use sp_std::convert::TryInto;

fn class_id_account() -> AccountId {
    <Runtime as Config>::PalletId::get().into_sub_account(CLASS_ID)
}

fn test_attr(x: u8) -> Attributes {
    let mut attr: Attributes = BTreeMap::new();
    attr.insert(vec![x, x + 10], vec![x, x + 1, x + 2]);
    attr.insert(vec![x + 1], vec![11]);
    attr
}

#[test]
fn create_class_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];

        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            metadata.clone(),
        ));

        System::assert_last_event(Event::NFTModule(crate::Event::CreatedClass(
            class_id_account(),
            CLASS_ID,
        )));

        assert_eq!(
            bholdus_lib_nft::Pallet::<Runtime>::classes(0)
                .unwrap()
                .metadata,
            metadata
        )
    });
}

#[test]
fn mint_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];
        let metadata_2 = vec![2, 3];
        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            metadata.clone(),
        ));
        System::assert_last_event(Event::NFTModule(crate::Event::CreatedClass(
            class_id_account(),
            CLASS_ID,
        )));

        assert_ok!(NFTModule::mint(
            Origin::signed(class_id_account()),
            BOB,
            CLASS_ID,
            test_attr(2),
            2
        ));

        System::assert_last_event(Event::NFTModule(crate::Event::MintedToken(
            class_id_account(),
            BOB,
            CLASS_ID,
            2,
        )));

        assert_eq!(
            bholdus_lib_nft::TokensByOwner::<Runtime>::iter_prefix((BOB,)).collect::<Vec<_>>(),
            vec![((0, 1), ()), ((0, 0), ())]
        );
    });
}

#[test]
fn mint_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];
        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            metadata.clone(),
        ));

        assert_noop!(
            NFTModule::mint(
                Origin::signed(ALICE),
                BOB,
                CLASS_ID_NOT_EXIST,
                Default::default(),
                2
            ),
            Error::<Runtime>::ClassIdNotFound
        );

        assert_noop!(
            NFTModule::mint(Origin::signed(BOB), BOB, CLASS_ID, Default::default(), 0),
            Error::<Runtime>::InvalidQuantity
        );

        assert_noop!(
            NFTModule::mint(Origin::signed(BOB), BOB, CLASS_ID, Default::default(), 2),
            Error::<Runtime>::NoPermission
        );

        bholdus_lib_nft::NextTokenId::<Runtime>::mutate(CLASS_ID, |id| {
            *id = <Runtime as bholdus_lib_nft::Config>::TokenId::max_value()
        });
        assert_noop!(
            NFTModule::mint(
                Origin::signed(class_id_account()),
                BOB,
                CLASS_ID,
                Default::default(),
                2
            ),
            bholdus_lib_nft::Error::<Runtime>::NoAvailableTokenId
        );
    });
}

#[test]
fn mint_should_fail_without_mintable() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];
        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            metadata.clone(),
        ));
    });
}

#[test]
fn transfer_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];
        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            metadata.clone(),
        ));

        assert_ok!(NFTModule::mint(
            Origin::signed(class_id_account()),
            BOB,
            CLASS_ID,
            Default::default(),
            2
        ));

        assert_ok!(NFTModule::transfer(
            Origin::signed(BOB),
            ALICE,
            (CLASS_ID, TOKEN_ID)
        ));
        System::assert_last_event(Event::NFTModule(crate::Event::TransferredToken(
            BOB, ALICE, CLASS_ID, TOKEN_ID,
        )));

        assert_ok!(NFTModule::transfer(
            Origin::signed(ALICE),
            BOB,
            (CLASS_ID, TOKEN_ID)
        ));
        System::assert_last_event(Event::NFTModule(crate::Event::TransferredToken(
            ALICE, BOB, CLASS_ID, TOKEN_ID,
        )));
    });
}

#[test]
fn transfer_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];
        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            metadata.clone(),
        ));
        assert_ok!(NFTModule::mint(
            Origin::signed(class_id_account()),
            BOB,
            CLASS_ID,
            Default::default(),
            1
        ));

        assert_noop!(
            NFTModule::transfer(Origin::signed(BOB), ALICE, (CLASS_ID_NOT_EXIST, TOKEN_ID)),
            Error::<Runtime>::ClassIdNotFound
        );

        assert_noop!(
            NFTModule::transfer(Origin::signed(BOB), ALICE, (CLASS_ID, TOKEN_ID_NOT_EXIST)),
            Error::<Runtime>::TokenIdNotFound
        );

        assert_noop!(
            NFTModule::transfer(Origin::signed(ALICE), BOB, (CLASS_ID, TOKEN_ID)),
            bholdus_lib_nft::Error::<Runtime>::NoPermission
        );
    });

    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];
        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            metadata.clone(),
        ));
    })
}

#[test]
fn burn_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];
        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            metadata.clone(),
        ));

        assert_ok!(NFTModule::mint(
            Origin::signed(class_id_account()),
            BOB,
            CLASS_ID,
            Default::default(),
            1
        ));
        assert_ok!(NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));
        System::assert_last_event(Event::NFTModule(crate::Event::BurnedToken(
            BOB, CLASS_ID, TOKEN_ID,
        )));
    })
}

#[test]
fn burn_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];
        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            metadata.clone(),
        ));
        assert_ok!(NFTModule::mint(
            Origin::signed(class_id_account()),
            BOB,
            CLASS_ID,
            Default::default(),
            1
        ));

        assert_noop!(
            NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID_NOT_EXIST)),
            Error::<Runtime>::TokenIdNotFound
        );

        assert_noop!(
            NFTModule::burn(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)),
            Error::<Runtime>::NoPermission
        );

        bholdus_lib_nft::Classes::<Runtime>::mutate(CLASS_ID, |class_info| {
            class_info.as_mut().unwrap().total_issuance = 0;
        });
        assert_noop!(
            NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
            ArithmeticError::Overflow,
        );
    });
}

#[test]
fn destroy_class_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];
        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            metadata.clone(),
        ));
        assert_ok!(NFTModule::mint(
            Origin::signed(class_id_account()),
            BOB,
            CLASS_ID,
            Default::default(),
            1
        ));

        assert_ok!(NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));
        assert_ok!(NFTModule::destroy_class(
            Origin::signed(class_id_account()),
            CLASS_ID,
        ));

        System::assert_last_event(Event::NFTModule(crate::Event::DestroyedClass(
            class_id_account(),
            CLASS_ID,
        )));
    })
}
#[test]
fn destroy_class_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];
        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            metadata.clone(),
        ));

        assert_ok!(NFTModule::mint(
            Origin::signed(class_id_account()),
            BOB,
            CLASS_ID,
            Default::default(),
            1
        ));

        assert_noop!(
            NFTModule::destroy_class(Origin::signed(class_id_account()), CLASS_ID_NOT_EXIST),
            Error::<Runtime>::ClassIdNotFound
        );

        assert_noop!(
            NFTModule::destroy_class(Origin::signed(BOB), CLASS_ID),
            Error::<Runtime>::NoPermission
        );

        assert_noop!(
            NFTModule::destroy_class(Origin::signed(class_id_account()), CLASS_ID),
            Error::<Runtime>::CannotDestroyClass
        );

        assert_ok!(NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));

        assert_ok!(NFTModule::destroy_class(
            Origin::signed(class_id_account()),
            CLASS_ID,
        ));
    });
}
