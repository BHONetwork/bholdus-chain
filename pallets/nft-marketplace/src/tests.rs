#[cfg(test)]
use super::*;
use frame_support::{assert_noop, assert_ok, traits::Currency};
use mock::{Event, *};
use sp_std::collections::btree_map::BTreeMap;

use bholdus_support_nft::{Error as SupportNFTError, TokenInfo};
use bholdus_support_nft_marketplace::{
	Error as SupportNFTMarketplaceError, ItemListing as SupportNFTMItemListing, ManagerRole,
	MemberRole, NFTState,
};
use common_primitives::Balance;
use sp_runtime::{traits::BlakeTwo256, ArithmeticError};

use bholdus_nft::Attributes;

fn test_attr(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 10], vec![x, x + 1, x + 2]);
	attr.insert(vec![x + 1], vec![11]);
	attr
}

fn create_nft() {
	let metadata = vec![1];
	let token_attr = test_attr(1u8);

	assert_ok!(NFT::create_class(Origin::signed(ALICE), test_attr(1)));
	assert_ok!(SupportNFT::create_group());
	assert_ok!(NFT::mint(Origin::signed(ALICE), ALICE, CLASS_ID, metadata, token_attr, 3));
}

fn create_nft_with_account(account: &AccountId) {
	let metadata = vec![1];
	let token_attr = test_attr(1u8);

	assert_ok!(NFT::create_class(Origin::signed(ALICE), test_attr(1)));
	assert_ok!(SupportNFT::create_group());
	assert_ok!(NFT::mint(
		Origin::signed(ALICE),
		account.clone(),
		CLASS_ID,
		metadata,
		token_attr,
		3
	));
}

fn create_token(admin: AccountId, amount: Balance) {
	Balances::make_free_balance_be(&admin, 10);
	Balances::make_free_balance_be(&DAVE, 10);
	assert_ok!(Tokens::create_and_mint(
		Origin::signed(DAVE),
		admin.clone(),
		vec![],
		vec![],
		18,
		admin.clone(),
		amount,
		10u128,
	));
	assert_eq!(Currencies::total_balance(ASSET_ID, &admin), 10000u128);
}

/*fn set_controller() {
	assert_ok!(NFTMarketplace::configure_pallet_management(Origin::signed(ALICE), ALICE));
}
*/

fn grant_admin_role(account: AccountId) {
	assert_ok!(NFTMarketplace::grant_role(
		Origin::root(),
		RoleType::Manager(ManagerRole::Admin),
		account
	));
}

fn revoke_admin_role(account: AccountId) {
	assert_ok!(NFTMarketplace::revoke_role(
		Origin::root(),
		RoleType::Manager(ManagerRole::Admin),
		account
	));
}

fn set_service_fee() {
	grant_admin_role(ALICE);
	assert_ok!(NFTMarketplace::set_marketplace_fee(Origin::signed(ALICE), (1000, 10000), ALICE,));
}

#[test]
fn create_fixed_price_listing_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		create_nft();
		let price = 10000u128;

		Timestamp::set_timestamp(100);
		set_service_fee();

		let info = FixedPriceSetting {
			price,
			currency_id: NFTCurrencyId::Native,
			expired_time: EXPIRED_TIME,
			royalty: Some(ROYALTY_VALUE),
		};

		assert_ok!(NFTMarketplace::create_fixed_price_listing(
			Origin::signed(ALICE),
			(CLASS_ID, TOKEN_ID),
			info
		));

		assert!(NFTMarketplace::is_lock(&ALICE, (CLASS_ID, TOKEN_ID)));

		/*
		let listing_info = PendingListingInfo {
				currency_id: NFTCurrencyId::Native,
				price,
				royalty: ROYALTY_VALUE,
				expired_time: EXPIRED_TIME,
				service_fee: SERVICE_FEE,
			};

			System::assert_last_event(Event::NFTMarketplace(crate::Event::NewFixedPriceNFTListing {
				owner: ALICE,
				token: (CLASS_ID, TOKEN_ID),
				listing_info,
			}));

			assert!(SupportNFTMarketplace::is_listing(
				&ALICE,
				(CLASS_ID, TOKEN_ID),
				MarketMode::FixedPrice,
			));
			assert!(NFTMarketplace::is_listing(&ALICE, (CLASS_ID, TOKEN_ID), MarketMode::FixedPrice));

			let listing_info = <bholdus_support_nft_marketplace::Pallet<Runtime>>::fixed_price_listing(
				(ALICE, CLASS_ID, TOKEN_ID),
			)
			.unwrap();
			assert_eq!(
				listing_info,
				FixedPriceListingInfo {
					owner: ALICE,
					price,
					currency_id: NFTCurrencyId::Native,
					royalty: ROYALTY_VALUE,
					status: NFTState::Pending,
					expired_time: EXPIRED_TIME,
					service_fee: SERVICE_FEE,
				}

			);*/
	});
}

#[test]
fn create_fixed_price_listing_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let price = 10000u128;
		set_service_fee();
		let info = FixedPriceSetting {
			price: PRICE,
			currency_id: NFTCurrencyId::Native,
			expired_time: EXPIRED_TIME,
			royalty: Some(ROYALTY_VALUE),
		};

		assert_noop!(
			NFTMarketplace::create_fixed_price_listing(
				Origin::signed(BOB),
				(CLASS_ID, TOKEN_ID),
				info.clone()
			),
			Error::<Runtime>::NoPermission
		);

		// Create a new NFT
		create_nft_with_account(&EVE);
		assert!(NFTMarketplace::is_owner(&EVE, (CLASS_ID, TOKEN_ID)));
		assert_noop!(
			NFTMarketplace::create_fixed_price_listing(
				Origin::signed(BOB),
				(CLASS_ID, TOKEN_ID),
				info.clone(),
			),
			Error::<Runtime>::NoPermission
		);

		// Ban user
		assert_ok!(NFTMarketplace::ban_user(Origin::signed(ALICE), EVE, vec![]));
		assert_noop!(
			NFTMarketplace::create_fixed_price_listing(
				Origin::signed(EVE),
				(CLASS_ID, TOKEN_ID),
				info.clone(),
			),
			Error::<Runtime>::UserBanned,
		);

		// Unban user
		assert_ok!(NFTMarketplace::unban_user(Origin::signed(ALICE), EVE));

		// Ban NFT
		assert_ok!(NFTMarketplace::ban(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID), vec![]));
		assert_noop!(
			NFTMarketplace::create_fixed_price_listing(
				Origin::signed(EVE),
				(CLASS_ID, TOKEN_ID),
				info.clone(),
			),
			Error::<Runtime>::NFTBanned
		);

		// Unban NFT
		assert_ok!(NFTMarketplace::unban(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)));
		assert_ok!(NFTMarketplace::create_fixed_price_listing(
			Origin::signed(EVE),
			(CLASS_ID, TOKEN_ID),
			info.clone(),
		));

		assert_noop!(
			SupportNFT::transfer(&EVE, &ALICE, (CLASS_ID, TOKEN_ID)),
			SupportNFTError::<Runtime>::IsLocked
		);
		assert_noop!(
			NFTMarketplace::create_fixed_price_listing(
				Origin::signed(EVE),
				(CLASS_ID, TOKEN_ID),
				info.clone(),
			),
			Error::<Runtime>::IsListing
		);
	})
}

#[test]
fn approve_listing_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		set_service_fee();
		create_nft();
		let price = 10000u128;
		Timestamp::set_timestamp(100);

		let info = FixedPriceSetting {
			price: PRICE,
			currency_id: NFTCurrencyId::Native,
			expired_time: EXPIRED_TIME,
			royalty: Some(ROYALTY_VALUE),
		};

		assert_ok!(NFTMarketplace::create_fixed_price_listing(
			Origin::signed(ALICE),
			(CLASS_ID, TOKEN_ID),
			info,
		));

		assert_eq!(
			FixedPriceListing::<Runtime>::get((ALICE, CLASS_ID, TOKEN_ID)).unwrap().status,
			NFTState::Pending
		);
		assert_ok!(NFTMarketplace::approve_listing(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)));
		assert!(FixedPriceListing::<Runtime>::contains_key((ALICE, CLASS_ID, TOKEN_ID)));

		assert_eq!(
			FixedPriceListing::<Runtime>::get((ALICE, CLASS_ID, TOKEN_ID)).unwrap().status,
			NFTState::Listing
		);
	})
}

#[test]
fn approve_listing_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		create_nft();
		let price = 10000u128;

		assert_noop!(
			NFTMarketplace::approve_listing(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::MissingPermission
		);
		Timestamp::set_timestamp(100);
		set_service_fee();

		let info = FixedPriceSetting {
			price: PRICE,
			currency_id: NFTCurrencyId::Native,
			expired_time: EXPIRED_TIME,
			royalty: Some(ROYALTY_VALUE),
		};

		assert_ok!(NFTMarketplace::create_fixed_price_listing(
			Origin::signed(ALICE),
			(CLASS_ID, TOKEN_ID),
			info,
		));
		Timestamp::set_timestamp(30000);

		assert_noop!(
			NFTMarketplace::approve_listing(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)),
			SupportNFTMarketplaceError::<Runtime>::ExpiredListing
		);

		Timestamp::set_timestamp(10000);

		assert_ok!(NFTMarketplace::approve_listing(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)));
		assert_noop!(
			NFTMarketplace::approve_listing(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)),
			SupportNFTMarketplaceError::<Runtime>::IsApproved
		);
	})
}

#[test]
fn cancel_listing_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		create_nft_with_account(&BOB);
		set_service_fee();
		let info = FixedPriceSetting {
			price: PRICE,
			currency_id: NFTCurrencyId::Native,
			expired_time: EXPIRED_TIME,
			royalty: Some(ROYALTY_VALUE),
		};

		assert_ok!(NFTMarketplace::create_fixed_price_listing(
			Origin::signed(BOB),
			(CLASS_ID, TOKEN_ID),
			info,
		));
		assert_noop!(
			SupportNFT::transfer(&BOB, &ALICE, (CLASS_ID, TOKEN_ID)),
			SupportNFTError::<Runtime>::IsLocked
		);
		assert_ok!(NFTMarketplace::cancel_listing(
			Origin::signed(BOB),
			(CLASS_ID, TOKEN_ID),
			vec![]
		));

		System::assert_last_event(Event::NFTMarketplace(crate::Event::CancelledListing {
			account: BOB,
			token: (CLASS_ID, TOKEN_ID),
			reason: vec![],
		}));

		assert!(!NFTMarketplace::is_listing(&BOB, (CLASS_ID, TOKEN_ID), MarketMode::FixedPrice));
		assert!(!SupportNFTMItemListing::<Runtime>::contains_key((&ALICE, CLASS_ID, TOKEN_ID)));

		assert_ok!(SupportNFT::transfer(&BOB, &ALICE, (CLASS_ID, TOKEN_ID)));
	})
}

#[test]
fn cancel_listing_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		create_nft_with_account(&BOB);
		set_service_fee();
		assert_noop!(
			NFTMarketplace::cancel_listing(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID), vec![]),
			Error::<Runtime>::NoPermission
		);

		assert_noop!(
			NFTMarketplace::cancel_listing(Origin::signed(BOB), (CLASS_ID, TOKEN_ID), vec![]),
			SupportNFTMarketplaceError::<Runtime>::NotFound
		);

		let info = FixedPriceSetting {
			price: PRICE,
			currency_id: NFTCurrencyId::Native,
			expired_time: EXPIRED_TIME,
			royalty: Some(ROYALTY_VALUE),
		};

		assert_ok!(NFTMarketplace::create_fixed_price_listing(
			Origin::signed(BOB),
			(CLASS_ID, TOKEN_ID),
			info,
		));

		assert_ok!(NFTMarketplace::cancel_listing(
			Origin::signed(BOB),
			(CLASS_ID, TOKEN_ID),
			vec![]
		));
		assert_noop!(
			NFTMarketplace::cancel_listing(Origin::signed(BOB), (CLASS_ID, TOKEN_ID), vec![]),
			SupportNFTMarketplaceError::<Runtime>::NotFound
		);
	})
}

#[test]
fn reject_listing_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		create_nft_with_account(&BOB);
		set_service_fee();

		let info = FixedPriceSetting {
			price: PRICE,
			currency_id: NFTCurrencyId::Native,
			expired_time: EXPIRED_TIME,
			royalty: Some(ROYALTY_VALUE),
		};

		assert_ok!(NFTMarketplace::create_fixed_price_listing(
			Origin::signed(BOB),
			(CLASS_ID, TOKEN_ID),
			info,
		));
		assert_noop!(
			SupportNFT::transfer(&BOB, &ALICE, (CLASS_ID, TOKEN_ID)),
			SupportNFTError::<Runtime>::IsLocked
		);
		assert_ok!(NFTMarketplace::reject_listing(
			Origin::signed(ALICE),
			(CLASS_ID, TOKEN_ID),
			vec![]
		));

		System::assert_last_event(Event::NFTMarketplace(crate::Event::CancelledListing {
			account: ALICE,
			token: (CLASS_ID, TOKEN_ID),
			reason: vec![],
		}));

		assert!(!NFTMarketplace::is_listing(&BOB, (CLASS_ID, TOKEN_ID), MarketMode::FixedPrice));
		assert!(!SupportNFTMItemListing::<Runtime>::contains_key((&ALICE, CLASS_ID, TOKEN_ID)));

		assert_ok!(SupportNFT::transfer(&BOB, &ALICE, (CLASS_ID, TOKEN_ID)));
	})
}

#[test]
fn reject_listing_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		create_nft_with_account(&BOB);
		assert_noop!(
			NFTMarketplace::reject_listing(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID), vec![]),
			Error::<Runtime>::MissingPermission
		);

		set_service_fee();
		assert_noop!(
			NFTMarketplace::reject_listing(Origin::signed(BOB), (CLASS_ID, TOKEN_ID), vec![]),
			Error::<Runtime>::MissingPermission
		);

		let info = FixedPriceSetting {
			price: PRICE,
			currency_id: NFTCurrencyId::Native,
			expired_time: EXPIRED_TIME,
			royalty: Some(ROYALTY_VALUE),
		};

		assert_ok!(NFTMarketplace::create_fixed_price_listing(
			Origin::signed(BOB),
			(CLASS_ID, TOKEN_ID),
			info,
		));
		assert_ok!(NFTMarketplace::reject_listing(
			Origin::signed(ALICE),
			(CLASS_ID, TOKEN_ID),
			vec![],
		));

		assert_noop!(
			NFTMarketplace::reject_listing(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID), vec![]),
			SupportNFTMarketplaceError::<Runtime>::NotFound
		);
	})
}

/*#[test]
fn configure_pallet_management_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let controller = ALICE;
		assert_ok!(NFTMarketplace::configure_pallet_management(
			Origin::signed(ALICE),
			controller.clone(),
		));

		System::assert_last_event(Event::NFTMarketplace(crate::Event::AddedManagementInfo {
			management_info: PalletManagementInfo { controller: controller.clone() },
		}));

		match PalletManagement::<Runtime>::get() {
			None => assert!(!PalletManagement::<Runtime>::exists()),
			Some(info) => {
				assert!(info.controller == controller.clone())
			}
		};

		assert!(PalletManagement::<Runtime>::exists());

		let controller = BOB;

		assert_ok!(NFTMarketplace::configure_pallet_management(
			Origin::signed(ALICE),
			controller.clone()
		));

		System::assert_last_event(Event::NFTMarketplace(crate::Event::UpdatedManagementInfo {
			management_info: PalletManagementInfo { controller: controller.clone() },
		}));

		match PalletManagement::<Runtime>::get() {
			None => assert!(!PalletManagement::<Runtime>::exists()),
			Some(info) => {
				assert!(info.controller == controller.clone())
			}
		};
	})
}

#[test]
fn configure_pallet_management_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(PalletManagement::<Runtime>::exists(), false);

		match PalletManagement::<Runtime>::get() {
			None => assert!(!PalletManagement::<Runtime>::exists()),
			Some(info) => {
				assert!(info.controller == ALICE,)
			}
		};

		assert_ok!(NFTMarketplace::configure_pallet_management(Origin::signed(ALICE), ALICE));

		match PalletManagement::<Runtime>::get() {
			None => assert!(!PalletManagement::<Runtime>::exists()),
			Some(info) => {
				assert!(info.controller == ALICE);
				assert_eq!(info.controller == BOB, false)
			}
		};
		assert_eq!(PalletManagement::<Runtime>::exists(), true);

		assert_noop!(
			NFTMarketplace::configure_pallet_management(Origin::signed(BOB), ALICE),
			Error::<Runtime>::AccountIdMustBeController
		);

		assert_ok!(NFTMarketplace::configure_pallet_management(Origin::signed(ALICE), BOB));

		assert_noop!(
			NFTMarketplace::configure_pallet_management(Origin::signed(ALICE), ALICE),
			Error::<Runtime>::AccountIdMustBeController
		);

		assert_ok!(NFTMarketplace::configure_pallet_management(Origin::signed(BOB), ALICE));
	})
}
*/

#[test]
fn set_marketplace_fee_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		grant_admin_role(ALICE);
		assert_ok!(NFTMarketplace::set_marketplace_fee(Origin::signed(ALICE), SERVICE_FEE, BOB,));

		System::assert_last_event(Event::NFTMarketplace(crate::Event::ConfiguredMarketplaceFee {
			controller: ALICE,
			marketplace_fee_info: MarketplaceFeeInfo { service_fee: SERVICE_FEE, beneficiary: BOB },
		}));

		assert_ok!(NFTMarketplace::set_marketplace_fee(Origin::signed(ALICE), SERVICE_FEE, ALICE));

		System::assert_last_event(Event::NFTMarketplace(crate::Event::ConfiguredMarketplaceFee {
			controller: ALICE,
			marketplace_fee_info: MarketplaceFeeInfo {
				service_fee: SERVICE_FEE,
				beneficiary: ALICE,
			},
		}));

		grant_admin_role(BOB);
		assert_ok!(NFTMarketplace::set_marketplace_fee(Origin::signed(BOB), SERVICE_FEE, BOB,));
	})
}

#[test]
fn set_marketplace_fee_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			NFTMarketplace::set_marketplace_fee(Origin::signed(ALICE), SERVICE_FEE, BOB),
			Error::<Runtime>::MissingPermission
		);

		assert_noop!(
			NFTMarketplace::set_marketplace_fee(Origin::signed(BOB), SERVICE_FEE, BOB),
			Error::<Runtime>::MissingPermission
		);
	})
}

#[test]
fn ban_user_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		create_nft();
		let price = 10_000u128;
		set_service_fee();

		let info = FixedPriceSetting {
			price,
			currency_id: NFTCurrencyId::Native,
			expired_time: EXPIRED_TIME,
			royalty: Some(ROYALTY_VALUE),
		};

		assert_ok!(NFTMarketplace::create_fixed_price_listing(
			Origin::signed(ALICE),
			(CLASS_ID, TOKEN_ID),
			info,
		));

		assert!(NFTMarketplace::is_owner(&ALICE, (CLASS_ID, TOKEN_ID)));

		assert!(NFTMarketplace::is_listing(&ALICE, (CLASS_ID, TOKEN_ID), MarketMode::FixedPrice));
		assert_ok!(NFTMarketplace::ban_user(Origin::signed(ALICE), ALICE, vec![]));

		System::assert_last_event(Event::NFTMarketplace(crate::Event::UserBanned {
			controller: ALICE,
			account: ALICE,
			reason: vec![],
		}));

		assert!(NFTMarketplace::is_banned_user(&ALICE));

		assert!(NFTMarketplace::is_owner(&ALICE, (CLASS_ID, TOKEN_ID)));
		assert!(!NFTMarketplace::is_listing(&ALICE, (CLASS_ID, TOKEN_ID), MarketMode::FixedPrice));
	})
}

#[test]
fn ban_user_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let price = 10_000u128;
		create_nft_with_account(&BOB);
		assert!(NFTMarketplace::is_owner(&BOB, (CLASS_ID, TOKEN_ID)));

		assert_noop!(
			NFTMarketplace::ban_user(Origin::signed(ALICE), BOB, vec![]),
			Error::<Runtime>::MissingPermission
		);
		// assert_ok!(NFTMarketplace::configure_pallet_management(Origin::signed(ALICE), ALICE));
		grant_admin_role(ALICE);
		assert_ok!(NFTMarketplace::ban_user(Origin::signed(ALICE), BOB, vec![]));
		assert!(NFTMarketplace::is_banned_user(&BOB));

		assert_noop!(
			NFTMarketplace::ban_user(Origin::signed(ALICE), BOB, vec![]),
			Error::<Runtime>::UserBanned
		);

		let info = FixedPriceSetting {
			price,
			currency_id: NFTCurrencyId::Native,
			expired_time: EXPIRED_TIME,
			royalty: Some(ROYALTY_VALUE),
		};

		assert_noop!(
			NFTMarketplace::create_fixed_price_listing(
				Origin::signed(BOB),
				(CLASS_ID, TOKEN_ID),
				info,
			),
			Error::<Runtime>::UserBanned
		);
	})
}

#[test]
fn unban_user_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		// assert_ok!(NFTMarketplace::configure_pallet_management(Origin::signed(ALICE), ALICE));
		grant_admin_role(ALICE);
		assert_ok!(NFTMarketplace::ban_user(Origin::signed(ALICE), BOB, vec![]));
		assert_ok!(NFTMarketplace::unban_user(Origin::signed(ALICE), BOB));

		System::assert_last_event(Event::NFTMarketplace(crate::Event::UserUnbanned {
			controller: ALICE,
			account: BOB,
		}));
	})
}

#[test]
fn unban_user_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			NFTMarketplace::unban_user(Origin::signed(ALICE), BOB),
			Error::<Runtime>::MissingPermission
		);
		grant_admin_role(ALICE);
		assert_noop!(
			NFTMarketplace::unban_user(Origin::signed(ALICE), BOB),
			Error::<Runtime>::NotFoundUserInBlacklist
		);

		assert_ok!(NFTMarketplace::ban_user(Origin::signed(ALICE), BOB, vec![]));

		assert_ok!(NFTMarketplace::unban_user(Origin::signed(ALICE), BOB));

		assert_noop!(
			NFTMarketplace::unban_user(Origin::signed(ALICE), BOB),
			Error::<Runtime>::NotFoundUserInBlacklist
		);
	})
}

#[test]
fn ban_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		create_nft();
		let price = 10_000u128;
		set_service_fee();

		let info = FixedPriceSetting {
			price,
			currency_id: NFTCurrencyId::Native,
			expired_time: EXPIRED_TIME,
			royalty: Some(ROYALTY_VALUE),
		};

		assert_ok!(NFTMarketplace::create_fixed_price_listing(
			Origin::signed(ALICE),
			(CLASS_ID, TOKEN_ID),
			info,
		));

		assert!(NFTMarketplace::is_listing(&ALICE, (CLASS_ID, TOKEN_ID), MarketMode::FixedPrice));

		assert!(SupportNFTMItemListing::<Runtime>::contains_key((ALICE, CLASS_ID, TOKEN_ID)));
		assert_ok!(NFTMarketplace::ban(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID), vec![],));

		System::assert_last_event(Event::NFTMarketplace(crate::Event::NFTBanned {
			controller: ALICE,
			token: (CLASS_ID, TOKEN_ID),
			reason: vec![],
		}));

		assert!(!NFTMarketplace::is_listing(&ALICE, (CLASS_ID, TOKEN_ID), MarketMode::FixedPrice));

		assert!(!SupportNFTMItemListing::<Runtime>::contains_key((ALICE, CLASS_ID, TOKEN_ID)));
	})
}

#[test]
fn ban_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		create_nft();
		let price = 10_000u128;
		assert_noop!(
			NFTMarketplace::ban(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID), vec![]),
			Error::<Runtime>::MissingPermission
		);

		set_service_fee();

		let info = FixedPriceSetting {
			price: 100u128,
			currency_id: NFTCurrencyId::Native,
			expired_time: EXPIRED_TIME,
			royalty: Some(ROYALTY_VALUE),
		};

		assert_ok!(NFTMarketplace::create_fixed_price_listing(
			Origin::signed(ALICE),
			(CLASS_ID, TOKEN_ID),
			info,
		));
		assert!(NFTMarketplace::is_listing(&ALICE, (CLASS_ID, TOKEN_ID), MarketMode::FixedPrice));

		assert_noop!(
			NFTMarketplace::ban(Origin::signed(BOB), (CLASS_ID, TOKEN_ID), vec![]),
			Error::<Runtime>::MissingPermission
		);

		assert_ok!(NFTMarketplace::ban(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID), vec![]));

		assert!(!NFTMarketplace::is_listing(&ALICE, (CLASS_ID, TOKEN_ID), MarketMode::FixedPrice));

		assert!(NFTBlacklist::<Runtime>::contains_key((CLASS_ID, TOKEN_ID)));

		assert_noop!(
			NFTMarketplace::ban(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID), vec![]),
			Error::<Runtime>::NFTBanned
		);
	})
}

#[test]
fn unban_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		create_nft();
		grant_admin_role(ALICE);
		assert_ok!(NFTMarketplace::ban(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID), vec![]));
		assert!(NFTBlacklist::<Runtime>::contains_key((CLASS_ID, TOKEN_ID)));

		assert_ok!(NFTMarketplace::unban(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID),));
		assert!(!NFTBlacklist::<Runtime>::contains_key((CLASS_ID, TOKEN_ID)));

		assert_ok!(NFTMarketplace::ban(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID), vec![]));
		assert!(NFTBlacklist::<Runtime>::contains_key((CLASS_ID, TOKEN_ID)));
	})
}

#[test]
fn set_lock_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		create_nft();
		SupportNFTMarketplace::lock_item(&ALICE, (CLASS_ID, TOKEN_ID));
		assert!(NFTMarketplace::is_lock(&ALICE, (CLASS_ID, TOKEN_ID)));
		assert_noop!(
			SupportNFT::transfer(&ALICE, &BOB, (CLASS_ID, TOKEN_ID)),
			SupportNFTError::<Runtime>::IsLocked
		);
	})
}

#[test]
fn buy_with_native_token_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = EVE;
		let royalty_recipient = EVE;
		let buyer = BOB;
		let beneficiary = ALICE;

		// Create a NFT
		create_nft_with_account(&EVE);
		Timestamp::set_timestamp(100);
		set_service_fee();

		let info = FixedPriceSetting {
			price: 100u128,
			currency_id: NFTCurrencyId::Native,
			expired_time: EXPIRED_TIME,
			royalty: Some((100, 10_0000)),
		};

		assert_ok!(NFTMarketplace::create_fixed_price_listing(
			Origin::signed(EVE),
			(CLASS_ID, TOKEN_ID),
			info,
		));
		assert!(NFTMarketplace::is_owner(&EVE, (CLASS_ID, TOKEN_ID)));

		assert_ok!(NFTMarketplace::approve_listing(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)));
		assert_eq!(Balances::free_balance(BOB), 0u128);
		assert_eq!(Balances::free_balance(EVE), 0u128);
		Balances::make_free_balance_be(&BOB, 101);
		Balances::make_free_balance_be(&ALICE, 0);
		assert_ok!(NFTMarketplace::buy_now(Origin::signed(BOB), (CLASS_ID, TOKEN_ID),));
		assert!(NFTMarketplace::is_owner(&BOB, (CLASS_ID, TOKEN_ID)));
		// Buyer
		assert_eq!(Balances::free_balance(buyer), 1u128);
		// Royalty Recipient
		assert_eq!(Balances::free_balance(royalty_recipient), 90u128);
		// Beneficiary
		assert_eq!(Balances::free_balance(beneficiary), 10u128);
		// Recipient
		assert_eq!(Balances::free_balance(EVE), 90u128);
	})
}

#[test]
fn buy_with_native_token_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		// Create a NFT
		create_nft_with_account(&EVE);
		Timestamp::set_timestamp(100);
		set_service_fee();

		let info = FixedPriceSetting {
			price: 1000u128,
			currency_id: NFTCurrencyId::Native,
			expired_time: EXPIRED_TIME,
			royalty: None,
		};

		assert_ok!(NFTMarketplace::create_fixed_price_listing(
			Origin::signed(EVE),
			(CLASS_ID, TOKEN_ID),
			info,
		));
		assert_ok!(NFTMarketplace::approve_listing(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)));
		Balances::make_free_balance_be(&BOB, 0);

		assert_noop!(
			NFTMarketplace::buy_now(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::InsufficientBalance
		);
		Balances::make_free_balance_be(&BOB, 100);
		assert_noop!(
			NFTMarketplace::buy_now(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::InsufficientBalance
		);
	})
}

#[test]
fn buy_with_token_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = EVE;
		let royalty_recipient = EVE;
		let buyer = BOB;
		let beneficiary = ALICE;
		// Create a NFT
		create_nft_with_account(&EVE);
		Timestamp::set_timestamp(100);
		set_service_fee();

		let info = FixedPriceSetting {
			price: 1000u128,
			currency_id: NFTCurrencyId::Token(ASSET_ID),
			expired_time: EXPIRED_TIME,
			royalty: Some((500, 10_000)),
		};

		assert_ok!(NFTMarketplace::create_fixed_price_listing(
			Origin::signed(EVE),
			(CLASS_ID, TOKEN_ID),
			info,
		));
		//
		// royalty_rate = 0.05
		// royalty_amount = 0.05*1000 = 50;
		assert_eq!(SupportNFTMarketplace::calc_amount(1000u128, (500, 10000)), 50u128);
		assert_eq!(SupportNFTMarketplace::calc_amount(1000u128, (1000, 10000)), 100);
		assert!(NFTMarketplace::is_owner(&EVE, (CLASS_ID, TOKEN_ID)));
		Balances::make_free_balance_be(&EVE, 10);

		// Admin approve
		assert_ok!(NFTMarketplace::approve_listing(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)));
		assert!(NFTMarketplace::is_lock(&EVE, (CLASS_ID, TOKEN_ID)));
		assert!(NFTMarketplace::is_listing(&EVE, (CLASS_ID, TOKEN_ID), MarketMode::FixedPrice));
		assert!(SupportNFTMItemListing::<Runtime>::contains_key((EVE, CLASS_ID, TOKEN_ID)));
		create_token(BOB, 10000u128);
		assert!(NFTMarketplace::is_owner(&EVE, (CLASS_ID, TOKEN_ID)));
		assert_eq!(Currencies::total_balance(ASSET_ID, &BOB), 10000u128);
		// Buy NFT
		assert_ok!(NFTMarketplace::buy_now(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));
		assert!(!NFTMarketplace::is_lock(&EVE, (CLASS_ID, TOKEN_ID)));
		assert!(!NFTMarketplace::is_listing(&EVE, (CLASS_ID, TOKEN_ID), MarketMode::FixedPrice));
		assert!(!SupportNFTMItemListing::<Runtime>::contains_key((EVE, CLASS_ID, TOKEN_ID)));
		assert!(NFTMarketplace::is_owner(&BOB, (CLASS_ID, TOKEN_ID)));
		assert_eq!(Currencies::total_balance(ASSET_ID, &BOB), 9000u128);
		assert_eq!(Currencies::total_balance(ASSET_ID, &beneficiary), 100u128);
		assert_eq!(Currencies::total_balance(ASSET_ID, &owner), 850u128 + 50u128);
	})
}

#[test]
fn buy_with_token_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			NFTMarketplace::buy_now(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::CannotBuyNFT
		);
		// Create NFT
		create_nft_with_account(&EVE);

		assert_noop!(
			NFTMarketplace::buy_now(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::NotFound
		);

		// Listing NFT
		Timestamp::set_timestamp(100);
		set_service_fee();

		let info = FixedPriceSetting {
			price: 1000u128,
			currency_id: NFTCurrencyId::Token(ASSET_ID),
			expired_time: EXPIRED_TIME,
			royalty: None,
		};

		assert_ok!(NFTMarketplace::create_fixed_price_listing(
			Origin::signed(EVE),
			(CLASS_ID, TOKEN_ID),
			info
		));

		assert!(NFTMarketplace::is_lock(&EVE, (CLASS_ID, TOKEN_ID)));

		assert_noop!(
			NFTMarketplace::buy_now(Origin::signed(EVE), (CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::CannotBuyNFT
		);

		assert_noop!(
			NFTMarketplace::buy_now(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::NotFound
		);

		// Approve a single listing
		assert_ok!(NFTMarketplace::approve_listing(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)));

		Timestamp::set_timestamp(30000);
		assert_noop!(
			NFTMarketplace::buy_now(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::ExpiredListing
		);
		Timestamp::set_timestamp(100);

		Balances::make_free_balance_be(&EVE, 10);
		create_token(BOB, 10000u128);
		assert_ok!(NFTMarketplace::buy_now(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));

		assert_noop!(
			NFTMarketplace::buy_now(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::NotFound
		);
	})
}

#[test]
fn calc_amount() {
	ExtBuilder::default().build().execute_with(|| {
		let amount = SupportNFTMarketplace::calc_amount(1000u128, (100, 10_000u32));
	})
}

#[test]
fn grant_role_admin_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTMarketplace::grant_role(
			Origin::root(),
			RoleType::Manager(ManagerRole::Admin),
			ALICE
		));

		System::assert_last_event(Event::NFTMarketplace(crate::Event::RoleGranted {
			account: ALICE,
			role: RoleType::Manager(ManagerRole::Admin),
		}));
	})
}

#[test]
fn grant_role_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTMarketplace::grant_role(
			Origin::root(),
			RoleType::Manager(ManagerRole::Admin),
			ALICE
		));

		assert_ok!(NFTMarketplace::grant_role(
			Origin::signed(ALICE),
			RoleType::Member(MemberRole::Mod),
			BOB,
		));

		System::assert_last_event(Event::NFTMarketplace(crate::Event::RoleGranted {
			account: BOB,
			role: RoleType::Member(MemberRole::Mod),
		}));
	})
}

#[test]
fn grant_role_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTMarketplace::grant_role(
			Origin::root(),
			RoleType::Manager(ManagerRole::Admin),
			ALICE,
		));

		assert_noop!(
			NFTMarketplace::grant_role(
				Origin::root(),
				RoleType::Manager(ManagerRole::Admin),
				ALICE
			),
			SupportNFTMarketplaceError::<Runtime>::RoleRedundant
		);
	})
}

#[test]
fn revoke_role_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTMarketplace::grant_role(
			Origin::root(),
			RoleType::Manager(ManagerRole::Admin),
			ALICE
		));

		assert_ok!(NFTMarketplace::revoke_role(
			Origin::root(),
			RoleType::Manager(ManagerRole::Admin),
			ALICE
		));

		System::assert_last_event(Event::NFTMarketplace(crate::Event::RoleRevoked {
			account: ALICE,
			role: RoleType::Manager(ManagerRole::Admin),
		}));
	});
}

// #[test]
fn revoke_role_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTMarketplace::revoke_role(
			Origin::signed(ALICE),
			RoleType::Member(MemberRole::Copywriter),
			BOB
		));
	});
}
