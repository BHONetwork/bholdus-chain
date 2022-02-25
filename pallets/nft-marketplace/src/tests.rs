#[cfg(test)]
use super::*;
use frame_support::traits::Currency;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};

use bholdus_primitives::Balance;
use bholdus_support_nft::TokenInfo;
use sp_runtime::{traits::BlakeTwo256, ArithmeticError};
use sp_std::convert::TryInto;

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

#[test]
fn list_item_on_market_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        create_nft();
        let price = Price::saturating_from_integer(10000u128);
        let royalty = (1000u32, 1000u32);

        assert_ok!(NFTMarketplace::list_item_on_market(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
            MarketMode::SellNow,
            price,
            Some(royalty.clone())
        ));

        let market_info = MarketInfo {
            market_mode: MarketMode::SellNow,
            price,
            royalty: royalty.clone(),
        };

        System::assert_last_event(Event::NFTMarketplace(crate::Event::ListedItem {
            owner: ALICE,
            token: (CLASS_ID, TOKEN_ID),
            market_info,
        }));

        assert!(SupportNFTMarketplace::is_listing((CLASS_ID, TOKEN_ID)));
        let listing_info = <bholdus_support_nft_marketplace::Pallet<Runtime>>::listing_on_market(
            CLASS_ID, TOKEN_ID,
        )
        .unwrap();
        assert_eq!(
            listing_info,
            ListingInfo {
                seller: ALICE,
                buyer: None,
                market_mode: MarketMode::SellNow,
                price,
                royalty: royalty.clone()
            }
        );

        let (numerator, denominator) = (listing_info.royalty.0, listing_info.royalty.1);
        let rate = FixedU128::from((numerator, denominator));
        let royalty_reward = listing_info.price * rate;
        let royalty = Price::saturating_from_integer(10000u128);
        assert_eq!(royalty_reward, royalty);
    });
}

#[test]
fn list_item_on_market_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let price = Price::saturating_from_integer(10000u128);
        assert_noop!(
            NFTMarketplace::list_item_on_market(
                Origin::signed(BOB),
                (CLASS_ID, TOKEN_ID),
                MarketMode::SellNow,
                price,
                Some((100u32, 100u32))
            ),
            Error::<Runtime>::NoPermission
        );

        create_nft();

        assert_eq!(
            bholdus_support_nft_marketplace::Pallet::<Runtime>::check_creator_or_owner(
                &BOB,
                (CLASS_ID, TOKEN_ID)
            ),
            (false, false)
        );

        assert_eq!(
            bholdus_support_nft_marketplace::Pallet::<Runtime>::check_creator_or_owner(
                &ALICE,
                (CLASS_ID, TOKEN_ID)
            ),
            (true, true)
        );

        assert_noop!(
            NFTMarketplace::list_item_on_market(
                Origin::signed(BOB),
                (CLASS_ID, TOKEN_ID),
                MarketMode::SellNow,
                price,
                Some((100u32, 100u32))
            ),
            Error::<Runtime>::NoPermission
        );
        assert_ok!(NFTMarketplace::list_item_on_market(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
            MarketMode::SellNow,
            price,
            Some((100u32, 100u32))
        ));

        assert_noop!(
            NFTMarketplace::list_item_on_market(
                Origin::signed(ALICE),
                (CLASS_ID, TOKEN_ID),
                MarketMode::SellNow,
                price,
                Some((100u32, 100u32))
            ),
            Error::<Runtime>::IsListing
        );
    })
}

#[test]
fn cancel_item_list_on_market_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        create_nft();
        let price = Price::saturating_from_integer(10000u128);
        assert_ok!(NFTMarketplace::list_item_on_market(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
            MarketMode::SellNow,
            price.clone(),
            Some((100u32, 100u32))
        ));
        assert_ok!(NFTMarketplace::cancel_item_list_on_market(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
        ));
        System::assert_last_event(Event::NFTMarketplace(crate::Event::CanceledItemListing {
            owner: ALICE,
            token: (CLASS_ID, TOKEN_ID),
        }));
    })
}

#[test]
fn cancel_item_list_on_market_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
    let price = Price::saturating_from_integer(10000u128);

    assert_noop!(
        NFTMarketplace::cancel_item_list_on_market(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID),),
        Error::<Runtime>::NoPermission);

    // Create a new NFT
    create_nft();

    // Case: NoPermission
    // - Current `owner`: ALICE
    // - Requester AccountId: BOB
    assert_noop!(
        NFTMarketplace::cancel_item_list_on_market(Origin::signed(BOB), (CLASS_ID, TOKEN_ID),),
        Error::<Runtime>::NoPermission
    );

    assert_noop!(
        NFTMarketplace::cancel_item_list_on_market(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID),),
        Error::<Runtime>::ItemMustBeListing
    );

    // List item on NFT marketplace
    assert_ok!(NFTMarketplace::list_item_on_market(
        Origin::signed(ALICE),
        (CLASS_ID, TOKEN_ID),
        MarketMode::SellNow,
        price.clone(),
        Some((100u32, 100u32))
    ));

    // Cancel item listing on NFT marketplace
    assert_ok!(NFTMarketplace::cancel_item_list_on_market(
        Origin::signed(ALICE),
        (CLASS_ID, TOKEN_ID),
    ));

    assert_noop!(
        NFTMarketplace::cancel_item_list_on_market(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID),),
        Error::<Runtime>::ItemMustBeListing
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
