#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Get;

use frame_support::{
    log::error,
    weights::{constants::RocksDbWeight, Weight},
};
use frame_system::RawOrigin;
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};

use sp_runtime::{traits::StaticLookup, DispatchError};

pub struct SampleExtensions;

impl<T> ChainExtension<T> for SampleExtensions
where
    T: SysConfig + pallet_contracts::Config + sample_extension::Config + pallet_balances::Config,
    <T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
{
    fn call<E: Ext>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
    where
        E: Ext<T = T>,
        <E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
    {
        let mut env = env.buf_in_buf_out();

        // Use the weight of `debug_message` as the baseline weight overhead for the chain extension
        // functions. `debug_message` is one reasonable choice as it immediately returns, which
        // represents the function of the chain extension well, as they don't do much beyond call an
        // already-weighted extrinsic.
        let extension_overhead = <T as pallet_contracts::Config>::Schedule::get()
            .host_fn_weights
            .debug_message;

        // Match on function id assigned in the contract
        match func_id {
            // ToDo: call transfer native
            1 => {
                // Retrieve arguments
                let base_weight = <T as pallet_contracts::Config>::Schedule::get()
                    .host_fn_weights
                    .call_transfer_surcharge;
                env.charge_weight(base_weight.saturating_add(extension_overhead))?;

                let (amount, target): (T::Balance, T::AccountId) = env.read_as()?;
                let caller = env.ext().caller().clone();
                let dest = T::Lookup::unlookup(target);

                pallet_balances::Pallet::<T>::transfer(
                    RawOrigin::Signed(caller).into(),
                    dest,
                    amount,
                )
                .map_err(|d| d.error)?;
            }
            _ => {
                error!("Called an unregistered `func_id`: {:}", func_id);
                return Err(DispatchError::Other("Unimplemented func_id"));
            }
        }
        // No error, return status code `0`, indicating `Ok(())`
        Ok(RetVal::Converging(0))
    }
}
