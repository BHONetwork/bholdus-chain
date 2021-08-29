use crate::{mock::*, Error, ProvisioningParameters, TradingPairStatus};
use bholdus_primitives::{ExchangeRate, TradingPair};
use bholdus_support::MultiCurrency;
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};
use sp_runtime::{
    traits::{BadOrigin, One, Zero},
    FixedPointNumber,
};

fn initialize_tokens(builder: &ExtBuilder) {
    assert_ok!(BholdusTokens::create(
        Origin::signed(BNB_ADMIN),
        BNB,
        BNB_ADMIN,
        1u128
    ));

    assert_ok!(BholdusTokens::create(
        Origin::signed(DOT_ADMIN),
        DOT,
        DOT_ADMIN,
        1u128
    ));

    assert_ok!(BholdusTokens::create(
        Origin::signed(ALICE),
        TradingPair::from_currency_ids(BHO, BNB)
            .unwrap()
            .dex_share_currency_id(),
        ALICE,
        1u128
    ));

    assert_ok!(BholdusTokens::create(
        Origin::signed(ALICE),
        TradingPair::from_currency_ids(BNB, DOT)
            .unwrap()
            .dex_share_currency_id(),
        ALICE,
        1u128
    ));

    for (currency_id, account_id, balance) in builder.balances.clone() {
        if currency_id == DOT {
            assert_ok!(BholdusTokens::mint(
                Origin::signed(DOT_ADMIN),
                currency_id,
                account_id,
                balance
            ));
        } else if currency_id == BNB {
            assert_ok!(BholdusTokens::mint(
                Origin::signed(BNB_ADMIN),
                currency_id,
                account_id,
                balance
            ));
        }
    }
}

#[test]
fn start_trading_pair_provisioning_should_work() {
    let builder = ExtBuilder::default();
    builder.build().execute_with(|| {
        System::set_block_number(1);

        initialize_tokens(&builder);

        // Only ListingOrigin can call
        assert_noop!(
            Dex::start_trading_pair_provisioning(
                Origin::signed(ALICE),
                BHO,
                BNB,
                1_000u128,
                2_000u128,
                1_000u128,
                2_000u128
            ),
            BadOrigin
        );
        assert_eq!(
            Dex::trading_pair_statuses(BHOBNBPair::get()),
            TradingPairStatus::<_>::Disabled
        );

        // Only valid trading pair is allowed
        assert_noop!(
            Dex::start_trading_pair_provisioning(
                Origin::root(),
                BHO,
                BHO,
                1_000u128,
                2_000u128,
                1_000u128,
                2_000u128
            ),
            crate::Error::<Runtime>::InvalidCurrencyId
        );
        assert_noop!(
            Dex::start_trading_pair_provisioning(
                Origin::root(),
                BHO,
                BHOBNBPair::get().dex_share_currency_id(),
                1_000u128,
                2_000u128,
                1_000u128,
                2_000u128
            ),
            crate::Error::<Runtime>::InvalidCurrencyId
        );

        // Valid trading pair should have provisioning status
        assert_ok!(Dex::start_trading_pair_provisioning(
            Origin::root(),
            BHO,
            BNB,
            1_000u128,
            2_000u128,
            1_000u128,
            2_000u128
        ));
        assert_eq!(
            Dex::trading_pair_statuses(BHOBNBPair::get()),
            TradingPairStatus::<_>::Provisioning(ProvisioningParameters {
                min_contribution: (2_000u128, 1_000u128),
                target_contribution: (2_000u128, 1_000u128),
                accumulated_contribution: Default::default()
            })
        );
        System::assert_last_event(Event::Dex(crate::Event::TradingPairProvisioning(
            TradingPair::from_currency_ids(BHO, BNB).unwrap(),
        )));

        // Only disabled trading pairs can become provisioning
        assert_noop!(
            Dex::start_trading_pair_provisioning(
                Origin::root(),
                BHO,
                BNB,
                1_000u128,
                2_000u128,
                1_000u128,
                2_000u128
            ),
            Error::<Runtime>::TradingPairMustBeDisabled
        );
    });
}

#[test]
fn update_provisioning_parameters_should_work() {
    let builder = ExtBuilder::default();
    builder.build().execute_with(|| {
        System::set_block_number(1);
        initialize_tokens(&builder);
        // Should throw error for invalid trading pair
        assert_noop!(
            Dex::update_trading_pair_provisioning_parameters(
                Origin::root(),
                BHO,
                BHOBNBPair::get().dex_share_currency_id(),
                1_000u128,
                2_000u128,
                1_000u128,
                2_000u128
            ),
            Error::<Runtime>::InvalidCurrencyId
        );

        // Only works for provisioning trading pairs
        assert_noop!(
            Dex::update_trading_pair_provisioning_parameters(
                Origin::root(),
                BHO,
                BNB,
                1_000u128,
                2_000u128,
                1_000u128,
                2_000u128
            ),
            Error::<Runtime>::TradingPairMustBeProvisioning
        );

        // Only works with correct listing origin
        assert_noop!(
            Dex::update_trading_pair_provisioning_parameters(
                Origin::signed(ALICE),
                BHO,
                BNB,
                1_000u128,
                2_000u128,
                1_000u128,
                2_000u128
            ),
            BadOrigin
        );

        // New parameters should store correctly
        assert_ok!(Dex::start_trading_pair_provisioning(
            Origin::root(),
            BHO,
            BNB,
            1_000u128,
            2_000u128,
            1_000u128,
            2_000u128
        ));
        assert_ok!(Dex::update_trading_pair_provisioning_parameters(
            Origin::root(),
            BHO,
            BNB,
            1_000u128,
            2_000u128,
            10_000u128,
            20_000u128
        ));
        assert_eq!(
            Dex::trading_pair_statuses(BHOBNBPair::get()),
            TradingPairStatus::<_>::Provisioning(ProvisioningParameters {
                min_contribution: (2_000u128, 1_000u128),
                target_contribution: (20_000u128, 10_000u128),
                accumulated_contribution: (0, 0),
            })
        );
    });
}

#[test]
fn enable_disabled_trading_pair_should_work() {
    let builder = ExtBuilder::default();
    builder.build().execute_with(|| {
        System::set_block_number(1);
        initialize_tokens(&builder);

        // Only works for listing origin
        assert_noop!(
            Dex::enable_trading_pair(Origin::signed(ALICE), BHO, BNB),
            BadOrigin
        );
        assert_eq!(
            Dex::trading_pair_statuses(BHOBNBPair::get()),
            TradingPairStatus::<_>::Disabled
        );
        // Should return error for invalid trading pair
        assert_noop!(
            Dex::enable_trading_pair(
                Origin::root(),
                BHO,
                BHOBNBPair::get().dex_share_currency_id()
            ),
            Error::<Runtime>::InvalidCurrencyId
        );

        // Enable with listing origin shoud works
        assert_ok!(Dex::enable_trading_pair(Origin::root(), BHO, BNB));
        assert_eq!(
            Dex::trading_pair_statuses(BHOBNBPair::get()),
            TradingPairStatus::Enabled
        );
    });
}

#[test]
fn enable_provisioning_trading_pair_without_provision_yet_should_work() {
    let builder = ExtBuilder::default();
    builder.build().execute_with(|| {
        System::set_block_number(1);

        initialize_tokens(&builder);

        assert_ok!(Dex::start_trading_pair_provisioning(
            Origin::root(),
            BHO,
            BNB,
            1_000u128,
            2_000u128,
            1_000u128,
            2_000u128
        ));
        assert_ok!(Dex::start_trading_pair_provisioning(
            Origin::root(),
            BNB,
            DOT,
            1_000u128,
            2_000u128,
            1_000u128,
            2_000u128
        ));

        // Should throw error for invalid trading pair
        assert_noop!(
            Dex::enable_trading_pair(Origin::signed(ALICE), BHO, BNB),
            BadOrigin
        );
        // Only listing origin can work
        assert_noop!(
            Dex::enable_trading_pair(Origin::signed(ALICE), BHO, BNB),
            BadOrigin
        );

        // Provisioning trading pair that are already provisioned should throw error
        assert_ok!(Dex::add_trading_pair_provision(
            Origin::signed(ALICE),
            BHO,
            BNB,
            1_000u128,
            2_000u128
        ));
        assert_noop!(
            Dex::enable_trading_pair(Origin::root(), BHO, BNB),
            Error::<Runtime>::TradingPairAlreadyProvisioned
        );
        assert_eq!(
            Dex::trading_pair_statuses(BHOBNBPair::get()),
            TradingPairStatus::<_>::Provisioning(ProvisioningParameters {
                min_contribution: (2_000u128, 1_000u128),
                target_contribution: (2_000u128, 1_000u128),
                accumulated_contribution: (2_000u128, 1_000u128)
            })
        );

        // Provisioning trading pair that are not yet provisioned shoud work
        assert_ok!(Dex::enable_trading_pair(Origin::root(), BNB, DOT));
        assert_eq!(
            Dex::trading_pair_statuses(BNBDOTPair::get()),
            TradingPairStatus::<_>::Enabled
        );
        System::assert_last_event(Event::Dex(crate::Event::TradingPairEnabled(
            BNBDOTPair::get(),
        )));
    });
}

#[test]
fn add_provision_should_work() {
    let builder = ExtBuilder::default();
    builder.build().execute_with(|| {
        System::set_block_number(1);
        initialize_tokens(&builder);

        assert_noop!(
            Dex::add_trading_pair_provision(
                Origin::signed(ALICE),
                BHO,
                BHOBNBPair::get().dex_share_currency_id(),
                1_000u128,
                2_000u128
            ),
            Error::<Runtime>::InvalidCurrencyId
        );

        // Disabled trading pairs are not allowed to add provision
        assert_noop!(
            Dex::add_trading_pair_provision(Origin::signed(ALICE), BHO, BNB, 1_000u128, 2_000u128),
            Error::<Runtime>::TradingPairMustBeProvisioning
        );
        assert_eq!(
            Dex::trading_pair_statuses(BHOBNBPair::get()),
            TradingPairStatus::<_>::Disabled
        );

        assert_ok!(Dex::start_trading_pair_provisioning(
            Origin::root(),
            BHO,
            BNB,
            1_000u128,
            2_000u128,
            1_000u128,
            2_000u128
        ));

        // Alice adds provision but not satisfies minimum contribution
        assert_noop!(
            Dex::add_trading_pair_provision(Origin::signed(ALICE), BHO, BNB, 500u128, 1_000u128),
            Error::<Runtime>::InvalidContributionIncrement
        );

        // Alice adds provision and it should work correcly
        assert_ok!(Dex::add_trading_pair_provision(
            Origin::signed(ALICE),
            BHO,
            BNB,
            1_000u128,
            2_000u128
        ));
        assert_eq!(
            Dex::provisioning_pool(BHOBNBPair::get(), ALICE),
            (2_000u128, 1_000u128)
        );
        assert_eq!(
            Dex::trading_pair_statuses(BHOBNBPair::get()),
            TradingPairStatus::Provisioning(ProvisioningParameters {
                min_contribution: (2_000u128, 1_000u128),
                target_contribution: (2_000u128, 1_000u128),
                accumulated_contribution: (2_000u128, 1_000u128),
            })
        );
        System::assert_last_event(Event::Dex(crate::Event::AddProvision(
            ALICE, BNB, 2_000u128, BHO, 1_000u128,
        )));

        // Alice adds provision second times
        assert_ok!(Dex::add_trading_pair_provision(
            Origin::signed(ALICE),
            BHO,
            BNB,
            3_000u128,
            4_000u128
        ));
        assert_eq!(
            Dex::provisioning_pool(BHOBNBPair::get(), ALICE),
            (6_000u128, 4_000u128)
        );
        assert_eq!(
            Dex::trading_pair_statuses(BHOBNBPair::get()),
            TradingPairStatus::Provisioning(ProvisioningParameters {
                min_contribution: (2_000u128, 1_000u128),
                target_contribution: (2_000u128, 1_000u128),
                accumulated_contribution: (6_000u128, 4_000u128),
            })
        );
        System::assert_last_event(Event::Dex(crate::Event::AddProvision(
            ALICE, BNB, 4_000u128, BHO, 3_000u128,
        )));

        // Bob adds provision
        assert_ok!(Dex::add_trading_pair_provision(
            Origin::signed(BOB),
            BHO,
            BNB,
            4_000u128,
            5_000u128
        ));
        assert_eq!(
            Dex::provisioning_pool(BHOBNBPair::get(), BOB),
            (5_000u128, 4_000u128)
        );
        assert_eq!(
            Dex::trading_pair_statuses(BHOBNBPair::get()),
            TradingPairStatus::Provisioning(ProvisioningParameters {
                min_contribution: (2_000u128, 1_000u128),
                target_contribution: (2_000u128, 1_000u128),
                accumulated_contribution: (11_000u128, 8_000u128),
            })
        );
        System::assert_last_event(Event::Dex(crate::Event::AddProvision(
            BOB, BNB, 5_000u128, BHO, 4_000u128,
        )));

        // Alice adds provision that out of her balance should show error
        assert_noop!(
            Dex::add_trading_pair_provision(
                Origin::signed(ALICE),
                BHO,
                BNB,
                2_000_000_000u128,
                2_000_000_000u128
            ),
            bholdus_tokens::Error::<Runtime>::BalanceLow
        );
    });
}

#[test]
fn enable_provisioning_trading_pair_should_work() {
    let builder = ExtBuilder::default();
    builder.build().execute_with(|| {
        System::set_block_number(1);

        initialize_tokens(&builder);

        // Only listing origin can enable provisioning trading pair
        assert_noop!(
            Dex::enable_provisioning_trading_pair(Origin::signed(ALICE), BHO, BNB),
            BadOrigin
        );

        // Enable invalid trading pair should throw error
        assert_noop!(
            Dex::enable_provisioning_trading_pair(
                Origin::root(),
                BHO,
                BHOBNBPair::get().dex_share_currency_id()
            ),
            Error::<Runtime>::InvalidCurrencyId
        );

        // Disabled trading pair should throw error
        assert_noop!(
            Dex::enable_provisioning_trading_pair(Origin::root(), BHO, BNB),
            Error::<Runtime>::TradingPairMustBeProvisioning
        );

        assert_ok!(Dex::start_trading_pair_provisioning(
            Origin::root(),
            BHO,
            BNB,
            0u128,
            0_000u128,
            1_000u128,
            2_000u128
        ));

        // Alice adds provision
        assert_ok!(Dex::add_trading_pair_provision(
            Origin::signed(ALICE),
            BHO,
            BNB,
            0u128,
            1_000u128
        ));
        // Enable trading pair at this moment should throw error since target contribution is not
        // reached
        assert_noop!(
            Dex::enable_provisioning_trading_pair(Origin::root(), BHO, BNB),
            Error::<Runtime>::UnqualifiedProvision
        );

        // Alice adds more provision
        assert_ok!(Dex::add_trading_pair_provision(
            Origin::signed(ALICE),
            BHO,
            BNB,
            0u128,
            1_000u128
        ));
        // Enable trading pair at this moment should throw error since target contribution is
        // reached but either one currency provision is zero
        assert_noop!(
            Dex::enable_provisioning_trading_pair(Origin::root(), BHO, BNB),
            Error::<Runtime>::UnqualifiedProvision
        );

        // Bobs add provision
        assert_ok!(Dex::add_trading_pair_provision(
            Origin::signed(BOB),
            BHO,
            BNB,
            1_000u128,
            1_000u128
        ));

        // Enable trading pair should work since target contribution is reached and neither
        // currency provision is zero
        assert_ok!(Dex::enable_provisioning_trading_pair(
            Origin::root(),
            BHO,
            BNB
        ));
        assert_eq!(
            Dex::trading_pair_statuses(BHOBNBPair::get()),
            TradingPairStatus::<_>::Enabled
        );
        // Initial share exchange rate should be stored so first liquidity providers can claim
        // their shares later
        assert_eq!(
            Dex::initial_share_exchange_rates(BHOBNBPair::get()),
            (
                ExchangeRate::one(),
                ExchangeRate::checked_from_rational(3_000u128, 1_000u128).unwrap()
            )
        );
        // Liquditiy pool must be set correctly
        assert_eq!(
            Dex::liquidity_pool(BHOBNBPair::get()),
            (3_000u128, 1_000u128)
        );
        // Event must be emitted
        System::assert_last_event(Event::Dex(
            crate::Event::TradingPairEnabledFromProvisioning(
                BNB, 3_000u128, BHO, 1_000u128, 6_000u128,
            ),
        ));
    });
}

#[test]
fn claim_dex_share_should_work() {
    let builder = ExtBuilder::default();
    builder.build().execute_with(|| {
        System::set_block_number(1);

        initialize_tokens(&builder);

        assert_ok!(Dex::start_trading_pair_provisioning(
            Origin::root(),
            BHO,
            BNB,
            1_000u128,
            2_000u128,
            1_000u128,
            2_000u128
        ));
        assert_ok!(Dex::add_trading_pair_provision(
            Origin::signed(ALICE),
            BHO,
            BNB,
            1_000u128,
            2_000u128
        ));
        assert_ok!(Dex::add_trading_pair_provision(
            Origin::signed(BOB),
            BHO,
            BNB,
            1_000u128,
            2_000u128
        ));
        assert_ok!(Dex::enable_provisioning_trading_pair(
            Origin::root(),
            BHO,
            BNB
        ));

        // Alice comes to claim her dex share and this should work
        assert_ok!(Dex::claim_dex_share(Origin::signed(ALICE), ALICE, BHO, BNB));
        assert_eq!(
            Dex::provisioning_pool(BHOBNBPair::get(), ALICE),
            (Zero::zero(), Zero::zero())
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(
                BHOBNBPair::get().dex_share_currency_id(),
                &ALICE
            ),
            4_000u128
        );
        // Bob claims his dex share and this should work
        assert_ok!(Dex::claim_dex_share(Origin::signed(BOB), BOB, BHO, BNB));
        assert_eq!(
            Dex::provisioning_pool(BHOBNBPair::get(), BOB),
            (Zero::zero(), Zero::zero())
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(
                BHOBNBPair::get().dex_share_currency_id(),
                &BOB
            ),
            4_000u128
        );
    });
}

#[test]
fn add_liquidity_should_work() {
    let builder = ExtBuilder::default();
    builder.build().execute_with(|| {
        System::set_block_number(1);
        initialize_tokens(&builder);

        // Alice adds liquidity to invalid trading pair. This should throw error
        assert_noop!(
            Dex::add_liquidity(
                Origin::signed(ALICE),
                BHO,
                BHOBNBPair::get().dex_share_currency_id(),
                1_000u128,
                2_000u128,
            ),
            Error::<Runtime>::InvalidCurrencyId
        );
        // Alice adds liquidity to disabled trading pair. This should throw error
        assert_noop!(
            Dex::add_liquidity(Origin::signed(ALICE), BHO, BNB, 1_000u128, 2_000u128,),
            Error::<Runtime>::TradingPairMustBeEnabled
        );
        // Alice ads liquidity to provisioning trading pair. This should show error
        assert_ok!(Dex::start_trading_pair_provisioning(
            Origin::root(),
            BHO,
            BNB,
            0u128,
            0u128,
            0u128,
            0u128
        ));
        assert_noop!(
            Dex::add_liquidity(Origin::signed(ALICE), BHO, BNB, 1_000u128, 2_000u128,),
            Error::<Runtime>::TradingPairMustBeEnabled
        );
        // Enable trading pair
        assert_ok!(Dex::enable_trading_pair(Origin::root(), BHO, BNB));
        // Alice adds liquidity but exceeds her balance so this should throw error
        assert_noop!(
            Dex::add_liquidity(
                Origin::signed(ALICE),
                BHO,
                BNB,
                2_000_000_000u128,
                2_000_000_000u128
            ),
            bholdus_tokens::Error::<Runtime>::BalanceLow
        );
        // Alice adds zero amount of either currency which is invalid
        assert_noop!(
            Dex::add_liquidity(Origin::signed(ALICE), BHO, BNB, 0, 1_000u128),
            Error::<Runtime>::InvalidLiquidityIncrement
        );

        // Alice adds liquidity successfully and is first liquidity provider
        assert_ok!(Dex::add_liquidity(
            Origin::signed(ALICE),
            BHO,
            BNB,
            1_000u128,
            2_000u128
        ));
        // Liquidity pool must be set correctly
        assert_eq!(
            Dex::liquidity_pool(BHOBNBPair::get()),
            (2_000u128, 1_000u128)
        );
        // Alice receives correct shares amount
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(
                BHOBNBPair::get().dex_share_currency_id(),
                &ALICE
            ),
            4_000u128
        );
        // Total share must be correct
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_issuance(
                BHOBNBPair::get().dex_share_currency_id(),
            ),
            4_000u128
        );
        // Alice should deposit her balance into liquidity pool
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BHO, &ALICE),
            999_999_000u128
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BNB, &ALICE),
            999_998_000u128
        );
        // Event must be emitted
        System::assert_last_event(Event::Dex(crate::Event::AddLiquidity(
            ALICE, BNB, 2_000u128, BHO, 1_000u128, 4_000u128,
        )));
        // Bob adds liquidity
        assert_ok!(Dex::add_liquidity(
            Origin::signed(BOB),
            BHO,
            BNB,
            2_000u128,
            4_000u128
        ));
        // Liquidity pool must be set correctly
        assert_eq!(
            Dex::liquidity_pool(BHOBNBPair::get()),
            (6_000u128, 3_000u128)
        );
        // Bob receives correct shares amount
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(
                BHOBNBPair::get().dex_share_currency_id(),
                &BOB
            ),
            8_000u128
        );
        // Total share must be correct
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_issuance(
                BHOBNBPair::get().dex_share_currency_id(),
            ),
            12_000u128
        );
        // Bob should deposit his balance into liquidity pool
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BHO, &BOB),
            999_998_000u128
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BNB, &BOB),
            999_996_000u128
        );
        // Event must be emitted
        System::assert_last_event(Event::Dex(crate::Event::AddLiquidity(
            BOB, BNB, 4_000u128, BHO, 2_000u128, 8_000u128,
        )));
        // Bob adds liquidity that has different exchange rate than current exchange rate. We should
        // match his exchange rate to current exchange rate.
        assert_ok!(Dex::add_liquidity(
            Origin::signed(BOB),
            BHO,
            BNB,
            3_000u128,
            7_000u128
        ));
        assert_eq!(
            Dex::liquidity_pool(BHOBNBPair::get()),
            (12_000u128, 6_000u128)
        );
        // Total share must be correct
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_issuance(
                BHOBNBPair::get().dex_share_currency_id(),
            ),
            24_000u128
        );
        // Bob receives correct shares amount
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(
                BHOBNBPair::get().dex_share_currency_id(),
                &BOB
            ),
            20_000u128
        );
        // Bob should deposit his balance into liquidity pool
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BHO, &BOB),
            999_995_000u128
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BNB, &BOB),
            999_990_000u128
        );
    });
}

#[test]
fn remove_liquidity_should_work() {
    let builder = ExtBuilder::default();
    builder.build().execute_with(|| {
        initialize_tokens(&builder);
        System::set_block_number(1);

        // Remove liquidity from invalid trading pair shoud throw error
        assert_noop!(
            Dex::remove_liquidity(
                Origin::signed(ALICE),
                BHO,
                BHOBNBPair::get().dex_share_currency_id(),
                10,
                10,
                10
            ),
            Error::<Runtime>::InvalidCurrencyId
        );

        // Remove zero share should throw error
        assert_noop!(
            Dex::remove_liquidity(
                Origin::signed(ALICE),
                BHO,
                BHOBNBPair::get().dex_share_currency_id(),
                0,
                10,
                10
            ),
            Error::<Runtime>::InvalidRemoveShareAmount
        );

        // Remove liquidity from trading pair that doesn't have any share should throw error
        assert_noop!(
            Dex::remove_liquidity(Origin::signed(ALICE), BHO, BNB, 100, 10, 10),
            Error::<Runtime>::ZeroTotalShare
        );

        assert_ok!(Dex::enable_trading_pair(Origin::root(), BHO, BNB));

        // Alice adds liquidity as first liquidity provider
        assert_ok!(Dex::add_liquidity(
            Origin::signed(ALICE),
            BHO,
            BNB,
            1_000_000u128,
            5_000_000u128
        ));

        // Alice removes liquidity with specified minimum withdrawn amount higher than actual
        // withdrawn amount should throw error
        assert_noop!(
            Dex::remove_liquidity(
                Origin::signed(ALICE),
                BHO,
                BNB,
                8_000_000u128,
                800_001u128,
                4_000_000u128
            ),
            Error::<Runtime>::UnacceptableWithdrawnAmount
        );
        assert_noop!(
            Dex::remove_liquidity(
                Origin::signed(ALICE),
                BHO,
                BNB,
                8_000_000u128,
                800_000u128,
                4_000_001u128
            ),
            Error::<Runtime>::UnacceptableWithdrawnAmount
        );

        // Alice removes liqudity successfully
        assert_ok!(Dex::remove_liquidity(
            Origin::signed(ALICE),
            BHO,
            BNB,
            8_000_000u128,
            800_000u128,
            4_000_000u128
        ));
        assert_eq!(
            Dex::liquidity_pool(BHOBNBPair::get()),
            (1_000_000u128, 200_000u128)
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(
                BHOBNBPair::get().dex_share_currency_id(),
                &ALICE
            ),
            2_000_000u128
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_issuance(
                BHOBNBPair::get().dex_share_currency_id(),
            ),
            2_000_000u128
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BHO, &ALICE),
            999_800_000u128
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BNB, &ALICE),
            999_000_000u128
        );
        System::assert_last_event(Event::Dex(crate::Event::RemoveLiqudity(
            ALICE,
            BNB,
            4_000_000u128,
            BHO,
            800_000u128,
            8_000_000u128,
        )));

        // Bob adds liquidity
        assert_ok!(Dex::add_liquidity(
            Origin::signed(BOB),
            BHO,
            BNB,
            200_000u128,
            1_000_000u128
        ));
        assert_eq!(
            Dex::liquidity_pool(BHOBNBPair::get()),
            (2_000_000u128, 400_000u128)
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_issuance(
                BHOBNBPair::get().dex_share_currency_id(),
            ),
            4_000_000u128
        );
        // Bob removes liquidity
        assert_ok!(Dex::remove_liquidity(
            Origin::signed(BOB),
            BHO,
            BNB,
            2_000_000u128,
            200_000u128,
            1_000_000u128
        ));
        assert_eq!(
            Dex::liquidity_pool(BHOBNBPair::get()),
            (1_000_000u128, 200_000u128)
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(
                BHOBNBPair::get().dex_share_currency_id(),
                &BOB
            ),
            0
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_issuance(
                BHOBNBPair::get().dex_share_currency_id(),
            ),
            2_000_000u128
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BHO, &BOB),
            1_000_000_000u128
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BNB, &BOB),
            1_000_000_000u128
        );
        System::assert_last_event(Event::Dex(crate::Event::RemoveLiqudity(
            BOB,
            BNB,
            1_000_000u128,
            BHO,
            200_000u128,
            2_000_000u128,
        )));
    });
}

#[test]
fn swap_with_exact_supply_should_work() {
    let builder = ExtBuilder::default();
    builder.build().execute_with(|| {
        System::set_block_number(1);
        initialize_tokens(&builder);

        // Swap with path exceeds trading path limit should throw error
        assert_noop!(
            Dex::swap_with_exact_supply(
                Origin::signed(ALICE),
                vec![BHO, BNB, DOT, BTC],
                1_000u128,
                1_000u128
            ),
            Error::<Runtime>::InvalidTradingPathLength
        );

        // Swap with path length that not at least form a trading pair should throw error
        assert_noop!(
            Dex::swap_with_exact_supply(Origin::signed(ALICE), vec![], 1_000u128, 1_000u128),
            Error::<Runtime>::InvalidTradingPathLength
        );
        assert_noop!(
            Dex::swap_with_exact_supply(Origin::signed(ALICE), vec![BHO], 1_000u128, 1_000u128),
            Error::<Runtime>::InvalidTradingPathLength
        );

        // Swap with trading path containing at least disabled/provisioning trading pair should throw
        // error
        assert_noop!(
            Dex::swap_with_exact_supply(
                Origin::signed(ALICE),
                vec![BHO, BNB, DOT],
                1_000u128,
                1_000u128
            ),
            Error::<Runtime>::TradingPairMustBeEnabled
        );
        assert_ok!(Dex::start_trading_pair_provisioning(
            Origin::root(),
            BHO,
            BNB,
            1_000u128,
            2_000u128,
            1_000u128,
            2_000u128
        ));
        assert_noop!(
            Dex::swap_with_exact_supply(
                Origin::signed(ALICE),
                vec![BHO, BNB, DOT],
                1_000u128,
                1_000u128
            ),
            Error::<Runtime>::TradingPairMustBeEnabled
        );

        assert_ok!(Dex::enable_trading_pair(Origin::root(), BHO, BNB));
        assert_ok!(Dex::enable_trading_pair(Origin::root(), BNB, DOT));

        // Swap with trading path that has zero liquidity (BHO-BNB) should throw error
        assert_noop!(
            Dex::swap_with_exact_supply(
                Origin::signed(ALICE),
                vec![BHO, BNB, DOT],
                1_000u128,
                1_000u128
            ),
            Error::<Runtime>::InsufficientLiquidity
        );

        assert_ok!(Dex::add_liquidity(
            Origin::signed(ALICE),
            BHO,
            BNB,
            100_000_000u128,
            200_000_000u128
        ));

        // Swap zero supply amount should throw error
        assert_noop!(
            Dex::swap_with_exact_supply(Origin::signed(ALICE), vec![BHO, BNB], 0u128, 0u128),
            Error::<Runtime>::ZeroTargetAmount
        );

        // Swap with trading path that has zero liquidity (BNB-DOT) should throw error
        assert_noop!(
            Dex::swap_with_exact_supply(
                Origin::signed(ALICE),
                vec![BHO, BNB, DOT],
                1_000u128,
                1_000u128
            ),
            Error::<Runtime>::InsufficientLiquidity
        );

        assert_ok!(Dex::add_liquidity(
            Origin::signed(ALICE),
            BNB,
            DOT,
            100_000_000u128,
            200_000_000u128
        ));

        // Alice swap BHO for BNB but her balance is not enough. This should throw error
        assert_noop!(
            Dex::swap_with_exact_supply(
                Origin::signed(ALICE),
                vec![BHO, BNB],
                1_000_000_000u128,
                0u128
            ),
            pallet_balances::Error::<Runtime>::InsufficientBalance
        );

        // Alice swaps BHO for BNB but she wants to receive at least BNB greater than what dex can
        // offer for current exchange rate and fee. This should throw error
        assert_noop!(
            Dex::swap_with_exact_supply(
                Origin::signed(ALICE),
                vec![BHO, BNB],
                100_000_000u128,
                200_000_000u128
            ),
            Error::<Runtime>::InsufficientTargetAmount
        );

        // Alice swaps successfully
        assert_ok!(Dex::swap_with_exact_supply(
            Origin::signed(ALICE),
            vec![BHO, BNB],
            100_000_000u128,
            90_000_000u128
        ));
        System::assert_last_event(Event::Dex(crate::Event::Swap(
            ALICE,
            vec![BHO, BNB],
            100_000_000u128,
            94_736_842u128,
        )));
        assert_eq!(
            Dex::liquidity_pool(BHOBNBPair::get()),
            (105_263_158u128, 200_000_000u128)
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BHO, &ALICE),
            800_000_000u128
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BNB, &ALICE),
            794_736_842u128
        );

        // Alice swaps BHO for DOT (BHO-BNB-DOT) successfully
        assert_ok!(Dex::swap_with_exact_supply(
            Origin::signed(ALICE),
            vec![BHO, BNB, DOT],
            100_000_000u128,
            40_000_000u128
        ));
        System::assert_last_event(Event::Dex(crate::Event::Swap(
            ALICE,
            vec![BHO, BNB, DOT],
            100_000_000u128,
            45_441_794u128,
        )));
        assert_eq!(
            Dex::liquidity_pool(BHOBNBPair::get()),
            (72_595_282u128, 300_000_000u128)
        );
        assert_eq!(
            Dex::liquidity_pool(BNBDOTPair::get()),
            (154_558_206u128, 132_667_876u128)
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BHO, &ALICE),
            700_000_000u128
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(DOT, &ALICE),
            845_441_794u128
        );
    });
}

#[test]
fn swap_with_exact_target_amount() {
    let builder = ExtBuilder::default();
    builder.build().execute_with(|| {
        System::set_block_number(1);
        initialize_tokens(&builder);

        // Swap with path exceeds trading path limit should throw error
        assert_noop!(
            Dex::swap_with_exact_target(
                Origin::signed(ALICE),
                vec![BHO, BNB, DOT, BTC],
                1_000u128,
                1_000u128
            ),
            Error::<Runtime>::InvalidTradingPathLength
        );

        // Swap with path length that not at least form a trading pair should throw error
        assert_noop!(
            Dex::swap_with_exact_target(Origin::signed(ALICE), vec![], 1_000u128, 1_000u128),
            Error::<Runtime>::InvalidTradingPathLength
        );
        assert_noop!(
            Dex::swap_with_exact_target(Origin::signed(ALICE), vec![BHO], 1_000u128, 1_000u128),
            Error::<Runtime>::InvalidTradingPathLength
        );

        // Swap with trading path containing at least disabled/provisioning trading pair should throw
        // error
        assert_noop!(
            Dex::swap_with_exact_target(
                Origin::signed(ALICE),
                vec![BHO, BNB, DOT],
                1_000u128,
                1_000u128
            ),
            Error::<Runtime>::TradingPairMustBeEnabled
        );
        assert_ok!(Dex::start_trading_pair_provisioning(
            Origin::root(),
            BHO,
            BNB,
            1_000u128,
            2_000u128,
            1_000u128,
            2_000u128
        ));
        assert_noop!(
            Dex::swap_with_exact_target(
                Origin::signed(ALICE),
                vec![BHO, BNB, DOT],
                1_000u128,
                1_000u128
            ),
            Error::<Runtime>::TradingPairMustBeEnabled
        );

        assert_ok!(Dex::enable_trading_pair(Origin::root(), BHO, BNB));
        assert_ok!(Dex::enable_trading_pair(Origin::root(), BNB, DOT));

        // Swap with trading path that has zero liquidity (BHO-BNB) should throw error
        assert_noop!(
            Dex::swap_with_exact_target(
                Origin::signed(ALICE),
                vec![BHO, BNB, DOT],
                1_000u128,
                1_000u128
            ),
            Error::<Runtime>::InsufficientLiquidity
        );

        assert_ok!(Dex::add_liquidity(
            Origin::signed(ALICE),
            BHO,
            BNB,
            100_000_000u128,
            200_000_000u128
        ));

        // Swap zero target amount should throw error
        assert_noop!(
            Dex::swap_with_exact_target(Origin::signed(ALICE), vec![BHO, BNB], 0u128, 0u128),
            Error::<Runtime>::ZeroSupplyAmount
        );

        // Swap with trading path that has zero liquidity (BNB-DOT) should throw error
        assert_noop!(
            Dex::swap_with_exact_target(
                Origin::signed(ALICE),
                vec![BHO, BNB, DOT],
                1_000u128,
                1_000u128
            ),
            Error::<Runtime>::InsufficientLiquidity
        );

        assert_ok!(Dex::add_liquidity(
            Origin::signed(ALICE),
            BNB,
            DOT,
            100_000_000u128,
            200_000_000u128
        ));

        // Alice swap BHO for BNB but her target amount is larger than the pool. This should throw error
        assert_noop!(
            Dex::swap_with_exact_target(
                Origin::signed(ALICE),
                vec![BHO, BNB],
                1_000_000_000u128,
                0u128
            ),
            Error::<Runtime>::InsufficientLiquidity
        );

        // Alice swaps BHO for BNB but she wants to spend at most some BHO that is smaller than what dex can
        // offer for current exchange rate and fee. This should throw error
        assert_noop!(
            Dex::swap_with_exact_target(
                Origin::signed(ALICE),
                vec![BHO, BNB],
                100_000_000u128,
                100_000_000u128
            ),
            Error::<Runtime>::InsufficientSupplyAmount
        );

        // Alice swaps successfully
        assert_ok!(Dex::swap_with_exact_target(
            Origin::signed(ALICE),
            vec![BHO, BNB],
            100_000_000u128,
            120_000_000u128
        ));
        System::assert_last_event(Event::Dex(crate::Event::Swap(
            ALICE,
            vec![BHO, BNB],
            111_111_112u128,
            100_000_000u128,
        )));
        assert_eq!(
            Dex::liquidity_pool(BHOBNBPair::get()),
            (100_000_000u128, 211_111_112u128)
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BHO, &ALICE),
            788_888_888u128
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BNB, &ALICE),
            800_000_000u128
        );

        // Alice swaps BHO for DOT (BHO-BNB-DOT) got error because of insufficient liquidity of
        // BHO-BNB.
        assert_noop!(
            Dex::swap_with_exact_target(
                Origin::signed(ALICE),
                vec![BHO, BNB, DOT],
                100_000_000u128,
                40_000_000u128
            ),
            Error::<Runtime>::InsufficientLiquidity
        );

        // Alice swaps successfully
        assert_ok!(Dex::swap_with_exact_target(
            Origin::signed(ALICE),
            vec![BHO, BNB, DOT],
            50_000_000u128,
            140_000_000u128
        ));
        System::assert_last_event(Event::Dex(crate::Event::Swap(
            ALICE,
            vec![BHO, BNB, DOT],
            137_981_125u128,
            50_000_000u128,
        )));
        assert_eq!(
            Dex::liquidity_pool(BHOBNBPair::get()),
            (62_962_962u128, 349_092_237u128)
        );
        assert_eq!(
            Dex::liquidity_pool(BNBDOTPair::get()),
            (150_000_000u128, 137_037_038u128)
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BHO, &ALICE),
            650_907_763u128
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(BNB, &ALICE),
            800_000_000u128,
        );
        assert_eq!(
            <Runtime as crate::Config>::Currency::total_balance(DOT, &ALICE),
            850_000_000u128
        );
    });
}
