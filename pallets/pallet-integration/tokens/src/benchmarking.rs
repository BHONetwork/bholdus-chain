//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller, account};
use frame_system::RawOrigin as SystemOrigin;
use pallet_contracts::chain_extension::UncheckedFrom;
use bholdus_primitives::{Balance, TokenId as CurrencyId};

const SEED: u32 = 0;

benchmarks! {
    where_clause {
        where
        T::AccountId: UncheckedFrom<T::Hash>,
        T::AccountId: AsRef<[u8]>,
     }
     transfer {
        let amount = Balance::from(100u32);
        let transfer_amount = Balance::from(90u32);

        let currency_id = 1;
        let caller: T::AccountId = whitelisted_caller();
        let caller_lookup = T::Lookup::unlookup(caller.clone());
        let target: T::AccountId = whitelisted_caller();
        let target_lookup = T::Lookup::unlookup(target.clone());

    }: _(SystemOrigin::Signed(caller.clone()), target_lookup, currency_id, transfer_amount)
}

impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
