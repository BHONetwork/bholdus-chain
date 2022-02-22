//! # NFT Marketplace
//! ### Functions
//!
//! - `add_item_to_market` - Add a new item to marketplace
//! - `remove_item_from_market` - Remove item from marketplace. Unsold item.
//! - `add_item_to_blacklist` - Add a item to blacklist. If item belongs blacklist, owner can't
//! list on NFT Marketplace.
//! - `remove_item_from_blacklist` - Remove item from blacklist.
//!

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{Decode, Encode};
use frame_support::{ensure, pallet_prelude::*, traits::Get, BoundedVec, Parameter};

use scale_info::TypeInfo;
use sp_runtime::{
    traits::{
        AtLeast32BitUnsigned, CheckedAdd, CheckedDiv, CheckedSub, MaybeSerializeDeserialize,
        Member, One, Zero,
    },
    ArithmeticError, DispatchError, DispatchResult, FixedPointNumber, FixedPointOperand, FixedU128,
    RuntimeDebug,
};

use bholdus_primitives::Balance;
use num_traits::float::Float;
use sp_std::fmt::Debug;
//use sp_std::vec::Vec;
use sp_std::{convert::TryInto, prelude::*, vec};

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

pub use support_module::*;

pub type TokenIdOf<T> = <T as bholdus_support_nft::Config>::TokenId;
pub type ClassIdOf<T> = <T as bholdus_support_nft::Config>::ClassId;
pub type Price = FixedU128;
pub type RoyaltyRate = (u32, u32);

/// Listing Info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ListingInfo<AccountId> {
    pub seller: AccountId,
    pub buyer: Option<AccountId>,
    pub market_mode: MarketMode,
    pub price: Price,
    pub royalty: RoyaltyRate,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum MarketMode {
    SellNow,
}

#[frame_support::pallet]
pub mod support_module {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + bholdus_support_nft::Config {
        type GetRoyaltyValue: Get<RoyaltyRate>;
    }
    pub type ListingInfoOf<T> = ListingInfo<<T as frame_system::Config>::AccountId>;

    /// Error for NFT Marketplace
    #[pallet::error]
    pub enum Error<T> {
        /// Item belonged to the blacklist.
        IntoBlacklist,
        NoPermission,
    }

    /// Listing NFT on marketplace
    #[pallet::storage]
    #[pallet::getter(fn listing_on_market)]
    pub type ListingOnMarket<T: Config> =
        StorageDoubleMap<_, Twox64Concat, T::ClassId, Twox64Concat, T::TokenId, ListingInfoOf<T>>;

    /// NFT Blacklist
    #[pallet::storage]
    #[pallet::getter(fn blacklist)]
    pub type Blacklist<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, ClassIdOf<T>>,
            NMapKey<Blake2_128Concat, TokenIdOf<T>>,
        ),
        (),
        ValueQuery,
    >;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
    pub fn check_item_status(token: (ClassIdOf<T>, TokenIdOf<T>)) -> (bool, bool) {
        (Self::is_listing(token), Self::is_item_blacklist(token))
    }

    pub fn check_creator_or_owner(
        account: &T::AccountId,
        token: (T::ClassId, T::TokenId),
    ) -> (bool, bool) {
        (
            Self::is_creator(account, token),
            Self::is_owner(account, token),
        )
    }

    pub fn is_creator(account: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> bool {
        bholdus_support_nft::Pallet::<T>::is_creator(account, token)
    }

    pub fn is_owner(account: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> bool {
        bholdus_support_nft::Pallet::<T>::is_owner(account, token)
    }

    pub fn is_listing(token: (ClassIdOf<T>, TokenIdOf<T>)) -> bool {
        ListingOnMarket::<T>::contains_key(token.0, token.1)
    }

    pub fn is_item_blacklist(token: (ClassIdOf<T>, TokenIdOf<T>)) -> bool {
        Blacklist::<T>::contains_key((token.0, token.1))
    }

    /// Add item to marketplace
    pub fn add_item_to_market(
        owner: &T::AccountId,
        creator: &T::AccountId,
        token: (ClassIdOf<T>, TokenIdOf<T>),
        market_mode: MarketMode,
        price: Price,
        royalty: RoyaltyRate,
    ) -> DispatchResult {
        let listing_info = ListingInfo {
            seller: owner.clone(),
            buyer: None,
            market_mode,
            price,
            royalty,
        };
        ListingOnMarket::<T>::insert(token.0, token.1, listing_info);
        Ok(())
    }

    /// Add item to blacklist
    pub fn add_item_to_blacklist(
        manager: &T::AccountId,
        token: (ClassIdOf<T>, TokenIdOf<T>),
    ) -> DispatchResult {
        Blacklist::<T>::insert((token.0, token.1), ());
        Ok(())
    }
}
