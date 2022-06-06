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
use frame_system::pallet_prelude::*;

use scale_info::TypeInfo;
use sp_core::U256;
use sp_runtime::{self, traits::Zero, DispatchResult, FixedU128, RuntimeDebug};

use bholdus_support::MultiCurrency;
use common_primitives::{Balance, TokenId as CurrencyId};
use sp_std::prelude::*;

pub mod access_control;
pub mod auction;
pub mod fixed_price;
pub mod traits;
pub use access_control::*;
pub use auction::*;
pub use fixed_price::*;
pub use traits::*;

pub type ExchangeRate = FixedU128;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

pub use support_module::*;

pub type Price = Balance;
pub type Denominator = u32;
pub type Numerator = u32;
pub type RoyaltyRate = (Numerator, Denominator);

/*#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PaymentInfo<AccountId> {
	pub actual_price: Balance,
	pub royalty_amount: Balance,
	pub fee_amount: Balance,
	pub fee_recipient: AccountId,
	pub royalty_recipient: AccountId,
	pub royalty: RoyaltyRate,
	pub service_fee: (Numerator, Denominator),
}
*/

/*/// Trading Information
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct FixedPriceOrderInfo<AccountId, CurrencyId, Moment> {
	pub seller: AccountId,
	pub buyer: AccountId,
	pub time: Moment,
	pub currency_id: CurrencyId,
	pub actual_price: Balance,
	pub fee_amount: Balance,
	pub royalty_amount: Balance,
	pub fee_recipient: AccountId,
	pub royalty_recipient: AccountId,
}
*/

#[frame_support::pallet]
pub mod support_module {
	use super::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_balances::Config<Balance = Balance>
		+ bholdus_support_nft::Config // + bholdus_tokens::Config
	{
		type GetRoyaltyValue: Get<RoyaltyRate>;
		type Time: Time;
		type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;
		//type NativeCurrency: ReservableCurrency<Self::AccountId>;
	}

	pub type PalletManagementInfoOf<T> =
		PalletManagementInfo<<T as frame_system::Config>::AccountId>;
	pub type MarketplaceFeeInfoOf<T> = MarketplaceFeeInfo<<T as frame_system::Config>::AccountId>;
	pub type RoyaltyInfoOf<T> = RoyaltyInfo<<T as frame_system::Config>::AccountId>;

	pub type FixedPriceListingInfoOf<T> = FixedPriceListingInfo<
		<T as frame_system::Config>::AccountId,
		//NFTCurrencyId<<T as bholdus_tokens::Config>::AssetId>,
		NFTCurrencyId<CurrencyId>,
		MomentOf<T>,
	>;

	pub type TimeAuctionListingInfoOf<T> = TimeAuctionListingInfo<
		<T as frame_system::Config>::AccountId,
		NFTCurrencyId<CurrencyId>,
		MomentOf<T>,
	>;
	/*pub type FixedPriceOrderInfoOf<T> = FixedPriceOrderInfo<
		  <T as frame_system::Config>::AccountId,
		  NFTCurrencyId<CurrencyId>,
		  MomentOf<T>,
	  >;
	*/
	pub type ItemListingOf<T> = ItemListingInfo<<T as frame_system::Config>::AccountId>;
	pub type TokenIdOf<T> = <T as bholdus_support_nft::Config>::TokenId;
	pub type ClassIdOf<T> = <T as bholdus_support_nft::Config>::ClassId;

	// pub type BHC20TokenIdOf<T> = <T as bholdus_tokens::Config>::AssetId;
	pub type BHC20TokenId = CurrencyId;
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
		InsufficientBalance,
		CannotBuyNFT,
		RoleRedundant,
		MissingRole,
		MissingPermission,
		InvalidRole,
		AuctionAlreadyConcluded,
	}

	#[pallet::storage]
	#[pallet::getter(fn pallet_management)]
	pub type PalletManagement<T: Config> = StorageValue<_, PalletManagementInfoOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn access_control)]
	pub type AccessControl<T: Config> =
		StorageDoubleMap<_, Twox64Concat, RoleType, Twox64Concat, T::AccountId, Vec<Permission>>;

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

	/// Auction Sale
	#[pallet::storage]
	#[pallet::getter(fn time_auction)]
	pub type TimeAuction<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::AccountId>,
			NMapKey<Blake2_128Concat, T::ClassId>,
			NMapKey<Blake2_128Concat, T::TokenId>,
		),
		TimeAuctionListingInfoOf<T>,
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
	#[pallet::getter(fn fixed_price_order)]
	pub type FixedPriceOrder<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::AccountId>, // owner
			NMapKey<Blake2_128Concat, T::ClassId>,
			NMapKey<Blake2_128Concat, T::TokenId>,
		),
		FixedPriceListingInfoOf<T>,
		//FixedPriceOrderInfoOf<T>,
	>;

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
	#[pallet::getter(fn set_royalty)]
	// pub type Royalty<T: Config> = StorageValue<_, (u32, u32), ValueQuery>;
	pub type Royalty<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::ClassId, Twox64Concat, T::TokenId, RoyaltyInfoOf<T>>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	/// Create fixed price listing
	pub fn new_fixed_price(
		token: (ClassIdOf<T>, TokenIdOf<T>),
		info: &FixedPriceListingInfoOf<T>,
	) -> DispatchResult {
		FixedPriceListing::<T>::insert((&info.owner, token.0, token.1), info);
		ItemListing::<T>::insert(
			(&info.owner, token.0, token.1),
			ItemListingInfo { owner: info.owner.clone(), mode: MarketMode::FixedPrice },
		);
		Self::lock_item(&info.owner, token);
		Ok(())
	}

	/// Auction Create
	pub fn new_time_auction(
		token: (ClassIdOf<T>, TokenIdOf<T>),
		info: &TimeAuctionListingInfoOf<T>,
	) -> DispatchResult {
		<TimeAuction<T>>::insert((&info.owner, token.0, token.1), info);
		ItemListing::<T>::insert(
			(&info.owner, token.0, token.1),
			ItemListingInfo {
				owner: info.owner.clone(),
				mode: MarketMode::Auction(AuctionType::English),
			},
		);
		Self::lock_item(&info.owner, token);
		Ok(())
	}

	/// Buy NFT
	pub fn buy_now(
		buyer: T::AccountId,
		token: (ClassIdOf<T>, TokenIdOf<T>),
		info: FixedPriceListingInfoOf<T>,
	) -> DispatchResult {
		let owner = &info.owner.clone();
		Self::unlock_item(&owner, token);
		Self::transfer(&owner, &buyer, token)?;

		<FixedPriceOrder<T>>::insert((&owner, token.0, token.1), info);

		// Clear listing information
		FixedPriceListing::<T>::remove((&owner, token.0, token.1));
		ItemListing::<T>::remove((&owner, token.0, token.1));

		Ok(())
	}

	/// Add item to blacklist
	pub fn add_item_to_blacklist(
		token: (ClassIdOf<T>, TokenIdOf<T>),
		reason: Vec<u8>,
	) -> DispatchResult {
		Blacklist::<T>::insert(token, reason);
		let owner = Self::owner(token);
		Self::maybe_do_delist(&owner, token)?;
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
	pub fn approve(token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
		let owner = Self::owner(token);
		let item_info =
			ItemListing::<T>::get((&owner, token.0, token.1)).ok_or(Error::<T>::NotFound)?;
		match item_info.mode {
			MarketMode::FixedPrice => Self::approve_fixed_price(&owner, token),
			MarketMode::Auction(_) => Self::approve_time_auction(&owner, token),
		}
	}

	/// Cancel product listing
	pub fn delist(owner: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
		let item_info =
			ItemListing::<T>::take((owner, token.0, token.1)).ok_or(Error::<T>::NotFound)?;
		match item_info.mode {
			MarketMode::FixedPrice => FixedPriceListing::<T>::remove((owner, token.0, token.1)),
			MarketMode::Auction(_) => TimeAuction::<T>::remove((owner, token.0, token.1)),
		};
		Self::unlock_item(&owner, token);
		Ok(())
	}
}

impl<T: Config> Pallet<T> {
	pub fn lock_item(owner: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) {
		bholdus_support_nft::Pallet::<T>::lock(owner, token)
	}

	pub fn unlock_item(owner: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) {
		bholdus_support_nft::Pallet::<T>::unlock(owner, token)
	}

	/// Remove item NFT from market place
	pub fn maybe_do_delist(
		owner: &T::AccountId,
		token: (ClassIdOf<T>, TokenIdOf<T>),
	) -> DispatchResult {
		if let Some(item_info) = ItemListing::<T>::take((owner, token.0, token.1)) {
			match item_info.mode {
				MarketMode::FixedPrice => FixedPriceListing::<T>::remove((owner, token.0, token.1)),
				MarketMode::Auction(_) => TimeAuction::<T>::remove((owner, token.0, token.1)),
			}
		};

		Self::unlock_item(&owner, token);
		Ok(())
	}

	pub fn transfer(
		from: &T::AccountId,
		to: &T::AccountId,
		token: (ClassIdOf<T>, TokenIdOf<T>),
	) -> DispatchResult {
		bholdus_support_nft::Pallet::<T>::transfer(from, to, token)
	}

	pub fn remove_by_owner(owner: &T::AccountId) {
		ItemListing::<T>::remove_prefix((owner,), None);
		FixedPriceListing::<T>::remove_prefix((owner,), None);
		TimeAuction::<T>::remove_prefix((owner,), None);
	}
}
