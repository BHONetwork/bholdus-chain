#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

use bholdus_primitives::{Balance, ExchangeRate};
use sp_runtime::{
    traits::{StaticLookup, AtLeast32BitUnsigned, Zero}
};
use sp_std::{
    vec::Vec,
    convert::TryInto
};
use frame_support::{
    traits::{Currency, ExistenceRequirement},
    PalletId,
    log,
    transactional,
};

use sp_runtime::{
    traits::{AccountIdConversion},
    ArithmeticError, FixedPointNumber,
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;
pub use types::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config
        + bholdus_tokens::Config<Balance = Balance>
    {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Admin Origin
        type AdminOrigin: EnsureOrigin<Self::Origin>;
        
        type PalletId: Get<PalletId>;

        /// Native Currency trait
        type CurrencyTrait: Currency<Self::AccountId, Balance = Balance>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
    // https://docs.substrate.io/v3/runtime/storage

    #[pallet::storage]
    #[pallet::getter(fn pool)]
    pub type Pool<T: Config> = StorageMap<
        _, 
        Blake2_128Concat,
        T::AssetId,
        PoolInfo<T::AccountId>,
        OptionQuery
    >;

    /// Admin MTS.
    /// Only Admin MTS can create a digital token
    #[pallet::storage]
    #[pallet::getter(fn admin_mts)]
    pub(super) type AdminMTS<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Some asset class was created. \[asset_id, pool_info\]
        Created(T::AssetId, PoolInfo::<T::AccountId>),
        /// Deposit success. \[currency_id, who, amount\]
        Deposited(T::AssetId, T::AccountId, Balance),
        /// Withdraw success. \[currency_id, who, amount\]
        Withdrawn(T::AssetId, T::AccountId, Balance),
        /// Change rate success. \[asset_id, pool_info\]
        ChangedRate(T::AssetId, PoolInfo::<T::AccountId>),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// The operator is not the owner of the token and has no permission
        NoPermission,
        /// Account balance must be greater than or equal to the transfer amount.
        BalanceLow
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_: T::BlockNumber) -> Weight {
            0
        }

        fn on_finalize(_n: <T as frame_system::Config>::BlockNumber) {}

        fn on_runtime_upgrade() -> Weight {
            0
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account()
        }

        fn sub_account_id(id: T::AssetId) -> T::AccountId {
            T::PalletId::get().into_sub_account(id)
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {

        #[pallet::weight(0)]
        #[transactional]
        pub fn create(
            origin: OriginFor<T>,
            name: Vec<u8>,
            symbol: Vec<u8>,
            decimals: u8,
            partner_admin: <T::Lookup as StaticLookup>::Source,
            rate: Rate,
        ) -> DispatchResult { 

            let who = ensure_signed(origin)?;

            // Only admin MTS are allowed
            ensure!(
                Self::admin_mts(who.clone()),
                Error::<T>::NoPermission
            );

            ExchangeRate::checked_from_rational(rate.numerator, rate.denominator)
                    .ok_or(ArithmeticError::Overflow)?;   

            ExchangeRate::checked_from_rational(rate.denominator, rate.numerator)
                    .ok_or(ArithmeticError::Overflow)?;

            let partner_admin = T::Lookup::lookup(partner_admin)?;
            let asset_id = bholdus_tokens::Pallet::<T>::next_asset_id();
            let address = Pallet::<T>::sub_account_id(asset_id);
            let result = bholdus_tokens::Pallet::<T>::do_create(
                &partner_admin,
                name,
                symbol,
                decimals,
                &partner_admin,
            );

            if result.is_ok() {
                let pool_info = PoolInfo {
                    rate,
                    account_pool: address,
                    partner_admin,
                };
                Pool::<T>::insert(asset_id, &pool_info);
                Self::deposit_event(Event::Created(asset_id, pool_info));
            }
            Ok(())
        }

        #[pallet::weight(0)]
        #[transactional]
        pub fn deposit(
            origin: OriginFor<T>,
            amount: Balance,
            asset_id: T::AssetId,
        ) -> DispatchResult {

            let who = ensure_signed(origin)?;

            let pool_info = Pool::<T>::get(&asset_id).unwrap();

            ExchangeRate::checked_from_rational(pool_info.rate.denominator, pool_info.rate.numerator)
                    .ok_or(ArithmeticError::Overflow)?;

            let mint_amount = amount
                                .saturating_mul(pool_info.rate.denominator)
                                .checked_div(pool_info.rate.numerator)
                                .and_then(|n| TryInto::<Balance>::try_into(n).ok())
                                .unwrap_or_else(Zero::zero);

            let result = bholdus_tokens::Pallet::<T>::do_mint(
                asset_id,
                &who,
                mint_amount,
                None,
            );

            if result.is_ok() {
                // Lock user tokens
                T::CurrencyTrait::transfer(
                    &who,
                    &pool_info.account_pool,
                    amount,
                    ExistenceRequirement::AllowDeath,
                )?;
                Self::deposit_event(Event::Deposited(asset_id, who, amount));
            }
            Ok(())
        }

        #[pallet::weight(0)]
        #[transactional]
        pub fn withdraw(
            origin: OriginFor<T>,
            amount: Balance,
            asset_id: T::AssetId,
        ) -> DispatchResult {
            let f = bholdus_tokens::DebitFlags {
                keep_alive: false,
                best_effort: true,
            };

            let who = ensure_signed(origin)?;

            let pool_info = Pool::<T>::get(&asset_id).unwrap();

            ExchangeRate::checked_from_rational(pool_info.rate.denominator, pool_info.rate.numerator)
                    .ok_or(ArithmeticError::Overflow)?;

            let burn_amount = amount 
                            .saturating_mul(pool_info.rate.denominator)
                            .checked_div(pool_info.rate.numerator)
                            .and_then(|n| TryInto::<Balance>::try_into(n).ok())
                            .unwrap_or_else(Zero::zero);

            let result = bholdus_tokens::Pallet::<T>::do_burn(
                asset_id,
                &who,
                burn_amount,
                None,
                f
            );
            
            if result.is_ok() {
                // Release user tokens
                T::CurrencyTrait::transfer(
                    &pool_info.account_pool,
                    &who,
                    amount,
                    ExistenceRequirement::AllowDeath,
                )?;
                Self::deposit_event(Event::Withdrawn(asset_id, who, amount));
            }
            
            Ok(())
        }

        #[pallet::weight(0)]
        #[transactional]
        pub fn change_rate(
            origin: OriginFor<T>,
            rate: Rate,
            asset_id: T::AssetId
        ) -> DispatchResult {

            ExchangeRate::checked_from_rational(rate.numerator, rate.denominator)
                    .ok_or(ArithmeticError::Overflow)?;   

            ExchangeRate::checked_from_rational(rate.denominator, rate.numerator)
                    .ok_or(ArithmeticError::Overflow)?;

            let who = ensure_signed(origin)?;
            let mut pool_info = Pool::<T>::get(&asset_id).unwrap();
            let total_current_mts = T::CurrencyTrait::free_balance(&pool_info.account_pool);
            
            if total_current_mts.is_zero() {

                // Only admin MTS are allowed
                ensure!(
                    Self::admin_mts(who.clone()),
                    Error::<T>::NoPermission
                );

                pool_info.rate = rate;
                Pool::<T>::insert(asset_id, &pool_info);
                Self::deposit_event(Event::ChangedRate(asset_id, pool_info));
            } 
            else {

                ensure!(who == pool_info.partner_admin, Error::<T>::NoPermission);

                ExchangeRate::checked_from_rational(pool_info.rate.denominator, pool_info.rate.numerator)
                    .ok_or(ArithmeticError::Overflow)?;

                let total_loyalty = total_current_mts
                                    .saturating_mul(pool_info.rate.denominator)
                                    .checked_div(pool_info.rate.numerator)
                                    .and_then(|n| TryInto::<Balance>::try_into(n).ok())
                                    .unwrap_or_else(Zero::zero);

                let total_new_pool_mts = total_loyalty
                                    .saturating_mul(rate.numerator)
                                    .checked_div(rate.denominator)
                                    .and_then(|n| TryInto::<Balance>::try_into(n).ok())
                                    .unwrap_or_else(Zero::zero);

                if total_new_pool_mts > total_current_mts {
                    let deposit_amount = total_new_pool_mts.checked_sub(total_current_mts)
                                                    .and_then(|n| TryInto::<Balance>::try_into(n).ok())
                                                    .unwrap_or_else(Zero::zero);

                    T::CurrencyTrait::transfer(
                        &who,
                        &pool_info.account_pool,
                        deposit_amount,
                        ExistenceRequirement::AllowDeath,
                    )?;

                    pool_info.rate = rate;
                    Pool::<T>::insert(asset_id, &pool_info);

                    Self::deposit_event(Event::ChangedRate(asset_id, pool_info));
                } 
                else if total_new_pool_mts < total_current_mts {
                    let withdraw_amount = total_current_mts.checked_sub(total_new_pool_mts)                                
                                                    .and_then(|n| TryInto::<Balance>::try_into(n).ok())
                                                    .unwrap_or_else(Zero::zero);

                    T::CurrencyTrait::transfer(
                        &pool_info.account_pool,
                        &who,
                        withdraw_amount,
                        ExistenceRequirement::AllowDeath,
                    )?;

                    pool_info.rate = rate;
                    Pool::<T>::insert(asset_id, &pool_info);

                    Self::deposit_event(Event::ChangedRate(asset_id, pool_info));
                }
            }
            Ok(())
        }

        /// Set partner admin
        /// Only `AdminMTS` can access this operation
        #[pallet::weight(0)]
        #[transactional]
        pub fn set_partner_admin(
            origin: OriginFor<T>,
            asset_id: T::AssetId,
            partner_admin: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResult { 

            let who = ensure_signed(origin)?;

            // Only admin MTS are allowed
            ensure!(
                Self::admin_mts(who.clone()),
                Error::<T>::NoPermission
            );

            let mut pool_info = Pool::<T>::get(&asset_id).unwrap();
            let partner_admin = T::Lookup::lookup(partner_admin)?;

            pool_info.partner_admin = partner_admin;
            Pool::<T>::insert(asset_id, &pool_info);

            Ok(())
        }

        /// Register admin MTS account
        /// Only `AdminOrigin` can access this operation
        #[pallet::weight(0)]
        #[transactional]
        pub fn force_register_admin_mts(
            origin: OriginFor<T>,
            relayer: T::AccountId,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            AdminMTS::<T>::insert(relayer, true);

            Ok(())
        }

        /// Unregister admin MTS account
        /// Only `AdminOrigin` can access this operation
        #[pallet::weight(0)]
        #[transactional]
        pub fn force_unregister_admin_mts(
            origin: OriginFor<T>,
            relayer: T::AccountId,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            AdminMTS::<T>::insert(relayer, false);

            Ok(())
        }
    }
}



