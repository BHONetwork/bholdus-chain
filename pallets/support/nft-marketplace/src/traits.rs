use crate::*;

/// Royalty Information
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct RoyaltyInfo<AccountId> {
	pub value: (Numerator, Denominator),
	pub creator: AccountId,
}

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

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ItemListingInfo<AccountId> {
	pub owner: AccountId,
	pub mode: MarketMode,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum MarketMode {
	FixedPrice,
	Auction(AuctionType),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum NFTCurrencyId<BHC20TokenId> {
	Native,
	Token(BHC20TokenId),
}

#[derive(Encode, Copy, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum NFTState {
	Pending,
	Listing,
}

impl<T: Config> Pallet<T> {
	pub fn calc_amount(amount: Balance, rate: (Numerator, Denominator)) -> Balance {
		let numerator = U256::from(amount).saturating_mul(U256::from(rate.0));
		let denominator = U256::from(rate.1);
		numerator
			.checked_div(denominator)
			.and_then(|n| TryInto::<Balance>::try_into(n).ok())
			.unwrap_or_else(Zero::zero)
	}

	pub fn calculate_royalty_amount(data: &FixedPriceListingInfoOf<T>) -> Balance {
		let price = data.price;
		let (royalty_numerator, royalty_denominator) = data.royalty;
		royalty_numerator
			.checked_div(royalty_denominator)
			.and_then(|n| TryInto::<Balance>::try_into(n).ok())
			.unwrap_or_else(Zero::zero)
			.saturating_mul(price)
	}

	pub fn get_royalty_value(
		token: (ClassIdOf<T>, TokenIdOf<T>),
		account: &T::AccountId,
		value: Option<(Numerator, Denominator)>,
	) -> (T::AccountId, (Numerator, Denominator)) {
		if let Some(royalty) = Royalty::<T>::get(token.0, token.1) {
			(royalty.creator, royalty.value)
		} else {
			let value = value.unwrap_or((0u32, 10_000u32));
			let data = RoyaltyInfo { value, creator: account.clone() };
			Royalty::<T>::insert(token.0, token.1, data);
			(account.clone(), value.clone())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn owner(token: (ClassIdOf<T>, TokenIdOf<T>)) -> T::AccountId {
		bholdus_support_nft::Pallet::<T>::owner(token)
	}

	pub fn is_existed(token: (ClassIdOf<T>, TokenIdOf<T>)) -> bool {
		bholdus_support_nft::Pallet::<T>::is_existed(token)
	}
	pub fn is_lock(account: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> bool {
		bholdus_support_nft::Pallet::<T>::is_lock(account, token)
	}

	pub fn is_owner(account: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> bool {
		bholdus_support_nft::Pallet::<T>::is_owner(account, token)
	}

	pub fn is_listing(
		owner: &T::AccountId,
		token: (ClassIdOf<T>, TokenIdOf<T>),
		market_mode: MarketMode,
	) -> bool {
		match market_mode {
			MarketMode::FixedPrice => {
				FixedPriceListing::<T>::contains_key((owner, token.0, token.1))
			}

			MarketMode::Auction(_) => TimeAuction::<T>::contains_key((owner, token.0, token.1)),
		}
	}

	pub fn is_approved(
		owner: &T::AccountId,
		token: (ClassIdOf<T>, TokenIdOf<T>),
		market_mode: MarketMode,
	) -> bool {
		match market_mode {
			MarketMode::FixedPrice => {
				if let Some(listing_info) = FixedPriceListing::<T>::get((owner, token.0, token.1)) {
					listing_info.status == NFTState::Listing
				} else {
					false
				}
			}
			MarketMode::Auction(_) => {
				if let Some(listing_info) = TimeAuction::<T>::get((owner, token.0, token.1)) {
					listing_info.status == NFTState::Listing
				} else {
					false
				}
			}
		}
	}

	pub fn is_user_blacklist(user: &T::AccountId) -> bool {
		UserBlacklist::<T>::contains_key(user)
	}

	pub fn is_item_blacklist(token: (ClassIdOf<T>, TokenIdOf<T>)) -> bool {
		Blacklist::<T>::contains_key((token.0, token.1))
	}

	pub fn is_bhc20(currency_id: NFTCurrencyId<BHC20TokenId>) -> bool {
		match currency_id {
			NFTCurrencyId::Token(_token_id) => true,
			_ => false,
		}
	}

	pub fn is_native(currency_id: NFTCurrencyId<BHC20TokenId>) -> bool {
		match currency_id {
			NFTCurrencyId::Native => true,
			_ => false,
		}
	}
}
