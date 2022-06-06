use super::*;

pub mod auction;
pub mod fixed_price;
pub use auction::*;
pub use fixed_price::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ bholdus_support_nft::Config
		// + bholdus_tokens::Config<Balance = Balance>
		+ bholdus_support_nft_marketplace::Config
		+ pallet_balances::Config<Balance = Balance>
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    	}

	pub type MomentOf<T> = <<T as bholdus_support_nft_marketplace::Config>::Time as Time>::Moment;
	pub type FixedPriceSettingOf<T> = FixedPriceSetting<NFTCurrencyId<BHC20TokenId>, MomentOf<T>>;
	pub type TimeAuctionSettingOf<T> = TimeAuctionSetting<NFTCurrencyId<BHC20TokenId>, MomentOf<T>>;
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
		InvalidRate,
		NotFound,
		CannotBuyNFT,
		ExpiredListing,
		InvalidTimeConfiguration,
		RoleRedundant,
		MissingRole,
		MissingPermission,
		InvalidRole,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Grant Role
		RoleGranted { account: T::AccountId, role: RoleType },
		/// Revoke Role
		RoleRevoked { account: T::AccountId, role: RoleType },

		/// Set Marketplace Fee Information
		ConfiguredMarketplaceFee {
			controller: T::AccountId,
			marketplace_fee_info: MarketplaceFeeInfoOf<T>,
		},
		/// Add item on marketplace
		NewFixedPriceNFTListing {
			token: (ClassIdOf<T>, TokenIdOf<T>),
			listing_info: FixedPriceListingInfoOf<T>,
		},

		/// Emit Time Auction Listing Event
		NewTimeAuctionNFTListing {
			token: (ClassIdOf<T>, TokenIdOf<T>),
			listing_info: TimeAuctionListingInfoOf<T>,
		},

		/// Approve listing
		ListingApproved { controller: T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>) },

		/// Cancel item list on marketplace
		CancelledListing {
			account: T::AccountId,
			token: (ClassIdOf<T>, TokenIdOf<T>),
			reason: Vec<u8>,
		},

		/// Buy NFT
		FixedPriceFulfilled {
			token: (ClassIdOf<T>, TokenIdOf<T>),
			order: FixedPriceListingInfoOf<T>,
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
		pub fn grant_role(
			origin: OriginFor<T>,
			role: RoleType,
			account: T::AccountId,
		) -> DispatchResult {
			Self::do_grant_role(origin, &role, &account)?;
			Self::deposit_event(Event::RoleGranted { account, role });
			Ok(())
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn revoke_role(
			origin: OriginFor<T>,
			role: RoleType,
			account: T::AccountId,
		) -> DispatchResult {
			Self::do_revoke_role(origin, &role, &account)?;
			Self::deposit_event(Event::RoleRevoked { account, role });
			Ok(())
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn set_marketplace_fee(
			origin: OriginFor<T>,
			service_fee: (Numerator, Denominator),
			beneficiary: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(
				Self::has_admin_permission(&Permission::UnbanUser, &who),
				Error::<T>::MissingPermission
			);
			let fee_info = MarketplaceFeeInfo { service_fee, beneficiary: beneficiary.clone() };
			MarketplaceFee::<T>::put(fee_info.clone());
			Self::deposit_event(Event::ConfiguredMarketplaceFee {
				controller: who,
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

			ensure!(
				Self::has_admin_permission(&Permission::BanUser, &who),
				Error::<T>::MissingPermission
			);

			ensure!(!Self::is_banned_user(&account), Error::<T>::UserBanned);

			Self::add_user_to_blacklist(&account, reason.clone())?;
			Self::deposit_event(Event::UserBanned {
				controller: who,
				account,
				reason: reason.clone(),
			});
			Ok(())
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn unban_user(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				Self::has_admin_permission(&Permission::UnbanUser, &who),
				Error::<T>::MissingPermission
			);

			UserBlacklist::<T>::take(&account).ok_or(Error::<T>::NotFoundUserInBlacklist)?;
			Self::deposit_event(Event::UserUnbanned { controller: who, account });

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

			ensure!(
				Self::has_admin_permission(&Permission::Ban, &who),
				Error::<T>::MissingPermission
			);
			ensure!(!Self::is_banned(token), Error::<T>::NFTBanned);
			Self::add_item_to_blacklist(token, reason.clone())?;
			Self::deposit_event(Event::NFTBanned {
				controller: who,
				token,
				reason: reason.clone(),
			});

			Ok(())
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn unban(origin: OriginFor<T>, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(
				Self::has_admin_permission(&Permission::Unban, &who),
				Error::<T>::MissingPermission
			);
			NFTBlacklist::<T>::take(&token).ok_or(Error::<T>::NFTBanned)?;
			Self::deposit_event(Event::NFTUnbanned { controller: who, token });
			Ok(())
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn create_fixed_price_listing(
			origin: OriginFor<T>,
			token: (ClassIdOf<T>, TokenIdOf<T>),
			info: FixedPriceSettingOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::check_new_fixed_price(&who, token, &info)?;
			Self::do_create_fixed_price(who, token, info)
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn create_time_auction_listing(
			origin: OriginFor<T>,
			token: (ClassIdOf<T>, TokenIdOf<T>),
			info: TimeAuctionSettingOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::check_new_time_auction(&who, token, &info)?;
			Self::do_create_time_auction(who, token, info)
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn buy_now(
			origin: OriginFor<T>,
			token: (ClassIdOf<T>, TokenIdOf<T>),
		) -> DispatchResult {
			ensure!(Self::is_existed(token), Error::<T>::CannotBuyNFT);
			Self::buy_fixed_price(origin, token)
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn approve_listing(
			origin: OriginFor<T>,
			token: (ClassIdOf<T>, TokenIdOf<T>),
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				Self::has_admin_permission(&Permission::ApproveListing, &who),
				Error::<T>::MissingPermission
			);

			Self::approve(token)?;
			Self::deposit_event(Event::ListingApproved { controller: who, token });
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
			ensure!(
				Self::has_admin_permission(&Permission::RejectListing, &who),
				Error::<T>::MissingPermission
			);
			let owner = Self::owner(token);
			Self::delist(&owner, token)?;
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
			let who = ensure_signed(origin)?;
			ensure!(Self::is_owner(&who, token), Error::<T>::NoPermission);
			Self::delist(&who, token)?;
			Self::deposit_event(Event::CancelledListing { account: who, token, reason });
			Ok(())
		}
	}
}
