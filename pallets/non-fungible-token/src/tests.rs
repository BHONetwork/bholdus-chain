#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok, traits::Currency};
use mock::{Event, *};

use bholdus_support_nft::TokenInfo;
use common_primitives::Balance;
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
		assert_eq!(
			bholdus_support_nft::Pallet::<Runtime>::classes(0).unwrap().data,
			ClassData { attributes: test_attr(1) }
		)
	});
}

#[test]
fn mint_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata = vec![1];
		let bounded_metadata = metadata.clone().try_into().unwrap();
		let class_attr = test_attr(2u8);

		let token_attr = test_attr(1u8);
		let data = ClassData { attributes: class_attr.clone() };

		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), class_attr.clone()));

		System::assert_last_event(Event::NFTModule(crate::Event::CreatedClass {
			owner: ALICE,
			class_id: CLASS_ID,
			data,
		}));

		// ClassID = 0;
		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			BOB,
			CLASS_ID,
			metadata.clone(),
			token_attr.clone(),
			3
		));

		let token_info = TokenInfo {
			metadata: bounded_metadata,
			owner: BOB,
			creator: BOB,
			data: TokenData { attributes: token_attr },
		};

		System::assert_last_event(Event::NFTModule(crate::Event::MintedToken {
			group_id: GROUP_ID,
			class_id: CLASS_ID,
			token_id: TOKEN_ID,
			token_info,
			quantity: 3,
		}));
	});
}

#[test]
fn mint_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata = vec![1];
		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), Default::default(),));

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

		bholdus_support_nft::NextTokenId::<Runtime>::mutate(|id| {
			*id = <Runtime as bholdus_support_nft::Config>::TokenId::max_value()
		});
		/*assert_noop!(
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
		*/
	});
}

#[test]
fn mint_should_fail_without_mintable() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata = vec![1];
		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), Default::default(),));
	});
}

#[test]
fn transfer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata = vec![1, 2, 3];
		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), Default::default(),));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			BOB,
			CLASS_ID,
			metadata.clone(),
			Default::default(),
			1
		));

		assert_ok!(NFTModule::transfer(Origin::signed(BOB), ALICE, (CLASS_ID, TOKEN_ID)));

		System::assert_last_event(Event::NFTModule(crate::Event::TransferredToken {
			from: BOB,
			to: ALICE,
			token: (CLASS_ID, TOKEN_ID),
		}));

		assert_ok!(NFTModule::transfer(Origin::signed(ALICE), DAVE, (CLASS_ID, TOKEN_ID)));

		System::assert_last_event(Event::NFTModule(crate::Event::TransferredToken {
			from: ALICE,
			to: DAVE,
			token: (CLASS_ID, TOKEN_ID),
		}));
	});
}

#[test]
fn transfer_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata = vec![1];
		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), Default::default(),));
		assert_ok!(NFTModule::mint(
			Origin::signed(
                        ALICE, //class_id_account()
                    ),
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
	});

	ExtBuilder::default().build().execute_with(|| {
		let metadata = vec![1];
		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), Default::default(),));
	})
}

#[test]
fn burn_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata = vec![1];
		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), Default::default(),));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			BOB,
			CLASS_ID,
			metadata.clone(),
			Default::default(),
			1
		));

		assert_ok!(NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));

		System::assert_last_event(Event::NFTModule(crate::Event::BurnedToken {
			owner: BOB,
			token: (CLASS_ID, TOKEN_ID),
		}));
	})
}

#[test]
fn burn_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata = vec![1];
		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), Default::default(),));
		assert_ok!(NFTModule::mint(
			Origin::signed(
				//class_id_account()
				ALICE
			),
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
		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), Default::default(),));

		assert_ok!(NFTModule::destroy_class(Origin::signed(ALICE), CLASS_ID));
		System::assert_last_event(Event::NFTModule(crate::Event::DestroyedClass {
			owner: ALICE,
			class_id: CLASS_ID,
		}));
	})
}
#[test]
fn destroy_class_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let metadata = vec![1];
		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), Default::default(),));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE,),
			BOB,
			CLASS_ID,
			metadata,
			Default::default(),
			1
		));

		assert_noop!(
			NFTModule::destroy_class(Origin::signed(ALICE), CLASS_ID_NOT_EXIST),
			Error::<Runtime>::ClassIdNotFound
		);

		assert_noop!(
			NFTModule::destroy_class(Origin::signed(BOB), CLASS_ID),
			Error::<Runtime>::NoPermission
		);

		assert_noop!(
			NFTModule::destroy_class(Origin::signed(ALICE), CLASS_ID),
			Error::<Runtime>::CannotDestroyClass,
		);
	});
}
