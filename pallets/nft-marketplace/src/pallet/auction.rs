use super::*;
#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq, TypeInfo)]
pub struct TimeAuctionSetting<CurrencyId, Moment> {
	pub min_price: Balance,
	pub currency_id: CurrencyId,
	pub auction_end: Moment,
	pub royalty: Option<(u32, u32)>,
}

impl<T: Config> Pallet<T> {
	pub fn do_create_time_auction(
		owner: T::AccountId,
		token: (ClassIdOf<T>, TokenIdOf<T>),
		info: TimeAuctionSettingOf<T>,
	) -> DispatchResult {
		let fee_info = MarketplaceFee::<T>::get().ok_or(Error::<T>::NotFoundServiceFee)?;
		let (f_numerator, f_denominator) = fee_info.service_fee;
		let (royalty_recipient, (r_numerator, r_denominator)) =
			Self::get_royalty_value(token, &owner, info.royalty);
		let fee_rate = FixedU128::checked_from_rational(f_numerator, f_denominator)
			.ok_or(ArithmeticError::Overflow)?;
		let royalty_rate = FixedU128::checked_from_rational(r_numerator, r_denominator)
			.ok_or(ArithmeticError::Overflow)?;

		ensure!(
			fee_rate.saturating_add(royalty_rate)
				< FixedU128::checked_from_rational(10_000, 10_000)
					.ok_or(ArithmeticError::Overflow)?,
			Error::<T>::InvalidRate
		);
		let listing_info = TimeAuctionListingInfo {
			owner,
			min_price: info.min_price,
			currency_id: info.currency_id,
			royalty: (r_numerator, r_denominator),
			status: NFTState::Pending,
			auction_end: info.auction_end,
			service_fee: (f_numerator, f_denominator),
			fee_recipient: fee_info.beneficiary,
			royalty_recipient,
			bid_count: 0u32,
		};
		Self::new_time_auction(token, &listing_info)?;
		// Emit Event
		Self::deposit_event(Event::NewTimeAuctionNFTListing { token, listing_info });
		Ok(())
	}
}
