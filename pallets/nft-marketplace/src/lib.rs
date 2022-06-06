#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::unused_unit)]
#![allow(clippy::unpper_case_acronyms)]

use frame_support::{pallet_prelude::*, traits::Time, transactional};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;

use sp_runtime::{
	traits::{Saturating, StaticLookup},
	ArithmeticError, DispatchResult, FixedPointNumber, FixedU128, RuntimeDebug,
};
use sp_std::{if_std, prelude::*, vec::Vec};

// #[cfg(feature = "runtime-benchmarks")]
// pub mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
// mod weights;

use bholdus_support::MultiCurrency;
use common_primitives::Balance;

use bholdus_support_nft_marketplace::{
	AuctionType, BHC20TokenId, Blacklist as NFTBlacklist, Denominator, FixedPriceListing,
	FixedPriceListingInfo, FixedPriceListingInfoOf, MarketMode, MarketplaceFee, MarketplaceFeeInfo,
	MarketplaceFeeInfoOf, NFTCurrencyId, NFTState, Numerator, Permission, Price, RoleType,
	RoyaltyRate, TimeAuctionListingInfo, TimeAuctionListingInfoOf, UserBlacklist,
};

pub mod pallet;
pub use pallet::{pallet::*, FixedPriceSetting};

pub type TokenIdOf<T> = <T as bholdus_support_nft::Config>::TokenId;
pub type ClassIdOf<T> = <T as bholdus_support_nft::Config>::ClassId;

impl<T: Config> Pallet<T> {
	pub fn check_new_listing(
		who: &T::AccountId,
		token: (ClassIdOf<T>, TokenIdOf<T>),
		market_mode: MarketMode,
	) -> DispatchResult {
		ensure!(Self::is_owner(who, token), Error::<T>::NoPermission);
		ensure!(!Self::is_banned_user(who), Error::<T>::UserBanned);
		ensure!(!Self::is_banned(token), Error::<T>::NFTBanned);
		ensure!(!Self::is_listing(who, token, market_mode), Error::<T>::IsListing);
		Ok(())
	}
	pub fn check_new_fixed_price(
		who: &T::AccountId,
		token: (ClassIdOf<T>, TokenIdOf<T>),
		info: &FixedPriceSettingOf<T>,
	) -> DispatchResult {
		let now = T::Time::now();
		ensure!(now < info.expired_time, Error::<T>::InvalidTimeConfiguration);
		Self::check_new_listing(who, token, MarketMode::FixedPrice)
	}

	pub fn check_new_time_auction(
		who: &T::AccountId,
		token: (ClassIdOf<T>, TokenIdOf<T>),
		info: &TimeAuctionSettingOf<T>,
	) -> DispatchResult {
		let now = T::Time::now();
		ensure!(now < info.auction_end, Error::<T>::InvalidTimeConfiguration);
		Self::check_new_listing(who, token, MarketMode::Auction(AuctionType::English))
	}

	pub fn is_available(time: MomentOf<T>) -> DispatchResult {
		let now = T::Time::now();
		ensure!(now < time, Error::<T>::ExpiredListing);
		Ok(())
	}
}

impl<T: Config> Pallet<T> {
	pub fn do_grant_role(
		origin: OriginFor<T>,
		role: &RoleType,
		account: &T::AccountId,
	) -> DispatchResult {
		bholdus_support_nft_marketplace::Pallet::<T>::grant_role(origin, role, account)
	}

	pub fn do_revoke_role(
		origin: OriginFor<T>,
		role: &RoleType,
		account: &T::AccountId,
	) -> DispatchResult {
		bholdus_support_nft_marketplace::Pallet::<T>::revoke_role(origin, role, account)
	}

	pub fn new_time_auction(
		token: (ClassIdOf<T>, TokenIdOf<T>),
		info: &TimeAuctionListingInfoOf<T>,
	) -> DispatchResult {
		bholdus_support_nft_marketplace::Pallet::<T>::new_time_auction(token, info)
	}

	pub fn new_fixed_price(
		token: (ClassIdOf<T>, TokenIdOf<T>),
		info: &FixedPriceListingInfoOf<T>,
	) -> DispatchResult {
		bholdus_support_nft_marketplace::Pallet::<T>::new_fixed_price(token, info)
	}

	pub fn approve(token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
		bholdus_support_nft_marketplace::Pallet::<T>::approve(token)
	}

	pub fn delist(owner: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
		bholdus_support_nft_marketplace::Pallet::<T>::delist(owner, token)
	}

	pub fn add_user_to_blacklist(account: &T::AccountId, reason: Vec<u8>) -> DispatchResult {
		bholdus_support_nft_marketplace::Pallet::<T>::add_user_to_blacklist(account, reason)
	}

	pub fn add_item_to_blacklist(
		token: (ClassIdOf<T>, TokenIdOf<T>),
		reason: Vec<u8>,
	) -> DispatchResult {
		bholdus_support_nft_marketplace::Pallet::<T>::add_item_to_blacklist(token, reason.clone())
	}
}

impl<T: Config> Pallet<T> {
	pub fn owner(token: (ClassIdOf<T>, TokenIdOf<T>)) -> T::AccountId {
		bholdus_support_nft_marketplace::Pallet::<T>::owner(token)
	}

	pub fn has_admin_permission(permission: &Permission, account: &T::AccountId) -> bool {
		bholdus_support_nft_marketplace::Pallet::<T>::has_admin_permission(permission, account)
	}

	pub fn is_owner(account: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> bool {
		bholdus_support_nft_marketplace::Pallet::<T>::is_owner(account, token)
	}

	pub fn is_banned_user(account: &T::AccountId) -> bool {
		bholdus_support_nft_marketplace::Pallet::<T>::is_user_blacklist(account)
	}

	pub fn is_banned(token: (ClassIdOf<T>, TokenIdOf<T>)) -> bool {
		bholdus_support_nft_marketplace::Pallet::<T>::is_item_blacklist(token)
	}

	pub fn is_listing(
		account: &T::AccountId,
		token: (ClassIdOf<T>, TokenIdOf<T>),
		mode: MarketMode,
	) -> bool {
		bholdus_support_nft_marketplace::Pallet::<T>::is_listing(account, token, mode)
	}

	pub fn is_lock(account: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> bool {
		bholdus_support_nft_marketplace::Pallet::<T>::is_lock(account, token)
	}

	pub fn get_royalty_value(
		token: (ClassIdOf<T>, TokenIdOf<T>),
		account: &T::AccountId,
		royalty: Option<(Numerator, Denominator)>,
	) -> (T::AccountId, RoyaltyRate) {
		bholdus_support_nft_marketplace::Pallet::<T>::get_royalty_value(token, account, royalty)
	}

	pub fn calc_amount(amount: Balance, rate: (Numerator, Denominator)) -> Balance {
		bholdus_support_nft_marketplace::Pallet::<T>::calc_amount(amount, rate)
	}

	pub fn calculate_royalty_amount(data: &FixedPriceListingInfoOf<T>) -> Balance {
		bholdus_support_nft_marketplace::Pallet::<T>::calculate_royalty_amount(data)
	}

	pub fn is_existed(token: (ClassIdOf<T>, TokenIdOf<T>)) -> bool {
		bholdus_support_nft_marketplace::Pallet::<T>::is_existed(token)
	}
}
