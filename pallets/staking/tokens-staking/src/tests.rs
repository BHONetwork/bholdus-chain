//! Unit tests

#![cfg(test)]

use super::{pallet::pallet::Error, Event, *};
use bholdus_primitives::{Amount, Balance, Rate};
use bholdus_support::{MultiCurrency, RewardHandler};
use bholdus_support_rewards::{PoolInfo, PoolOwner, Pools};
use frame_support::{
    assert_noop, assert_ok,
    traits::{OnInitialize, OnUnbalanced},
};
use mock::{Balances, *};
use sp_runtime::{
    traits::{BadOrigin, Zero},
    FixedPointNumber,
};
use sp_std::if_std;

/// Used to get total
pub(crate) fn get_total_reward_per_era() -> Balance {
    BLOCKS_REWARD * BLOCKS_PER_ERA as Balance
}

#[test]
fn test_bholdus_tokens() {
    ExtBuilder::default().build().execute_with(|| {
        let asset_id = BTokens::next_asset_id();
        let minimum_balance = 1;
        let amount = 100;
        assert_ok!(BTokens::force_create(
            Origin::root(),
            asset_id,
            ALICE::get(),
            true,
            minimum_balance.clone()
        ));
        assert_ok!(BTokens::mint(
            Origin::signed(ALICE::get()),
            asset_id,
            ALICE::get(),
            amount.clone()
        ));
        assert_eq!(
            BTokens::total_balance(asset_id, &ALICE::get()),
            amount.clone()
        );
        assert_eq!(BTokens::total_issuance(asset_id), amount.clone());
    })
}

/*#[test]
fn add_pool() {
    ExtBuilder::default().build().execute_with(|| {
        let pool_id = PoolId::Dex(BTC);
        let invalid_pool_id = PoolId::Dex(ACA);
        assert_ok!(BStakingTokens::add_pool(Origin::signed(ALICE::get()), BTC));
        // key: (owner, pool_id)
        assert!(PoolOwner::<Runtime>::contains_key((ALICE::get(), pool_id)));
        // key: (pool_id)
        assert!(Pools::<Runtime>::contains_key(pool_id));
        assert!(!Pools::<Runtime>::contains_key(invalid_pool_id));
    })
}
*/

#[test]
fn add_pool_should_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        // let pool_id = PoolId::Dex(BTC);
        assert_noop!(
            BStakingTokens::add_pool(Origin::signed(ALICE::get()), BTC),
            Error::<Runtime>::NoPermission
        );
        deposit_for_account(ALICE::get(), BTC, 10000);
        assert_eq!(BTokens::is_owner(&ALICE::get(), BTC), true);
        assert_ok!(BTokens::verify_asset(Origin::root(), BTC));
        assert_eq!(BTokens::is_verifiable(BTC), true);
        // assert_ok!(BStakingTokens::add_pool(Origin::signed(ALICE::get()), BTC));
    })
}

#[test]
fn on_initialize_is_ok() {
    ExtBuilder::default().build().execute_with(|| {
        // Before we start, era is zero
        assert!(BStakingTokens::current_era().is_zero());

        // We initialize the firt block and advance to second one.
        // New era must be triggered.
        initialize_first_block();
        let current_era = BStakingTokens::current_era();
        assert_eq!(1, current_era);

        // Now advance by history limit. Ensure that rewards for era 1 still exist
        let previous_era = current_era;
        advance_to_era(previous_era + HistoryDepth::get() + 1);

        // Check that all reward&stakes are as expected
        let current_era = BStakingTokens::current_era();
        for era in 1..current_era {}
    })
}

/* #[test]
fn staking_info_is_ok() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_first_block();

        // Prepare a little scenario
        // staker 1 --> stakes starting era, doesn't unstake
        // staker 2 --> stakes starting era, unstake everything before final era
        // staker 3 --> stakes after starting era, doesn't unstake

        let starting_era = 3;
        advance_to_era(starting_era);
        deposit_for_account(ALICE::get(), BTC_AUSD_LP, 10000);
        assert_eq!(BTokens::total_balance(BTC_AUSD_LP, &ALICE::get()), 10000);
        assert_eq!(
            BTokens::total_balance(BTC_AUSD_LP, &BStakingTokens::account_id()),
            0
        );
        assert_eq!(
            SupportReward::era_pool_infos(PoolId::Dex(BTC_AUSD_LP), starting_era),
            PoolInfo::default(),
        );

        assert_eq!(
            SupportReward::era_shares_and_withdrawn_rewards((
                PoolId::Dex(BTC_AUSD_LP),
                starting_era,
                ALICE::get()
            )),
            Default::default(),
        );

        assert_ok!(BStakingTokens::deposit_dex_share(
            Origin::signed(ALICE::get()),
            BTC_AUSD_LP,
            10000
        ));

        System::assert_last_event(mock::Event::BStakingTokens(Event::DepositDexShare(
            ALICE::get(),
            BTC_AUSD_LP,
            10000,
        )));

        assert_eq!(BTokens::total_balance(BTC_AUSD_LP, &ALICE::get()), 0);

        assert_eq!(
            BTokens::total_balance(BTC_AUSD_LP, &BStakingTokens::account_id()),
            10000
        );

        assert_eq!(
            SupportReward::era_pool_infos(PoolId::Dex(BTC_AUSD_LP), starting_era),
            PoolInfo {
                total_shares: 10000,
                ..Default::default()
            }
        );

        assert_eq!(
            SupportReward::era_shares_and_withdrawn_rewards((
                PoolId::Dex(BTC_AUSD_LP),
                starting_era,
                ALICE::get()
            )),
            (10000, Default::default())
        );
    });
}
*/

/* #[test]
fn withdraw_dex_share_works() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_first_block();

        let current_era = BStakingTokens::current_era();
        /*if_std!(
            println!("withdraw_dex_share_works: current_era {:?}", current_era);
        );
        */
        deposit_for_account(ALICE::get(), BTC_AUSD_LP, 10000);
        assert_eq!(BTokens::total_balance(BTC_AUSD_LP, &ALICE::get()), 10000);
        assert_eq!(
            BTokens::total_balance(BTC_AUSD_LP, &BStakingTokens::account_id()),
            0
        );

        assert_noop!(
            BStakingTokens::withdraw_dex_share(Origin::signed(BOB::get()), BTC_AUSD_LP, 10000),
            Error::<Runtime>::NotEnough,
        );

        assert_ok!(BStakingTokens::deposit_dex_share(
            Origin::signed(ALICE::get()),
            BTC_AUSD_LP,
            10000
        ));

        assert_eq!(BTokens::total_balance(BTC_AUSD_LP, &ALICE::get()), 0);
        assert_eq!(
            BTokens::total_balance(BTC_AUSD_LP, &BStakingTokens::account_id()),
            10000
        );
        assert_eq!(
            SupportReward::era_pool_infos(PoolId::Dex(BTC_AUSD_LP), current_era),
            PoolInfo {
                total_shares: 10000,
                ..Default::default()
            }
        );

        assert_eq!(
            SupportReward::era_shares_and_withdrawn_rewards((
                PoolId::Dex(BTC_AUSD_LP),
                current_era,
                ALICE::get()
            )),
            (10000, Default::default())
        );

        assert_ok!(BStakingTokens::withdraw_dex_share(
            Origin::signed(ALICE::get()),
            BTC_AUSD_LP,
            8000
        ));
        System::assert_last_event(mock::Event::BStakingTokens(Event::WithdrawDexShare(
            ALICE::get(),
            BTC_AUSD_LP,
            8000,
        )));

        assert_eq!(BTokens::total_balance(BTC_AUSD_LP, &ALICE::get()), 8000);
        assert_eq!(
            BTokens::total_balance(BTC_AUSD_LP, &BStakingTokens::account_id()),
            2000
        );

        assert_eq!(
            SupportReward::era_pool_infos(PoolId::Dex(BTC_AUSD_LP), current_era),
            PoolInfo {
                total_shares: 2000,
                ..Default::default()
            }
        );

        assert_eq!(
            SupportReward::era_shares_and_withdrawn_rewards((
                PoolId::Dex(BTC_AUSD_LP),
                current_era,
                ALICE::get()
            )),
            (2000, Default::default())
        );
    });
}

*/

#[test]
fn payout_works() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_first_block();
        let start_era = BStakingTokens::current_era();
        assert_eq!(
            BStakingTokens::pending_multi_rewards((PoolId::Dex(BTC), start_era, ALICE::get())),
            BTreeMap::default()
        );
        BStakingTokens::payout(&ALICE::get(), &PoolId::Dex(BTC), start_era, ACA, 1000);

        assert_eq!(
            BStakingTokens::pending_multi_rewards((PoolId::Dex(BTC), start_era, ALICE::get())),
            vec![(ACA, 1000)].into_iter().collect()
        );
        BStakingTokens::payout(&ALICE::get(), &PoolId::Dex(BTC), start_era, ACA, 1000);

        assert_eq!(
            BStakingTokens::pending_multi_rewards((PoolId::Dex(BTC), start_era, ALICE::get())),
            vec![(ACA, 2000)].into_iter().collect()
        );
    });
}

#[test]
fn claim_rewards_works() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_first_block();

        deposit_for_account(VAULT::get(), ACA, 10000);
        deposit_for_account(VAULT::get(), AUSD, 10000);
        deposit_for_account(VAULT::get(), LDOT, 10000);

        let start_era = BStakingTokens::current_era();

        assert!(!Pools::<Runtime>::contains_key(&PoolId::Dex(BTC)));

        assert_noop!(
            BStakingTokens::deposit_dex_share(Origin::signed(ALICE::get()), BTC, 100),
            Error::<Runtime>::PoolDoesNotExist
        );
        /*assert_ok!(BStakingTokens::deposit_dex_share(
            Origin::signed(ALICE::get()),
            BTC,
            100
        ));
        */

        /* assert_ok!(SupportReward::era_add_share(
            &ALICE::get(),
            &PoolId::Dex(BTC),
            start_era,
            100
        ));
        */
        // SupportReward::era_add_share(&ALICE::get(), &PoolId::Dex(BTC_AUSD_LP), start_era, 100);

        // bob add shares before accumulate rewards
        // SupportReward::era_add_share(&BOB::get(), &PoolId::Dex(BTC_AUSD_LP), start_era, 100);

        // accumulate LDOT rewards for PoolId::Dex(BTC)
        // assert_ok!(SupportReward::era_accumulate_reward(
        //     &PoolId::Dex(BTC),
        //     start_era,
        //     ACA,
        //     2000
        // ));
        // assert_ok!(SupportReward::era_accumulate_reward(
        //     &PoolId::Dex(BTC_AUSD_LP),
        //     start_era,
        //     ACA,
        //     1000
        // ));
        // assert_ok!(SupportReward::era_accumulate_reward(
        //     &PoolId::Dex(BTC_AUSD_LP),
        //     start_era,
        //     AUSD,
        //     2000
        // ));

        // bob add share after accumulate rewards
        // SupportReward::era_add_share(&BOB::get(), &PoolId::Dex(BTC), start_era, 100);

        // accumulate LDOT rewards for PoolId::Dex(BTC)
        // assert_ok!(SupportReward::era_accumulate_reward(
        //     &PoolId::Dex(BTC),
        //     start_era,
        //     LDOT,
        //     500
        // ));
    })
}

/*

        // alice claim rewards for PoolId::Dex(BTC)
        assert_eq!(
            SupportReward::era_pool_infos(PoolId::Dex(BTC), start_era),
            PoolInfo {
                total_shares: 200,
                rewards: vec![(ACA, (2000, 0)), (LDOT, (500, 0))]
                    .into_iter()
                    .collect(),
            }
        );

        assert_eq!(
            SupportReward::era_shares_and_withdrawn_rewards((
                PoolId::Dex(BTC),
                start_era,
                ALICE::get()
            )),
            (100, Default::default())
        );

        assert_eq!(BTokens::total_balance(ACA, &VAULT::get()), 10000);
        assert_eq!(BTokens::total_balance(LDOT, &VAULT::get()), 10000);
        assert_eq!(BTokens::total_balance(ACA, &ALICE::get()), 0);
        assert_eq!(BTokens::total_balance(LDOT, &ALICE::get()), 0);
        assert_ok!(BStakingTokens::claim_rewards(
            Origin::signed(ALICE::get()),
            PoolId::Dex(BTC),
            start_era,
        ));

        /*System::assert_has_event(mock::Event::BStakingTokens(Event::ClaimRewards(
            ALICE::get(),
            PoolId::Dex(BTC),
            ACA,
            200,
        )));*/

        /*System::assert_has_event(mock::Event::BStakingTokens(Event::ClaimRewards(
            ALICE::get(),
            PoolId::Dex(BTC),
            LDOT,
            25,
        )));*/

        assert_eq!(
            SupportReward::era_pool_infos(PoolId::Dex(BTC), start_era),
            PoolInfo {
                total_shares: 200,
                rewards: vec![(ACA, (4000, 4000)), (LDOT, (500, 250))]
                    .into_iter()
                    .collect(),
            }
        );

        assert_eq!(
            SupportReward::era_shares_and_withdrawn_rewards((
                PoolId::Dex(BTC),
                start_era,
                ALICE::get()
            )),
            (100, vec![(ACA, 2000), (LDOT, 250)].into_iter().collect())
        );

        assert_eq!(BTokens::total_balance(ACA, &VAULT::get()), 8000);
        assert_eq!(BTokens::total_balance(LDOT, &VAULT::get()), 9750);
        assert_eq!(BTokens::total_balance(ACA, &ALICE::get()), 2000);
        assert_eq!(BTokens::total_balance(LDOT, &ALICE::get()), 250);

        // bob claim rewards for PoolId::Dex(BTC)
        assert_eq!(
            SupportReward::era_shares_and_withdrawn_rewards((
                PoolId::Dex(BTC),
                start_era,
                BOB::get()
            )),
            (100, vec![(ACA, 2000)].into_iter().collect())
        );

        assert_eq!(BTokens::total_balance(ACA, &BOB::get()), 0);
        assert_ok!(BStakingTokens::claim_rewards(
            Origin::signed(BOB::get()),
            PoolId::Dex(BTC),
            start_era,
        ));

        assert_eq!(
            SupportReward::era_pool_infos(PoolId::Dex(BTC), start_era),
            PoolInfo {
                total_shares: 200,
                rewards: vec![(ACA, (4000, 4000)), (LDOT, (500, 500))]
                    .into_iter()
                    .collect(),
            }
        );

        assert_eq!(
            SupportReward::era_shares_and_withdrawn_rewards((
                PoolId::Dex(BTC),
                start_era,
                BOB::get()
            )),
            (100, vec![(ACA, 2000), (LDOT, 250)].into_iter().collect())
        );
    })
}
*/

#[test]
fn update_incentive_rewards_works() {
    ExtBuilder::default().build().execute_with(|| {
        initialize_first_block();

        deposit_for_account(ALICE::get(), DOT_AUSD_LP, 10000);
        deposit_for_account(ALICE::get(), DOT, 10000);
        assert_eq!(BTokens::total_balance(DOT, &ALICE::get()), 10000);
        // assert_ok!(BStakingTokens::add_pool(Origin::signed(ALICE::get()), DOT));

        /*assert_noop!(
            BStakingTokens::update_incentive_rewards(Origin::signed(ALICE::get()), vec![]),
            BadOrigin
        );
        */

        assert_eq!(
            BStakingTokens::incentive_reward_amounts(PoolId::Dex(DOT_AUSD_LP), ACA),
            0
        );

        assert_eq!(
            BStakingTokens::incentive_reward_amounts(PoolId::Dex(DOT_AUSD_LP), DOT),
            0
        );
        assert_ok!(BStakingTokens::update_incentive_rewards(
            Origin::signed(ALICE::get()),
            vec![
                (PoolId::Dex(DOT_AUSD_LP), vec![(ACA, 1000), (DOT, 100)]),
                (PoolId::Dex(DOT), vec![(ACA, 500)]),
            ],
        ));

        System::assert_has_event(mock::Event::BStakingTokens(
            Event::IncentiveRewardAmountUpdated(PoolId::Dex(DOT_AUSD_LP), ACA, 1000),
        ));

        System::assert_has_event(mock::Event::BStakingTokens(
            Event::IncentiveRewardAmountUpdated(PoolId::Dex(DOT), ACA, 500),
        ));

        assert_eq!(
            BStakingTokens::incentive_reward_amounts(PoolId::Dex(DOT_AUSD_LP), ACA),
            1000
        );

        assert_eq!(
            BStakingTokens::incentive_reward_amounts(PoolId::Dex(DOT_AUSD_LP), DOT),
            100
        );

        assert_eq!(
            BStakingTokens::incentive_reward_amounts(PoolId::Dex(DOT), ACA),
            500
        );

        assert_ok!(BStakingTokens::update_incentive_rewards(
            Origin::signed(ROOT::get()),
            vec![
                (PoolId::Dex(DOT_AUSD_LP), vec![(ACA, 200), (DOT, 0)]),
                (PoolId::Dex(DOT), vec![(ACA, 500)]),
            ],
        ));

        System::assert_has_event(mock::Event::BStakingTokens(
            Event::IncentiveRewardAmountUpdated(PoolId::Dex(DOT_AUSD_LP), ACA, 200),
        ));

        System::assert_has_event(mock::Event::BStakingTokens(
            Event::IncentiveRewardAmountUpdated(PoolId::Dex(DOT_AUSD_LP), DOT, 0),
        ));

        assert_eq!(
            BStakingTokens::incentive_reward_amounts(PoolId::Dex(DOT_AUSD_LP), ACA),
            200
        );

        assert_eq!(
            BStakingTokens::incentive_reward_amounts(PoolId::Dex(DOT), ACA),
            500
        );
    })
}
