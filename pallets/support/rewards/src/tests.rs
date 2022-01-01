//! Unit tests for the rewards

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

#[test]
fn add_share_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        /* assert_eq!(SupportReward::pool_infos(BHO_POOL), Default::default());

        assert_eq!(
            SupportReward::shares_and_withdrawn_rewards(BHO_POOL, ALICE),
            Default::default()
        );
        */

        SupportReward::add_share(&ALICE, &BHO_POOL, 0);

        assert_eq!(
            SupportReward::shares_and_withdrawn_rewards(BHO_POOL, ALICE),
            Default::default()
        );

        SupportReward::add_share(&ALICE, &BHO_POOL, 100);
        assert_eq!(
            SupportReward::pool_infos(BHO_POOL),
            PoolInfo {
                total_shares: 100,
                ..Default::default()
            }
        );

        assert_eq!(
            SupportReward::shares_and_withdrawn_rewards(&BHO_POOL, &ALICE),
            (100, Default::default())
        );

        PoolInfos::<Runtime>::mutate(BHO_POOL, |pool_info| {
            pool_info.rewards.insert(NATIVE_COIN, (5_000, 2_000));
        });

        assert_eq!(
            SupportReward::pool_infos(BHO_POOL),
            PoolInfo {
                total_shares: 100,
                rewards: vec![(NATIVE_COIN, (5_000, 2_000))].into_iter().collect()
            }
        );

        assert_eq!(
            SupportReward::shares_and_withdrawn_rewards(BHO_POOL, BOB),
            Default::default()
        );

        SupportReward::add_share(&BOB, &BHO_POOL, 50);

        assert_eq!(
            SupportReward::pool_infos(BHO_POOL),
            PoolInfo {
                total_shares: 150,
                rewards: vec![(NATIVE_COIN, (7_500, 4_500))].into_iter().collect()
            }
        );

        assert_eq!(
            SupportReward::shares_and_withdrawn_rewards(BHO_POOL, BOB),
            (50, vec![(NATIVE_COIN, 2_500)].into_iter().collect())
        );

        SupportReward::add_share(&ALICE, &BHO_POOL, 100);
        assert_eq!(
            SupportReward::pool_infos(BHO_POOL),
            PoolInfo {
                total_shares: 250,
                rewards: vec![(NATIVE_COIN, (12_500, 9_500))].into_iter().collect()
            }
        );
    });
}

#[test]
fn claim_rewards_should_not_create_empty_records() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(PoolInfos::<Runtime>::contains_key(&BHO_POOL), false);
        assert_eq!(
            SharesAndWithdrawnRewards::<Runtime>::contains_key(&BHO_POOL, &ALICE),
            false
        );
        SupportReward::claim_rewards(&ALICE, &BHO_POOL);
        assert_eq!(PoolInfos::<Runtime>::contains_key(&BHO_POOL), false);
        assert_eq!(
            SharesAndWithdrawnRewards::<Runtime>::contains_key(&BHO_POOL, &ALICE),
            false
        );
    })
}

#[test]
fn remove_share_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        SupportReward::add_share(&ALICE, &BHO_POOL, 100);
        SupportReward::add_share(&BOB, &BHO_POOL, 100);
        PoolInfos::<Runtime>::mutate(BHO_POOL, |pool_info| {
            pool_info.rewards.insert(NATIVE_COIN, (10_000, 0));
        });

        assert_eq!(
            SupportReward::pool_infos(BHO_POOL),
            PoolInfo {
                total_shares: 200,
                rewards: vec![(NATIVE_COIN, (10_000, 0))].into_iter().collect()
            }
        );
        assert_eq!(
            SupportReward::shares_and_withdrawn_rewards(BHO_POOL, ALICE),
            (100, Default::default())
        );

        assert_eq!(
            SupportReward::shares_and_withdrawn_rewards(BHO_POOL, BOB),
            (100, Default::default())
        );

        assert_eq!(
            RECEIVED_PAYOUT.with(|v| *v
                .borrow()
                .get(&(BHO_POOL, ALICE, NATIVE_COIN))
                .unwrap_or(&0)),
            0
        );
        assert_eq!(
            RECEIVED_PAYOUT.with(|v| *v
                .borrow()
                .get(&(BHO_POOL, ALICE, NATIVE_COIN))
                .unwrap_or(&0)),
            0
        );

        // remove amount is zero, do not claim interest
        SupportReward::remove_share(&ALICE, &BHO_POOL, 0);
        assert_eq!(
            SupportReward::pool_infos(BHO_POOL),
            PoolInfo {
                total_shares: 200,
                rewards: vec![(NATIVE_COIN, (10_000, 0))].into_iter().collect()
            }
        );

        assert_eq!(
            SupportReward::shares_and_withdrawn_rewards(BHO_POOL, ALICE),
            (100, Default::default())
        );

        assert_eq!(
            RECEIVED_PAYOUT.with(|v| *v
                .borrow()
                .get(&(BHO_POOL, ALICE, NATIVE_COIN))
                .unwrap_or(&0)),
            0
        );

        SupportReward::remove_share(&BOB, &BHO_POOL, 50);
        assert_eq!(
            SupportReward::pool_infos(BHO_POOL),
            PoolInfo {
                total_shares: 150,
                rewards: vec![(NATIVE_COIN, (7_500, 2_500))].into_iter().collect()
            }
        );

        assert_eq!(
            SupportReward::shares_and_withdrawn_rewards(BHO_POOL, BOB),
            (50, vec![(NATIVE_COIN, 2_500)].into_iter().collect())
        );

        assert_eq!(
            RECEIVED_PAYOUT.with(|v| *v.borrow().get(&(BHO_POOL, BOB, NATIVE_COIN)).unwrap_or(&0)),
            5_000
        );

        SupportReward::remove_share(&ALICE, &BHO_POOL, 101);
        assert_eq!(
            SupportReward::pool_infos(BHO_POOL),
            PoolInfo {
                total_shares: 50,
                rewards: vec![(NATIVE_COIN, (2_500, 2_500))].into_iter().collect()
            }
        );

        assert_eq!(
            SupportReward::pool_infos(BHO_POOL),
            PoolInfo {
                total_shares: 50,
                rewards: vec![(NATIVE_COIN, (2_500, 2_500))].into_iter().collect()
            }
        );

        assert_eq!(
            SupportReward::shares_and_withdrawn_rewards(BHO_POOL, ALICE),
            (0, Default::default())
        );

        assert_eq!(
            SharesAndWithdrawnRewards::<Runtime>::contains_key(&BHO_POOL, &ALICE),
            false
        );

        assert_eq!(
            RECEIVED_PAYOUT.with(|v| *v
                .borrow()
                .get(&(BHO_POOL, ALICE, NATIVE_COIN))
                .unwrap_or(&0)),
            5_000
        );

        // remove all shares will remove entries
        SupportReward::remove_share(&BOB, &BHO_POOL, 100);
        assert_eq!(SupportReward::pool_infos(BHO_POOL), PoolInfo::default());
        assert_eq!(PoolInfos::<Runtime>::contains_key(BHO_POOL), false);
        assert_eq!(PoolInfos::<Runtime>::iter().count(), 0);
        assert_eq!(
            SharesAndWithdrawnRewards::<Runtime>::contains_key(&BHO_POOL, &BOB),
            false
        );
        assert_eq!(SharesAndWithdrawnRewards::<Runtime>::iter().count(), 0);
    });
}

#[test]
fn claim_rewards_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        SupportReward::add_share(&ALICE, &BHO_POOL, 100);
        SupportReward::add_share(&BOB, &BHO_POOL, 100);
        PoolInfos::<Runtime>::mutate(BHO_POOL, |pool_info| {
            pool_info.rewards.insert(NATIVE_COIN, (5_000, 0));
        });
        SupportReward::add_share(&CAROL, &BHO_POOL, 200);
        assert_eq!(
            SupportReward::pool_infos(BHO_POOL),
            PoolInfo {
                total_shares: 400,
                rewards: vec![(NATIVE_COIN, (10_000, 5_000))].into_iter().collect()
            }
        );

        assert_eq!(
            SupportReward::shares_and_withdrawn_rewards(BHO_POOL, ALICE),
            (100, Default::default())
        );

        assert_eq!(
            SupportReward::shares_and_withdrawn_rewards(BHO_POOL, BOB),
            (100, Default::default())
        );

        assert_eq!(
            SupportReward::shares_and_withdrawn_rewards(BHO_POOL, CAROL),
            (200, vec![(NATIVE_COIN, 5_000)].into_iter().collect())
        );

        SupportReward::claim_rewards(&ALICE, &BHO_POOL);
        assert_eq!(
            SupportReward::pool_infos(BHO_POOL),
            PoolInfo {
                total_shares: 400,
                rewards: vec![(NATIVE_COIN, (10_000, 7_500))].into_iter().collect()
            }
        );

        assert_eq!(
            SupportReward::shares_and_withdrawn_rewards(BHO_POOL, ALICE),
            (100, vec![(NATIVE_COIN, 2_500)].into_iter().collect())
        );

        assert_eq!(
            RECEIVED_PAYOUT.with(|v| *v
                .borrow()
                .get(&(BHO_POOL, ALICE, NATIVE_COIN))
                .unwrap_or(&0)),
            2_500
        );
    })
}
