#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
use bholdus_support::{MultiCurrency, MultiCurrencyExtended};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use bholdus_chainbridge as bridge;
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement},
    };
    use frame_system::pallet_prelude::*;
    use sp_core::U256;
    use sp_runtime::SaturatedConversion;
    use sp_std::prelude::*;

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type ResourceId = bridge::ResourceId;
    type ChainId = bridge::ChainId;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + bridge::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Currency mechanism
        type Currency: Currency<Self::AccountId>;
        /// Origin are allowed to transfer from the bridge
        type BridgeOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;
        /// Origin are allowed to proceed admin operations
        type AdminOrigin: EnsureOrigin<Self::Origin>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage
    #[pallet::storage]
    #[pallet::getter(fn something)]
    // Learn more about declaring storage items:
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
    pub type NativeResourceId<T> = StorageValue<_, ResourceId, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(Default)]
    pub struct GenesisConfig {
        pub native_resource_id: ResourceId,
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            <NativeResourceId<T>>::put(self.native_resource_id);
        }
    }

    #[cfg(feature = "std")]
    impl GenesisConfig {
        /// Direct implementation of `GenesisBuild::build_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn build_storage<T: Config>(&self) -> Result<sp_runtime::Storage, String> {
            <Self as GenesisBuild<T>>::build_storage(self)
        }

        /// Direct implementation of `GenesisBuild::assimilate_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn assimilate_storage<T: Config>(
            &self,
            storage: &mut sp_runtime::Storage,
        ) -> Result<(), String> {
            <Self as GenesisBuild<T>>::assimilate_storage(self, storage)
        }
    }

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        SomethingStored(u32, T::AccountId),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        //
        // Use these to initiate crosschain transfer
        //

        /// Inititate native token crosschain transfer
        #[pallet::weight(195_000)]
        pub fn transfer_native_token(
            origin: OriginFor<T>,
            to: Vec<u8>,
            amount: BalanceOf<T>,
            dest_id: bridge::ChainId,
        ) -> DispatchResult {
            let from_id = ensure_signed(origin)?;
            let resource_id = <NativeResourceId<T>>::get();
            T::Currency::transfer(
                &from_id,
                &<bridge::Pallet<T>>::account_id(),
                amount.into(),
                ExistenceRequirement::AllowDeath,
            )?;
            <bridge::Pallet<T>>::transfer_fungible(
                dest_id,
                resource_id,
                to,
                U256::from(amount.saturated_into::<u128>()),
            )
        }

        //
        // Relayers dispatch these to process transfers from other chains
        //

        /// Release native token locked in ChainBridge pallet to user.
        /// Relayer will use the name of this call to make a proposal to ChainBridge.
        /// Only ChainBridge pallet can execute this call as a result of accepted proposal.
        #[pallet::weight(195_000_000)]
        pub fn release_native_token(
            origin: OriginFor<T>,
            to: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            T::BridgeOrigin::ensure_origin(origin)?;
            Self::_release_native_token(to, amount)?;
            Ok(())
        }

        /// Same as `release_native_token` but for admin
        /// Use this call to refund locked tokens to users.
        /// Useful in cases when errors happened and user's funds locked in pallet.
        #[pallet::weight(0)]
        pub fn admin_release_token(
            origin: OriginFor<T>,
            to: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            <T as Config>::AdminOrigin::ensure_origin(origin)?;
            Self::_release_native_token(to, amount)?;
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn _release_native_token(to: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
            let bridge_id = bridge::Pallet::<T>::account_id();
            T::Currency::transfer(
                &bridge_id,
                &to,
                amount.into(),
                ExistenceRequirement::AllowDeath,
            )
        }
    }
}
