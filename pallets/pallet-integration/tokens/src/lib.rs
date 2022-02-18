#![cfg_attr(not(feature = "std"), no_std)]
use bholdus_primitives::{Balance, TokenId as CurrencyId};
use bholdus_support::MultiCurrency;
use frame_support::{inherent::Vec, pallet_prelude::*, traits::Currency};
use frame_system::pallet_prelude::*;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use pallet_contracts::chain_extension::UncheckedFrom;
use sp_runtime::{
    traits::{CheckedSub, MaybeSerializeDeserialize, StaticLookup, Zero},
    DispatchError, DispatchResult,
};

use sp_core::Bytes;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
pub mod weights;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    type BalanceOf<T> = <<T as pallet_contracts::Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_contracts::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;
        type WeightInfo: WeightInfo;
    }

    // Some const value to compare inputs of unknown size to
    pub const MAX_LENGTH: usize = 50;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn get_value)]
    pub(super) type ContractEntry<T> = StorageValue<_, Vec<u8>, ValueQuery>;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event to display when call is made from extrinsic to a smart contract
        CalledContractFromPallet(T::AccountId),
        /// Event to display when call is made from a smart contract to the extrinsic
        CalledPalletFromContract(u32),
        /// Some assets were transferred. \[asset_id, owner, total_supply\]
        Transferred(CurrencyId, T::AccountId, T::AccountId, Balance),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        InputTooLarge,
        InvalidAmount,
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
        /// This can be retrived from the compiled `metadata.json`.
        /// * `arg` - An argument to be passed to the smart contract.
        /// * `gas_limit` - The gas limit passed to the smart contract bare_call.
        pub fn call_smart_contract(
            origin: OriginFor<T>,
            dest: T::AccountId,
            mut selector: Vec<u8>,
            arg: u8,
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
            let result = pallet_contracts::Pallet::<T>::bare_call(
                who,
                dest.clone(),
                value,
                gas_limit,
                data,
                false,
            )
            .result
            .unwrap();
            let val: Vec<u8> = result.data.to_vec();
            ContractEntry::<T>::put(val);
            Self::deposit_event(Event::CalledContractFromPallet(dest));
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn transfer(
            origin: OriginFor<T>,
            dest: <T::Lookup as StaticLookup>::Source,
            currency_id: CurrencyId,
            amount: Balance,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;
            let to = T::Lookup::lookup(dest)?;
            ensure!(amount > 0, Error::<T>::InvalidAmount);
            <T as pallet::Config>::Currency::transfer(currency_id, &from, &to, amount);
            Self::deposit_event(Event::Transferred(currency_id, from, to, amount));
            Ok(())
        }
    }
}
