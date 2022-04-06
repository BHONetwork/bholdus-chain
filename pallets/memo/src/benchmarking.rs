//! Benchmarking setup for pallet-template

#![cfg(feature = "runtime-benchmarks")]

pub use super::*;

#[allow(unused)]
use crate::Pallet as Vaults;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
    dispatch::UnfilteredDispatchable,
    traits::{EnsureOrigin, Get},
};
use frame_system::RawOrigin as SystemOrigin;
use scale_info::prelude::string::String;
use sp_std::vec;

const UNIT: u128 = 10_u128.pow(18);

fn funded_account<T: Config>(caller: &T::AccountId, amount: u128) -> T::AccountId {
    T::Currency::make_free_balance_be(caller, amount);
    caller.clone()
}

benchmarks! {
    create {
        let n in 0 .. 1000;
        let s in 0 .. 1000;

        let caller: T::AccountId = whitelisted_caller();
        let caller_lookup = T::Lookup::unlookup(caller.clone());

        let content: Vec<u8> = vec![0u8; n as usize];
        let chain_id: ChainId = 10;
        let txn_hash: TxnHash = vec![0u8; s as usize];
        let sender: T::AccountId = whitelisted_caller();
        let sender_lookup = T::Lookup::unlookup(sender.clone());
        let receiver: T::AccountId = whitelisted_caller();
        let receiver_lookup = T::Lookup::unlookup(receiver.clone());
        funded_account::<T>(&caller, UNIT);

    }: _(SystemOrigin::Signed(caller.clone()), chain_id, txn_hash, content, sender_lookup.clone(), receiver_lookup.clone())

    // update {
    //     let n in 0 .. 1000;

    //     let caller: T::AccountId = whitelisted_caller();
    //     let caller_lookup = T::Lookup::unlookup(caller.clone());

    //     let content: Vec<u8> = String::from("TEST").into_bytes();
    //     let chain_id: ChainId = 10;
    //     let txn_hash: TxnHash = String::from("HASH").into_bytes();
    //     let sender: T::AccountId = whitelisted_caller();
    //     let sender_lookup = T::Lookup::unlookup(sender.clone());
    //     let receiver: T::AccountId = whitelisted_caller();
    //     let receiver_lookup = T::Lookup::unlookup(receiver.clone());
    //     funded_account::<T>(&caller, UNIT);

    //     Pallet::<T>::create(SystemOrigin::Signed(caller.clone()).into(), chain_id, txn_hash.clone(), content, sender_lookup.clone(), receiver_lookup.clone())?;

    //     let new_content: Vec<u8> = vec![0u8; n as usize];

    // }: _(SystemOrigin::Signed(caller.clone()), chain_id, txn_hash, new_content)
}

impl_benchmark_test_suite!(
    Vaults,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
