#![cfg_attr(not(feature = "std"), no_std)]
use common_primitives::{Balance, TokenId};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::Output;
use frame_support::traits::{Get, Randomness};
pub type CurrencyId = TokenId;

use frame_support::{
    log::{error, trace},
    weights::{constants::RocksDbWeight, Weight},
};
use frame_system::RawOrigin;
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::{traits::StaticLookup, DispatchError};

pub struct IntegrationExtensions;

impl<T> ChainExtension<T> for IntegrationExtensions
where
    T: SysConfig
        + pallet_contracts::Config
        + pallet_balances::Config
        + pallet_randomness_collective_flip::Config,
    <T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
{
    fn call<E: Ext>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
    where
        E: Ext<T = T>,
        <E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
    {
        let mut env = env.buf_in_buf_out();
        let extension_overhead = <T as pallet_contracts::Config>::Schedule::get()
            .host_fn_weights
            .debug_message;

        // Match on function id assigned in the contract
        match func_id {
            // do_balance_transfer
            1 => {
                // Retrieve arguments
                let base_weight = <T as pallet_contracts::Config>::Schedule::get()
                    .host_fn_weights
                    .call_transfer_surcharge;
                env.charge_weight(base_weight.saturating_add(extension_overhead))?;

                let (from, to, value): (T::AccountId, T::AccountId, T::Balance) = env.read_as()?;
                let recipient = T::Lookup::unlookup(to);
                let address = env.ext().address().clone();

                pallet_balances::Pallet::<T>::transfer(
                    RawOrigin::Signed(address).into(),
                    recipient,
                    value,
                )
                .map_err(|d| d.error)?;
            }

            2 => {
                let arg: [u8; 32] = env.read_as()?;
                let random_seed = <pallet_randomness_collective_flip::Pallet<T>>::random(&arg).0;
                let random_slice = random_seed.encode();
                trace!(
                    target: "runtime",
                    "[ChainExtension]|call|func_id:{:}",
                    func_id
                );

                env.write(&random_slice, false, None)
                    .map_err(|_| DispatchError::Other("ChainExtension failed to call random"))?
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
