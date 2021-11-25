//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as BridgeNativeTransfer;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

const UNIT: u128 = 10_u128.pow(18);

fn funded_account<T: Config>(caller: &T::AccountId, amount: u128) -> T::AccountId {
    T::Currency::make_free_balance_be(caller, amount);
    caller.clone()
}

benchmarks! {
    initiate_transfer {
        let s in 1 .. 1000;
        let caller: T::AccountId = whitelisted_caller();
        let to = vec![0x2e, 0x86, 0x88, 0x27, 0xCC, 0xb7, 0xB0, 0x15, 0x55, 0x2a, 0x98, 0x17, 0xca, 0x3E, 0x9E, 0xE3, 0xa0, 0x8A, 0xe5, 0x96];
        let amount = UNIT.checked_mul(s.into()).unwrap();
        Pallet::<T>::force_register_chain(RawOrigin::Root.into(), 1)?;
        funded_account::<T>(&caller, amount.checked_mul(10).unwrap());

    }: _(RawOrigin::Signed(caller), to.clone(), amount, 1)

    confirm_transfer {
        let s in 1 .. 1000;
        let caller: T::AccountId = whitelisted_caller();
        let to = vec![0x2e, 0x86, 0x88, 0x27, 0xCC, 0xb7, 0xB0, 0x15, 0x55, 0x2a, 0x98, 0x17, 0xca, 0x3E, 0x9E, 0xE3, 0xa0, 0x8A, 0xe5, 0x96];
        let amount = UNIT.checked_mul(s.into()).unwrap();
        Pallet::<T>::force_register_chain(RawOrigin::Root.into(), 1)?;
        Pallet::<T>::force_register_relayer(RawOrigin::Root.into(), caller.clone())?;
        funded_account::<T>(&caller, amount.checked_mul(10).unwrap());
        funded_account::<T>(&Pallet::<T>::pallet_account_id(), amount.checked_mul(10).unwrap());
        Pallet::<T>::initiate_transfer(RawOrigin::Signed(caller.clone()).into(),to.clone(),amount,1)?;

    }: _(RawOrigin::Signed(caller), 0)

    release_tokens {
        let s in 1 .. 1000;
        let caller: T::AccountId = whitelisted_caller();
        let from = vec![0x2e, 0x86, 0x88, 0x27, 0xCC, 0xb7, 0xB0, 0x15, 0x55, 0x2a, 0x98, 0x17, 0xca, 0x3E, 0x9E, 0xE3, 0xa0, 0x8A, 0xe5, 0x96];
        let amount = UNIT.checked_mul(s.into()).unwrap();
        Pallet::<T>::force_register_chain(RawOrigin::Root.into(), 1)?;
        Pallet::<T>::force_register_relayer(RawOrigin::Root.into(), caller.clone())?;
        funded_account::<T>(&caller, amount.checked_mul(10).unwrap());
        funded_account::<T>(&Pallet::<T>::pallet_account_id(), amount.checked_mul(10).unwrap());

    }: _(RawOrigin::Signed(caller.clone()), 0, from.clone(), caller.clone(), amount)

    force_register_relayer {
        let s in 1 .. 1000;
        let caller: T::AccountId = whitelisted_caller();

    }: _(RawOrigin::Root, caller)

    force_unregister_relayer {
        let s in 1 .. 1000;
        let caller: T::AccountId = whitelisted_caller();

    }: _(RawOrigin::Root, caller)

    force_register_chain {
        let s in 1 .. 1000;
        let caller: T::AccountId = whitelisted_caller();

    }: _(RawOrigin::Root, 1)

    force_unregister_chain {
        let s in 1 .. 1000;
        let caller: T::AccountId = whitelisted_caller();

    }: _(RawOrigin::Root, 1)

    force_set_service_fee {
        let s in 1 .. 1000;
        let caller: T::AccountId = whitelisted_caller();

    }: _(RawOrigin::Root, (3,10))

    force_withdraw {
        let s in 1 .. 1000;
        let caller: T::AccountId = whitelisted_caller();

    }: _(RawOrigin::Root, caller)

}

impl_benchmark_test_suite!(
    BridgeNativeTransfer,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
