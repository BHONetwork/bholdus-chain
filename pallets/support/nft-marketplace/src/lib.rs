//! # NFT Marketplace
//! ### Functions
//!
//! - `add_item_to_market` - Add a new item to marketplace
//! - `remove_item_from_market` - Remove item from marketplace. Unsold item.
//! - `add_item_to_blacklist` - Add a item to blacklist. If item belongs blacklist, owner can't
//! list on NFT Marketplace.
//! - `remove_item_from_blacklist` - Remove item from blacklist.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{Decode, Encode};
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{Get, Time},
};

use scale_info::TypeInfo;
use sp_runtime::{self, DispatchResult, RuntimeDebug};

use common_primitives::Balance;
use sp_std::prelude::*;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

pub use support_module::*;

pub type Price = Balance;
pub type Denominator = u32;
pub type Numerator = u32;
pub type RoyaltyRate = (Numerator, Denominator);

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct PalletManagementInfo<AccountId> {
	pub controller: AccountId,
}

/// MarketPlace Fee Information
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MarketplaceFeeInfo<AccountId> {
	pub service_fee: (Numerator, Denominator),
	pub beneficiary: AccountId,
}

/// Listing Info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct FixedPriceListingInfo<AccountId, CurrencyId, Moment> {
	pub owner: AccountId,
	pub price: Price,
	pub currency_id: CurrencyId,
	pub royalty: RoyaltyRate,
	pub status: NFTState,
	pub expired_time: Moment,
	pub service_fee: (Numerator, Denominator),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ItemListingInfo<AccountId> {
	pub owner: AccountId,
	pub mode: MarketMode,
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
		type Time: Time;
	}

	pub type PalletManagementInfoOf<T> =
		PalletManagementInfo<<T as frame_system::Config>::AccountId>;
	pub type MarketplaceFeeInfoOf<T> = MarketplaceFeeInfo<<T as frame_system::Config>::AccountId>;

	pub type FixedPriceListingInfoOf<T> = FixedPriceListingInfo<
		<T as frame_system::Config>::AccountId,
		NFTCurrencyId<<T as bholdus_tokens::Config>::AssetId>,
		MomentOf<T>,
	>;
	pub type TradingInfoOf<T> = TradingInfo<<T as frame_system::Config>::AccountId>;
	pub type ItemListingOf<T> = ItemListingInfo<<T as frame_system::Config>::AccountId>;
	pub type TokenIdOf<T> = <T as bholdus_support_nft::Config>::TokenId;
	pub type ClassIdOf<T> = <T as bholdus_support_nft::Config>::ClassId;
	pub type BHC20TokenIdOf<T> = <T as bholdus_tokens::Config>::AssetId;
	pub type MomentOf<T> = <<T as Config>::Time as Time>::Moment;

	/// Error for NFT Marketplace
	#[pallet::error]
	pub enum Error<T> {
		/// Item belonged to the blacklist.
		NFTBanned,
		NoPermission,
		IsListing,
		UnknownMode,
		NotFound,
		IsApproved,
		ExpiredListing,
	}

	#[pallet::storage]
	#[pallet::getter(fn pallet_management)]
	pub type PalletManagement<T: Config> = StorageValue<_, PalletManagementInfoOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn marketplace_fee)]
	pub type MarketplaceFee<T: Config> = StorageValue<_, MarketplaceFeeInfoOf<T>>;

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

	#[pallet::storage]
	#[pallet::getter(fn item_listing)]
	pub type ItemListing<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::AccountId>, // owner
			NMapKey<Blake2_128Concat, T::ClassId>,
			NMapKey<Blake2_128Concat, T::TokenId>,
		),
		ItemListingOf<T>,
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
		(NMapKey<Blake2_128Concat, ClassIdOf<T>>, NMapKey<Blake2_128Concat, TokenIdOf<T>>),
		Vec<u8>,
	>;

	/// Royalty
	#[pallet::storage]
	#[pallet::getter(fn set_royalty_value)]
	pub type RoyaltyValue<T: Config> = StorageValue<_, (u32, u32), ValueQuery>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	/// Create fixed price listing
	pub fn create_fixed_price_listing(
		owner: &T::AccountId,
		token: (ClassIdOf<T>, TokenIdOf<T>),
		price: Price,
		currency_id: NFTCurrencyId<BHC20TokenIdOf<T>>,
		royalty: RoyaltyRate,
		expired_time: MomentOf<T>,
		service_fee: (Numerator, Denominator),
	) -> DispatchResult {
		let now = T::Time::now();
		ensure!(expired_time >= now, Error::<T>::ExpiredListing);

		let listing_info = FixedPriceListingInfo {
			owner: owner.clone(),
			price,
			currency_id,
			royalty,
			status: NFTState::Pending,
			expired_time,
			service_fee,
		};

		FixedPriceListing::<T>::insert((owner, token.0, token.1), listing_info);
		ItemListing::<T>::insert(
			(owner, token.0, token.1),
			ItemListingInfo { owner: owner.clone(), mode: MarketMode::FixedPrice },
		);
		Self::lock_item(owner, token);
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
	/// Remove all listing NFTs on market
	pub fn add_user_to_blacklist(account: &T::AccountId, reason: Vec<u8>) -> DispatchResult {
		UserBlacklist::<T>::insert(account, reason);
		Self::remove_by_owner(account);
		Ok(())
	}

	/// Approve Listing
	pub fn approve_item_listing(token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
		let now = T::Time::now();
		let owner = Self::owner(token);
		let item_info =
			ItemListing::<T>::get((&owner, token.0, token.1)).ok_or(Error::<T>::NotFound)?;
		match item_info.mode {
			MarketMode::FixedPrice => FixedPriceListing::<T>::try_mutate_exists(
				(item_info.owner, token.0, token.1),
				|listing_info| -> DispatchResult {
					let info = listing_info.as_mut().ok_or(Error::<T>::NotFound)?;
					/*if_std!(println!(
						"expired_time: ExpiredTime: {:?}, Now: {:?}",
						info.expired_time, now
					));
					*/
					ensure!(info.expired_time > now, Error::<T>::ExpiredListing);
					ensure!(info.status == NFTState::Pending, Error::<T>::IsApproved);
					info.status = NFTState::Listing;
					Ok(())
				},
			),
			_ => return Ok(()),
		}
	}

	/// Cancel product listing
	pub fn delist(owner: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
		let item_info =
			ItemListing::<T>::take((owner, token.0, token.1)).ok_or(Error::<T>::NotFound)?;
		match item_info.mode {
			MarketMode::FixedPrice => {
				FixedPriceListing::<T>::take((owner, token.0, token.1))
					.ok_or(Error::<T>::NotFound)?;
				Self::unlock_item(&owner, token);
				Ok(())
			},
			_ => return Ok(()),
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn owner(token: (ClassIdOf<T>, TokenIdOf<T>)) -> T::AccountId {
		bholdus_support_nft::Pallet::<T>::owner(token)
	}

	pub fn lock_item(owner: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) {
		bholdus_support_nft::Pallet::<T>::lock(owner, token)
	}

	pub fn unlock_item(owner: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) {
		bholdus_support_nft::Pallet::<T>::unlock(owner, token)
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
		if ItemListing::<T>::contains_key((owner, token.0, token.1)) {
			let item_info =
				ItemListing::<T>::take((owner, token.0, token.1)).ok_or(Error::<T>::UnknownMode)?;
			match item_info.mode {
				MarketMode::FixedPrice => {
					FixedPriceListing::<T>::remove((owner, token.0, token.1));
					Ok(())
				},
				_ =>
				// TODO
				// AuctionListing::<T>::remove((owner, token.0, token.1));
					Ok(()),
			}
		} else {
			Ok(())
		}
	}

	pub fn remove_by_owner(owner: &T::AccountId) {
		ItemListing::<T>::remove_prefix((owner,), None);
		FixedPriceListing::<T>::remove_prefix((owner,), None);
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
			MarketMode::FixedPrice =>
				FixedPriceListing::<T>::contains_key((owner, token.0, token.1)),
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
			NFTCurrencyId::Token(_token_id) => true,
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
