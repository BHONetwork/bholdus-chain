#![cfg_attr(not(feature = "std"), no_std)]

use bholdus_chainbridge as bridge;
use bholdus_primitives::{Balance, CurrencyId};
use bholdus_support::MultiCurrency;
use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement},
    transactional,
};
use frame_system::pallet_prelude::*;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;
use sp_core::U256;
use sp_runtime::SaturatedConversion;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
    pub type ResourceId = bridge::ResourceId;
    pub type ChainId = bridge::ChainId;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + bridge::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Currency mechanism
        type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;
        /// Origin are allowed to transfer from the bridge
        type BridgeOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;
        /// Origin are allowed to proceed admin operations
        type AdminOrigin: EnsureOrigin<Self::Origin>;
        /// What is Native currency id
        type NativeCurrencyId: Get<CurrencyId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Mapping from CurrencyId to ResourceId
    #[pallet::storage]
    #[pallet::getter(fn currency_id_to_resource_id)]
    pub type CurrencyIdToResourceId<T> =
        StorageMap<_, Blake2_128Concat, CurrencyId, ResourceId, OptionQuery>;

    /// Mapping from ResourceId to CurrencyId
    #[pallet::storage]
    #[pallet::getter(fn resource_id_to_currency_id)]
    pub type ResourceIdToCurrencyId<T> =
        StorageMap<_, Blake2_128Concat, ResourceId, CurrencyId, OptionQuery>;

    #[pallet::genesis_config]
    #[derive(Default)]
    pub struct GenesisConfig {}

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {}
    }

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Resource id is registered successfully. [resource_id, currency_id]
        ResourceIdRegistered(ResourceId, CurrencyId),
        /// Resource id is unregistered successfully. [resource_id, currency_id]
        ResourceIdUnregistered(ResourceId, CurrencyId),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Resource id not registered
        ResourceIdNotRegistered,
        /// Resource id is already registered
        ResourceIdAlreadyRegistered,
        /// Invalid Destination Chain Id, maybe whitelisted is needed?
        InvalidDestChainId,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register resource id with currency id
        #[pallet::weight(195_000)]
        #[transactional]
        pub fn register_resource_id(
            origin: OriginFor<T>,
            resource_id: ResourceId,
            currency_id: CurrencyId,
        ) -> DispatchResult {
            <T as Config>::AdminOrigin::ensure_origin(origin)?;
            let maybe_resource_id = CurrencyIdToResourceId::<T>::get(currency_id);
            ensure!(
                maybe_resource_id.is_none(),
                Error::<T>::ResourceIdAlreadyRegistered
            );
            CurrencyIdToResourceId::<T>::insert(currency_id, resource_id);
            ResourceIdToCurrencyId::<T>::insert(resource_id, currency_id);

            Self::deposit_event(Event::ResourceIdRegistered(resource_id, currency_id));

            Ok(())
        }

        /// Unregister resource id
        #[pallet::weight(195_000)]
        #[transactional]
        pub fn remove_resource_id(origin: OriginFor<T>, resource_id: ResourceId) -> DispatchResult {
            <T as Config>::AdminOrigin::ensure_origin(origin)?;
            let maybe_currency_id = ResourceIdToCurrencyId::<T>::get(resource_id);
            ensure!(
                maybe_currency_id.is_some(),
                Error::<T>::ResourceIdNotRegistered
            );
            let currency_id = maybe_currency_id.unwrap();

            CurrencyIdToResourceId::<T>::remove(currency_id);
            ResourceIdToCurrencyId::<T>::remove(resource_id);

            Self::deposit_event(Event::ResourceIdUnregistered(resource_id, currency_id));

            Ok(())
        }

        /// Initiate token transfer
        #[pallet::weight(195_000)]
        #[transactional]
        pub fn transfer_to_bridge(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            dest_id: ChainId,
            to: Vec<u8>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;
            Self::do_transfer_to_bridge(&from, currency_id, dest_id, to, amount)?;
            Ok(())
        }

        #[pallet::weight(195_000)]
        #[transactional]
        pub fn transfer_native_to_bridge(
            origin: OriginFor<T>,
            dest_chain_id: ChainId,
            recipient: Vec<u8>,
            amount: Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_transfer_to_bridge(
                &who,
                T::NativeCurrencyId::get(),
                dest_chain_id,
                recipient,
                amount,
            )?;
            Ok(())
        }

        /// Transfer token locked in ChainBridge pallet to user.
        /// Relayer will use the name of this call to make a proposal to ChainBridge.
        /// Only ChainBridge pallet can execute this call as a result of accepted proposal.
        #[pallet::weight(195_000_000)]
        #[transactional]
        pub fn transfer_from_bridge(
            origin: OriginFor<T>,
            to: T::AccountId,
            amount: BalanceOf<T>,
            resource_id: ResourceId,
        ) -> DispatchResult {
            T::BridgeOrigin::ensure_origin(origin)?;
            let currency_id = Self::resource_id_to_currency_id(resource_id)
                .ok_or(Error::<T>::ResourceIdNotRegistered)?;

            if Self::is_origin_chain_resource(resource_id) {
                // Unlocking the tokens for resource orginated from bholdus
                T::Currency::transfer(
                    currency_id,
                    &<bridge::Pallet<T>>::account_id(),
                    &to,
                    amount,
                )?;
            } else {
                // Mint to tokens for foregin tokens since we don't know beforehand how many tokens
                // are there
                T::Currency::deposit(currency_id, &to, amount)?;
            }

            Ok(())
        }

        /// Same as `transfer_from_bridge` but for admin
        /// Use this call to refund locked tokens to users.
        /// Useful in cases when errors happened and user's funds locked in pallet.
        #[pallet::weight(0)]
        #[transactional]
        pub fn admin_transfer_from_bridge(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            to: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            <T as Config>::AdminOrigin::ensure_origin(origin)?;
            T::Currency::transfer(currency_id, &<bridge::Pallet<T>>::account_id(), &to, amount)?;

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn do_transfer_to_bridge(
        from: &T::AccountId,
        currency_id: CurrencyId,
        dest_id: ChainId,
        to: Vec<u8>,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        ensure!(
            <bridge::Pallet<T>>::chain_whitelisted(dest_id),
            Error::<T>::InvalidDestChainId
        );
        let resource_id = Self::currency_id_to_resource_id(currency_id)
            .ok_or(Error::<T>::ResourceIdNotRegistered)?;

        if Self::is_origin_chain_resource(resource_id) {
            // If resource's origin is from bholdus, we use locking mechanism
            T::Currency::transfer(
                currency_id,
                &from,
                &<bridge::Pallet<T>>::account_id(),
                amount.into(),
            )?;
        } else {
            // If resource's origin is from another chain (Binance Smart Chain), burn the tokens
            T::Currency::withdraw(currency_id, from, amount)?;
        }

        <bridge::Pallet<T>>::transfer_fungible(
            dest_id,
            resource_id,
            to,
            U256::from(amount.saturated_into::<u128>()),
        )?;
        Ok(())
    }

    pub fn is_origin_chain_resource(resource_id: ResourceId) -> bool {
        let chain_id = <T as bholdus_chainbridge::Config>::ChainId::get();
        resource_id[31] == chain_id
    }
}
