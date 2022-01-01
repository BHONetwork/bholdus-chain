use super::*;

use bholdus_primitives::{Amount, Balance, Rate};

/// Counter for the number of eras that have passed.
use bholdus_support::{
    CurrencyDetails, DEXIncentives, EmergencyShutdown, MultiCurrency, RewardHandler,
};
use frame_support::{
    log,
    pallet_prelude::*,
    traits::{Get, OnUnbalanced},
    transactional, PalletId,
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AccountIdConversion, One, Saturating, UniqueSaturatedInto, Zero},
    DispatchResult, FixedPointNumber, RuntimeDebug,
};
use sp_std::convert::From;
use sp_std::if_std;
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + bholdus_support_rewards::Config<
            Share = Balance,
            Balance = Balance,
            PoolId = PoolId,
            CurrencyId = CurrencyId,
        >
    {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Number of blocks per era.
        #[pallet::constant]
        type BlockPerEra: Get<BlockNumberFor<Self>>;

        /// The period to accumulate rewards
        #[pallet::constant]
        type AccumulatePeriod: Get<Self::BlockNumber>;

        /// The source account for native token rewards.
        #[pallet::constant]
        type RewardsSource: Get<Self::AccountId>;

        /// The origin which may update incentive related params
        type UpdateOrigin: EnsureOrigin<Self::Origin>;

        /// The reward type for dex saving.
        #[pallet::constant]
        type StableCurrencyId: Get<CurrencyId>;

        /// Currency for transfer/issue assets
        type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>
            + CurrencyDetails<Self::AccountId, CurrencyId = CurrencyId>;

        /// Number of eras that are valid when claim rewards.
        ///
        /// All the rest will be either claimed by the treasury or discarded.
        #[pallet::constant]
        type HistoryDepth: Get<u32>;

        /// The module id, keep DexShare LP
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Weight information for the extrinsic
        type WeightInfo: WeightInfo;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// No permission
        NoPermission,
        /// Token is unverified
        UnVerified,
        /// Share amount is not enough
        NotEnough,
        /// Invalid currency id
        InvalidCurrencyId,
        /// Invalid pool id
        InvalidPoolId,

        /// Not found pool
        PoolDoesNotExist,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Create a new pool\[who, pool_id\]
        CreatedPool(T::AccountId, PoolId),
        /// Deposit DEX share. \[who, dex_share_type, deposit_amount\]
        DepositDexShare(T::AccountId, CurrencyId, Balance),
        /// Withdraw DEX share. \[who, dex_share_type, withdraw_amount\]
        WithdrawDexShare(T::AccountId, CurrencyId, Balance),
        /// Claim rewads. \[who, pool_id, reward_currency_id, actual_amount\]
        ClaimRewards(T::AccountId, PoolId, CurrencyId, Balance),
        /// Incentive reward amount updated. \[pool_id, reward_currency_id,
        /// reward_amount_per_period\]
        IncentiveRewardAmountUpdated(PoolId, CurrencyId, Balance),
        /// Era parameter is out of bounds
        EraOutOfBounds,
    }

    /// The current era index.
    #[pallet::storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T> = StorageValue<_, EraIndex, ValueQuery>;

    #[pallet::type_value]
    pub fn ForceEraOnEmpty() -> Forcing {
        Forcing::ForceNone
    }

    #[pallet::storage]
    #[pallet::getter(fn force_era)]
    pub type ForceEra<T> = StorageValue<_, Forcing, ValueQuery, ForceEraOnEmpty>;

    /// Mapping from pool to its fixed incentive amounts of multi currencies per period.
    ///
    /// IncentiveRewardAmounts: double_map Pool, RewardsCurrencyId => RewardAmountPerPeriod
    #[pallet::storage]
    #[pallet::getter(fn incentive_reward_amounts)]
    pub type IncentiveRewardAmounts<T: Config> =
        StorageDoubleMap<_, Twox64Concat, PoolId, Twox64Concat, CurrencyId, Balance, ValueQuery>;

    /// The pending rewards amount, actual available rewards amount may be deducted
    ///
    /// PendingMultiRewards: double_map PoolId, AccountId => BTreeMap<CurrencyId, Balance>
    #[pallet::storage]
    #[pallet::getter(fn pending_multi_rewards)]
    pub type PendingMultiRewards<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::PoolId>,    // PoolId,
            NMapKey<Blake2_128Concat, EraIndex>,     // EraIndex,
            NMapKey<Blake2_128Concat, T::AccountId>, // staker,
        ),
        BTreeMap<CurrencyId, Balance>,
        ValueQuery,
    >;

    #[pallet::pallet]
    #[pallet::generate_store(pub(crate) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    /* // Negative imbalance type of this pallet.
    type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::NegativeImbalance;

    impl<T: Config> OnUnbalanced<NegativeImbalanceOf<T>> for Pallet<T> {
        fn on_nonzero_unbalanced(block_reward: NegativeImbalanceOf<T>) {

        }
    }
    */

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            let force_new_era = Self::force_era().eq(&Forcing::ForceNew);
            let blocks_per_era = T::BlockPerEra::get();
            let previous_era = Self::current_era();
            // Value is compared to 1 since genesis block is igonered
            if now % blocks_per_era == BlockNumberFor::<T>::from(1u32)
                || force_new_era
                || previous_era.is_zero()
            {
                let next_era = previous_era + 1;
                CurrentEra::<T>::put(next_era);
                for (pool_id, pool_detail) in bholdus_support_rewards::Pools::<T>::iter() {
                    if !pool_detail.total_shares.is_zero() {
                        if_std!(
                            println!("on_initialize: pool_id {:?}; pool_detail {:?}", pool_id, pool_detail);
                        );
                        Self::accumulate_incentives(pool_id, previous_era)
                    }
                }

                if force_new_era {
                    ForceEra::<T>::put(Forcing::ForceNone);
                }
            }
            T::WeightInfo::on_initialize(5)
        }
    }
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        #[transactional]
        pub fn add_pool(origin: OriginFor<T>, lp_currency_id: CurrencyId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let pool_id = PoolId::Dex(lp_currency_id);
            ensure!(
                T::Currency::is_owner(&who, lp_currency_id),
                Error::<T>::NoPermission
            );
            ensure!(
                T::Currency::is_verifiable(lp_currency_id),
                Error::<T>::UnVerified
            );
            bholdus_support_rewards::Pallet::<T>::add_pool_owner(&who, &pool_id)?;
            Self::deposit_event(Event::CreatedPool(who.clone(), pool_id.clone()));
            Ok(().into())
        }
        /// Stake LP token to add shares of Pool::Dex
        ///
        /// The dispatch origin of this call must be `Signed` by the transactor.
        ///
        /// - `lp_currency_id`: LP token type
        /// - `amount`: amount to stake
        #[pallet::weight(<T as Config>::WeightInfo::deposit_dex_share())]
        #[transactional]
        pub fn deposit_dex_share(
            origin: OriginFor<T>,
            lp_currency_id: CurrencyId,
            #[pallet::compact] amount: Balance,
        ) -> DispatchResult {
            let staker = ensure_signed(origin)?;
            Self::do_deposit_dex_share(&staker, lp_currency_id, amount)?;
            Ok(())
        }

        /// Unstake LP token to remove shares of Pool::Dex
        ///
        /// The dispatch origin of this call must be `Signed` by the transactor.
        ///
        /// - `lp_currency_id`: LP token type
        /// - `amount`: amount to unstake
        #[pallet::weight(0)]
        #[transactional]
        pub fn withdraw_dex_share(
            origin: OriginFor<T>,
            lp_currency_id: CurrencyId,
            #[pallet::compact] amount: Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_withdraw_dex_share(&who, lp_currency_id, amount)?;
            Ok(())
        }

        /// Claim all available multi currencies rewards for specific PoolId.
        ///
        /// The dispatch origin of this call must be `Signed` by the transactor.
        ///
        /// -`pool_id`: pool type
        #[pallet::weight(0)]
        #[transactional]
        pub fn claim_rewards(
            origin: OriginFor<T>,
            pool_id: PoolId,
            era: EraIndex,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let current_era = Self::current_era();
            // let era_low_bound = current_era.saturating_sub(T::HistoryDepth::get());

            // ensure!(
            //     era < current_era && era >= era_low_bound
            //     Error::<T>::EraOutOfBounds,
            // );

            // bholdus support rewards for all currencies rewards
            <bholdus_support_rewards::Pallet<T>>::era_claim_rewards(&who, &pool_id, era);
            let pending_multi_rewards: BTreeMap<CurrencyId, Balance> =
                PendingMultiRewards::<T>::take((&pool_id, current_era, &who));

            for (currency_id, actual_amount) in pending_multi_rewards {
                if actual_amount.is_zero() {
                    continue;
                }

                // transfer the actual reward to user from the pool.
                // it should not affect the process, ignore the result to continue. If it fails,
                // just the user will not be rewarded, there will not increase user balance.
                T::Currency::transfer(currency_id, &Self::account_id(), &who, actual_amount)?;

                Self::deposit_event(Event::ClaimRewards(
                    who.clone(),
                    pool_id,
                    currency_id,
                    actual_amount,
                ));
            }
            Ok(())
        }

        /// Update incentive reward amount for specific PoolId
        ///
        /// The dispatch origin of this call must be `UpdateOrigin`.
        ///
        /// - `updates`: Vec<(PoolId, Vec<(RewardsCurrencyId, FixedPointNumber)>)>
        #[pallet::weight(0)]
        #[transactional]
        pub fn update_incentive_rewards(
            origin: OriginFor<T>,
            updates: Vec<(PoolId, Vec<(CurrencyId, Balance)>)>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            for (pool_id, update_list) in updates {
                ensure!(
                    <bholdus_support_rewards::Pallet<T>>::is_pool_owner(&who, &pool_id),
                    Error::<T>::NoPermission
                );

                for (currency_id, amount) in update_list {
                    IncentiveRewardAmounts::<T>::mutate_exists(
                        pool_id,
                        currency_id,
                        |maybe_amount| {
                            let mut v = maybe_amount.unwrap_or_default();
                            if amount != v {
                                v = amount;
                                Self::deposit_event(Event::IncentiveRewardAmountUpdated(
                                    pool_id,
                                    currency_id,
                                    amount,
                                ));
                            }

                            if v.is_zero() {
                                *maybe_amount = None;
                            } else {
                                *maybe_amount = Some(v);
                            }
                        },
                    );
                }
            }

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    fn accumulate_incentives(pool_id: PoolId, era: EraIndex) {
        for (reward_currency_id, reward_amount) in IncentiveRewardAmounts::<T>::iter_prefix(pool_id)
        {
            if reward_amount.is_zero() {
                continue;
            }

            let res = T::Currency::transfer(
                reward_currency_id,
                &T::RewardsSource::get(),
                &Self::account_id(),
                reward_amount,
            );

            match res {
                Ok(_) => {
                    let _ = <bholdus_support_rewards::Pallet<T>>::era_accumulate_reward(
                        &pool_id,
                        era,
                        reward_currency_id,
                        reward_amount,
                    ).map_err(|e| {
                        log::error!(
                            target: "tokens-staking",
                            "accumulate_reward: failed to accumulate reward to non-existen pool {:?}, reward_currency_id {:?}, reward_amount {:?}: {:?}",
                            pool_id, reward_currency_id, reward_amount, e
                        );
                    });
                }
                Err(e) => {
                    log::warn!(
                        target: "tokens-staking",
                        "transfer: failed to transfer {:?} {:?} from {:?} to {:?}: {:?}. \
                        This is unexpected but should be safe",
                        reward_amount, reward_currency_id, T::RewardsSource::get(), Self::account_id(), e
                    );
                }
            }
        }
    }
}

impl<T: Config> DEXIncentives<T::AccountId, CurrencyId, Balance> for Pallet<T> {
    fn do_deposit_dex_share(
        who: &T::AccountId,
        lp_currency_id: CurrencyId,
        amount: Balance,
    ) -> DispatchResult {
        // Get the latest era staking
        let current_era = Self::current_era();
        if_std!(
            println!("deposit_dex_share: current_era {:?}", current_era);
        );

        ensure!(
            <bholdus_support_rewards::Pools<T>>::contains_key(&PoolId::Dex(lp_currency_id)),
            Error::<T>::PoolDoesNotExist
        );

        T::Currency::transfer(lp_currency_id, who, &Self::account_id(), amount)?;
        <bholdus_support_rewards::Pallet<T>>::era_add_share(
            who,
            &PoolId::Dex(lp_currency_id),
            current_era,
            amount.unique_saturated_into(),
        );
        Self::deposit_event(Event::DepositDexShare(who.clone(), lp_currency_id, amount));
        Ok(())
    }

    fn do_withdraw_dex_share(
        who: &T::AccountId,
        lp_currency_id: CurrencyId,
        amount: Balance,
    ) -> DispatchResult {
        // Get the latest era staking
        let current_era = Self::current_era();
        if_std!(
            println!("withdraw_dex_share: current_era {:?}", current_era);
        );
        ensure!(
            <bholdus_support_rewards::Pallet<T>>::era_shares_and_withdrawn_rewards((
                &PoolId::Dex(lp_currency_id),
                current_era,
                &who
            ))
            .0 >= amount,
            Error::<T>::NotEnough,
        );

        T::Currency::transfer(lp_currency_id, &Self::account_id(), who, amount)?;
        <bholdus_support_rewards::Pallet<T>>::era_remove_share(
            who,
            &PoolId::Dex(lp_currency_id),
            current_era,
            amount.unique_saturated_into(),
        );

        Self::deposit_event(Event::WithdrawDexShare(who.clone(), lp_currency_id, amount));
        Ok(())
    }
}

impl<T: Config> RewardHandler<T::AccountId, CurrencyId> for Pallet<T> {
    type Balance = Balance;
    type PoolId = PoolId;

    fn payout(
        who: &T::AccountId,
        pool_id: &Self::PoolId,
        era: EraIndex,
        currency_id: CurrencyId,
        payout_amount: Self::Balance,
    ) {
        if payout_amount.is_zero() {
            return;
        }
        PendingMultiRewards::<T>::mutate((pool_id, era, who), |rewards| {
            rewards
                .entry(currency_id)
                .and_modify(|current| *current = current.saturating_add(payout_amount))
                .or_insert(payout_amount);
        });
    }
}
