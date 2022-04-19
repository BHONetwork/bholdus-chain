#[cfg(test)]
use super::*;
use frame_support::traits::Currency;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};

use bholdus_primitives::Balance;
use bholdus_support_nft::{Error as SupportNFTError, TokenInfo};
use bholdus_support_nft_marketplace::{
    Error as SupportNFTMarketplaceError, ItemListing as SupportNFTMItemListing, NFTState,
};
use sp_runtime::{traits::BlakeTwo256, ArithmeticError};

fn create_nft() {
    assert_ok!(SupportNFT::create_class(&ALICE, ()));
    assert_ok!(SupportNFT::create_group());
    assert_ok!(SupportNFT::mint_to_group(
        &ALICE,
        CLASS_ID,
        GROUP_ID,
        vec![1],
        ()
    ));
}

fn create_nft_with_account(account: &AccountId) {
    assert_ok!(SupportNFT::create_class(account, ()));
    assert_ok!(SupportNFT::create_group());
    assert_ok!(SupportNFT::mint_to_group(
        account,
        CLASS_ID,
        GROUP_ID,
        vec![1],
        ()
    ));
}

fn set_controller() {
    assert_ok!(NFTMarketplace::configure_pallet_management(
        Origin::signed(ALICE),
        ALICE
    ));
}

fn set_service_fee() {
    set_controller();
    assert_ok!(NFTMarketplace::set_marketplace_fee(
        Origin::signed(ALICE),
        (10_000, 10_000),
        BOB,
    ));
}

#[test]
fn create_fixed_price_listing_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        create_nft();
        let price = 10000u128;
        let royalty = (1000u32, 1000u32);

        Timestamp::set_timestamp(100);
        set_service_fee();

        assert_ok!(NFTMarketplace::create_fixed_price_listing(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
            price,
            NFTCurrencyId::Native,
            Some(royalty.clone()),
            EXPIRED_TIME
        ));

        assert!(NFTMarketplace::is_lock(&ALICE, (CLASS_ID, TOKEN_ID)));

        let listing_info = PendingListingInfo {
            currency_id: NFTCurrencyId::Native,
            price,
            royalty: royalty.clone(),
            expired_time: EXPIRED_TIME,
            service_fee: (10_000, 10_000),
        };

        System::assert_last_event(Event::NFTMarketplace(
            crate::Event::NewFixedPriceNFTListing {
                owner: ALICE,
                token: (CLASS_ID, TOKEN_ID),
                listing_info,
            },
        ));

        assert!(SupportNFTMarketplace::is_listing(
            &ALICE,
            (CLASS_ID, TOKEN_ID),
            MarketMode::FixedPrice,
        ));
        assert!(NFTMarketplace::is_listing(
            &ALICE,
            (CLASS_ID, TOKEN_ID),
            MarketMode::FixedPrice
        ));

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
                royalty: royalty.clone(),
                status: NFTState::Pending,
                expired_time: EXPIRED_TIME,
                service_fee: (10_000, 10_000),
            }
        );
    });
}

#[test]
fn create_fixed_price_listing_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let price = 10000u128;

        assert_noop!(
            NFTMarketplace::create_fixed_price_listing(
                Origin::signed(BOB),
                (CLASS_ID, TOKEN_ID),
                price,
                NFTCurrencyId::Native,
                Some((100u32, 100u32)),
                EXPIRED_TIME,
            ),
            Error::<Runtime>::NoPermission
        );

        // Create a new NFT
        create_nft_with_account(&EVE);
        set_controller();
        assert!(NFTMarketplace::is_owner(&EVE, (CLASS_ID, TOKEN_ID)));

        assert_noop!(
            NFTMarketplace::create_fixed_price_listing(
                Origin::signed(BOB),
                (CLASS_ID, TOKEN_ID),
                price,
                NFTCurrencyId::Native,
                Some((100u32, 100u32)),
                EXPIRED_TIME,
            ),
            Error::<Runtime>::NoPermission
        );

        // Ban user
        assert_ok!(NFTMarketplace::ban_user(Origin::signed(ALICE), EVE, vec![]));

        assert_noop!(
            NFTMarketplace::create_fixed_price_listing(
                Origin::signed(EVE),
                (CLASS_ID, TOKEN_ID),
                price,
                NFTCurrencyId::Native,
                Some((100u32, 100u32)),
                EXPIRED_TIME,
            ),
            Error::<Runtime>::UserBanned,
        );

        // Unban user
        assert_ok!(NFTMarketplace::unban_user(Origin::signed(ALICE), EVE));

        // Ban NFT
        assert_ok!(NFTMarketplace::ban(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
            vec![]
        ));

        assert_noop!(
            NFTMarketplace::create_fixed_price_listing(
                Origin::signed(EVE),
                (CLASS_ID, TOKEN_ID),
                price,
                NFTCurrencyId::Native,
                Some((100u32, 100u32)),
                EXPIRED_TIME,
            ),
            Error::<Runtime>::NFTBanned
        );

        // Unban NFT
        assert_ok!(NFTMarketplace::unban(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID)
        ));

        set_service_fee();

        assert_ok!(NFTMarketplace::create_fixed_price_listing(
            Origin::signed(EVE),
            (CLASS_ID, TOKEN_ID),
            price,
            NFTCurrencyId::Native,
            Some((100u32, 100u32)),
            EXPIRED_TIME,
        ));

        assert_noop!(
            SupportNFT::transfer(&EVE, &ALICE, (CLASS_ID, TOKEN_ID)),
            SupportNFTError::<Runtime>::IsLocked
        );

        assert_noop!(
            NFTMarketplace::create_fixed_price_listing(
                Origin::signed(EVE),
                (CLASS_ID, TOKEN_ID),
                price,
                NFTCurrencyId::Native,
                Some((100u32, 100u32)),
                EXPIRED_TIME,
            ),
            Error::<Runtime>::IsListing
        );
    })
}

#[test]
fn approve_listing_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        create_nft();
        let price = 10000u128;
        let royalty = (1000u32, 1000u32);

        Timestamp::set_timestamp(100);
        set_service_fee();

        assert_ok!(NFTMarketplace::create_fixed_price_listing(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
            price,
            NFTCurrencyId::Native,
            Some(royalty.clone()),
            EXPIRED_TIME,
        ));

        assert_eq!(
            FixedPriceListing::<Runtime>::get((ALICE, CLASS_ID, TOKEN_ID))
                .unwrap()
                .status,
            NFTState::Pending
        );

        set_controller();
        assert_ok!(NFTMarketplace::approve_listing(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID)
        ));

        assert!(FixedPriceListing::<Runtime>::contains_key((
            ALICE, CLASS_ID, TOKEN_ID
        )));

        assert_eq!(
            FixedPriceListing::<Runtime>::get((ALICE, CLASS_ID, TOKEN_ID))
                .unwrap()
                .status,
            NFTState::Listing
        );
    })
}

#[test]
fn approve_listing_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        create_nft();
        let price = 10000u128;
        let royalty = (1000u32, 1000u32);

        assert_noop!(
            NFTMarketplace::approve_listing(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)),
            Error::<Runtime>::NotFoundPalletManagementInfo
        );

        set_controller();

        assert_noop!(
            NFTMarketplace::approve_listing(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
            Error::<Runtime>::NoPermission
        );

        assert_noop!(
            NFTMarketplace::approve_listing(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)),
            SupportNFTMarketplaceError::<Runtime>::NotFound
        );

        Timestamp::set_timestamp(100);
        set_service_fee();

        assert_ok!(NFTMarketplace::create_fixed_price_listing(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
            price,
            NFTCurrencyId::Native,
            Some(royalty.clone()),
            EXPIRED_TIME,
        ));
        Timestamp::set_timestamp(30000);

        assert_noop!(
            NFTMarketplace::approve_listing(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)),
            SupportNFTMarketplaceError::<Runtime>::ExpiredListing
        );

        Timestamp::set_timestamp(10000);

        assert_ok!(NFTMarketplace::approve_listing(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID)
        ));
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
        assert_ok!(NFTMarketplace::create_fixed_price_listing(
            Origin::signed(BOB),
            (CLASS_ID, TOKEN_ID),
            PRICE,
            NFTCurrencyId::Native,
            Some(ROYALTY_VALUE),
            EXPIRED_TIME,
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

        assert!(!NFTMarketplace::is_listing(
            &BOB,
            (CLASS_ID, TOKEN_ID),
            MarketMode::FixedPrice
        ));
        assert!(!SupportNFTMItemListing::<Runtime>::contains_key((
            &ALICE, CLASS_ID, TOKEN_ID
        )));

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

        assert_ok!(NFTMarketplace::create_fixed_price_listing(
            Origin::signed(BOB),
            (CLASS_ID, TOKEN_ID),
            PRICE,
            NFTCurrencyId::Native,
            Some(ROYALTY_VALUE),
            EXPIRED_TIME,
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
        assert_ok!(NFTMarketplace::create_fixed_price_listing(
            Origin::signed(BOB),
            (CLASS_ID, TOKEN_ID),
            PRICE,
            NFTCurrencyId::Native,
            Some(ROYALTY_VALUE),
            EXPIRED_TIME,
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

        assert!(!NFTMarketplace::is_listing(
            &BOB,
            (CLASS_ID, TOKEN_ID),
            MarketMode::FixedPrice
        ));
        assert!(!SupportNFTMItemListing::<Runtime>::contains_key((
            &ALICE, CLASS_ID, TOKEN_ID
        )));

        assert_ok!(SupportNFT::transfer(&BOB, &ALICE, (CLASS_ID, TOKEN_ID)));
    })
}

#[test]
fn reject_listing_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        create_nft_with_account(&BOB);
        assert_noop!(
            NFTMarketplace::reject_listing(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID), vec![]),
            Error::<Runtime>::NotFoundPalletManagementInfo
        );

        set_service_fee();
        assert_noop!(
            NFTMarketplace::reject_listing(Origin::signed(BOB), (CLASS_ID, TOKEN_ID), vec![]),
            Error::<Runtime>::NoPermission
        );

        assert_noop!(
            NFTMarketplace::reject_listing(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID), vec![]),
            SupportNFTMarketplaceError::<Runtime>::NotFound
        );

        assert_ok!(NFTMarketplace::create_fixed_price_listing(
            Origin::signed(BOB),
            (CLASS_ID, TOKEN_ID),
            PRICE,
            NFTCurrencyId::Native,
            Some(ROYALTY_VALUE),
            EXPIRED_TIME,
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

#[test]
fn configure_pallet_management_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let controller = ALICE;
        assert_ok!(NFTMarketplace::configure_pallet_management(
            Origin::signed(ALICE),
            controller.clone(),
        ));

        System::assert_last_event(Event::NFTMarketplace(crate::Event::AddedManagementInfo {
            management_info: PalletManagementInfo {
                controller: controller.clone(),
            },
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
            management_info: PalletManagementInfo {
                controller: controller.clone(),
            },
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

        assert_ok!(NFTMarketplace::configure_pallet_management(
            Origin::signed(ALICE),
            ALICE
        ));

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

        assert_ok!(NFTMarketplace::configure_pallet_management(
            Origin::signed(ALICE),
            BOB
        ));

        assert_noop!(
            NFTMarketplace::configure_pallet_management(Origin::signed(ALICE), ALICE),
            Error::<Runtime>::AccountIdMustBeController
        );

        assert_ok!(NFTMarketplace::configure_pallet_management(
            Origin::signed(BOB),
            ALICE
        ));
    })
}

#[test]
fn set_marketplace_fee_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let service_fee = (1000u32, 10_000u32);
        assert_ok!(NFTMarketplace::configure_pallet_management(
            Origin::signed(ALICE),
            ALICE
        ));
        assert_ok!(NFTMarketplace::set_marketplace_fee(
            Origin::signed(ALICE),
            service_fee.clone(),
            BOB,
        ));

        System::assert_last_event(Event::NFTMarketplace(
            crate::Event::ConfiguredMarketplaceFee {
                controller: ALICE,
                marketplace_fee_info: MarketplaceFeeInfo {
                    service_fee: service_fee.clone(),
                    beneficiary: BOB,
                },
            },
        ));

        assert_ok!(NFTMarketplace::set_marketplace_fee(
            Origin::signed(ALICE),
            service_fee.clone(),
            ALICE
        ));

        System::assert_last_event(Event::NFTMarketplace(
            crate::Event::ConfiguredMarketplaceFee {
                controller: ALICE,
                marketplace_fee_info: MarketplaceFeeInfo {
                    service_fee: service_fee.clone(),
                    beneficiary: ALICE,
                },
            },
        ));

        assert_ok!(NFTMarketplace::configure_pallet_management(
            Origin::signed(ALICE),
            BOB
        ));
        assert_ok!(NFTMarketplace::set_marketplace_fee(
            Origin::signed(BOB),
            service_fee.clone(),
            BOB,
        ));
    })
}

#[test]
fn set_marketplace_fee_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let service_fee = (1000u32, 10_000u32);
        assert_noop!(
            NFTMarketplace::set_marketplace_fee(Origin::signed(ALICE), service_fee.clone(), BOB),
            Error::<Runtime>::NotFoundPalletManagementInfo
        );

        assert_ok!(NFTMarketplace::configure_pallet_management(
            Origin::signed(ALICE),
            ALICE
        ));

        assert_noop!(
            NFTMarketplace::set_marketplace_fee(Origin::signed(BOB), service_fee.clone(), BOB),
            Error::<Runtime>::NoPermission
        );

        assert_ok!(NFTMarketplace::set_marketplace_fee(
            Origin::signed(ALICE),
            service_fee.clone(),
            BOB,
        ));
    })
}

#[test]
fn ban_user_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        create_nft();
        let price = 10_000u128;
        set_service_fee();
        assert_ok!(NFTMarketplace::create_fixed_price_listing(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
            price.clone(),
            NFTCurrencyId::Native,
            Some((100u32, 100u32)),
            EXPIRED_TIME,
        ));

        assert!(NFTMarketplace::is_owner(&ALICE, (CLASS_ID, TOKEN_ID)));

        assert!(NFTMarketplace::is_listing(
            &ALICE,
            (CLASS_ID, TOKEN_ID),
            MarketMode::FixedPrice
        ));

        assert_ok!(NFTMarketplace::configure_pallet_management(
            Origin::signed(ALICE),
            ALICE,
        ));

        assert_ok!(NFTMarketplace::ban_user(
            Origin::signed(ALICE),
            ALICE,
            vec![]
        ));
        System::assert_last_event(Event::NFTMarketplace(crate::Event::UserBanned {
            controller: ALICE,
            account: ALICE,
            reason: vec![],
        }));

        assert!(NFTMarketplace::is_banned_user(&ALICE));

        assert!(NFTMarketplace::is_owner(&ALICE, (CLASS_ID, TOKEN_ID)));
        assert!(!NFTMarketplace::is_listing(
            &ALICE,
            (CLASS_ID, TOKEN_ID),
            MarketMode::FixedPrice
        ));
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
            Error::<Runtime>::NotFoundPalletManagementInfo
        );
        assert_ok!(NFTMarketplace::configure_pallet_management(
            Origin::signed(ALICE),
            ALICE
        ));
        assert_noop!(
            NFTMarketplace::ban_user(Origin::signed(EVE), BOB, vec![]),
            Error::<Runtime>::NoPermission
        );

        assert_ok!(NFTMarketplace::ban_user(Origin::signed(ALICE), BOB, vec![]));
        assert!(NFTMarketplace::is_banned_user(&BOB));

        assert_noop!(
            NFTMarketplace::ban_user(Origin::signed(ALICE), BOB, vec![]),
            Error::<Runtime>::UserBanned
        );

        assert_noop!(
            NFTMarketplace::create_fixed_price_listing(
                Origin::signed(BOB),
                (CLASS_ID, TOKEN_ID),
                price.clone(),
                NFTCurrencyId::Native,
                None,
                EXPIRED_TIME
            ),
            Error::<Runtime>::UserBanned
        );
    })
}

#[test]
fn unban_user_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(NFTMarketplace::configure_pallet_management(
            Origin::signed(ALICE),
            ALICE
        ));

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
            Error::<Runtime>::NotFoundPalletManagementInfo
        );

        // Controller
        assert_ok!(NFTMarketplace::configure_pallet_management(
            Origin::signed(ALICE),
            ALICE
        ));

        assert_noop!(
            NFTMarketplace::unban_user(Origin::signed(BOB), BOB),
            Error::<Runtime>::NoPermission
        );

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
        assert_ok!(NFTMarketplace::create_fixed_price_listing(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
            price.clone(),
            NFTCurrencyId::Native,
            Some((100u32, 100u32)),
            EXPIRED_TIME
        ));

        assert!(NFTMarketplace::is_listing(
            &ALICE,
            (CLASS_ID, TOKEN_ID),
            MarketMode::FixedPrice
        ));

        assert!(SupportNFTMItemListing::<Runtime>::contains_key((
            ALICE, CLASS_ID, TOKEN_ID
        )));

        assert_ok!(NFTMarketplace::configure_pallet_management(
            Origin::signed(ALICE),
            ALICE,
        ));

        assert_ok!(NFTMarketplace::ban(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
            vec![],
        ));

        System::assert_last_event(Event::NFTMarketplace(crate::Event::NFTBanned {
            controller: ALICE,
            token: (CLASS_ID, TOKEN_ID),
            reason: vec![],
        }));

        assert!(!NFTMarketplace::is_listing(
            &ALICE,
            (CLASS_ID, TOKEN_ID),
            MarketMode::FixedPrice
        ));

        assert!(!SupportNFTMItemListing::<Runtime>::contains_key((
            ALICE, CLASS_ID, TOKEN_ID
        )));
    })
}

#[test]
fn ban_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        create_nft();
        let price = 10_000u128;
        assert_noop!(
            NFTMarketplace::ban(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID), vec![]),
            Error::<Runtime>::NotFoundPalletManagementInfo
        );

        set_service_fee();

        assert_ok!(NFTMarketplace::create_fixed_price_listing(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
            price.clone(),
            NFTCurrencyId::Native,
            Some((100u32, 100u32)),
            EXPIRED_TIME
        ));

        assert!(NFTMarketplace::is_listing(
            &ALICE,
            (CLASS_ID, TOKEN_ID),
            MarketMode::FixedPrice
        ));

        assert_ok!(NFTMarketplace::configure_pallet_management(
            Origin::signed(ALICE),
            ALICE,
        ));

        assert_noop!(
            NFTMarketplace::ban(Origin::signed(BOB), (CLASS_ID, TOKEN_ID), vec![]),
            Error::<Runtime>::NoPermission
        );

        assert_ok!(NFTMarketplace::ban(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
            vec![]
        ));

        assert!(!NFTMarketplace::is_listing(
            &ALICE,
            (CLASS_ID, TOKEN_ID),
            MarketMode::FixedPrice
        ));

        assert!(NFTBlacklist::<Runtime>::contains_key((CLASS_ID, TOKEN_ID)));

        assert_noop!(
            NFTMarketplace::ban(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID), vec![]),
            Error::<Runtime>::NFTBanned
        );
    })
}

#[test]
fn unban() {
    ExtBuilder::default().build().execute_with(|| {
        create_nft();
        assert_noop!(
            NFTMarketplace::unban(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)),
            Error::<Runtime>::NotFoundPalletManagementInfo
        );

        // Set controller
        assert_ok!(NFTMarketplace::configure_pallet_management(
            Origin::signed(ALICE),
            ALICE,
        ));

        assert_noop!(
            NFTMarketplace::unban(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
            Error::<Runtime>::NoPermission
        );

        assert_ok!(NFTMarketplace::ban(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
            vec![]
        ));

        assert!(NFTBlacklist::<Runtime>::contains_key((CLASS_ID, TOKEN_ID)));

        assert_ok!(NFTMarketplace::unban(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
        ));
        assert!(!NFTBlacklist::<Runtime>::contains_key((CLASS_ID, TOKEN_ID)));

        assert_ok!(NFTMarketplace::ban(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
            vec![]
        ));
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
