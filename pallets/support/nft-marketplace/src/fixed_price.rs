use crate::*;

/// Fixed Price Listing Info
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct FixedPriceListingInfo<AccountId, CurrencyId, Moment> {
	pub owner: AccountId,
	pub buyer: Option<AccountId>,
	pub price: Price,
	pub currency_id: CurrencyId,
	pub royalty: RoyaltyRate,
	pub status: NFTState,
	pub expired_time: Moment,
	pub service_fee: (Numerator, Denominator),
	pub actual_price: Balance,
	pub royalty_amount: Balance,
	pub fee_amount: Balance,
	pub fee_recipient: AccountId,
	pub royalty_recipient: AccountId,
	pub order_time: Option<Moment>,
}

impl<T: Config> Pallet<T> {
	pub fn approve_fixed_price(
		owner: &T::AccountId,
		token: (ClassIdOf<T>, TokenIdOf<T>),
	) -> DispatchResult {
		let now = T::Time::now();

		FixedPriceListing::<T>::try_mutate_exists(
			(owner, token.0, token.1),
			|info| -> DispatchResult {
				let listing_info = info.as_mut().ok_or(Error::<T>::NotFound)?;
				ensure!(now < listing_info.expired_time, Error::<T>::ExpiredListing);
				ensure!(listing_info.status == NFTState::Pending, Error::<T>::IsApproved);
				listing_info.status = NFTState::Listing;
				Ok(())
			},
		)
	}
}
