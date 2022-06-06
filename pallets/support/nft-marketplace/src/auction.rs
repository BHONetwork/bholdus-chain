use crate::*;
use codec::{Decode, Encode};

use sp_std::{
	fmt::{Debug, Display, Formatter},
	result,
	vec::Vec,
};

#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum AuctionType {
	English,
	Dutch,
}

impl Display for AuctionType {
	fn fmt(&self, f: &mut Formatter) -> sp_std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl Default for AuctionType {
	fn default() -> Self {
		AuctionType::English
	}
}

#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq, TypeInfo)]
pub struct TimeAuctionListingInfo<AccountId, CurrencyId, Moment> {
	pub owner: AccountId,
	pub min_price: Price,
	pub currency_id: CurrencyId,
	pub royalty: (Numerator, Denominator),
	pub status: NFTState,
	pub auction_end: Moment,
	pub service_fee: (Numerator, Denominator),
	pub fee_recipient: AccountId,
	pub royalty_recipient: AccountId,
	pub bid_count: u32,
}

impl<T: Config> Pallet<T> {
	pub fn approve_time_auction(
		owner: &T::AccountId,
		token: (ClassIdOf<T>, TokenIdOf<T>),
	) -> DispatchResult {
		let now = T::Time::now();
		TimeAuction::<T>::try_mutate_exists((owner, token.0, token.1), |info| -> DispatchResult {
			let listing_info = info.as_mut().ok_or(Error::<T>::NotFound)?;
			ensure!(now < listing_info.auction_end, Error::<T>::AuctionAlreadyConcluded);
			listing_info.status = NFTState::Listing;
			Ok(())
		})
	}
}
