#![cfg(test)]

use super::*;
use frame_support::traits::Currency;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};

use bholdus_primitives::Balance;
use bholdus_support_nft::TokenInfo;
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

        assert_ok!(NFTModule::create_class(Origin::signed(ALICE), test_attr(1)));

        System::assert_last_event(Event::NFTModule(crate::Event::CreatedClass(
            class_id_account(),
            CLASS_ID,
        )));

        assert_eq!(
            bholdus_support_nft::Pallet::<Runtime>::classes(0)
                .unwrap()
                .data,
            ClassData {
                attributes: test_attr(1),
            }
        )
        // let class_info = bholdus_support_nft::Classes::<Runtime>::get(CLASS_ID);
        // println!("create_class_should_work {:?}", class_info);
    });
}

#[test]
fn mint_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];
        let metadata_2 = vec![2, 3];

        assert_ok!(NFTModule::create_class(Origin::signed(ALICE), test_attr(1),));

        System::assert_last_event(Event::NFTModule(crate::Event::CreatedClass(
            class_id_account(),
            CLASS_ID,
        )));

        assert_ok!(NFTModule::mint(
            Origin::signed(class_id_account()),
            BOB,
            CLASS_ID,
            metadata_2.clone(),
            test_attr(2),
            3
        ));

        System::assert_last_event(Event::NFTModule(crate::Event::MintedToken(
            class_id_account(),
            BOB,
            CLASS_ID,
            TOKEN_ID,
            3,
        )));

        // Test TokensByOwner

        assert_eq!(
            bholdus_support_nft::TokensByOwner::<Runtime>::iter_values().collect::<Vec<_>>(),
            vec![(BOB, 1), (BOB, 0), (BOB, 2)]
        );

        // println!(
        //     "transfer_should_work: tokens_by_owner {:#?}",
        //     bholdus_support_nft::TokensByOwner::<Runtime>::iter_values().collect::<Vec<_>>()
        // );

        // Test TokensByGroup
        //
        // assert_eq!(
        //     bholdus_support_nft::TokensByGroup::<Runtime>::contains_key((
        //         GROUP_ID, CLASS_ID, TOKEN_ID
        //     )),
        //     true
        // );
        assert_eq!(
            bholdus_support_nft::TokensByGroup::<Runtime>::contains_key((
                GROUP_ID, CLASS_ID, TOKEN_ID
            )),
            true
        );

        println!(
            "mint_should_work: tokens_by_group{:#?}",
            bholdus_support_nft::TokensByGroup::<Runtime>::iter_values().collect::<Vec<_>>()
        );

        println!(
            "mint_should_work: test prefix tokens_by_group{:#?}",
            bholdus_support_nft::TokensByGroup::<Runtime>::iter_prefix((GROUP_ID, CLASS_ID,))
                .collect::<Vec<_>>()
        );

        // assert_eq!(
        //     bholdus_support_nft::TokensByGroup::<Runtime>::iter_prefix((GROUP_ID))
        //         .collect::<Vec<_>>(),
        //     vec![(BOB, 1), (BOB, 0), (BOB, 2)]
        // );

        // Test OwnedTokens

        assert_eq!(
            bholdus_support_nft::OwnedTokens::<Runtime>::iter_values().collect::<Vec<_>>(),
            vec![(BOB, 1), (BOB, 0), (BOB, 2)]
        )

        // println!(
        //     "mint_should_work: owned_tokens {:#?}",
        //     bholdus_support_nft::OwnedTokens::<Runtime>::iter_values().collect::<Vec<_>>()
        // );
        //
    });
}

#[test]
fn mint_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];
        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            Default::default(),
        ));

        assert_noop!(
            NFTModule::mint(
                Origin::signed(ALICE),
                BOB,
                CLASS_ID_NOT_EXIST,
                metadata.clone(),
                Default::default(),
                2
            ),
            Error::<Runtime>::ClassIdNotFound
        );

        assert_noop!(
            NFTModule::mint(
                Origin::signed(BOB),
                BOB,
                CLASS_ID,
                metadata.clone(),
                Default::default(),
                0
            ),
            Error::<Runtime>::InvalidQuantity
        );

        assert_noop!(
            NFTModule::mint(
                Origin::signed(BOB),
                BOB,
                CLASS_ID,
                metadata.clone(),
                Default::default(),
                101
            ),
            Error::<Runtime>::InvalidQuantity
        );

        // assert_noop!(
        //     NFTModule::mint(
        //         Origin::signed(BOB),
        //         BOB,
        //         CLASS_ID,
        //         metadata.clone(),
        //         Default::default(),
        //         2
        //     ),
        //     Error::<Runtime>::NoPermission
        // );

        bholdus_support_nft::NextTokenId::<Runtime>::mutate(CLASS_ID, |id| {
            *id = <Runtime as bholdus_support_nft::Config>::TokenId::max_value()
        });
        assert_noop!(
            NFTModule::mint(
                Origin::signed(class_id_account()),
                BOB,
                CLASS_ID,
                metadata.clone(),
                Default::default(),
                2
            ),
            bholdus_support_nft::Error::<Runtime>::NoAvailableTokenId
        );
    });
}

#[test]
fn mint_should_fail_without_mintable() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];
        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            Default::default(),
        ));
    });
}

#[test]
fn transfer_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1, 2, 3];
        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            Default::default(),
        ));

        assert_ok!(NFTModule::mint(
            Origin::signed(class_id_account()),
            BOB,
            CLASS_ID,
            metadata.clone(),
            Default::default(),
            1
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
            DAVE,
            (CLASS_ID, TOKEN_ID)
        ));

        System::assert_last_event(Event::NFTModule(crate::Event::TransferredToken(
            ALICE, DAVE, CLASS_ID, TOKEN_ID,
        )));

        assert_eq!(
            bholdus_support_nft::TokensByOwner::<Runtime>::iter_values().collect::<Vec<_>>(),
            vec![(DAVE, 0)]
        );

        assert_eq!(
            bholdus_support_nft::OwnedTokens::<Runtime>::iter_values().collect::<Vec<_>>(),
            vec![(DAVE, 0), (ALICE, 0), (BOB, 0)]
        )
    });
}

#[test]
fn transfer_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];
        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            Default::default(),
        ));
        assert_ok!(NFTModule::mint(
            Origin::signed(class_id_account()),
            BOB,
            CLASS_ID,
            metadata,
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

        // assert_noop!(
        //     NFTModule::transfer(Origin::signed(ALICE), BOB, (CLASS_ID, TOKEN_ID)),
        //     bholdus_support_nft::Error::<Runtime>::NoPermission
        // );
    });

    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];
        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            Default::default(),
        ));
    })
}

#[test]
fn burn_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];
        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            Default::default(),
        ));

        assert_ok!(NFTModule::mint(
            Origin::signed(class_id_account()),
            BOB,
            CLASS_ID,
            metadata.clone(),
            Default::default(),
            1
        ));

        assert_ok!(NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));

        System::assert_last_event(Event::NFTModule(crate::Event::BurnedToken(
            BOB, CLASS_ID, TOKEN_ID,
        )));

        assert_eq!(
            bholdus_support_nft::TokensByOwner::<Runtime>::iter_values().collect::<Vec<_>>(),
            vec![]
        );

        assert_eq!(
            bholdus_support_nft::OwnedTokens::<Runtime>::iter_values().collect::<Vec<_>>(),
            vec![(BOB, 0)]
        )
    })
}

#[test]
fn burn_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        let metadata = vec![1];
        assert_ok!(NFTModule::create_class(
            Origin::signed(ALICE),
            Default::default(),
        ));
        assert_ok!(NFTModule::mint(
            Origin::signed(class_id_account()),
            BOB,
            CLASS_ID,
            metadata,
            Default::default(),
            1
        ));

        assert_noop!(
            NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID_NOT_EXIST)),
            Error::<Runtime>::TokenIdNotFound
        );

        // assert_noop!(
        //     NFTModule::burn(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)),
        //     Error::<Runtime>::NoPermission
        // );

        bholdus_support_nft::Classes::<Runtime>::mutate(CLASS_ID, |class_info| {
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
            Default::default(),
        ));
        assert_ok!(NFTModule::mint(
            Origin::signed(class_id_account()),
            BOB,
            CLASS_ID,
            metadata,
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
            // metadata.clone(),
            Default::default(),
        ));

        assert_ok!(NFTModule::mint(
            Origin::signed(class_id_account()),
            BOB,
            CLASS_ID,
            metadata,
            Default::default(),
            1
        ));

        assert_noop!(
            NFTModule::destroy_class(Origin::signed(class_id_account()), CLASS_ID_NOT_EXIST),
            Error::<Runtime>::ClassIdNotFound
        );

        // assert_noop!(
        //     NFTModule::destroy_class(Origin::signed(BOB), CLASS_ID),
        //     Error::<Runtime>::NoPermission
        // );

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
