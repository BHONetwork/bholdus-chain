#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::unused_unit)]
#![allow(clippy::unpper_case_acronyms)]

use frame_support::{pallet_prelude::*, traits::Time, transactional};

use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{DispatchResult, RuntimeDebug};

use sp_std::vec::Vec;

// #[cfg(feature = "runtime-benchmarks")]
// pub mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
// mod weights;

use common_primitives::Balance;

use bholdus_support_nft_marketplace::{
	BHC20TokenIdOf, Blacklist as NFTBlacklist, Denominator, FixedPriceListing,
	FixedPriceListingInfo, MarketMode, MarketplaceFee, MarketplaceFeeInfo, MarketplaceFeeInfoOf,
	NFTCurrencyId, Numerator, PalletManagement, PalletManagementInfo, PalletManagementInfoOf,
	Price, RoyaltyRate, UserBlacklist,
};

pub use support_module::*;

pub type TokenIdOf<T> = <T as bholdus_support_nft::Config>::TokenId;
pub type ClassIdOf<T> = <T as bholdus_support_nft::Config>::ClassId;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct PendingListingInfo<NFTCurrencyId, Moment> {
	pub currency_id: NFTCurrencyId,
	pub price: Price,
	pub royalty: RoyaltyRate,
	pub expired_time: Moment,
	pub service_fee: (Numerator, Denominator),
}

#[frame_support::pallet]
pub mod support_module {
	use super::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ bholdus_support_nft::Config
		+ bholdus_tokens::Config<Balance = Balance>
		+ bholdus_support_nft_marketplace::Config
		+ pallet_balances::Config<Balance = Balance>
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	pub type MomentOf<T> = <<T as bholdus_support_nft_marketplace::Config>::Time as Time>::Moment;
	pub type PendingListingInfoOf<T> =
		PendingListingInfo<NFTCurrencyId<BHC20TokenIdOf<T>>, MomentOf<T>>;
	#[pallet::error]
	pub enum Error<T> {
		IsListing,
		ItemMustBeListing,
		AccountIdMustBeController,
		NotFoundPalletManagementInfo,
		NotFoundMarketplaceFeeInfo,
		BadPrice,
		NoPermission,
		InsufficientBalance,
		UserBanned,
		NotFoundUserInBlacklist,
		NFTBanned,
		BadRequest,
		NotFoundServiceFee,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Add pallet management info
		AddedManagementInfo { management_info: PalletManagementInfoOf<T> },
		/// Update pallet management info
		UpdatedManagementInfo { management_info: PalletManagementInfoOf<T> },

		/// Set Marketplace Fee Information
		ConfiguredMarketplaceFee {
			controller: T::AccountId,
			marketplace_fee_info: MarketplaceFeeInfoOf<T>,
		},
		/// Add item on marketplace
		NewFixedPriceNFTListing {
			owner: T::AccountId,
			token: (ClassIdOf<T>, TokenIdOf<T>),
			listing_info: PendingListingInfoOf<T>,
		},
		/// Approve listing
		ListingApproved { controller: T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>) },

		/// Cancel item list on marketplace
		CancelledListing {
			account: T::AccountId,
			token: (ClassIdOf<T>, TokenIdOf<T>),
			reason: Vec<u8>,
		},

		/// Add a NFT item to blacklist
		NFTBanned { controller: T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>), reason: Vec<u8> },

		/// Remove a NFT from blacklist
		NFTUnbanned { controller: T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>) },

		/// Remove user from blacklist
		UserUnbanned { controller: T::AccountId, account: T::AccountId },

		/// Add a user to blacklist
		UserBanned { controller: T::AccountId, account: T::AccountId, reason: Vec<u8> },
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		#[transactional]
		pub fn configure_pallet_management(
			origin: OriginFor<T>,
			controller: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if PalletManagement::<T>::exists() {
				PalletManagement::<T>::try_mutate(|management_info| -> DispatchResult {
					let info = management_info.as_mut().ok_or(Error::<T>::BadRequest)?;
					ensure!(info.controller == who, Error::<T>::AccountIdMustBeController);
					if info.controller == controller.clone() {
						// no change needed
						return Ok(());
					}
					info.controller = controller.clone();
					Self::deposit_event(Event::UpdatedManagementInfo {
						management_info: PalletManagementInfo { controller: controller.clone() },
					});
					Ok(())
				})
			} else {
				PalletManagement::<T>::put(PalletManagementInfo { controller: controller.clone() });
				Self::deposit_event(Event::AddedManagementInfo {
					management_info: PalletManagementInfo { controller: controller.clone() },
				});
				Ok(())
			}
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn set_marketplace_fee(
			origin: OriginFor<T>,
			service_fee: (Numerator, Denominator),
			beneficiary: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let management_info =
				PalletManagement::<T>::get().ok_or(Error::<T>::NotFoundPalletManagementInfo)?;
			ensure!(management_info.controller == who, Error::<T>::NoPermission);
			let fee_info = MarketplaceFeeInfo { service_fee, beneficiary: beneficiary.clone() };
			MarketplaceFee::<T>::put(fee_info.clone());
			Self::deposit_event(Event::ConfiguredMarketplaceFee {
				controller: management_info.controller,
				marketplace_fee_info: fee_info.clone(),
			});

			Ok(())
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn ban_user(
			origin: OriginFor<T>,
			account: T::AccountId,
			reason: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let management_info =
				PalletManagement::<T>::get().ok_or(Error::<T>::NotFoundPalletManagementInfo)?;
			ensure!(management_info.controller == who, Error::<T>::NoPermission);

			ensure!(!Self::is_banned_user(&account), Error::<T>::UserBanned);

			Self::add_user_to_blacklist(&account, reason.clone())?;
			Self::deposit_event(Event::UserBanned {
				controller: management_info.controller,
				account,
				reason: reason.clone(),
			});
			Ok(())
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn unban_user(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			let controller = ensure_signed(origin)?;
			let management_info =
				PalletManagement::<T>::get().ok_or(Error::<T>::NotFoundPalletManagementInfo)?;
			ensure!(management_info.controller == controller, Error::<T>::NoPermission);
			UserBlacklist::<T>::take(&account).ok_or(Error::<T>::NotFoundUserInBlacklist)?;
			Self::deposit_event(Event::UserUnbanned { controller, account });

			Ok(())
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn ban(
			origin: OriginFor<T>,
			token: (ClassIdOf<T>, TokenIdOf<T>),
			reason: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let management_info =
				PalletManagement::<T>::get().ok_or(Error::<T>::NotFoundPalletManagementInfo)?;
			ensure!(management_info.controller == who, Error::<T>::NoPermission);
			ensure!(!Self::is_banned(token), Error::<T>::NFTBanned);
			Self::add_item_to_blacklist(token, reason.clone())?;
			Self::deposit_event(Event::NFTBanned {
				controller: management_info.controller,
				token,
				reason: reason.clone(),
			});

			Ok(())
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn unban(origin: OriginFor<T>, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
			let controller = ensure_signed(origin)?;
			let management_info =
				PalletManagement::<T>::get().ok_or(Error::<T>::NotFoundPalletManagementInfo)?;
			ensure!(management_info.controller == controller, Error::<T>::NoPermission);

			NFTBlacklist::<T>::take(&token).ok_or(Error::<T>::NFTBanned)?;
			Self::deposit_event(Event::NFTUnbanned {
				controller: management_info.controller,
				token,
			});
			Ok(())
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn create_fixed_price_listing(
			origin: OriginFor<T>,
			token: (ClassIdOf<T>, TokenIdOf<T>),
			price: Price,
			currency_id: NFTCurrencyId<BHC20TokenIdOf<T>>,
			royalty: Option<(Numerator, Denominator)>,
			expired_time: MomentOf<T>,
		) -> DispatchResult {
			let account = ensure_signed(origin)?;

			ensure!(Self::is_owner(&account, token), Error::<T>::NoPermission);
			ensure!(!Self::is_banned_user(&account), Error::<T>::UserBanned);
			ensure!(!Self::is_banned(token), Error::<T>::NFTBanned);
			ensure!(
				!Self::is_listing(&account, token, MarketMode::FixedPrice),
				Error::<T>::IsListing
			);

			let fee_info = MarketplaceFee::<T>::get().ok_or(Error::<T>::NotFoundServiceFee)?;
			let service_fee = fee_info.service_fee;

			let royalty_value = Self::get_royalty_value(royalty);
			bholdus_support_nft_marketplace::Pallet::<T>::create_fixed_price_listing(
				&account,
				token,
				price.clone(),
				currency_id.clone(),
				royalty_value,
				expired_time,
				service_fee,
			)?;

			let listing_info = PendingListingInfo {
				currency_id: currency_id.clone(),
				price: price.clone(),
				royalty: royalty_value,
				expired_time,
				service_fee,
			};

			Self::deposit_event(Event::NewFixedPriceNFTListing {
				owner: account,
				token,
				listing_info,
			});
			Ok(())
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn approve_listing(
			origin: OriginFor<T>,
			token: (ClassIdOf<T>, TokenIdOf<T>),
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let management_info =
				PalletManagement::<T>::get().ok_or(Error::<T>::NotFoundPalletManagementInfo)?;
			ensure!(management_info.controller == who, Error::<T>::NoPermission);
			Self::approve_item_listing(token)?;
			Self::deposit_event(Event::ListingApproved {
				controller: management_info.controller,
				token,
			});
			Ok(())
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn reject_listing(
			origin: OriginFor<T>,
			token: (ClassIdOf<T>, TokenIdOf<T>),
			reason: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let management_info =
				PalletManagement::<T>::get().ok_or(Error::<T>::NotFoundPalletManagementInfo)?;
			ensure!(management_info.controller == who, Error::<T>::NoPermission);
			let owner = Self::owner(token);
			Self::delist(&owner, token)?;
			// Emit Event
			Self::deposit_event(Event::CancelledListing { account: who, token, reason });
			Ok(())
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn cancel_listing(
			origin: OriginFor<T>,
			token: (ClassIdOf<T>, TokenIdOf<T>),
			reason: Vec<u8>,
		) -> DispatchResult {
			// Only allow owner can cancel product listing
			let who = ensure_signed(origin)?;
			ensure!(Self::is_owner(&who, token), Error::<T>::NoPermission);
			Self::delist(&who, token)?;
			// Emit Event
			Self::deposit_event(Event::CancelledListing { account: who, token, reason });
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn owner(token: (ClassIdOf<T>, TokenIdOf<T>)) -> T::AccountId {
		bholdus_support_nft_marketplace::Pallet::<T>::owner(token)
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

	pub fn get_royalty_value(royalty: Option<(Numerator, Denominator)>) -> RoyaltyRate {
		bholdus_support_nft_marketplace::Pallet::<T>::get_loyalty_value(royalty)
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

	pub fn approve_item_listing(token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
		bholdus_support_nft_marketplace::Pallet::<T>::approve_item_listing(token)
	}

	pub fn delist(owner: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
		bholdus_support_nft_marketplace::Pallet::<T>::delist(owner, token)
	}
}
