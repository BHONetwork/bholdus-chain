// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Functions for the Assets pallet.

use super::*;
use scale_info::prelude::string::String;

// The main implementation block for the module.
impl<T: Config<I>, I: 'static> Pallet<T, I> {
	pub fn is_valid_symbol(symbol: Vec<u8>) -> bool {
		let str_trim = String::from_utf8(symbol.clone()).unwrap().replace(" ", "");
		let val: Vec<u8> = str_trim.into_bytes();
		symbol == val
	}

	pub fn get_name(name: Vec<u8>) -> Vec<u8> {
		let str_trim = String::from_utf8(name.clone()).unwrap().replace(" ", "");
		str_trim.into_bytes()
	}

	pub fn maybe_add_metadata(
		origin: &T::AccountId,
		id: T::AssetId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	) -> DispatchResult {
		let _blacklist = AssetsBlacklist::<T, I>::get().contains(&(name.clone(), symbol.clone()));
		ensure!(
			!AssetsBlacklist::<T, I>::get().contains(&(name.clone(), symbol.clone())),
			Error::<T, I>::AssetBlacklist
		);

		let bounded_name: BoundedVec<u8, T::StringLimit> =
			name.clone().try_into().map_err(|_| Error::<T, I>::BadMetadata)?;
		let bounded_symbol: BoundedVec<u8, T::StringLimit> =
			symbol.clone().try_into().map_err(|_| Error::<T, I>::BadMetadata)?;

		Metadata::<T, I>::mutate(id, |metadata| {
			let new_deposit = T::MetadataDepositPerByte::get()
				.saturating_mul(((name.len() + symbol.len()) as u32).into())
				.saturating_add(T::MetadataDepositBase::get());
			T::Currency::reserve(&origin, new_deposit)?;
			*metadata = AssetMetadata {
				deposit: new_deposit,
				name: bounded_name,
				symbol: bounded_symbol,
				decimals,
				is_frozen: false,
			};

			Ok(())
		})
	}
	// Public immutables

	/// Return the extra "sid-car" data for `id`/`who`, or `None` if the account doesn't exist.
	pub fn adjust_extra(
		id: T::AssetId,
		who: impl sp_std::borrow::Borrow<T::AccountId>,
	) -> Option<ExtraMutator<T, I>> {
		ExtraMutator::maybe_new(id, who)
	}

	///The total issuance of a token type.
	pub fn total_issuance(id: T::AssetId) -> T::Balance {
		Asset::<T, I>::get(id).map(|x| x.supply).unwrap_or_else(Zero::zero)
	}

	pub(super) fn new_account(
		who: &T::AccountId,
		d: &mut AssetDetails<T::Balance, T::AccountId, DepositBalanceOf<T, I>>,
	) -> Result<bool, DispatchError> {
		let accounts = d.accounts.checked_add(1).ok_or(ArithmeticError::Overflow)?;
		let is_sufficient = if d.is_sufficient {
			frame_system::Pallet::<T>::inc_sufficients(who);
			d.sufficients += 1;
			true
		} else {
			frame_system::Pallet::<T>::inc_consumers(who).map_err(|_| Error::<T, I>::NoProvider)?;
			false
		};
		d.accounts = accounts;
		Ok(is_sufficient)
	}

	pub(super) fn dead_account(
		what: T::AssetId,
		who: &T::AccountId,
		d: &mut AssetDetails<T::Balance, T::AccountId, DepositBalanceOf<T, I>>,
		sufficient: bool,
	) {
		if sufficient {
			d.sufficients = d.sufficients.saturating_sub(1);
			frame_system::Pallet::<T>::dec_sufficients(who);
		} else {
			frame_system::Pallet::<T>::dec_consumers(who);
		}
		d.accounts = d.accounts.saturating_sub(1);
		T::Freezer::died(what, who)
	}

	pub(super) fn can_increase(
		id: T::AssetId,
		who: &T::AccountId,
		amount: T::Balance,
		increase_supply: bool,
	) -> DepositConsequence {
		let details = match Asset::<T, I>::get(id) {
			Some(details) => details,
			None => return DepositConsequence::UnknownAsset,
		};
		if increase_supply && details.supply.checked_add(&amount).is_none() {
			return DepositConsequence::Overflow;
		}
		let account = Account::<T, I>::get(id, who);
		if account.free.checked_add(&amount).is_none() {
			return DepositConsequence::Overflow;
		}

		if account.free.is_zero() {
			if amount < details.min_balance {
				return DepositConsequence::BelowMinimum;
			}
			if !details.is_sufficient && frame_system::Pallet::<T>::providers(who) == 0 {
				return DepositConsequence::CannotCreate;
			}
			if details.is_sufficient && details.sufficients.checked_add(1).is_none() {
				return DepositConsequence::Overflow;
			}
		}

		DepositConsequence::Success
	}

	/// Return the consequence of a withdraw.
	pub(super) fn can_decrease(
		id: T::AssetId,
		who: &T::AccountId,
		amount: T::Balance,
		_keep_alive: bool,
		action: Action,
	) -> WithdrawConsequence<T::Balance> {
		use WithdrawConsequence::*;
		let details = match Asset::<T, I>::get(id) {
			Some(details) => details,
			None => return UnknownAsset,
		};

		if details.supply.checked_sub(&amount).is_none() {
			return Underflow;
		}

		if details.is_frozen {
			return Frozen;
		}
		let account = Account::<T, I>::get(id, who);
		if account.is_frozen {
			return Frozen;
		}
		if let Some(rest) = account.free.checked_sub(&amount) {
			if let Some(frozen) = T::Freezer::frozen_balance(id, who) {
				match frozen.checked_add(&details.min_balance) {
					Some(required) if rest < required => return Frozen,
					None => return Overflow,
					_ => {},
				}
			}

			/*let is_provider = false;
			let is_required = is_provider && !frame_system::Pallet::<T>::can_dec_provider(who);
			let must_keep_alive = keep_alive || is_required;
			*/

			if rest < details.min_balance {
				/*if must_keep_alive {
				   WouldDie
				} else {
					ReducedToZero(rest)
				}
				*/

				match action {
					Action::Burn => ReducedToZero(rest),
					Action::Transfer => NoFunds,
					_ => ReducedToZero(rest),
				}
			} else {
				Success
			}
		} else {
			NoFunds
		}
	}

	// Maximum `amount` that can be passed into `can_withdraw` to result in a `WithdrawConsequence`
	// of `Success`.
	pub(super) fn reducible_balance(
		id: T::AssetId,
		who: &T::AccountId,
		keep_alive: bool,
	) -> Result<T::Balance, DispatchError> {
		let details = Asset::<T, I>::get(id).ok_or_else(|| Error::<T, I>::Unknown)?;
		ensure!(!details.is_frozen, Error::<T, I>::Frozen);

		let account = Account::<T, I>::get(id, who);
		ensure!(!account.is_frozen, Error::<T, I>::Frozen);

		let amount = if let Some(frozen) = T::Freezer::frozen_balance(id, who) {
			// Frozen balance: account CANNOT be deleted
			let required =
				frozen.checked_add(&details.min_balance).ok_or(ArithmeticError::Overflow)?;
			account.free.saturating_sub(required)
		} else {
			let is_provider = false;
			let is_required = is_provider && !frame_system::Pallet::<T>::can_dec_provider(who);

			if keep_alive || is_required {
				// We want to keep the account around.
				account.free.saturating_sub(details.min_balance)
			} else {
				// Don't care if the account dies
				account.free
			}
		};
		Ok(amount.min(details.supply))
	}

	/// Make preparatory checks for debiting some funds from an account. Flags indicate requirements
	/// of the debit.
	///
	/// - `amount`: The amount desired to be debited. The actual amount returned for debit may be
	///   less (in the case of `best_effort` being `true`) or greater by up to the minimum balance
	///   less one.
	/// - `keep_alive`: Require that `target` must stay alive.
	/// - `respect_freezer`: Respect any freezes on the account or token (or not).
	/// - `best_effort`: The debit amount may be less than `amount`.
	///
	/// On success, the amount which should be debited (this will always be at least `amount` unless
	/// `best_effort` is `true`) together with an optional value indicating the argument which must
	/// be passed into the `melted` function of the `T::Freezer` if `Some`.
	///
	/// If no valid debit can be made then return an `Err`.
	pub(super) fn prep_debit(
		id: T::AssetId,
		target: &T::AccountId,
		amount: T::Balance,
		f: DebitFlags,
		action: Action,
	) -> Result<T::Balance, DispatchError> {
		let asset_details = Asset::<T, I>::get(id).ok_or_else(|| Error::<T, I>::Unknown)?;
		ensure!(asset_details.supply >= amount, Error::<T, I>::ExceedTotalSupply);
		let actual = Self::reducible_balance(id, target, f.keep_alive)?.min(amount);
		ensure!(f.best_effort || actual >= amount, Error::<T, I>::BalanceLow);

		let conseq = Self::can_decrease(id, target, actual, f.keep_alive, action);
		let actual = match conseq.into_result() {
			Ok(dust) => actual.saturating_add(dust), //< guaranteed by reducible_balance
			Err(e) => {
				debug_assert!(false, "passed from reducible_balance; qed");
				return Err(e.into());
			},
		};

		Ok(actual)
	}

	/// Make preparatory checks for crediting some funds from an account. Flags indicate
	/// requirements of the credit.
	///
	/// - `amount`: The amount desired to be credited.
	/// - `debit`: The amount by which some other account has been debited. If this is greater than
	///   `amount`, then the `burn_dust` parameter takes effect.
	/// - `burn_dust`: Indicates that in the case of debit being greater than amount, the additional
	///   (dust) value should be burned, rather than credited.
	///
	/// On success, the amount which should be credited (this will always be at least `amount`)
	/// together with an optional value indicating the value which should be burned. The latter
	/// will always be `None` as long as `burn_dust` is `false` or `debit` is no greater than
	/// `amount`.
	///
	/// If no valid credit can be made then return an `Err`.
	pub(super) fn prep_credit(
		id: T::AssetId,
		dest: &T::AccountId,
		amount: T::Balance,
		debit: T::Balance,
		burn_dust: bool,
	) -> Result<(T::Balance, Option<T::Balance>), DispatchError> {
		let (credit, maybe_burn) = match (burn_dust, debit.checked_sub(&amount)) {
			(true, Some(dust)) => (amount, Some(dust)),
			_ => (debit, None),
		};
		Self::can_increase(id, &dest, credit, false).into_result()?;
		Ok((credit, maybe_burn))
	}

	/// Increases the asset `id` balance of `beneficiary` by `amount`.
	///
	/// This alters the registered supply of the asset and emits an event.
	///
	/// Will return an error or will increase the amount by exactly `amount`.
	pub(super) fn do_mint(
		id: T::AssetId,
		beneficiary: &T::AccountId,
		amount: T::Balance,
		maybe_check_issuer: Option<T::AccountId>,
	) -> DispatchResult {
		Self::increase_balance(id, beneficiary, amount, |details| -> DispatchResult {
			if let Some(check_issuer) = maybe_check_issuer {
				ensure!(&check_issuer == &details.issuer, Error::<T, I>::NoPermission);
			}
			debug_assert!(
				T::Balance::max_value() - details.supply >= amount,
				"checked in prep; qed"
			);
			details.supply = details.supply.saturating_add(amount);
			Ok(())
		})?;
		Self::deposit_event(Event::Issued(id, beneficiary.clone(), amount));
		Ok(())
	}

	/// Increases the asset `id` balance of `beneficiary` by `amount`.
	///
	/// LOW-LEVEL: Does not alter the supply of asset or emit an event. Use `do_mint` if you need
	/// that. This is not intended to be used alone.
	///
	/// Will return an error or will increase the amount by exactly `amount`.
	pub(super) fn increase_balance(
		id: T::AssetId,
		beneficiary: &T::AccountId,
		amount: T::Balance,
		check: impl FnOnce(
			&mut AssetDetails<T::Balance, T::AccountId, DepositBalanceOf<T, I>>,
		) -> DispatchResult,
	) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}

		Self::can_increase(id, beneficiary, amount, true).into_result()?;
		Asset::<T, I>::try_mutate(id, |maybe_details| -> DispatchResult {
			let details = maybe_details.as_mut().ok_or(Error::<T, I>::Unknown)?;

			check(details)?;

			Account::<T, I>::try_mutate(id, beneficiary, |t| -> DispatchResult {
				let new_balance = t.free.saturating_add(amount);
				ensure!(new_balance >= details.min_balance, TokenError::BelowMinimum);
				if t.free.is_zero() {
					t.sufficient = Self::new_account(beneficiary, details)?;
				}
				t.free = new_balance;
				Ok(())
			})?;
			Ok(())
		})?;
		Ok(())
	}

	/// Set GenesisConfig
	pub(super) fn do_set_genesis(
		who: &T::AccountId,
		amount: T::Balance,
	) -> Result<bool, DispatchError> {
		let asset_id =
			NextAssetId::<T, I>::try_mutate(|id| -> Result<T::AssetId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(&One::one()).ok_or(Error::<T, I>::NoAvailableTokenId)?;
				Ok(current_id)
			})?;
		Asset::<T, I>::insert(
			asset_id,
			AssetDetails {
				owner: who.clone(),
				issuer: who.clone(),
				admin: who.clone(),
				freezer: who.clone(),
				supply: Zero::zero(),
				deposit: Zero::zero(),
				min_balance: Zero::zero(),
				is_sufficient: true,
				accounts: 0,
				sufficients: 0,
				approvals: 0,
				is_frozen: false,
			},
		);

		Self::increase_balance(asset_id, who, amount, |details| -> DispatchResult {
			details.supply = details.supply.saturating_add(amount);
			Ok(())
		})?;

		Ok(true)
	}

	/// Reduces asset `id` balance of `target` by `amount`. Flags `f` can be given to alter whether
	/// it attempts a `best_effort` or makes sure to `keep_alive` the account.
	///
	/// This alters the registered supply of the asset and emits an event.
	///
	/// Will return an error and do nothing or will decrease the amount and return the amount
	/// reduced by.
	pub(super) fn do_burn(
		id: T::AssetId,
		target: &T::AccountId,
		amount: T::Balance,
		maybe_check_admin: Option<T::AccountId>,
		f: DebitFlags,
	) -> Result<T::Balance, DispatchError> {
		let actual =
			Self::decrease_balance(id, target, amount, f, Action::Burn, |actual, details| {
				// Check admin rights.
				if let Some(check_admin) = maybe_check_admin {
					ensure!(&check_admin == &details.admin, Error::<T, I>::NoPermission);
				}
				debug_assert!(details.supply >= actual, "checked in prep; qed");
				details.supply = details.supply.saturating_sub(actual);

				Ok(())
			})?;
		Self::deposit_event(Event::Burned(id, target.clone(), actual));
		Ok(actual)
	}

	/// Reduces asset `id` balance of `target` by `amount`. Flags `f` can be given to alter whether
	/// it attempts a `best_effort` or makes sure to `keep_alive` the account.
	///
	/// LOW-LEVEL: Does not alter the supply of asset or emit an event. Use `do_burn` if you need
	/// that. This is not intended to be used alone.
	///
	/// Will return an error and do nothing or will decrease the amount and return the amount
	/// reduced by.
	pub(super) fn decrease_balance(
		id: T::AssetId,
		target: &T::AccountId,
		amount: T::Balance,
		f: DebitFlags,
		action: Action,
		check: impl FnOnce(
			T::Balance,
			&mut AssetDetails<T::Balance, T::AccountId, DepositBalanceOf<T, I>>,
		) -> DispatchResult,
	) -> Result<T::Balance, DispatchError> {
		if amount.is_zero() {
			return Ok(amount);
		}

		let actual = Self::prep_debit(id, target, amount, f, action)?;
		Asset::<T, I>::try_mutate(id, |maybe_details| -> DispatchResult {
			let details = maybe_details.as_mut().ok_or(Error::<T, I>::Unknown)?;

			check(actual, details)?;

			Account::<T, I>::try_mutate_exists(id, target, |maybe_account| -> DispatchResult {
				let mut account = maybe_account.take().unwrap_or_default();
				debug_assert!(account.free >= actual, "checked in prep; qed");

				// Make the debit.
				account.free = account.free.saturating_sub(actual);
				*maybe_account = if account.free < details.min_balance {
					debug_assert!(account.free.is_zero(), "checked in prep; qed");
					Self::dead_account(id, target, details, account.sufficient);
					None
				} else {
					Some(account)
				};
				Ok(())
			})?;

			Ok(())
		})?;

		Ok(actual)
	}

	/// Transfer some free balance from `from` to `to`.
	pub(crate) fn do_transfer(
		currency_id: T::AssetId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: T::Balance,
		existence_requirement: ExistenceRequirement,
		f: TransferFlags,
	) -> DispatchResult {
		if amount.is_zero() || from == to {
			return Ok(());
		}
		// Figure out the debit and credit, together with side-effects.
		let debit = Self::prep_debit(currency_id, &from, amount, f.into(), Action::Transfer)?;
		let (credit, maybe_burn) = Self::prep_credit(currency_id, &to, amount, debit, f.burn_dust)?;

		Asset::<T, I>::try_mutate(currency_id, |maybe_details| -> DispatchResult {
			let details = maybe_details.as_mut().ok_or(Error::<T, I>::Unknown)?;
			// Burn any dust if needed.
			if let Some(burn) = maybe_burn {
				// Debit dust from supply; this will not saturate since it's already checked in
				// prep.
				debug_assert!(details.supply >= burn, "checked in prep; qed");
				details.supply = details.supply.saturating_sub(burn);
			}
			Self::try_mutate_account(to, currency_id, |to_account, _existed| -> DispatchResult {
				Self::try_mutate_account(
					from,
					currency_id,
					|from_account, _existed| -> DispatchResult {
						from_account.free = from_account
							.free
							.checked_sub(&debit)
							.ok_or(Error::<T, I>::BalanceLow)?;

						// Calculate new balance; this will not saturate since it's already checked
						// in prep.
						debug_assert!(
							to_account.free.checked_add(&credit).is_some(),
							"checked in prep; qed"
						);

						// Create a new account if there wasn't one already.
						if to_account.free.is_zero() {
							to_account.sufficient = Self::new_account(&to, details)?;
						}

						to_account.free = to_account
							.free
							.checked_add(&credit)
							.ok_or(ArithmeticError::Overflow)?;
						let ed = T::ExistentialDeposits::get(&currency_id);
						// if to_account non_zero total is below existential deposit, would return
						// an error.
						ensure!(to_account.total() >= ed, Error::<T, I>::ExistentialDeposit);
						Self::ensure_can_withdraw(currency_id, from, amount)?;
						let allow_death = existence_requirement == ExistenceRequirement::AllowDeath;
						let allow_death =
							allow_death && frame_system::Pallet::<T>::can_dec_provider(from);
						// if from_account does not allow death and non_zero total is below
						// existential deposit, would return an error.
						ensure!(
							allow_death || from_account.total() >= ed,
							Error::<T, I>::KeepAlive
						);
						Ok(())
					},
				)?;
				Ok(())
			})?;
			Ok(())
		})
	}
}
