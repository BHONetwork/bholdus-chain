#![cfg_attr(not(feature = "std"), no_std)]
use bholdus_primitives::{Balance, TokenId as CurrencyId};
use bholdus_support::MultiCurrency;
use frame_support::{inherent::Vec, pallet_prelude::*, traits::Currency};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use pallet_contracts::chain_extension::UncheckedFrom;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, CheckedSub, MaybeSerializeDeserialize, StaticLookup, Zero},
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

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + integration_tokens::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {}

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
            integration_tokens::Pallet::<T>::transfer(origin, dest, currency_id, amount)
        }
    }
}
