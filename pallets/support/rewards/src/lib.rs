#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{FullCodec, HasCompact};
use frame_support::{pallet_prelude::*, transactional, PalletId};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_core::U256;

use bholdus_primitives::{Balance, CurrencyId, EraIndex, Rate, Ratio};
use bholdus_support::{MultiCurrency, RewardHandler};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
    traits::{
        AccountIdConversion, AtLeast32BitUnsigned, CheckedDiv, MaybeSerializeDeserialize, Member,
        Saturating, UniqueSaturatedInto, Zero,
    },
    ArithmeticError, DispatchError, DispatchResult, FixedPointNumber, FixedPointOperand,
    RuntimeDebug, SaturatedConversion,
};
use sp_std::{borrow::ToOwned, collections::btree_map::BTreeMap, fmt::Debug, if_std, prelude::*};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

/// The Reward Pool Info.
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct PoolInfo<Share: HasCompact, Balance: HasCompact, CurrencyId: Ord> {
    /// Total shares amount
    pub total_shares: Share,
    /// Reward infos <reward_currency, (total_reward, total_withdrawn_reward)>
    pub rewards: BTreeMap<CurrencyId, (Balance, Balance)>,
}

/// The Pool Detail
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct PoolDetails<AccountId, Share: HasCompact, Balance: HasCompact, CurrencyId: Ord> {
    pub owner: AccountId,
    /// Total shares amount
    pub total_shares: Share,
    /// Reward infos <reward_currency, (total_reward, total_withdrawn_reward)>
    pub rewards: BTreeMap<CurrencyId, (Balance, Balance)>,
}

impl<Share, Balance, CurrencyId> Default for PoolInfo<Share, Balance, CurrencyId>
where
    Share: Default + HasCompact,
    Balance: HasCompact,
    CurrencyId: Ord,
{
    fn default() -> Self {
        Self {
            total_shares: Default::default(),
            rewards: BTreeMap::new(),
        }
    }
}

/// The configurable params of staking token.
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, Default, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Params {
    /// The base rate fee for redemption.
    pub base_fee_rate: Rate,
}

#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum Phase {}

/// The ledger of staking pool.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, Default)]
pub struct Ledger {
    /// The amount of total bonded.
    pub bonded: Balance,
    /// The amount of total unbonding to free pool.
    pub unbonding_to_free: Balance,
    /// The amount of free pool.
    pub free_pool: Balance,
    /// The amount to unbond when next era beginning.
    pub to_unbond_next_era: (Balance, Balance),
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::dispatch::DispatchResult;
    #[pallet::config]
    pub trait Config: frame_system::Config {
        // type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        // /// The staking currency id
        // #[pallet::constant]
        // type StakingCurrencyId: Get<CurrencyId>;
        // /// The staking pool's pallet id, keep all staking currency
        // #[pallet::constant]
        // type PalletId: Get<PalletId>;
        // /// The currency for managing assets related
        // type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

        /// *Rewards*
        /// The share type of pool.
        type Share: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + FixedPointOperand;
        /// The reward balance type.
        type Balance: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + FixedPointOperand;

        /// The reward pool ID type
        type PoolId: Parameter + Member + Clone + FullCodec;

        type CurrencyId: Parameter
            + Member
            + Copy
            + Default
            + MaybeSerializeDeserialize
            + Ord
            + AtLeast32BitUnsigned;

        /// The `RewardHandler`
        type Handler: RewardHandler<
            Self::AccountId,
            Self::CurrencyId,
            Balance = Self::Balance,
            PoolId = Self::PoolId,
        >;
    }

    pub type PoolDetailsOf<T> = PoolDetails<
        <T as frame_system::Config>::AccountId,
        <T as Config>::Share,
        <T as Config>::Balance,
        <T as Config>::Balance,
    >;

    #[pallet::error]
    pub enum Error<T> {
        /// The era index is invalid.
        InvalidEra,
        /// Failed to calculate redemption fee.
        GetFeeFailed,
        /// Invalid config.
        InvalidConfig,
        /// Rebalance process is unfinished.
        RebalanceUnfinished,

        /// Pool does not exist
        PoolDoesNotExist,
    }

    // #[pallet::event]
    // #[pallet::generate_deposit(pub(super) fn deposit_event)]
    // pub enum Event<T: Config> {
    //     /// Deposit staking currency to staking pool and issue liquid currency. \[who,
    //     /// staking_amount_deposited, liquid_amount_issued\]
    //     MintLiquid(T::AccountId, Balance, Balance),
    // }

    /// Pool Owner
    #[pallet::storage]
    #[pallet::getter(fn pool_owner)]
    pub type PoolOwner<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>, //owner
            NMapKey<Blake2_128Concat, T::PoolId>,    // pool_id
        ),
        (),
        ValueQuery,
    >;

    /// Pool
    #[pallet::storage]
    #[pallet::getter(fn pools)]
    pub type Pools<T: Config> = StorageMap<_, Twox64Concat, T::PoolId, PoolDetailsOf<T>>;

    /// Record reward pool info.
    ///
    /// map PoolId => PoolInfo
    #[pallet::storage]
    #[pallet::getter(fn pool_infos)]
    pub type PoolInfos<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::PoolId,
        PoolInfo<T::Share, T::Balance, T::CurrencyId>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn era_pool_infos)]
    pub type EraPoolInfos<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::PoolId,
        Twox64Concat,
        EraIndex,
        PoolInfo<T::Share, T::Balance, T::CurrencyId>,
        ValueQuery,
    >;

    /// Record share amount, reward currency and withdrawn reward amount for
    /// specific `AccountId` under `PoolId`.
    ///
    /// double_map (PoolId, AccountId) => (Share, BTreeMap<CurrencyId, Balance>)
    #[pallet::storage]
    #[pallet::getter(fn shares_and_withdrawn_rewards)]
    pub type SharesAndWithdrawnRewards<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::PoolId,
        Twox64Concat,
        T::AccountId,
        (T::Share, BTreeMap<T::CurrencyId, T::Balance>),
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn era_shares_and_withdrawn_rewards)]
    pub type EraSharesAndWithdrawnRewards<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::PoolId>,    // PoolId,
            NMapKey<Blake2_128Concat, EraIndex>,     // EraIndex,
            NMapKey<Blake2_128Concat, T::AccountId>, // staker,
        ),
        (T::Share, BTreeMap<T::CurrencyId, T::Balance>),
        ValueQuery,
    >;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// The params of staking pool.
    ///
    /// StakingPoolParams: Params
    #[pallet::storage]
    #[pallet::getter(fn staking_pool_params)]
    pub type StakingPoolParams<T: Config> = StorageValue<_, Params, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
    pub fn is_pool_owner(account_id: &T::AccountId, pool_id: &T::PoolId) -> bool {
        PoolOwner::<T>::contains_key((account_id, pool_id))
    }

    pub fn add_pool_owner(owner: &T::AccountId, pool_id: &T::PoolId) -> DispatchResult {
        // ensure `owner` tokens owner
        let pool_detail = PoolDetails {
            owner: owner.clone(),
            total_shares: Zero::zero(),
            rewards: Default::default(),
        };
        if_std!(
            println!("add_pool_owner: {:?}", pool_detail);
        );
        PoolOwner::<T>::insert((owner, pool_id), ());
        Pools::<T>::insert(pool_id, pool_detail);
        Ok(())
    }
    pub fn accumulate_reward(
        pool: &T::PoolId,
        reward_currency: T::CurrencyId,
        reward_increment: T::Balance,
    ) -> DispatchResult {
        if reward_increment.is_zero() {
            return Ok(());
        }

        PoolInfos::<T>::mutate_exists(pool, |maybe_pool_info| -> DispatchResult {
            let pool_info = maybe_pool_info
                .as_mut()
                .ok_or(Error::<T>::PoolDoesNotExist)?;
            pool_info
                .rewards
                .entry(reward_currency)
                .and_modify(|(total_reward, _)| {
                    *total_reward = total_reward.saturating_add(reward_increment);
                })
                .or_insert((reward_increment, Zero::zero()));
            Ok(())
        })
    }

    pub fn era_accumulate_reward(
        pool: &T::PoolId,
        era: EraIndex,
        reward_currency: T::CurrencyId,
        reward_increment: T::Balance,
    ) -> DispatchResult {
        if reward_increment.is_zero() {
            return Ok(());
        }

        Pools::<T>::mutate_exists(pool, |pool_detail| -> DispatchResult {
            pool_detail.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;

            // Prepare info for the next era
            EraPoolInfos::<T>::mutate(pool, era + 1, |pool_info| -> DispatchResult {
                pool_info.total_shares = Zero::zero();
                pool_info
                    .rewards
                    .entry(reward_currency)
                    .and_modify(|(total_reward, _)| {
                        *total_reward = total_reward.saturating_add(reward_increment);
                    })
                    .or_insert((reward_increment, Zero::zero()));
                Ok(())
            });

            EraPoolInfos::<T>::mutate(pool, era, |pool_info| -> DispatchResult {
                /* let pool_info = maybe_pool_info
                    .as_mut()
                    .ok_or(Error::<T>::PoolDoesNotExist)?;
                */
                pool_info
                    .rewards
                    .entry(reward_currency)
                    .and_modify(|(total_reward, _)| {
                        *total_reward = total_reward.saturating_add(reward_increment);
                    })
                    .or_insert((reward_increment, Zero::zero()));
                Ok(())
            });

            Ok(())
        })
    }

    pub fn era_add_share(
        who: &T::AccountId,
        pool: &T::PoolId,
        era: EraIndex,
        add_amount: T::Share,
    ) -> DispatchResult {
        if add_amount.is_zero() {
            return Ok(());
        }

        if_std!(println!("era_add_share: {:?}", add_amount));
        Pools::<T>::mutate_exists(pool, |pool_detail| -> DispatchResult {
            if_std!(
                println!("era_add_share: pool_detail {:?}", pool_detail);
            );
            pool_detail.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;

            EraPoolInfos::<T>::mutate_exists(pool, era, |maybe_pool_info| -> DispatchResult {
                let pool_info = maybe_pool_info
                    .as_mut()
                    .ok_or(Error::<T>::PoolDoesNotExist)?;

                let initial_total_shares = pool_info.total_shares;

                pool_info.total_shares = pool_info.total_shares.saturating_add(add_amount);

                let mut withdrawn_inflation = Vec::<(T::CurrencyId, T::Balance)>::new();

                pool_info.rewards.iter_mut().for_each(
                    |(reward_currency, (total_reward, total_withdrawn_reward))| {
                        let reward_inflation = if initial_total_shares.is_zero() {
                            Zero::zero()
                        } else {
                            U256::from(add_amount.to_owned().saturated_into::<u128>())
                                .saturating_mul(
                                    total_reward.to_owned().saturated_into::<u128>().into(),
                                )
                                .checked_div(
                                    initial_total_shares
                                        .to_owned()
                                        .saturated_into::<u128>()
                                        .into(),
                                )
                                .unwrap_or_default()
                                .as_u128()
                                .saturated_into()
                        };

                        *total_reward = total_reward.saturating_add(reward_inflation);
                        *total_withdrawn_reward =
                            total_withdrawn_reward.saturating_add(reward_inflation);
                        withdrawn_inflation.push((*reward_currency, reward_inflation));
                    },
                );

                EraSharesAndWithdrawnRewards::<T>::mutate(
                    (pool, era, who),
                    |(share, withdrawn_rewards)| {
                        *share = share.saturating_add(add_amount);
                        // update withdrawn inflation for each reward currency
                        withdrawn_inflation.into_iter().for_each(
                            |(reward_currency, reward_inflation)| {
                                withdrawn_rewards
                                    .entry(reward_currency)
                                    .and_modify(|withdrawn_reward| {
                                        *withdrawn_reward =
                                            withdrawn_reward.saturating_add(reward_inflation);
                                    })
                                    .or_insert(reward_inflation);
                            },
                        );
                    },
                );

                Ok(())
            });
            Ok(())
        })
    }

    pub fn era_remove_share(
        who: &T::AccountId,
        pool: &T::PoolId,
        era: EraIndex,
        remove_amount: T::Share,
    ) {
        if remove_amount.is_zero() {
            return;
        }

        // claim rewards firstly
        Self::era_claim_rewards(who, pool, era);

        EraSharesAndWithdrawnRewards::<T>::mutate_exists((pool, era, who), |share_info| {
            if let Some((mut share, mut withdrawn_rewards)) = share_info.take() {
                let remove_amount = remove_amount.min(share);
                if remove_amount.is_zero() {
                    return;
                }

                EraPoolInfos::<T>::mutate_exists(pool, era, |maybe_pool_info| {
                    if let Some(mut pool_info) = maybe_pool_info.take() {
                        let removing_share = U256::from(remove_amount.saturated_into::<u128>());

                        pool_info.total_shares =
                            pool_info.total_shares.saturating_sub(remove_amount);

                        // update withdrawn rewards for each reward currency.
                        withdrawn_rewards.iter_mut().for_each(
                            |(reward_currency, withdrawn_reward)| {
                                let withdrawn_reward_to_remove: T::Balance = removing_share
                                    .saturating_mul(
                                        withdrawn_reward.to_owned().saturated_into::<u128>().into(),
                                    )
                                    .checked_div(share.saturated_into::<u128>().into())
                                    .unwrap_or_default()
                                    .as_u128()
                                    .saturated_into();
                                if let Some((total_reward, total_withdrawn_reward)) =
                                    pool_info.rewards.get_mut(reward_currency)
                                {
                                    *total_reward =
                                        total_reward.saturating_sub(withdrawn_reward_to_remove);
                                    *total_withdrawn_reward = total_withdrawn_reward
                                        .saturating_sub(withdrawn_reward_to_remove);

                                    // remove if all reward is withdrawn
                                    if total_reward.is_zero() {
                                        pool_info.rewards.remove(reward_currency);
                                    }
                                }
                                *withdrawn_reward =
                                    withdrawn_reward.saturating_sub(withdrawn_reward_to_remove);
                            },
                        );

                        if !pool_info.total_shares.is_zero() {
                            *maybe_pool_info = Some(pool_info);
                        }
                    }
                });

                share = share.saturating_sub(remove_amount);
                if !share.is_zero() {
                    *share_info = Some((share, withdrawn_rewards));
                }
            }
        });
    }

    pub fn era_claim_rewards(who: &T::AccountId, pool: &T::PoolId, era: EraIndex) {
        EraSharesAndWithdrawnRewards::<T>::mutate_exists(
            (pool, era, who),
            |maybe_share_withdrawn| {
                if_std!(println!(
                    "SupportRewards: claim_rewards maybe_share_withdrawn {:?}",
                    maybe_share_withdrawn
                ));
                if let Some((share, withdrawn_rewards)) = maybe_share_withdrawn {
                    if_std!(println!(
                        "SupportRewards: claim_rewards share {:?};  withdrawn_rewards {:?}",
                        share, withdrawn_rewards
                    ));
                    if share.is_zero() {
                        return;
                    }

                    EraPoolInfos::<T>::mutate(pool, era, |pool_info| {
                        if_std!(println!(
                            "SupportRewards: claim_rewards pool_info {:?}",
                            pool_info
                        ));
                        let total_shares =
                            U256::from(pool_info.total_shares.to_owned().saturated_into::<u128>());
                        if_std!(println!(
                            "SupportRewards: claim_rewards total_shares {:?}; rewards {:?}",
                            total_shares, withdrawn_rewards
                        ));
                        pool_info.rewards.iter_mut().for_each(
                    |(reward_currency, (total_reward, total_withdrawn_reward))| {
                        let withdrawn_reward = withdrawn_rewards
                            .get(reward_currency)
                            .copied()
                            .unwrap_or_default();

                        /*let total_reward_proportion_0: T::Balance =
                            U256::from(share.to_owned().saturated_into::<u128>())
                            .saturating_mul(U256::from(total_reward.to_owned().saturated_into::<u128>()))
                            .as_u128()
                            .unique_saturated_into();
                        */

                        let total_reward_proportion: T::Balance =
                            U256::from(share.to_owned().saturated_into::<u128>())
                            .saturating_mul(U256::from(
                                    total_reward.to_owned().saturated_into::<u128>(),
                            ))
                            .checked_div(total_shares)
                            .unwrap_or_default()
                            .as_u128()
                            .unique_saturated_into();


                        if_std! {
                            println!("SupportReward: claim_rewards total_reward: {:?}", total_reward);
                            println!("SupportReward: claim_rewards total_reward_proportion: {:?}", total_reward_proportion)
                        };
                        let reward_to_withdraw = total_reward_proportion
                            .saturating_sub(withdrawn_reward)
                            .min(total_reward.saturating_sub(*total_withdrawn_reward));

                        if_std! {
                            println!("SupportReward: claim_rewards reward_to_withdraw {:?}", reward_to_withdraw)
                        };

                        if reward_to_withdraw.is_zero() {
                            return;
                        }

                        *total_withdrawn_reward =
                            total_withdrawn_reward.saturating_add(reward_to_withdraw);
                        withdrawn_rewards.insert(
                            *reward_currency,
                            withdrawn_reward.saturating_add(reward_to_withdraw),
                        );

                        // pay reward to `who`
                        T::Handler::payout(who, pool, era, *reward_currency, reward_to_withdraw);
                    },
                )
                    })
                }
            },
        )
    }
}
