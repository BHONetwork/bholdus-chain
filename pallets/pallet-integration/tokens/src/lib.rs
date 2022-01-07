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

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;
// pub mod weights;
// pub use weights::WeightInfo;

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
        // type WeightInfo: WeightInfo;
    }

    // Some const value to compare inputs of unknown size to

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
        /// Some assets were transferred. \[asset_id, owner, total_supply\]
        Transferred(CurrencyId, T::AccountId, T::AccountId, Balance),
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
        #[pallet::weight(0)]
        pub fn transfer(
            origin: OriginFor<T>,
            dest: <T::Lookup as StaticLookup>::Source,
            currency_id: CurrencyId,
            amount: Balance,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;
            let to = T::Lookup::lookup(dest)?;
            <T as pallet::Config>::Currency::transfer(currency_id, &from, &to, amount);
            Self::deposit_event(Event::Transferred(currency_id, from, to, amount));
            Ok(())
        }
        
    }
}

