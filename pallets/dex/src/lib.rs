#![cfg_attr(not(feature = "std"), no_std)]

use bholdus_primitives::{Balance, CurrencyId, TradingPair};
use bholdus_support::traits::MultiCurrency;
use frame_support::{dispatch::DispatchResult, pallet_prelude::*, transactional, PalletId};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use sp_core::U256;
use sp_runtime::{
    traits::{AccountIdConversion, One, Zero},
    ArithmeticError,
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(Encode, Decode, Default, Clone, PartialRq, RuntimeDebug)]
pub struct ProvisioningParameters {
    pub min_contribution: (Balance, Balance),
    pub target_contribution: (Balance, Balance),
}

#[derive(Encode, Decode, Default, Clone, PartialEq, RuntimeDebug)]
pub enum TradingPairStatus {
    Disabled,
    Provisioning,
    Enabled,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The origin that may disable, enable, listing provisioning trading pair
        type ListingOrigin: EnsureOrigin<Self::Origin>;
        /// Pallet Id turned into Account Id to hold tokens
        #[pallet::constant]
        type PalletId: Get<PalletId>;
        #[pallet::constant]
        /// Exchange fee. Use seperate numerator, denominator to achieve result more accurate
        type ExchangeFee: Get<(u32, u32)>;
        /// Trading path limit
        type TradingPath: Get<u32>;
        /// Multi currency mechanism
        type Currency: MultiCurrency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Reserves of trading pairs
    #[pallet::storage]
    #[pallet::getter(fn liquidity_pool)]
    pub type LiquidityPool<T: Config> =
        StorageMap<_, Blake2_128Concat, TradingPair, (Balance, Balance), ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn trading_pair_statuses)]
    pub type TradingPairStatuses<T: Config> =
        StorageMap<_, Blake2_128Concat, TradingPair, TradingPairStatus, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn provisioning_pool)]
    pub type ProvisioningPool<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        TradingPair,
        T::AccountId,
        (Balance, Balance),
        ValueQuery,
    >;

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
        /// Currency provided is not supported for swapping.
        /// Only "Token" currency are supported.
        InvalidCurrencyId,
        /// Target amount returned by a swap not reach minimum target amount.
        InsufficientTargetAmount,
        /// Target amount is zero.
        /// Error is thrown when:
        /// - Amount user receives after swapping calculation is nothing.
        ZeroTargetAmount,
        /// Insufficient liqudity pool for swapping.
        InsufficientLiquidity,
        /// Trading path limit is reached.
        InvalidTradingPathLength,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Trading with DEX, swap with exact supply amount
        ///
        /// - `path`: trading path
        /// - `supply_amount`: exact supply amount
        /// - `min_target_amount`: acceptable target amount
        #[pallet::weight(0)]
        #[transactional]
        pub fn swap_with_exact_supply(
            origin: OriginFor<T>,
            path: Vec<CurrencyId>,
            #[pallet::compact] supply_amount: Balance,
            #[pallet::compact] min_target_amount: Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_swap_with_exact_supply(&who, &path, supply_amount, min_target_amount)?;
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    /// Implementation of `swap_with_exact_supply`
    ///
    /// Returns a `Result`:
    /// - `target_amount`: calculated target amount when successful
    /// - `error`: dispatch error when failed
    #[transactional]
    pub fn do_swap_with_exact_supply(
        who: &T::AccountId,
        path: &[CurrencyId],
        supply_amount: Balance,
        min_target_amount: Balance,
    ) -> Result<Balance, DispatchError> {
        let target_amounts = Self::get_target_amounts(path, supply_amount)?;
        ensure!(
            target_amount[target_amounts.len() - 1] >= min_target_amount,
            Error::<T>::InsufficientTargetAmount
        );
        let pallet_account_id = Self::account_id();
        let actual_target_amount = target_amounts[target_amounts.len() - 1];
        T::Currency::transfer(path[0], who, &pallet_account_id, supply_amount)?;
        Self::_swap_by_path(&path, &amounts)?;
        T::Currency::transfer(
            path[path.len() - 1],
            &pallet_account_id,
            who,
            actual_target_amount,
        )?;

        // TODO: Define Event::Swap
        Self::deposit_event(Event::Swap());
        Ok(actual_target_amount)
    }

    /// Returns reserves of two currencies
    pub fn get_liquidity(
        currency_id_0: &CurrencyId,
        currency_id_1: &CurrencyId,
    ) -> (Balance, Balance) {
        if let Some(trading_pair) =
            TradingPair::from_currency_ids(currency_id_0.clone(), currency_id_1.clone())
        {
            let (pool_0, pool_1) = Self::liquidity_pool(trading_pair);
            if currency_id_0 == trading_pair.first() {
                (pool_0, pool_1)
            } else {
                (pool_1, pool_0)
            }
        } else {
            (Zero::zero(), Zero::zero())
        }
    }

    /// Returns target amount,
    /// given supply reserves, target reserves, supply amount
    pub fn get_target_amount(
        supply_pool: Balance,
        target_pool: Balance,
        supply_amount: Balance,
    ) -> Balance {
        if supply_pool.is_zero() || target_pool.is_zero() || supply_amount.is_zero() {
            Zero::zero()
        } else {
            // The following formula is actually the original formula with simplification process
            let (fee_numerator, fee_denominator) = T::ExchangeFee::get();
            let supply_amount_with_fee = U256::from(supply_amount)
                .saturating_mul(U256::from(fee_denominator.saturating_sub(fee_numerator)));
            let numerator =
                U256::from(supply_amount_with_fee).saturating_mul(U256::from(target_pool));
            let denominator = U256::from(fee_denominator)
                .saturating_mul(U256::from(supply_pool))
                .saturating_add(U256::from(supply_amount_with_fee));

            numerator
                .checked_div(denominator)
                .and_then(|n| TryInto::<Balance>::try_into(n).ok())
                .unwrap_or_else(Zero::zero)
        }
    }

    /// Returns list of target amounts with each is the target amount of a trading pair
    /// of given trading path
    pub fn get_target_amounts(
        path: &[CurrencyId],
        supply_amount: Balance,
    ) -> Result<Vec<Balance>, DispatchError> {
        let path_length = path.len();
        ensure!(
            path_length >= 2 && path_length <= T::TradingPath::get().saturated_into(),
            Error::<T>::InvalidTradingPathLength
        );

        let mut target_amounts: Vec<Balance> = vec![Zero::zero(); path_length];
        target_amounts[0] = supply_amount;

        let mut i: usize = 0;
        while i + 1 < path_length {
            let trading_pair = TradingPair::from_currency_ids(path[i], path[i + 1])
                .ok_or(Error::<T>::InvalidCurrencyId)?;
            ensure!(
                matches!(
                    Self::trading_pair_statuses(trading_pair),
                    TradingPairStatus::Enabled
                ),
                Error::<T>::TradingPairMustBeEnabled
            );

            let (supply_pool, target_pool) = Self::get_liquidity(&path[i], &path[i + 1]);
            ensure!(
                !supply_pool.is_zero() && !target_pool.is_zero(),
                Error::<T>::InsufficientLiquidity
            );

            let target_amount = Self::get_target_amount(supply_pool, target_pool, supply_amount);
            ensure!(!target_amount.is_zero(), Error::<T>::ZeroTargetAmount);

            target_amounts[i + 1] = target_amount;
            i += 1;
        }

        return target_amounts;
    }

    /// Do storage changes for swapping a trading pair
    fn _swap(
        supply_currency: CurrencyId,
        target_currency: CurrencyId,
        supply_increment: Balance,
        target_decrement: Balance,
    ) -> DispatchResult {
        if let Some(trading_pair) =
            TradingPair::from_currency_ids(supply_currency.clone(), target_currency.clone())
        {
            LiquidityPool::<T>::try_mutate(trading_pair, |(pool_0, pool_1)| {
                let invariant_before_swap = U256::from(*pool_0).saturaing_mul(U256::from(*pool_1));

                if supply_currency_id == trading_pair.first() {
                    *pool_0 = pool_0
                        .checked_add(supply_increment)
                        .ok_or(ArithmeticError::Overflow)?;
                    *pool_1 = pool_1
                        .checked_sub(target_decrement)
                        .ok_or(ArithmeticError::Underflow)?;
                } else {
                    *pool_0 = pool_0
                        .checked_sub(target_decrement)
                        .ok_or(ArithmeticError::Underflow)?;
                    *pool_1 = pool_1
                        .checked_add(supply_increment)
                        .ok_or(ArithmeticError::Overflow)?;
                }

                let invariant_after_swap = U256::from(*pool_0).saturating_mul(U256::from(*pool_1));
                ensure!(
                    invariant_after_swap >= invariant_before_swap,
                    Error::<T>::InvariantAfterCheckFailed
                );

                Ok(())
            })?;
        }
        Ok(())
    }

    /// Do storage changes for swapping given trading path
    fn _swap_by_path(path: &[CurrencyId], amounts: &[Balance]) -> DispatchResult {
        let mut i: usize = 0;
        while i + 1 < path.len() {
            let supply_currency = path[i];
            let target_currency = path[i + 1];
            let supply_increment = amounts[i];
            let target_decrement = amounts[i + 1];

            Self::_swap(
                supply_currency,
                target_currency,
                supply_increment,
                target_decrement,
            )?;

            i += 1;
        }
        Ok(())
    }
}
