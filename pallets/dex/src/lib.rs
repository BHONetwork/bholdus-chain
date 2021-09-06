#![cfg_attr(not(feature = "std"), no_std)]

use bholdus_primitives::{Balance, CurrencyId, ExchangeRate, Ratio, TradingPair};
use bholdus_support::MultiCurrency;
use frame_support::{
    dispatch::DispatchResult, log, pallet_prelude::*, transactional, weights::DispatchClass,
    PalletId,
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use sp_core::U256;
use sp_runtime::{
    traits::{AccountIdConversion, One, Zero},
    ArithmeticError, FixedPointNumber, SaturatedConversion,
};
use sp_std::{convert::TryInto, prelude::*, vec};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ProvisioningParameters<Balance> {
    pub min_contribution: (Balance, Balance),
    pub target_contribution: (Balance, Balance),
    pub accumulated_contribution: (Balance, Balance),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum TradingPairStatus<Balance> {
    Disabled,
    Provisioning(ProvisioningParameters<Balance>),
    Enabled,
}

impl<Balance> Default for TradingPairStatus<Balance> {
    fn default() -> Self {
        Self::Disabled
    }
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
        /// Exchange fee. Use seperate numerator, denominator to achieve result more accurate
        #[pallet::constant]
        type ExchangeFee: Get<(u32, u32)>;
        /// Trading path limit
        #[pallet::constant]
        type TradingPathLimit: Get<u32>;
        /// Multi currency mechanism
        type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Reserves of trading pairs.
    #[pallet::storage]
    #[pallet::getter(fn liquidity_pool)]
    pub type LiquidityPool<T: Config> =
        StorageMap<_, Blake2_128Concat, TradingPair, (Balance, Balance), ValueQuery>;

    /// Status of trading pairs.
    #[pallet::storage]
    #[pallet::getter(fn trading_pair_statuses)]
    pub type TradingPairStatuses<T: Config> =
        StorageMap<_, Blake2_128Concat, TradingPair, TradingPairStatus<Balance>, ValueQuery>;

    /// Provision of each user added to a trading pair when that trading pair is in `provisioning`
    /// status.
    #[pallet::storage]
    #[pallet::getter(fn provisioning_pool)]
    pub type ProvisioningPool<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        TradingPair,
        Blake2_128Concat,
        T::AccountId,
        (Balance, Balance),
        ValueQuery,
    >;

    /// Initial share exchange rates for each trading pair.
    /// Used to calculate share amount for first liqudity providers.
    #[pallet::storage]
    #[pallet::getter(fn initial_share_exchange_rates)]
    pub type InitialShareExchangeRate<T: Config> =
        StorageMap<_, Blake2_128Concat, TradingPair, (ExchangeRate, ExchangeRate), ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_provisioning_trading_pairs:
            Vec<(TradingPair, (Balance, Balance), (Balance, Balance))>,
        pub initial_enabled_trading_pairs: Vec<TradingPair>,
        pub initial_added_liquidity_pools:
            Vec<(T::AccountId, Vec<(TradingPair, (Balance, Balance))>)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig {
                initial_provisioning_trading_pairs: vec![],
                initial_enabled_trading_pairs: vec![],
                initial_added_liquidity_pools: vec![],
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            self.initial_provisioning_trading_pairs.iter().for_each(
                |(trading_pair, min_contribution, target_contribution)| {
                    TradingPairStatuses::<T>::insert(
                        trading_pair,
                        TradingPairStatus::<_>::Provisioning(ProvisioningParameters {
                            min_contribution: *min_contribution,
                            target_contribution: *target_contribution,
                            accumulated_contribution: Default::default(),
                        }),
                    );
                },
            );

            self.initial_enabled_trading_pairs
                .iter()
                .for_each(|trading_pair| {
                    TradingPairStatuses::<T>::insert(trading_pair, TradingPairStatus::<_>::Enabled);
                });

            self.initial_added_liquidity_pools
                .iter()
                .for_each(|(who, trading_pairs_data)| {
                    trading_pairs_data.iter().for_each(
                        |(trading_pair, (deposit_amount_0, deposit_amount_1))| {
                            let result = match <Pallet<T>>::trading_pair_statuses(trading_pair) {
                                TradingPairStatus::<_>::Enabled => <Pallet<T>>::do_add_liquidity(
                                    &who,
                                    trading_pair.first(),
                                    trading_pair.second(),
                                    *deposit_amount_0,
                                    *deposit_amount_1,
                                ),
                                _ => Err(Error::<T>::TradingPairMustBeEnabled.into()),
                            };

                            assert!(result.is_ok(), "genesis add lidquidity pool failed.");
                        },
                    );
                });
        }
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Swap is successful. \[who, trading_path, supply_amount, target_amount\]
        Swap(T::AccountId, Vec<CurrencyId>, Balance, Balance),
        /// Trading pair is enabled from provisioning successfully. \[currency_id_0,
        /// pool_0_amount, currency_id_1, pool_1_amount, total_share_amount\]
        TradingPairEnabledFromProvisioning(CurrencyId, Balance, CurrencyId, Balance, Balance),
        /// Trading pair is disabled. \[trading_pair\]
        TradingPairDisabled(TradingPair),
        /// Trading pair is enabled. Either from disabled or zero accumulated provisioning trading
        /// pair. \[trading_pair\]
        TradingPairEnabled(TradingPair),
        /// Trading pair is in provisioning stage. \[trading_pair\]
        TradingPairProvisioning(TradingPair),
        /// Add provision to a trading pair successfully.
        /// \[who, currency_id_0, contribution_0, currency_id_1, contribution_1\]
        AddProvision(T::AccountId, CurrencyId, Balance, CurrencyId, Balance),
        /// Add liqudity to a trading pair successfully. \[who, currency_id_0, pool_0_increment, currency_id_1,
        /// pool_1_increment, share_increment\]
        AddLiquidity(
            T::AccountId,
            CurrencyId,
            Balance,
            CurrencyId,
            Balance,
            Balance,
        ),
        /// Remove liquidity to a trading pair successfully. \[who, currency_id_0,
        /// pool_0_decrement, currency_id_1, pool_1_decrement, share_decrement\]
        RemoveLiqudity(
            T::AccountId,
            CurrencyId,
            Balance,
            CurrencyId,
            Balance,
            Balance,
        ),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Currency provided is not supported for swapping.
        /// Only "Token" currency are supported.
        InvalidCurrencyId,
        /// Trading pair must be disabled to work.
        TradingPairMustBeDisabled,
        /// Trading pair must be in provisioning.
        TradingPairMustBeProvisioning,
        /// Trading pair must be enabled to work.
        TradingPairMustBeEnabled,
        /// Calculated target amount returned by a swap less than target amount user accepts to.
        /// receive.
        InsufficientTargetAmount,
        /// Calculated supply amount returned by a swap greater than supply amount user can pay.
        InsufficientSupplyAmount,
        /// Target amount is zero.
        ZeroTargetAmount,
        /// Supply amount is zero.
        ZeroSupplyAmount,
        /// Insufficient liqudity pool for swapping.
        InsufficientLiquidity,
        /// Trading path limit is reached.
        InvalidTradingPathLength,
        /// New invariant after swap is smaller than invariant before swap.
        /// This may happen because of arithmetic error.
        InvariantAfterCheckFailed,
        /// Provision contribution not sastisfies minimum contribution requirements.
        InvalidContributionIncrement,
        /// Invalid liquidity increment.
        InvalidLiquidityIncrement,
        /// Provisioning trading pair is not qualified to be enabled yet.
        UnqualifiedProvision,
        /// Trading pair is already enabled.
        TradingPairAlreadyEnabled,
        /// Trading pair is already provisioned so it can't be enabled.
        TradingPairAlreadyProvisioned,
        /// Remove share amount is invalid.
        InvalidRemoveShareAmount,
        /// Withdrawn amount is unacceptable by user.
        UnacceptableWithdrawnAmount,
        /// User didn't provide liquidity for this trading pair in provisioning but try to claim dex share.
        UserNotAddProvision,
        /// Total share amount is zero.
        ZeroTotalShare,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Trading with DEX, swap with exact supply amount.
        ///
        /// - `path`: trading path.
        /// - `supply_amount`: exact supply amount.
        /// - `min_target_amount`: acceptable target amount.
        ///
        /// TODO: Weight benchmarking
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

        /// Trading with DEX, swap with exact target amount.
        ///
        /// - `path`: trading path.
        /// - `target_amount`: target amount user wants to receive.
        /// - `max_supply_amount`: maximum supply amount that user willing to pay.
        ///
        /// Weight benchmarking
        #[pallet::weight(0)]
        #[transactional]
        pub fn swap_with_exact_target(
            origin: OriginFor<T>,
            path: Vec<CurrencyId>,
            #[pallet::compact] target_amount: Balance,
            #[pallet::compact] max_supply_amount: Balance,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;
            Self::do_swap_with_exact_target(&from, &path, target_amount, max_supply_amount)?;
            Ok(())
        }

        /// Start provisioning stage of a trading pair
        ///
        /// TODO: Weight benchmarking
        #[pallet::weight((0, DispatchClass::Operational))]
        #[transactional]
        pub fn start_trading_pair_provisioning(
            origin: OriginFor<T>,
            currency_id_0: CurrencyId,
            currency_id_1: CurrencyId,
            #[pallet::compact] min_contribution_0: Balance,
            #[pallet::compact] min_contribution_1: Balance,
            #[pallet::compact] target_contribution_0: Balance,
            #[pallet::compact] target_contribution_1: Balance,
        ) -> DispatchResult {
            T::ListingOrigin::ensure_origin(origin)?;
            Self::do_start_trading_pair_provisioning(
                currency_id_0,
                currency_id_1,
                min_contribution_0,
                min_contribution_1,
                target_contribution_0,
                target_contribution_1,
            )?;
            Ok(())
        }

        /// Update provisioning parameters of provisioning trading pair
        ///
        /// TODO: Weight benchmarking
        #[pallet::weight((0, DispatchClass::Operational))]
        #[transactional]
        pub fn update_trading_pair_provisioning_parameters(
            origin: OriginFor<T>,
            currency_id_0: CurrencyId,
            currency_id_1: CurrencyId,
            #[pallet::compact] min_contribution_0: Balance,
            #[pallet::compact] min_contribution_1: Balance,
            #[pallet::compact] target_contribution_0: Balance,
            #[pallet::compact] target_contribution_1: Balance,
        ) -> DispatchResult {
            T::ListingOrigin::ensure_origin(origin)?;
            Self::do_update_trading_pair_provisioning_parameters(
                currency_id_0,
                currency_id_1,
                min_contribution_0,
                min_contribution_1,
                target_contribution_0,
                target_contribution_1,
            )?;
            Ok(())
        }

        /// Add provision to a provisioning trading pair
        ///
        /// TODO: Weight benchmarking
        #[pallet::weight((0,DispatchClass::Operational))]
        #[transactional]
        pub fn add_trading_pair_provision(
            origin: OriginFor<T>,
            currency_id_0: CurrencyId,
            currency_id_1: CurrencyId,
            #[pallet::compact] contribution_0: Balance,
            #[pallet::compact] contribution_1: Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_add_trading_pair_provision(
                &who,
                currency_id_0,
                currency_id_1,
                contribution_0,
                contribution_1,
            )?;
            Ok(())
        }

        /// Enable provisioning trading pair
        ///
        /// TODO: Weight benchmarking
        #[pallet::weight((0, DispatchClass::Operational))]
        #[transactional]
        pub fn enable_provisioning_trading_pair(
            origin: OriginFor<T>,
            currency_id_0: CurrencyId,
            currency_id_1: CurrencyId,
        ) -> DispatchResult {
            T::ListingOrigin::ensure_origin(origin)?;
            Self::do_enable_provisioning_trading_pair(currency_id_0, currency_id_1)?;
            Ok(())
        }

        /// Enable trading pair from disabled or provisioning with zero accumulated provision
        ///
        /// TODO: Weight benchmarking
        #[pallet::weight((0, DispatchClass::Operational))]
        #[transactional]
        pub fn enable_trading_pair(
            origin: OriginFor<T>,
            currency_id_0: CurrencyId,
            currency_id_1: CurrencyId,
        ) -> DispatchResult {
            T::ListingOrigin::ensure_origin(origin)?;
            Self::do_enable_trading_pair(currency_id_0, currency_id_1)?;
            Ok(())
        }

        /// Add liquidity to enabled trading pair
        ///
        /// TODO: Weight benchmarking
        #[pallet::weight(0)]
        #[transactional]
        pub fn add_liquidity(
            origin: OriginFor<T>,
            currency_id_0: CurrencyId,
            currency_id_1: CurrencyId,
            #[pallet::compact] max_amount_0: Balance,
            #[pallet::compact] max_amount_1: Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_add_liquidity(
                &who,
                currency_id_0,
                currency_id_1,
                max_amount_0,
                max_amount_1,
            )?;
            Ok(())
        }

        /// Remove liquidity
        ///
        /// TODO: Weight benchmarking
        #[pallet::weight(0)]
        #[transactional]
        pub fn remove_liquidity(
            origin: OriginFor<T>,
            currency_id_0: CurrencyId,
            currency_id_1: CurrencyId,
            #[pallet::compact] remove_share_amount: Balance,
            #[pallet::compact] min_withdrawn_amount_0: Balance,
            #[pallet::compact] min_withdrawn_amount_1: Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_remove_liquidity(
                &who,
                currency_id_0,
                currency_id_1,
                remove_share_amount,
                min_withdrawn_amount_0,
                min_withdrawn_amount_1,
            )?;
            Ok(())
        }

        /// Claim dex share. Founders that add provision to trading pair in provisioning stage can
        /// use this call to claim their dex share
        #[pallet::weight(0)]
        #[transactional]
        pub fn claim_dex_share(
            origin: OriginFor<T>,
            owner: T::AccountId,
            currency_id_0: CurrencyId,
            currency_id_1: CurrencyId,
        ) -> DispatchResult {
            ensure_signed(origin)?;
            Self::do_claim_dex_share(&owner, currency_id_0, currency_id_1)?;
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    /// Get share and pool increment
    ///
    /// - `pool_0`: current liquidity pool of currency_0
    /// - `pool_1`: current liquidity pool of currency_1
    /// - `max_amount_0`: maximum amount of currency_0 users wants to add liquidity
    /// - `max_amount_1`: maximum amount of currency_1 users wants to add liquidity
    /// - `total_share_amount`: current total shares of this trading pair.
    ///
    /// `total_share_amount = 0` indicates that calculation for first liquidity providers should be
    /// taken. In this case, `pool_0` and `pool_1` is ignored.
    ///
    /// Returns:
    /// (pool_0_increment, pool_1_increment, share_increment)
    pub fn get_share_and_pool_increment(
        pool_0: Balance,
        pool_1: Balance,
        max_amount_0: Balance,
        max_amount_1: Balance,
        total_share_amount: Balance,
    ) -> sp_std::result::Result<(Balance, Balance, Balance), DispatchError> {
        if total_share_amount.is_zero() {
            // Calculate total share for first liquidity providers.

            // This is the exchange rate to convert from token amount to share amount.
            // We use first token for base rate. Meaning 1-to-1 mapping between first token and
            // share token.
            let (exchange_rate_0, exchange_rate_1) = (
                ExchangeRate::one(),
                ExchangeRate::checked_from_rational(max_amount_0, max_amount_1)
                    .ok_or(ArithmeticError::Overflow)?,
            );

            let share_from_token_0 = exchange_rate_0
                .checked_mul_int(max_amount_0)
                .ok_or(ArithmeticError::Overflow)?;
            let share_from_token_1 = exchange_rate_1
                .checked_mul_int(max_amount_1)
                .ok_or(ArithmeticError::Overflow)?;

            let share_amount = share_from_token_0
                .checked_add(share_from_token_1)
                .ok_or(ArithmeticError::Overflow)?;

            Ok((max_amount_0, max_amount_1, share_amount))
        } else {
            // Calculate share amount for successor.

            // This is the current exchange rate between first token and second token.
            let exchange_rate_1_0 = ExchangeRate::checked_from_rational(pool_0, pool_1)
                .ok_or(ArithmeticError::Overflow)?;
            // This is the exchange rate between first token and second token user willing to add
            // liqudity.
            let input_exchange_rate_1_0 =
                ExchangeRate::checked_from_rational(max_amount_0, max_amount_1)
                    .ok_or(ArithmeticError::Overflow)?;

            // We want to match exchange rate supplied by user to match current exchange rate.
            if input_exchange_rate_1_0 <= exchange_rate_1_0 {
                // In this case, we have two solutions:
                // 1. Increase numerator of input exchange rate which is first token amount user
                // willing to add liquidity.
                // 2. Decrease denominator of input exchange rate which is second token amount user
                // willing to add liquidity
                // However, increasing first token amount can be eliminated because the amount
                // given is already the maximum amount user willing to pay.
                // So we go with option 2 by calculating new second token amount.

                let exchange_rate_0_1 = ExchangeRate::checked_from_rational(pool_1, pool_0)
                    .ok_or(ArithmeticError::Overflow)?;
                let amount_1 = exchange_rate_0_1
                    .checked_mul_int(max_amount_0)
                    .ok_or(ArithmeticError::Overflow)?;
                let share_amount = Ratio::checked_from_rational(amount_1, pool_1)
                    .and_then(|r| r.checked_mul_int(total_share_amount))
                    .ok_or(ArithmeticError::Overflow)?;

                Ok((max_amount_0, amount_1, share_amount))
            } else {
                // With same explanation, we will calculate new first token amount in this case.

                let amount_0 = exchange_rate_1_0
                    .checked_mul_int(max_amount_1)
                    .ok_or(ArithmeticError::Overflow)?;
                let share_amount = Ratio::checked_from_rational(amount_0, pool_0)
                    .and_then(|r| r.checked_mul_int(total_share_amount))
                    .ok_or(ArithmeticError::Overflow)?;

                Ok((amount_0, max_amount_1, share_amount))
            }
        }
    }

    /// Return share and pool decrement when users withdraw liquidity token
    /// - `pool_0`: liquidity pool of currency_0
    /// - `pool_1`: liqudity pool of currency_1
    /// - `remove_share_amount`: share amount to remove
    /// - `total_share_amount`: total share amount
    ///
    /// Returns:
    /// (pool_0_decrement, pool_1_decrement, share_decrement)
    pub fn get_share_and_pool_decrement(
        pool_0: Balance,
        pool_1: Balance,
        remove_share_amount: Balance,
        total_share_amount: Balance,
    ) -> sp_std::result::Result<(Balance, Balance, Balance), DispatchError> {
        let share_proportion =
            Ratio::checked_from_rational(remove_share_amount, total_share_amount)
                .ok_or(ArithmeticError::Overflow)?;
        let withdrawn_amount_0 = share_proportion
            .checked_mul_int(pool_0)
            .ok_or(ArithmeticError::Overflow)?;
        let withdrawn_amount_1 = share_proportion
            .checked_mul_int(pool_1)
            .ok_or(ArithmeticError::Overflow)?;

        Ok((withdrawn_amount_0, withdrawn_amount_1, remove_share_amount))
    }

    #[transactional]
    pub fn do_claim_dex_share(
        owner: &T::AccountId,
        currency_id_0: CurrencyId,
        currency_id_1: CurrencyId,
    ) -> DispatchResult {
        let trading_pair = TradingPair::from_currency_ids(currency_id_0, currency_id_1)
            .ok_or(Error::<T>::InvalidCurrencyId)?;
        ensure!(
            matches!(
                Self::trading_pair_statuses(trading_pair),
                TradingPairStatus::<_>::Enabled
            ),
            Error::<T>::TradingPairMustBeEnabled
        );

        ProvisioningPool::<T>::try_mutate_exists(
            trading_pair,
            owner.clone(),
            |maybe_provision| -> DispatchResult {
                if !maybe_provision.is_some() {
                    return Err(Error::<T>::UserNotAddProvision.into());
                }
                let (provision_0, provision_1) = maybe_provision.unwrap_or_default();
                let (exchange_rate_0, exchange_rate_1) =
                    Self::initial_share_exchange_rates(trading_pair);
                let share_from_token_0 = exchange_rate_0
                    .checked_mul_int(provision_0)
                    .ok_or(ArithmeticError::Overflow)?;
                let share_from_token_1 = exchange_rate_1
                    .checked_mul_int(provision_1)
                    .ok_or(ArithmeticError::Overflow)?;
                let share_amount = share_from_token_0
                    .checked_add(share_from_token_1)
                    .ok_or(ArithmeticError::Overflow)?;
                let share_currency_id = trading_pair.dex_share_currency_id();

                T::Currency::transfer(share_currency_id, &Self::account_id(), owner, share_amount)?;

                *maybe_provision = None;
                frame_system::Pallet::<T>::dec_consumers(owner);

                Ok(())
            },
        )?;

        Ok(())
    }

    #[transactional]
    pub fn do_remove_liquidity(
        who: &T::AccountId,
        currency_id_0: CurrencyId,
        currency_id_1: CurrencyId,
        remove_share_amount: Balance,
        min_withdrawn_amount_0: Balance,
        min_withdrawn_amount_1: Balance,
    ) -> DispatchResult {
        ensure!(
            !remove_share_amount.is_zero(),
            Error::<T>::InvalidRemoveShareAmount
        );
        let trading_pair = TradingPair::from_currency_ids(currency_id_0, currency_id_1)
            .ok_or(Error::<T>::InvalidCurrencyId)?;
        let share_currency_id = trading_pair.dex_share_currency_id();
        let pallet_account_id = Self::account_id();
        let total_share_amount = T::Currency::total_issuance(share_currency_id);

        ensure!(!total_share_amount.is_zero(), Error::<T>::ZeroTotalShare);

        LiquidityPool::<T>::try_mutate(trading_pair, |(pool_0, pool_1)| -> DispatchResult {
            let min_withdrawn_amount = if currency_id_0 == trading_pair.first() {
                (min_withdrawn_amount_0, min_withdrawn_amount_1)
            } else {
                (min_withdrawn_amount_1, min_withdrawn_amount_0)
            };

            let (pool_0_decrement, pool_1_decrement, share_decrement) =
                Self::get_share_and_pool_decrement(
                    *pool_0,
                    *pool_1,
                    remove_share_amount,
                    total_share_amount,
                )?;

            ensure!(
                pool_0_decrement >= min_withdrawn_amount.0
                    && pool_1_decrement >= min_withdrawn_amount.1,
                Error::<T>::UnacceptableWithdrawnAmount
            );

            T::Currency::transfer(
                trading_pair.first(),
                &pallet_account_id,
                who,
                pool_0_decrement,
            )?;
            T::Currency::transfer(
                trading_pair.second(),
                &pallet_account_id,
                who,
                pool_1_decrement,
            )?;
            T::Currency::withdraw(share_currency_id, who, share_decrement)?;

            *pool_0 = pool_0
                .checked_sub(pool_0_decrement)
                .ok_or(ArithmeticError::Underflow)?;
            *pool_1 = pool_1
                .checked_sub(pool_1_decrement)
                .ok_or(ArithmeticError::Underflow)?;

            Self::deposit_event(Event::RemoveLiqudity(
                who.clone(),
                trading_pair.first(),
                pool_0_decrement,
                trading_pair.second(),
                pool_1_decrement,
                share_decrement,
            ));

            Ok(())
        })?;

        Ok(())
    }

    pub fn do_add_liquidity(
        who: &T::AccountId,
        currency_id_0: CurrencyId,
        currency_id_1: CurrencyId,
        max_amount_0: Balance,
        max_amount_1: Balance,
    ) -> DispatchResult {
        let trading_pair = TradingPair::from_currency_ids(currency_id_0, currency_id_1)
            .ok_or(Error::<T>::InvalidCurrencyId)?;

        ensure!(
            matches!(
                Self::trading_pair_statuses(trading_pair),
                TradingPairStatus::<_>::Enabled
            ),
            Error::<T>::TradingPairMustBeEnabled
        );

        ensure!(
            !max_amount_0.is_zero() && !max_amount_1.is_zero(),
            Error::<T>::InvalidLiquidityIncrement
        );

        LiquidityPool::<T>::try_mutate(trading_pair, |(pool_0, pool_1)| -> DispatchResult {
            let max_amount = if currency_id_0 == trading_pair.first() {
                (max_amount_0, max_amount_1)
            } else {
                (max_amount_1, max_amount_0)
            };

            let share_currency_id = trading_pair.dex_share_currency_id();
            let total_shares = T::Currency::total_issuance(share_currency_id);
            let (pool_0_increment, pool_1_increment, share_increment) =
                Self::get_share_and_pool_increment(
                    *pool_0,
                    *pool_1,
                    max_amount.0,
                    max_amount.1,
                    total_shares,
                )?;

            let pallet_account_id = Self::account_id();
            T::Currency::transfer(
                trading_pair.first(),
                who,
                &pallet_account_id,
                pool_0_increment,
            )?;
            T::Currency::transfer(
                trading_pair.second(),
                who,
                &pallet_account_id,
                pool_1_increment,
            )?;
            T::Currency::deposit(share_currency_id, who, share_increment)?;

            *pool_0 = pool_0
                .checked_add(pool_0_increment)
                .ok_or(ArithmeticError::Overflow)?;
            *pool_1 = pool_1
                .checked_add(pool_1_increment)
                .ok_or(ArithmeticError::Overflow)?;

            Self::deposit_event(Event::AddLiquidity(
                who.clone(),
                trading_pair.first(),
                pool_0_increment,
                trading_pair.second(),
                pool_1_increment,
                share_increment,
            ));

            Ok(())
        })?;

        Ok(())
    }

    #[transactional]
    pub fn do_enable_trading_pair(
        currency_id_0: CurrencyId,
        currency_id_1: CurrencyId,
    ) -> DispatchResult {
        let trading_pair = TradingPair::from_currency_ids(currency_id_0, currency_id_1)
            .ok_or(Error::<T>::InvalidCurrencyId)?;

        match Self::trading_pair_statuses(trading_pair) {
            TradingPairStatus::<_>::Provisioning(provisioning_parameters) => {
                let (contribution_0, contribution_1) =
                    provisioning_parameters.accumulated_contribution;
                ensure!(
                    contribution_0.is_zero() && contribution_1.is_zero(),
                    Error::<T>::TradingPairAlreadyProvisioned
                );
            }
            TradingPairStatus::<_>::Disabled => (),
            TradingPairStatus::<_>::Enabled => {
                return Err(Error::<T>::TradingPairAlreadyEnabled.into())
            }
        }

        TradingPairStatuses::<T>::insert(trading_pair, TradingPairStatus::<_>::Enabled);

        Self::deposit_event(Event::TradingPairEnabled(trading_pair));

        Ok(())
    }

    #[transactional]
    pub fn do_enable_provisioning_trading_pair(
        currency_id_0: CurrencyId,
        currency_id_1: CurrencyId,
    ) -> DispatchResult {
        let trading_pair = TradingPair::from_currency_ids(currency_id_0, currency_id_1)
            .ok_or(Error::<T>::InvalidCurrencyId)?;

        let provisioning_parameters = match Self::trading_pair_statuses(trading_pair) {
            TradingPairStatus::<_>::Provisioning(p) => p,
            _ => return Err(Error::<T>::TradingPairMustBeProvisioning.into()),
        };

        let (total_provision_0, total_provision_1) =
            provisioning_parameters.accumulated_contribution;
        ensure!(
            !total_provision_0.is_zero()
                && !total_provision_1.is_zero()
                && (total_provision_0 >= provisioning_parameters.target_contribution.0
                    || total_provision_1 >= provisioning_parameters.target_contribution.1),
            Error::<T>::UnqualifiedProvision
        );

        let (pool_0_increment, pool_1_increment, share_amount_to_issue) =
            Self::get_share_and_pool_increment(
                0,
                0,
                total_provision_0,
                total_provision_1,
                Zero::zero(),
            )?;
        ensure!(
            !pool_0_increment.is_zero()
                && !pool_1_increment.is_zero()
                && !share_amount_to_issue.is_zero(),
            Error::<T>::InvalidLiquidityIncrement
        );

        let share_currency_id = trading_pair.dex_share_currency_id();
        let pallet_account_id = Self::account_id();
        T::Currency::deposit(share_currency_id, &pallet_account_id, share_amount_to_issue)?;

        LiquidityPool::<T>::try_mutate(trading_pair, |(pool_0, pool_1)| -> DispatchResult {
            *pool_0 = pool_0
                .checked_add(pool_0_increment)
                .ok_or(ArithmeticError::Overflow)?;
            *pool_1 = pool_1
                .checked_add(pool_1_increment)
                .ok_or(ArithmeticError::Overflow)?;

            Ok(())
        })?;

        TradingPairStatuses::<T>::insert(trading_pair, TradingPairStatus::<_>::Enabled);

        InitialShareExchangeRate::<T>::insert(
            trading_pair,
            (
                ExchangeRate::one(),
                ExchangeRate::checked_from_rational(total_provision_0, total_provision_1)
                    .ok_or(ArithmeticError::Overflow)?,
            ),
        );

        Self::deposit_event(Event::TradingPairEnabledFromProvisioning(
            trading_pair.first(),
            total_provision_0,
            trading_pair.second(),
            total_provision_1,
            share_amount_to_issue,
        ));

        Ok(())
    }

    #[transactional]
    pub fn do_add_trading_pair_provision(
        who: &T::AccountId,
        currency_id_0: CurrencyId,
        currency_id_1: CurrencyId,
        contribution_0: Balance,
        contribution_1: Balance,
    ) -> DispatchResult {
        let trading_pair = TradingPair::from_currency_ids(currency_id_0, currency_id_1)
            .ok_or(Error::<T>::InvalidCurrencyId)?;

        let mut provisioning_parameters = match Self::trading_pair_statuses(trading_pair) {
            TradingPairStatus::<_>::Provisioning(provisioning_parameters) => {
                provisioning_parameters
            }
            _ => return Err(Error::<T>::TradingPairMustBeProvisioning.into()),
        };

        let contribution = if currency_id_0 == trading_pair.first() {
            (contribution_0, contribution_1)
        } else {
            (contribution_1, contribution_0)
        };

        ensure!(
            contribution.0 >= provisioning_parameters.min_contribution.0
                || contribution.1 >= provisioning_parameters.min_contribution.1,
            Error::<T>::InvalidContributionIncrement
        );

        ProvisioningPool::<T>::try_mutate_exists(
            trading_pair,
            who.clone(),
            |maybe_pool| -> DispatchResult {
                let existed = maybe_pool.is_some();
                let mut pool = maybe_pool.unwrap_or_default();
                pool.0 = pool
                    .0
                    .checked_add(contribution.0)
                    .ok_or(ArithmeticError::Overflow)?;
                pool.1 = pool
                    .1
                    .checked_add(contribution.1)
                    .ok_or(ArithmeticError::Overflow)?;

                let pallet_account_id = Self::account_id();
                T::Currency::transfer(
                    trading_pair.first(),
                    who,
                    &pallet_account_id,
                    contribution.0,
                )?;
                T::Currency::transfer(
                    trading_pair.second(),
                    who,
                    &pallet_account_id,
                    contribution.1,
                )?;

                *maybe_pool = Some(pool);

                if !existed && maybe_pool.is_some() {
                    if frame_system::Pallet::<T>::inc_consumers(who).is_err() {
                        log::warn!("Warning: Attempts to lock provider account but no providers");
                    }
                }

                provisioning_parameters.accumulated_contribution.0 = provisioning_parameters
                    .accumulated_contribution
                    .0
                    .checked_add(contribution.0)
                    .ok_or(ArithmeticError::Overflow)?;

                provisioning_parameters.accumulated_contribution.1 = provisioning_parameters
                    .accumulated_contribution
                    .1
                    .checked_add(contribution.1)
                    .ok_or(ArithmeticError::Overflow)?;

                TradingPairStatuses::<T>::insert(
                    trading_pair,
                    TradingPairStatus::<_>::Provisioning(ProvisioningParameters {
                        min_contribution: provisioning_parameters.min_contribution,
                        target_contribution: provisioning_parameters.target_contribution,
                        accumulated_contribution: provisioning_parameters.accumulated_contribution,
                    }),
                );

                Self::deposit_event(Event::AddProvision(
                    who.clone(),
                    trading_pair.first(),
                    contribution.0,
                    trading_pair.second(),
                    contribution.1,
                ));

                Ok(())
            },
        )
    }

    #[transactional]
    pub fn do_update_trading_pair_provisioning_parameters(
        currency_id_0: CurrencyId,
        currency_id_1: CurrencyId,
        min_contribution_0: Balance,
        min_contribution_1: Balance,
        target_contribution_0: Balance,
        target_contribution_1: Balance,
    ) -> DispatchResult {
        let trading_pair = TradingPair::from_currency_ids(currency_id_0, currency_id_1)
            .ok_or(Error::<T>::InvalidCurrencyId)?;

        match Self::trading_pair_statuses(trading_pair) {
            TradingPairStatus::<_>::Provisioning(provisioning_parameters) => {
                let (min_contribution, target_contribution) =
                    if currency_id_0 == trading_pair.first() {
                        (
                            (min_contribution_0, min_contribution_1),
                            (target_contribution_0, target_contribution_1),
                        )
                    } else {
                        (
                            (min_contribution_1, min_contribution_0),
                            (target_contribution_1, target_contribution_0),
                        )
                    };

                TradingPairStatuses::<T>::insert(
                    trading_pair,
                    TradingPairStatus::<_>::Provisioning(ProvisioningParameters {
                        min_contribution,
                        target_contribution,
                        accumulated_contribution: provisioning_parameters.accumulated_contribution,
                    }),
                );
            }
            _ => return Err(Error::<T>::TradingPairMustBeProvisioning.into()),
        }

        Ok(())
    }

    /// Implementation of `start_trading_pair_provisioning`
    #[transactional]
    pub fn do_start_trading_pair_provisioning(
        currency_id_0: CurrencyId,
        currency_id_1: CurrencyId,
        min_contribution_0: Balance,
        min_contribution_1: Balance,
        target_contribution_0: Balance,
        target_contribution_1: Balance,
    ) -> DispatchResult {
        let trading_pair = TradingPair::from_currency_ids(currency_id_0, currency_id_1)
            .ok_or(Error::<T>::InvalidCurrencyId)?;
        ensure!(
            matches!(
                Self::trading_pair_statuses(trading_pair),
                TradingPairStatus::<_>::Disabled
            ),
            Error::<T>::TradingPairMustBeDisabled
        );

        let (min_contribution, target_contribution) = if currency_id_0 == trading_pair.first() {
            (
                (min_contribution_0, min_contribution_1),
                (target_contribution_0, target_contribution_1),
            )
        } else {
            (
                (min_contribution_1, min_contribution_0),
                (target_contribution_1, target_contribution_0),
            )
        };

        TradingPairStatuses::<T>::insert(
            trading_pair,
            TradingPairStatus::<_>::Provisioning(ProvisioningParameters {
                min_contribution,
                target_contribution,
                accumulated_contribution: Default::default(),
            }),
        );

        Self::deposit_event(Event::TradingPairProvisioning(trading_pair));

        Ok(())
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
    ) -> sp_std::result::Result<Balance, DispatchError> {
        let amounts = Self::get_target_amounts(path, supply_amount)?;
        ensure!(
            amounts[amounts.len() - 1] >= min_target_amount,
            Error::<T>::InsufficientTargetAmount
        );
        let pallet_account_id = Self::account_id();
        let actual_target_amount = amounts[amounts.len() - 1];
        T::Currency::transfer(path[0], who, &pallet_account_id, supply_amount)?;
        Self::_swap_by_path(&path, &amounts)?;
        T::Currency::transfer(
            path[path.len() - 1],
            &pallet_account_id,
            who,
            actual_target_amount,
        )?;

        Self::deposit_event(Event::Swap(
            who.clone(),
            path.to_vec(),
            supply_amount,
            actual_target_amount,
        ));
        Ok(actual_target_amount)
    }

    /// Implementation of `swap_with_exact_target`
    ///
    /// Returns a `Result`:
    /// - `supply_amount`: Calculated amount that user must pay
    /// - `error`: dispatch error when failed
    #[transactional]
    pub fn do_swap_with_exact_target(
        who: &T::AccountId,
        path: &[CurrencyId],
        target_amount: Balance,
        max_supply_amount: Balance,
    ) -> sp_std::result::Result<Balance, DispatchError> {
        let amounts = Self::get_supply_amounts(path, target_amount)?;
        let actual_supply_amount = amounts[0];
        ensure!(
            actual_supply_amount <= max_supply_amount,
            Error::<T>::InsufficientSupplyAmount
        );

        let pallet_account_id = Self::account_id();
        T::Currency::transfer(path[0], who, &pallet_account_id, actual_supply_amount)?;
        Self::_swap_by_path(path, &amounts)?;
        T::Currency::transfer(
            path[path.len() - 1],
            &pallet_account_id,
            who,
            amounts[amounts.len() - 1],
        )?;

        Self::deposit_event(Event::Swap(
            who.clone(),
            path.to_vec(),
            actual_supply_amount,
            amounts[amounts.len() - 1],
        ));
        Ok(actual_supply_amount)
    }

    /// Returns reserves of two currencies.
    pub fn get_liquidity(
        currency_id_0: &CurrencyId,
        currency_id_1: &CurrencyId,
    ) -> (Balance, Balance) {
        if let Some(trading_pair) =
            TradingPair::from_currency_ids(currency_id_0.clone(), currency_id_1.clone())
        {
            let (pool_0, pool_1) = Self::liquidity_pool(trading_pair);
            if *currency_id_0 == trading_pair.first() {
                (pool_0, pool_1)
            } else {
                (pool_1, pool_0)
            }
        } else {
            (Zero::zero(), Zero::zero())
        }
    }

    /// Returns target amount,
    /// given supply reserves, target reserves, supply amount.
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
    /// of given trading path.
    pub fn get_target_amounts(
        path: &[CurrencyId],
        supply_amount: Balance,
    ) -> sp_std::result::Result<Vec<Balance>, DispatchError> {
        let path_length = path.len();
        ensure!(
            path_length >= 2 && path_length <= T::TradingPathLimit::get().saturated_into(),
            Error::<T>::InvalidTradingPathLength
        );

        let mut amounts: Vec<Balance> = vec![Zero::zero(); path_length];
        amounts[0] = supply_amount;

        let mut i: usize = 0;
        while i + 1 < path_length {
            let trading_pair = TradingPair::from_currency_ids(path[i], path[i + 1])
                .ok_or(Error::<T>::InvalidCurrencyId)?;
            ensure!(
                matches!(
                    Self::trading_pair_statuses(trading_pair),
                    TradingPairStatus::<_>::Enabled
                ),
                Error::<T>::TradingPairMustBeEnabled
            );

            let (supply_pool, target_pool) = Self::get_liquidity(&path[i], &path[i + 1]);
            ensure!(
                !supply_pool.is_zero() && !target_pool.is_zero(),
                Error::<T>::InsufficientLiquidity
            );

            let target_amount = Self::get_target_amount(supply_pool, target_pool, amounts[i]);
            ensure!(!target_amount.is_zero(), Error::<T>::ZeroTargetAmount);

            amounts[i + 1] = target_amount;
            i += 1;
        }

        Ok(amounts)
    }

    pub fn get_supply_amount(
        supply_pool: Balance,
        target_pool: Balance,
        target_amount: Balance,
    ) -> Balance {
        if target_amount.is_zero() || supply_pool.is_zero() || target_pool.is_zero() {
            Zero::zero()
        } else {
            let (fee_numerator, fee_denominator) = T::ExchangeFee::get();
            let numerator: U256 = U256::from(target_amount)
                .saturating_mul(U256::from(supply_pool))
                .saturating_mul(U256::from(fee_denominator));
            let denominator: U256 = U256::from(target_pool)
                .saturating_sub(U256::from(target_amount))
                .saturating_mul(U256::from(fee_denominator.saturating_sub(fee_numerator)));

            numerator
                .checked_div(denominator)
                .and_then(|r| r.checked_add(U256::one()))
                .and_then(|n| TryInto::<Balance>::try_into(n).ok())
                .unwrap_or_else(Zero::zero)
        }
    }

    pub fn get_supply_amounts(
        path: &[CurrencyId],
        target_amount: Balance,
    ) -> sp_std::result::Result<Vec<Balance>, DispatchError> {
        let path_length = path.len();
        ensure!(
            path_length >= 2 && path_length <= T::TradingPathLimit::get().saturated_into(),
            Error::<T>::InvalidTradingPathLength
        );

        let mut amounts: Vec<Balance> = vec![0; path.len()];
        amounts[path_length - 1] = target_amount;

        let mut i: usize = path_length - 1;
        while i > 0 {
            let trading_pair = TradingPair::from_currency_ids(path[i - 1], path[i])
                .ok_or(Error::<T>::InvalidCurrencyId)?;
            ensure!(
                matches!(
                    Self::trading_pair_statuses(trading_pair),
                    TradingPairStatus::<_>::Enabled
                ),
                Error::<T>::TradingPairMustBeEnabled
            );

            let (supply_pool, target_pool) = Self::get_liquidity(&path[i - 1], &path[i]);
            ensure!(
                !supply_pool.is_zero() && !target_pool.is_zero(),
                Error::<T>::InsufficientLiquidity
            );
            ensure!(target_pool >= amounts[i], Error::<T>::InsufficientLiquidity);

            let supply_amount = Self::get_supply_amount(supply_pool, target_pool, amounts[i]);
            ensure!(!supply_amount.is_zero(), Error::<T>::ZeroSupplyAmount);

            amounts[i - 1] = supply_amount;
            i -= 1;
        }

        Ok(amounts)
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
            LiquidityPool::<T>::try_mutate(
                trading_pair.clone(),
                |(pool_0, pool_1)| -> DispatchResult {
                    let invariant_before_swap =
                        U256::from(*pool_0).saturating_mul(U256::from(*pool_1));

                    if supply_currency == trading_pair.first() {
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

                    let invariant_after_swap =
                        U256::from(*pool_0).saturating_mul(U256::from(*pool_1));
                    ensure!(
                        invariant_after_swap >= invariant_before_swap,
                        Error::<T>::InvariantAfterCheckFailed
                    );

                    Ok(())
                },
            )?;
        }
        Ok(())
    }

    /// Do storage changes for swapping given trading path
    fn _swap_by_path(path: &[CurrencyId], amounts: &[Balance]) -> DispatchResult {
        let mut i: usize = 0;
        while i + 1 < path.len() {
            let supply_currency = path[i].clone();
            let target_currency = path[i + 1].clone();
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
