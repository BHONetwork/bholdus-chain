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

use bholdus_support::MultiCurrency;
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
use sp_std::fmt::Debug;
use sp_std::{convert::TryInto, prelude::*, vec};

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

pub use support_module::*;

pub type Price = Balance;
pub type Denominator = u32;
pub type Numerator = u32;
pub type RoyaltyRate = (Numerator, Denominator);

/// Listing Info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct FixedPriceListingInfo<AccountId, CurrencyId> {
    pub owner: AccountId,
    pub price: Price,
    pub currency_id: CurrencyId,
    pub royalty: RoyaltyRate,
    pub status: NFTState,
}

/// Trading Information
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct TradingInfo<AccountId> {
    pub seller: AccountId,
    pub buyer: AccountId,
    pub market_mode: MarketMode,
    pub price: Price,
    pub royalty: RoyaltyRate,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum MarketMode {
    FixedPrice,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum NFTCurrencyId<BHC20TokenId> {
    Native,
    Token(BHC20TokenId),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum NFTState {
    Pending,
    Listing,
}

#[frame_support::pallet]
pub mod support_module {
    use super::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config + bholdus_support_nft::Config + bholdus_tokens::Config
    {
        type GetRoyaltyValue: Get<RoyaltyRate>;
    }
    pub type FixedPriceListingInfoOf<T> = FixedPriceListingInfo<
        <T as frame_system::Config>::AccountId,
        NFTCurrencyId<<T as bholdus_tokens::Config>::AssetId>,
    >;
    pub type TradingInfoOf<T> = TradingInfo<<T as frame_system::Config>::AccountId>;
    pub type TokenIdOf<T> = <T as bholdus_support_nft::Config>::TokenId;
    pub type ClassIdOf<T> = <T as bholdus_support_nft::Config>::ClassId;
    pub type BHC20TokenIdOf<T> = <T as bholdus_tokens::Config>::AssetId;

    /// Error for NFT Marketplace
    #[pallet::error]
    pub enum Error<T> {
        /// Item belonged to the blacklist.
        NFTBanned,
        NoPermission,
        IsListing,
    }

    /// Listing NFT on marketplace
    #[pallet::storage]
    #[pallet::getter(fn fixed_price_listing)]
    pub type FixedPriceListing<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>, // owner
            NMapKey<Blake2_128Concat, T::ClassId>,
            NMapKey<Blake2_128Concat, T::TokenId>,
        ),
        FixedPriceListingInfoOf<T>,
    >;

    /// NFT listed on market check by owner, class ID, token ID
    #[pallet::storage]
    #[pallet::getter(fn token_listing_by_owner)]
    pub type TokenListingByOwner<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>, // owner
            NMapKey<Blake2_128Concat, T::ClassId>,
            NMapKey<Blake2_128Concat, T::TokenId>,
        ),
        (),
        ValueQuery,
    >;

    /// History Trading Information
    #[pallet::storage]
    #[pallet::getter(fn trading)]
    pub type Trading<T: Config> =
        StorageDoubleMap<_, Twox64Concat, T::ClassId, Twox64Concat, T::TokenId, TradingInfoOf<T>>;

    /// User Blacklist
    #[pallet::storage]
    #[pallet::getter(fn user_blacklist)]
    pub type UserBlacklist<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<u8>>;

    /// NFT Blacklist
    #[pallet::storage]
    #[pallet::getter(fn blacklist)]
    pub type Blacklist<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, ClassIdOf<T>>,
            NMapKey<Blake2_128Concat, TokenIdOf<T>>,
        ),
        Vec<u8>,
    >;

    /// Royalty
    #[pallet::storage]
    #[pallet::getter(fn set_royalty_value)]
    pub type RoyaltyValue<T: Config> = StorageValue<_, (u32, u32), ValueQuery>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
    /// Create fixed price listing
    pub fn create_fixed_price_listing(
        account: &T::AccountId,
        token: (ClassIdOf<T>, TokenIdOf<T>),
        price: Price,
        currency_id: NFTCurrencyId<BHC20TokenIdOf<T>>,
        royalty: RoyaltyRate,
    ) -> DispatchResult {
        let listing_info = FixedPriceListingInfo {
            owner: account.clone(),
            price,
            currency_id,
            royalty,
            status: NFTState::Pending,
        };

        FixedPriceListing::<T>::insert((account, token.0, token.1), listing_info);
        Ok(())
    }

    /// Add item to blacklist
    pub fn add_item_to_blacklist(
        token: (ClassIdOf<T>, TokenIdOf<T>),
        reason: Vec<u8>,
    ) -> DispatchResult {
        Blacklist::<T>::insert(token, reason);
        let owner = Self::owner(token);
        Self::remove_item_from_market(&owner, token)?;
        Ok(())
    }

    /// Add user to blacklist
    pub fn add_user_to_blacklist(account: &T::AccountId, reason: Vec<u8>) -> DispatchResult {
        UserBlacklist::<T>::insert(account, reason);
        // remove all listing NFTs on market
        Self::remove_tokens_by_owner(account);
        Ok(())
    }
}

impl<T: Config> Pallet<T> {
    pub fn owner(token: (ClassIdOf<T>, TokenIdOf<T>)) -> T::AccountId {
        bholdus_support_nft::Pallet::<T>::owner(token)
    }

    pub fn get_loyalty_value(value: Option<(Numerator, Denominator)>) -> (Numerator, Denominator) {
        if RoyaltyValue::<T>::exists() {
            RoyaltyValue::<T>::get()
        } else {
            if let Some(v) = value {
                v
            } else {
                (0, 10_000)
            }
        }
    }

    /// Remove item NFT from market place
    pub fn remove_item_from_market(
        owner: &T::AccountId,
        token: (ClassIdOf<T>, TokenIdOf<T>),
    ) -> DispatchResult {
        FixedPriceListing::<T>::remove((owner, token.0, token.1));
        // TODO
        // AuctionListing::<T>::remove((owner, token.0, token.1));
        Ok(())
    }

    pub fn remove_tokens_from_market(owner: &[T::AccountId]) {
        for removed in owner {
            TokenListingByOwner::<T>::remove_prefix((removed,), None);
        }
    }

    pub fn remove_tokens_by_owner(owner: &T::AccountId) {
        // remove fixed price token
        FixedPriceListing::<T>::remove_prefix((owner,), None);
        // remove EnglishAuctionListingInfo
    }
}

impl<T: Config> Pallet<T> {
    pub fn is_lock(account: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> bool {
        bholdus_support_nft::Pallet::<T>::is_lock(account, token)
    }

    pub fn is_owner(account: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> bool {
        bholdus_support_nft::Pallet::<T>::is_owner(account, token)
    }

    pub fn is_listing(
        owner: &T::AccountId,
        token: (ClassIdOf<T>, TokenIdOf<T>),
        type_of_listing: MarketMode,
    ) -> bool {
        match type_of_listing {
            MarketMode::FixedPrice => {
                FixedPriceListing::<T>::contains_key((owner, token.0, token.1))
            }
            _ => false,
        }
    }

    pub fn is_user_blacklist(user: &T::AccountId) -> bool {
        UserBlacklist::<T>::contains_key(user)
    }

    pub fn is_item_blacklist(token: (ClassIdOf<T>, TokenIdOf<T>)) -> bool {
        Blacklist::<T>::contains_key((token.0, token.1))
    }

    pub fn is_bhc20(currency_id: NFTCurrencyId<BHC20TokenIdOf<T>>) -> bool {
        match currency_id {
            NFTCurrencyId::Token(token_id) => true,
            _ => false,
        }
    }

    pub fn is_native(currency_id: NFTCurrencyId<BHC20TokenIdOf<T>>) -> bool {
        match currency_id {
            NFTCurrencyId::Native => true,
            _ => false,
        }
    }
}
