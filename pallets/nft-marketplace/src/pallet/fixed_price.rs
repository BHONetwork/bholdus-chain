use super::*;

#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq, TypeInfo)]
pub struct FixedPriceSetting<CurrencyId, Moment> {
	pub price: Balance,
	pub currency_id: CurrencyId,
	pub expired_time: Moment,
	pub royalty: Option<(u32, u32)>,
}

impl<T: Config> Pallet<T> {
	pub fn do_create_fixed_price(
		owner: T::AccountId,
		token: (ClassIdOf<T>, TokenIdOf<T>),
		info: FixedPriceSettingOf<T>,
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

		let fee_amount: Balance = Self::calc_amount(info.price, fee_info.service_fee);
		let royalty_amount: Balance = Self::calc_amount(info.price, (r_numerator, r_denominator));
		let actual_price: Balance =
			info.price.saturating_sub(royalty_amount).saturating_sub(fee_amount);

		let listing_info = FixedPriceListingInfo {
			owner,
			buyer: None,
			price: info.price,
			currency_id: info.currency_id,
			royalty: (r_numerator, r_denominator),
			status: NFTState::Pending,
			expired_time: info.expired_time,
			service_fee: (f_numerator, f_denominator),
			actual_price,
			order_time: None,
			royalty_amount,
			fee_amount,
			fee_recipient: fee_info.beneficiary,
			royalty_recipient,
		};
		Self::new_fixed_price(token, &listing_info)?;
		Self::deposit_event(Event::NewFixedPriceNFTListing { token, listing_info });
		Ok(())
	}

	pub fn buy_fixed_price(
		origin: OriginFor<T>,
		token: (ClassIdOf<T>, TokenIdOf<T>),
	) -> DispatchResult {
		let buyer = ensure_signed(origin.clone())?;
		let now = T::Time::now();
		let owner = Self::owner(token);
		let listing_info =
			FixedPriceListing::<T>::get((&owner, token.0, token.1)).ok_or(Error::<T>::NotFound)?;
		Self::is_available(listing_info.expired_time)?;
		let actual_price = listing_info.actual_price;
		let royalty_amount = listing_info.royalty_amount;
		let fee_amount = listing_info.fee_amount;

		ensure!(buyer != listing_info.owner, Error::<T>::CannotBuyNFT);
		ensure!(listing_info.status == NFTState::Listing, Error::<T>::NotFound);
		ensure!(Self::is_lock(&owner, token), Error::<T>::NotFound);

		match listing_info.currency_id {
			NFTCurrencyId::Token(token_id) => {
				T::Currency::transfer(token_id, &buyer, &listing_info.fee_recipient, fee_amount)?;
				T::Currency::transfer(
					token_id,
					&buyer,
					&listing_info.royalty_recipient,
					royalty_amount,
				)?;
				T::Currency::transfer(token_id, &buyer, &listing_info.owner, actual_price)?;
			}
			NFTCurrencyId::Native => {
				let amount = pallet_balances::Pallet::<T>::free_balance(&buyer);
				ensure!(amount > listing_info.price, Error::<T>::InsufficientBalance);

				pallet_balances::Pallet::<T>::transfer_keep_alive(
					origin.clone(),
					T::Lookup::unlookup(listing_info.fee_recipient.clone()),
					fee_amount,
				);

				pallet_balances::Pallet::<T>::transfer_keep_alive(
					origin.clone(),
					T::Lookup::unlookup(listing_info.royalty_recipient.clone()),
					royalty_amount,
				);

				pallet_balances::Pallet::<T>::transfer_keep_alive(
					origin,
					T::Lookup::unlookup(owner),
					actual_price,
				);
			}
		};

		let order = FixedPriceListingInfo {
			buyer: Some(buyer.clone()),
			order_time: Some(now),
			..listing_info
		};

		bholdus_support_nft_marketplace::Pallet::<T>::buy_now(buyer, token, order.clone())?;
		Self::deposit_event(Event::FixedPriceFulfilled { token, order });
		Ok(())
	}
}
