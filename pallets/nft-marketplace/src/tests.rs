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
        let royalty_numerator = 1000u32;

        assert_ok!(NFTMarketplace::list_item_on_market(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
            MarketMode::SellNow,
            price,
            Some(royalty_numerator)
        ));

        let market_info = MarketInfo {
            market_mode: MarketMode::SellNow,
            price,
            royalty: (royalty_numerator, ROYALTY_DENOMINATOR),
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
                royalty: (royalty_numerator, ROYALTY_DENOMINATOR)
            }
        );

        let (numerator, denominator) = (listing_info.royalty.0, listing_info.royalty.1);
        let rate = FixedU128::from((numerator, denominator));
        let royalty_reward = listing_info.price * rate;
        let royalty = Price::saturating_from_integer(10000u128 / 10u128);
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
                Some(100u32)
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
                Some(100u32)
            ),
            Error::<Runtime>::NoPermission
        );
        assert_ok!(NFTMarketplace::list_item_on_market(
            Origin::signed(ALICE),
            (CLASS_ID, TOKEN_ID),
            MarketMode::SellNow,
            price,
            Some(100u32)
        ));

        assert_noop!(
            NFTMarketplace::list_item_on_market(
                Origin::signed(ALICE),
                (CLASS_ID, TOKEN_ID),
                MarketMode::SellNow,
                price,
                Some(100u32)
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
            Some(100u32)
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
        Some(100u32)
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
