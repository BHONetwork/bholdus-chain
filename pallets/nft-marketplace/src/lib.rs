#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::unused_unit)]
#![allow(clippy::unpper_case_acronyms)]

use enumflags2::BitFlags;
use frame_support::{
    log,
    pallet_prelude::*,
    require_transactional,
    traits::{
        Currency,
        ExistenceRequirement::{AllowDeath, KeepAlive},
        NamedReservableCurrency,
    },
    transactional, PalletId,
};

use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
    traits::{AccountIdConversion, Hash, Saturating, StaticLookup, Zero},
    DispatchResult, FixedPointNumber, FixedPointOperand, FixedU128, RuntimeDebug,
};

use sp_std::if_std;

// #[cfg(feature = "runtime-benchmarks")]
// pub mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use bholdus_primitives::Balance;
use bholdus_support_nft_marketplace::{
    ListingInfo, ListingOnMarket, MarketMode, Numerator, Price, RoyaltyRate,
};

pub use support_module::*;

pub type TokenIdOf<T> = <T as bholdus_support_nft::Config>::TokenId;
pub type ClassIdOf<T> = <T as bholdus_support_nft::Config>::ClassId;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct MarketInfo {
    pub market_mode: MarketMode,
    pub price: Price,
    pub royalty: RoyaltyRate,
}

#[frame_support::pallet]
pub mod support_module {
    use super::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config + bholdus_support_nft::Config + bholdus_support_nft_marketplace::Config
    {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::error]
    pub enum Error<T> {
        IsListing,
        ItemMustBeListing,
        NoPermission,
        IsBlacklist,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Add item on marketplace
        ListedItem {
            owner: T::AccountId,
            token: (ClassIdOf<T>, TokenIdOf<T>),
            market_info: MarketInfo,
        },
        /// Cancel item list on marketplace
        CanceledItemListing {
            owner: T::AccountId,
            token: (ClassIdOf<T>, TokenIdOf<T>),
        },
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        #[transactional]
        pub fn list_item_on_market(
            origin: OriginFor<T>,
            token: (ClassIdOf<T>, TokenIdOf<T>),
            market_mode: MarketMode,
            price: Price,
            royalty: Option<Numerator>,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            // Check permission

            let (is_creator, is_owner) =
                bholdus_support_nft_marketplace::Pallet::<T>::check_creator_or_owner(&owner, token);
            let is_passed_role = Self::is_passed_role(is_creator, is_owner);
            ensure!(is_passed_role, Error::<T>::NoPermission);

            // Check status

            let (is_listing, is_item_blacklist) =
                bholdus_support_nft_marketplace::Pallet::<T>::check_item_status(token);
            ensure!(!is_listing, Error::<T>::IsListing);
            ensure!(!is_item_blacklist, Error::<T>::IsBlacklist);

            // Mapping royalty

            let royalty_bounded = Self::mapping_royalty(is_creator, is_owner, royalty);
            bholdus_support_nft_marketplace::Pallet::<T>::add_item_to_market(
                &owner,
                &owner,
                token,
                market_mode.clone(),
                price.clone(),
                royalty_bounded.clone(),
            );
            let market_info = MarketInfo {
                market_mode: market_mode.clone(),
                price: price.clone(),
                royalty: royalty_bounded.clone(),
            };
            Self::deposit_event(Event::ListedItem {
                owner,
                token,
                market_info,
            });
            Ok(())
        }

        #[pallet::weight(0)]
        #[transactional]
        pub fn cancel_item_list_on_market(
            origin: OriginFor<T>,
            token: (ClassIdOf<T>, TokenIdOf<T>),
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;
            let is_owner = bholdus_support_nft_marketplace::Pallet::<T>::is_owner(&owner, token);
            ensure!(is_owner, Error::<T>::NoPermission);
            ListingOnMarket::<T>::take(token.0, token.1).ok_or(Error::<T>::ItemMustBeListing)?;
            Self::deposit_event(Event::CanceledItemListing { owner, token });
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn is_passed_role(is_creator: bool, is_owner: bool) -> bool {
        if is_creator == false && is_owner == false {
            false
        } else {
            true
        }
    }
    pub fn mapping_royalty(
        is_creator: bool,
        is_owner: bool,
        royalty: Option<Numerator>,
    ) -> RoyaltyRate {
        if is_creator {
            if royalty.is_some() {
                (royalty.unwrap(), 10_000u32)
            } else {
                (10_000, 10_000)
            }
        } else {
            (10_000, 10_000)
        }
    }
}
