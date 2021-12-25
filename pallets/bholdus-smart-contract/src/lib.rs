#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::{
		dispatch::DispatchResult, inherent::Vec, pallet_prelude::*, traits::Currency,
	};
	use frame_system::pallet_prelude::*;
	use pallet_contracts::chain_extension::UncheckedFrom;

	type BalanceOf<T> = <<T as pallet_contracts::Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_contracts::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
		type WeightInfo: WeightInfo;
	}

	// Some const value to compare inputs of unknown size to
	pub const MAX_LENGTH: usize = 50;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_value)]
	pub(super) type ContractEntry<T> = StorageValue<_, u32, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event to display when call is made from the extrinsic to a smart contract
		CalledContractFromPallet(T::AccountId),
		/// Event to display when call is made from a smart contract to the extrinsic
		CalledPalletFromContract(u32),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		InputTooLarge,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T::AccountId: UncheckedFrom<T::Hash>,
		T::AccountId: AsRef<[u8]>,
	{
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		/// A generic extrinsic to wrap
		/// [pallet_contracts::bare_call](https://github.com/paritytech/substrate/blob/352c46a648a5f2d4526e790a184daa4a1ffdb3bf/frame/contracts/src/lib.rs#L545-L562)
		///
		/// * `dest` - A destination account id for the contract being targeted
		/// * `selector` - The 'selector' of the ink! smart contract function.
		/// This can be retrived from the compiled `metadata.json`. It's possible to
		/// [specify a selector](https://paritytech.github.io/ink-docs/macros-attributes/selector/) in
		/// the smart contract itself.
		/// * `arg` - An argument to be passed to the smart contract.
		/// * `gas_limit` - The gas limit passed to the contract bare_call. This example should work
		///   when given a value of around 10000000000
		pub fn call_smart_contract(
			origin: OriginFor<T>,
			dest: T::AccountId,
			mut selector: Vec<u8>,
			arg: u32,
			#[pallet::compact] gas_limit: Weight,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// Check against unbounded input
			ensure!(selector.len() < MAX_LENGTH, Error::<T>::InputTooLarge);
			// Amount to transfer
			let value: BalanceOf<T> = Default::default();
			let mut arg_enc: Vec<u8> = arg.encode();
			let mut data = Vec::new();
			data.append(&mut selector);
			data.append(&mut arg_enc);

			// Do the actual call to the smart contract function
			pallet_contracts::Pallet::<T>::bare_call(
				who,
				dest.clone(),
				value,
				gas_limit,
				data,
				false,
			)
			.result?;

			Self::deposit_event(Event::CalledContractFromPallet(dest));
			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::insert_number(*val))]
		/// A storage extrinsic for demonstrating calls originating from a smart contract
		/// * `val` - Some integer to be stored.
		pub fn insert_number(origin: OriginFor<T>, val: u32) -> DispatchResult {
			ensure_signed(origin)?;
			// Do something with the value
			ContractEntry::<T>::put(val);
			Self::deposit_event(Event::CalledPalletFromContract(val));
			Ok(())
		}
	}
}
