//! Benchmarking setup for integration-tokens

use super::*;

#[allow(unused)]
use crate::Pallet as IntegrationTokens;
use frame_benchmarking::{
    account, benchmarks_instance_pallet, impl_benchmark_test_suite, whitelist_account,
    whitelisted_caller, benchmarks
};
use frame_system::RawOrigin as SystemOrigin;
use pallet_contracts::chain_extension::UncheckedFrom;
use bholdus_primitives::{Balance, TokenId as CurrencyId};

const SEED: u32 = 0;

benchmarks! {
     transfer {
        let amount = Balance::from(100u32);
        let transfer_amount = Balance::from(90u32);

        let currency_id = CurrencyId::from(1);
        let caller: T::AccountId = whitelisted_caller();
        let caller_lookup = T::Lookup::unlookup(caller.clone());
        
        let target: T::AccountId = account("target", 0, SEED);
        let target_lookup = T::Lookup::unlookup(target.clone());

    }: _(SystemOrigin::Signed(caller.clone()), currency_id, target_lookup, transfer_amount)
}

impl_benchmark_test_suite!(IntegrationTokens, crate::mock::new_test_ext(), crate::mock::Test);
