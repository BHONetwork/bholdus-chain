//! Digital tokens pallet benchmaking.
//!
#![cfg(feature = "runtime-benchmarks")]

pub use super::*;

use frame_benchmarking::{
    account, benchmarks_instance_pallet, impl_benchmark_test_suite, whitelist_account,
    whitelisted_caller,
};

use frame_support::{
    dispatch::UnfilteredDispatchable,
    traits::{EnsureOrigin, Get},
};

use frame_system::RawOrigin as SystemOrigin;
use sp_runtime::traits::Bounded;
use sp_std::prelude::*;

use crate::Pallet as Assets;

const SEED: u32 = 0;

fn create_default_asset<T: Config<I>, I: 'static>(
    is_sufficient: bool,
) -> (T::AccountId, <T::Lookup as StaticLookup>::Source) {
    let caller: T::AccountId = whitelisted_caller();
    let caller_lookup = T::Lookup::unlookup(caller.clone());
    let root = SystemOrigin::Root.into();
    assert!(Assets::<T, I>::force_create(
        root,
        Default::default(),
        caller_lookup.clone(),
        is_sufficient,
        1u32.into(),
    )
    .is_ok());
    (caller, caller_lookup)
}

fn create_default_minted_asset<T: Config<I>, I: 'static>(
    is_sufficient: bool,
    supply: T::Balance,
) -> (T::AccountId, <T::Lookup as StaticLookup>::Source) {
    let (caller, caller_lookup) = create_default_asset::<T, I>(is_sufficient);
    // let n in 0 .. T::StringLimit::get();
    // let s in 0 .. T::StringLimit::get();

    let name = vec![0u8; 1];
    let symbol = vec![0u8; 2];
    let decimals = 12;
    if !is_sufficient {
        T::Currency::make_free_balance_be(&caller, T::Currency::minimum_balance());
    }
    assert!(Assets::<T, I>::create_and_mint(
        SystemOrigin::Signed(caller.clone()).into(), // origin
        caller_lookup.clone(),                       //beneficiary
        name,                                        // name
        symbol,                                      // symbol
        decimals,                                    // decimal
        caller_lookup.clone(),                       //beneficiary
        supply,                                      // supply
        1u32.into(),                                 // minimum balance
    )
    .is_ok());
    (caller, caller_lookup)
}

fn swap_is_sufficient<T: Config<I>, I: 'static>(s: &mut bool) {
    Asset::<T, I>::mutate(&T::AssetId::default(), |maybe_a| {
        if let Some(ref mut a) = maybe_a {
            sp_std::mem::swap(s, &mut a.is_sufficient)
        }
    });
}

fn add_consumers<T: Config<I>, I: 'static>(minter: T::AccountId, n: u32) {
    let origin = SystemOrigin::Signed(minter);
    let mut s = false;
    swap_is_sufficient::<T, I>(&mut s);
    for i in 0..n {
        let target = account("consumer", i, SEED);
        T::Currency::make_free_balance_be(&target, T::Currency::minimum_balance());
        let target_lookup = T::Lookup::unlookup(target);
        assert!(Assets::<T, I>::mint(
            origin.clone().into(),
            Default::default(),
            target_lookup,
            100u32.into()
        )
        .is_ok());
    }
    swap_is_sufficient::<T, I>(&mut s);
}

fn add_sufficients<T: Config<I>, I: 'static>(minter: T::AccountId, n: u32) {
    let origin = SystemOrigin::Signed(minter);
    let mut s = true;
    swap_is_sufficient::<T, I>(&mut s);
    for i in 0..n {
        let target = account("sufficient", i, SEED);
        let target_lookup = T::Lookup::unlookup(target);
        assert!(Assets::<T, I>::mint(
            origin.clone().into(),
            Default::default(),
            target_lookup,
            100u32.into()
        )
        .is_ok());
    }
    swap_is_sufficient::<T, I>(&mut s);
}

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn assert_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::Event) {
    frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

benchmarks_instance_pallet! {
    create {
        let caller: T::AccountId = whitelisted_caller();
        let caller_lookup = T::Lookup::unlookup(caller.clone());
        T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T, I>::max_value());
    }: _(SystemOrigin::Signed(caller.clone()), caller_lookup, 1u32.into())
    verify {
        assert_last_event::<T, I>(Event::Created(Default::default(), caller.clone(), caller).into());
    }

    force_create {
        let caller: T::AccountId = whitelisted_caller();
        let caller_lookup = T::Lookup::unlookup(caller.clone());
    }: _(SystemOrigin::Root, Default::default(), caller_lookup, true, 1u32.into())

    verify {
        assert_last_event::<T, I>(Event::ForceCreated(Default::default(), caller).into());
    }

    destroy {
        let c in 0 .. 5_000;
        let s in 0 .. 5_000;
        let a in 0 .. 5_00;

        let(caller, _) = create_default_asset::<T,I>(true);
        add_consumers::<T, I>(caller.clone(), c);
        add_sufficients::<T, I>(caller.clone(), s);
        let witness = Asset::<T, I>::get(T::AssetId::default()).unwrap().destroy_witness();

    }: _(SystemOrigin::Signed(caller), Default::default(), witness)
    verify {
        assert_last_event::<T, I>(Event::Destroyed(Default::default()).into());
    }

    mint {
        let (caller, caller_lookup) = create_default_asset::<T, I>(true);
        let amount = T::Balance::from(100u32);
    }: _(SystemOrigin::Signed(caller.clone()), Default::default(), caller_lookup, amount)

    verify {
        assert_last_event::<T, I>(Event::Issued(Default::default(), caller, amount).into());
    }

    transfer {
        let amount = T::Balance::from(100u32);
        let (caller, caller_lookup) = create_default_minted_asset::<T, I>(true, amount);
        let target: T::AccountId = account("target", 0, SEED);
        let target_lookup = T::Lookup::unlookup(target.clone());
    }: _(SystemOrigin::Signed(caller.clone()), Default::default(), target_lookup, amount)
    verify {
        assert_last_event::<T, I>(Event::Transferred(Default::default(), caller, target, amount).into());
    }

    freeze {
        let (caller, caller_lookup) = create_default_minted_asset::<T, I>(true, 100u32.into());
    }: _(SystemOrigin::Signed(caller.clone()), Default::default(), caller_lookup)

    verify {
        assert_last_event::<T, I>(Event::Frozen(Default::default(), caller).into());
    }

    thaw {
        let (caller, caller_lookup) = create_default_minted_asset::<T, I>(true, 100u32.into());
        Assets::<T, I>::freeze(
            SystemOrigin::Signed(caller.clone()).into(),
            Default::default(),
            caller_lookup.clone(),
        )?;
    }: _(SystemOrigin::Signed(caller.clone()), Default::default(), caller_lookup)
    verify {
        assert_last_event::<T, I>(Event::Thawed(Default::default(), caller).into());
    }

    freeze_asset {
        let (caller, caller_lookup) = create_default_minted_asset::<T, I>(true, 100u32.into());
    }: _(SystemOrigin::Signed(caller.clone()), Default::default())
    verify {
        assert_last_event::<T, I>(Event::AssetFrozen(Default::default()).into());
    }

    thaw_asset {
        let (caller, caller_lookup) = create_default_minted_asset::<T, I>(true, 100u32.into());
        Assets::<T, I>::freeze_asset(
            SystemOrigin::Signed(caller.clone()).into(),
            Default::default(),
        )?;
    }: _(SystemOrigin::Signed(caller.clone()), Default::default())
    verify {
        assert_last_event::<T, I>(Event::AssetThawed(Default::default()).into());
    }

    set_metadata {
        let n in 0 .. T::StringLimit::get();
        let s in 0 .. T::StringLimit::get();

        let name = vec![0u8; n as usize];
        let symbol = vec![0u8; s as usize];
        let decimals = 12;

        let (caller, _) = create_default_asset::<T, I>(true);
        T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T, I>::max_value());
    }: _(SystemOrigin::Signed(caller), Default::default(), name.clone(), symbol.clone(), decimals)
    verify {
        let id = Default::default();
        assert_last_event::<T, I>(Event::MetadataSet(id, name, symbol, decimals, false).into());
    }

    clear_metadata {
        let (caller, _) = create_default_asset::<T, I>(true);
        T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T, I>::max_value());
        let dummy = vec![0u8; T::StringLimit::get() as usize];
        let origin = SystemOrigin::Signed(caller.clone()).into();
        Assets::<T, I>::set_metadata(origin, Default::default(), dummy.clone(), dummy, 12)?;
    }: _(SystemOrigin::Signed(caller), Default::default())
    verify {
        assert_last_event::<T, I>(Event::MetadataCleared(Default::default()).into());
    }

    set_identity{
        let (caller, _) = create_default_asset::<T, I>(true);
    }: _(SystemOrigin::Signed(caller), Default::default(), Default::default())
    verify {
        let id = Default::default();
        assert_last_event::<T, I>(Event::IdentitySet(id).into());
    }

    set_blacklist{
        let n in 0 .. T::StringLimit::get();
        let s in 0 .. T::StringLimit::get();

        let name = vec![0u8; n as usize];
        let symbol = vec![0u8; s as usize];
        let (caller, _) = create_default_asset::<T, I>(true);
    }: _(SystemOrigin::Root, name.clone(), symbol.clone())
    verify {
        assert_last_event::<T, I>(Event::BlacklistSet(name.clone(), symbol.clone()).into());
    }

    verify_asset {
        let (caller, _) = create_default_minted_asset::<T, I>(true, 100u32.into());
    }: _(SystemOrigin::Root, Default::default())
    verify {
        assert_last_event::<T, I>(Event::AssetVerified(Default::default()).into());
    }

}
