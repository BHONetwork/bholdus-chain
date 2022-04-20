//! Benchmarks for the nft module.

#![cfg(feature = "runtime-benchmarks")]

use sp_std::vec;

use frame_benchmarking::{account, benchmarks};
use frame_support::{dispatch::DispatchErrorWithPostInfo, traits::Get, weights::DispatchClass};
use frame_system::RawOrigin;
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};

pub use crate::*;
use common_primitives::Balance;

pub struct Module<T: Config>(crate::Pallet<T>);

const SEED: u32 = 0;

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}

fn test_attr() -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	for i in 0..30 {
		attr.insert(vec![i], vec![0; 64]);
	}
	attr
}

fn create_token_class<T: Config>(
	caller: T::AccountId,
) -> Result<T::AccountId, DispatchErrorWithPostInfo> {
	let module_account: T::AccountId =
		T::PalletId::get().into_sub_account(bholdus_support_nft::Pallet::<T>::next_class_id());

	crate::Pallet::<T>::create_class(RawOrigin::Signed(caller).into(), test_attr())?;
	Ok(module_account)
}

benchmarks! {
	// create NFT class
	create_class {
		let caller: T::AccountId = account("caller", 0, SEED);
	}: _(RawOrigin::Signed(caller), test_attr())

	// mint NFT token
	mint {
		let i in 1 .. 99;

		let caller: T::AccountId = account("caller", 0, SEED);
		let to: T::AccountId = account("to", 0, SEED);
		let to_lookup = T::Lookup::unlookup(to);
		let account = create_token_class::<T>(caller)?;

	}: _(RawOrigin::Signed(account), to_lookup, 0u32.into(), vec![1], test_attr(), i)

	// transfer NFT token to another account
	transfer {
		let caller: T::AccountId = account("caller", 0, SEED);
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		let to: T::AccountId = account("to", 0, SEED);
		let to_lookup = T::Lookup::unlookup(to.clone());
		let account = create_token_class::<T>(caller)?;

		crate::Pallet::<T>::mint(
			RawOrigin::Signed(account).into(),
			to_lookup,
			0u32.into(),
			vec![1],
			test_attr(),
			1)?;
	}: _(RawOrigin::Signed(to), caller_lookup, (0u32.into(), 0u32.into()))

	// burn NFT token
	burn {
		let caller: T::AccountId = account("caller", 0, SEED);
		let to: T::AccountId = account("to", 0, SEED);
		let to_lookup = T::Lookup::unlookup(to.clone());
		let account = create_token_class::<T>(caller)?;
		crate::Pallet::<T>::mint(RawOrigin::Signed(account).into(), to_lookup, 0u32.into(), vec![1], test_attr(), 1)?;
	}: _(RawOrigin::Signed(to), (0u32.into(), 0u32.into()))

	// destroy NFT class
	destroy_class {
		let caller: T::AccountId = account("caller", 0, SEED);
		create_token_class::<T>(caller.clone())?;
	}: _(RawOrigin::Signed(caller.clone()), 0u32.into())
}

#[cfg(test)]
mod tests {
	use super::{mock::*, *};
	use frame_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(Pallet, super::mock::ExtBuilder::default().build(), super::Runtime,);
}
