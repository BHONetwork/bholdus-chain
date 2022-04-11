//! Unit tests for the non-fungible-token pallet.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

#[test]
fn create_class_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BholdusNFT::create_class(&ALICE, ()));
    });
}

#[test]
fn create_class_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        NextClassId::<Runtime>::mutate(|id| *id = <Runtime as Config>::ClassId::max_value());
        assert_noop!(
            BholdusNFT::create_class(&ALICE, ()),
            Error::<Runtime>::NoAvailableClassId
        );
    });
}

#[test]
fn create_group_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BholdusNFT::create_group());
    })
}

/*#[test]
fn mint_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let next_class_id = BholdusNFT::next_class_id();
        println!("NEXT CLASS ID {}", next_class_id);
        assert_eq!(next_class_id, CLASS_ID);

        // Create class
        assert_ok!(BholdusNFT::create_class(&ALICE, ()));
        assert_eq!(BholdusNFT::next_token_id(), 0); //assert_eq!(BholdusNFT::next_token_id(CLASS_ID), 0);
        assert_ok!(BholdusNFT::mint(&BOB, CLASS_ID, vec![1], ()));
        assert_eq!(BholdusNFT::next_token_id(), 1); //assert_eq!(BholdusNFT::next_token_id(CLASS_ID), 1);
        assert_ok!(BholdusNFT::mint(&BOB, CLASS_ID, vec![1], ()));
        assert_eq!(BholdusNFT::next_token_id(), 2); //assert_eq!(BholdusNFT::next_token_id(CLASS_ID), 2);

        let next_class_id = BholdusNFT::next_class_id();
        assert_ok!(BholdusNFT::create_class(&ALICE, ()));
        assert_eq!(BholdusNFT::next_token_id(), 0); //assert_eq!(BholdusNFT::next_token_id(next_class_id), 0);
        assert_ok!(BholdusNFT::mint(&BOB, next_class_id, vec![1], ()));
        assert_eq!(BholdusNFT::next_token_id(), 1); //assert_eq!(BholdusNFT::next_token_id(next_class_id), 1);

        assert_eq!(BholdusNFT::next_token_id(), 2); //assert_eq!(BholdusNFT::next_token_id(CLASS_ID), 2);
    });
}
*/

#[test]
fn mint_to_group_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(BholdusNFT::next_class_id(), CLASS_ID);
        assert_ok!(BholdusNFT::create_class(&ALICE, ()));

        assert_eq!(BholdusNFT::next_group_id(), GROUP_ID);
        assert_ok!(BholdusNFT::create_group());

        assert_ok!(BholdusNFT::mint_to_group(
            &ALICE,
            CLASS_ID,
            GROUP_ID,
            vec![1],
            ()
        ));
        assert_eq!(BholdusNFT::next_class_id(), 1);
        assert_eq!(BholdusNFT::next_group_id(), 1);

        assert_ok!(BholdusNFT::mint_to_group(
            &ALICE,
            CLASS_ID,
            GROUP_ID,
            vec![1],
            ()
        ));

        assert_ok!(BholdusNFT::mint_to_group(
            &ALICE,
            CLASS_ID,
            GROUP_ID,
            vec![1],
            ()
        ));

        // assert_eq!(
        //     TokensByGroup::<Runtime>::contains_key((
        //         GROUP_ID, CLASS_ID, TOKEN_ID
        //     )),
        //     true
        // );

        assert_eq!(
            TokensByGroup::<Runtime>::contains_key((GROUP_ID, CLASS_ID, TOKEN_ID)),
            true
        );

        // let tokens_by_group = TokensByGroup::<Runtime>::get((GROUP_ID, CLASS_ID, TOKEN_ID));
        // println!("tokens-by-group{:#?}", tokens_by_group);

        let tokens_by_group =
            TokensByGroup::<Runtime>::iter_prefix((GROUP_ID, CLASS_ID)).collect::<Vec<_>>();
        // println!("tokens-by-group{:#?}", tokens_by_group);
        // assert_eq!(
        //     tokens_by_group,
        //     vec![((0, 2), ()), ((0, 1), ()), ((0, 0), ())]
        // )
    });
}

#[test]
fn mint_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BholdusNFT::create_class(&ALICE, ()));
        Classes::<Runtime>::mutate(CLASS_ID, |class_info| {
            class_info.as_mut().unwrap().total_issuance = <Runtime as Config>::TokenId::max_value();
        });
        /*assert_noop!(
            BholdusNFT::mint(&BOB, CLASS_ID, vec![1], ()),
            ArithmeticError::Overflow,
        );

        NextTokenId::<Runtime>::mutate(|id| *id = <Runtime as Config>::TokenId::max_value());
        assert_noop!(
            BholdusNFT::mint(&BOB, CLASS_ID, vec![1], ()),
            Error::<Runtime>::NoAvailableTokenId
        );
        */
    });
}

#[test]
fn transfer_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BholdusNFT::create_class(&ALICE, ()));

        assert_ok!(BholdusNFT::mint(&BOB, CLASS_ID, vec![1], ()));
        assert_ok!(BholdusNFT::transfer(&BOB, &ALICE, (CLASS_ID, TOKEN_ID)));
        assert_ok!(BholdusNFT::transfer(&ALICE, &BOB, (CLASS_ID, TOKEN_ID)));
        // assert!(BholdusNFT::is_owner(&BOB, (CLASS_ID, TOKEN_ID)));

        // let token_owner = TokensByOwner::<Runtime>::get((ALICE, CLASS_ID, TOKEN_ID));
        // println!("tokens-owner {:#?}", token_owner);

        /*println!(
            "transferred: tokens {:?} && tokens_owner {:?}",
            Tokens::<Runtime>::iter_values().collect::<Vec<_>>(),
            TokensByOwner::<Runtime>::iter_values().collect::<Vec<_>>()
        );
        */
    });
}

#[test]
fn transfer_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BholdusNFT::create_class(&ALICE, ()));
        assert_ok!(BholdusNFT::mint(&BOB, CLASS_ID, vec![1], ()));
        assert_noop!(
            BholdusNFT::transfer(&BOB, &ALICE, (CLASS_ID, TOKEN_ID_NOT_EXIST)),
            Error::<Runtime>::TokenNotFound
        );

        assert_noop!(
            BholdusNFT::transfer(&ALICE, &BOB, (CLASS_ID, TOKEN_ID)),
            Error::<Runtime>::NoPermission
        );

        /*assert_noop!(
            BholdusNFT::mint(&BOB, CLASS_ID_NOT_EXIST, vec![1], ()),
            Error::<Runtime>::ClassNotFound
        );
        */

        assert_noop!(
            BholdusNFT::transfer(&ALICE, &ALICE, (CLASS_ID, TOKEN_ID)),
            Error::<Runtime>::NoPermission
        );

        LockableNFT::<Runtime>::insert((&BOB, CLASS_ID, TOKEN_ID), ());
        assert_noop!(
            BholdusNFT::transfer(&BOB, &ALICE, (CLASS_ID, TOKEN_ID)),
            Error::<Runtime>::IsLocked
        );
    });
}

#[test]
fn burn_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BholdusNFT::create_class(&ALICE, ()));
        assert_ok!(BholdusNFT::create_class(&ALICE, ()));

        // token_id = 0
        assert_ok!(BholdusNFT::mint(&BOB, CLASS_ID, vec![1], ()));

        assert!(BholdusNFT::is_owner(&BOB, (CLASS_ID, TOKEN_ID)));
        // assert!(BholdusNFT::is_owner(&ALICE, (CLASS_ID, 1)));

        // token_id = 1
        assert_ok!(BholdusNFT::mint(&ALICE, 1, vec![2], ()));

        assert_ok!(BholdusNFT::burn(&BOB, (CLASS_ID, TOKEN_ID)));

        assert_eq!(BholdusNFT::is_owner(&BOB, (CLASS_ID, TOKEN_ID)), false);
    });
}

#[test]
fn burn_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BholdusNFT::create_class(&ALICE, ()));
        assert_ok!(BholdusNFT::mint(&BOB, CLASS_ID, vec![1], ()));
        assert_noop!(
            BholdusNFT::burn(&ALICE, (CLASS_ID, TOKEN_ID)),
            Error::<Runtime>::NoPermission
        );

        LockableNFT::<Runtime>::insert((&BOB, CLASS_ID, TOKEN_ID), ());
        assert_noop!(
            BholdusNFT::transfer(&BOB, &ALICE, (CLASS_ID, TOKEN_ID)),
            Error::<Runtime>::IsLocked
        );
    });

    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BholdusNFT::create_class(&ALICE, ()));
        assert_ok!(BholdusNFT::mint(&BOB, CLASS_ID, vec![1], ()));

        Classes::<Runtime>::mutate(CLASS_ID, |class_info| {
            class_info.as_mut().unwrap().total_issuance = 0;
        });
        assert_noop!(
            BholdusNFT::burn(&BOB, (CLASS_ID, TOKEN_ID)),
            ArithmeticError::Overflow,
        );
    });
}

#[test]
fn destroy_class_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BholdusNFT::create_class(&ALICE, ()));
        assert_ok!(BholdusNFT::mint(&BOB, CLASS_ID, vec![1], ()));
        assert_ok!(BholdusNFT::burn(&BOB, (CLASS_ID, TOKEN_ID)));
        assert_ok!(BholdusNFT::destroy_class(&ALICE, CLASS_ID));
        assert_eq!(Classes::<Runtime>::contains_key(CLASS_ID), false);
        assert_eq!(NextTokenIdByClass::<Runtime>::contains_key(TOKEN_ID), false);
    });
}

#[test]
fn destroy_class_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BholdusNFT::create_class(&ALICE, ()));
        assert_ok!(BholdusNFT::mint(&BOB, CLASS_ID, vec![1], ()));
        assert_noop!(
            BholdusNFT::destroy_class(&ALICE, CLASS_ID_NOT_EXIST),
            Error::<Runtime>::ClassNotFound
        );

        assert_noop!(
            BholdusNFT::destroy_class(&ALICE, CLASS_ID),
            Error::<Runtime>::CannotDestroyClass
        );

        assert_ok!(BholdusNFT::burn(&BOB, (CLASS_ID, TOKEN_ID)));
        assert_ok!(BholdusNFT::destroy_class(&ALICE, CLASS_ID));
        assert_eq!(Classes::<Runtime>::contains_key(CLASS_ID), false);

        Classes::<Runtime>::get(CLASS_ID);
        Classes::<Runtime>::iter_values().collect::<Vec<_>>();
        TokensByOwner::<Runtime>::get((BOB, CLASS_ID, TOKEN_ID));
        TokensByOwner::<Runtime>::iter_values().collect::<Vec<_>>();
        Tokens::<Runtime>::iter_values().collect::<Vec<_>>();
    });
}

// #[test]
// fn exceeding_max_metadata_should_fail() {
//     ExtBuilder::default().build().execute_with(|| {
//         assert_noop!(
//             BholdusNFT::create_class(&ALICE, vec![1, 2]),
//             Error::<Runtime>::MaxMetadataExceeded
//         );
//         assert_ok!(BholdusNFT::create_class(&ALICE, vec![1]));
//     });
// }

#[test]
fn set_lock_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BholdusNFT::create_class(&ALICE, ()));
        assert_ok!(BholdusNFT::mint(&BOB, CLASS_ID, vec![1], ()));
        LockableNFT::<Runtime>::insert((&BOB, CLASS_ID, TOKEN_ID), ());
        BholdusNFT::set_lock((&BOB, CLASS_ID, TOKEN_ID));
        assert!(LockableNFT::<Runtime>::contains_key((
            &BOB, CLASS_ID, TOKEN_ID
        )));
        assert!(BholdusNFT::is_lock(&BOB, (CLASS_ID, TOKEN_ID)));
    })
}

#[test]
fn unlock_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BholdusNFT::create_class(&ALICE, ()));
        assert_ok!(BholdusNFT::mint(&BOB, CLASS_ID, vec![1], ()));
        LockableNFT::<Runtime>::insert((&BOB, CLASS_ID, TOKEN_ID), ());
        BholdusNFT::unlock(&BOB, (CLASS_ID, TOKEN_ID));
        assert!(!LockableNFT::<Runtime>::contains_key((
            &BOB, CLASS_ID, TOKEN_ID
        )));
    })
}
