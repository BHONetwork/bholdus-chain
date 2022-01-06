#![cfg_attr(not(feature = "std"), no_std)]
use bholdus_primitives::{Balance, TokenId};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Get;
pub type CurrencyId = TokenId;

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
    T: SysConfig
        + bholdus_currencies::Config
        + pallet_contracts::Config
        + sample_extension::Config
        + pallet_balances::Config,
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
            // do_store_in_runtime
            1 => {
                use sample_extension::WeightInfo;
                // retrieve argument that was passed in smart contract invocation
                let value: u32 = env.read_as()?;
                // Capture weight for the main action being performed by the extrinsic
                let base_weight: Weight =
                    <T as sample_extension::Config>::WeightInfo::insert_number(value);
                env.charge_weight(base_weight.saturating_add(extension_overhead))?;
                let caller: T::AccountId = env.ext().caller().clone();
                sample_extension::Pallet::<T>::insert_number(
                    RawOrigin::Signed(caller).into(),
                    value,
                )?;
            }
            // do_balance_transfer
            2 => {
                // Retrieve arguments
                let base_weight = <T as pallet_contracts::Config>::Schedule::get()
                    .host_fn_weights
                    .call_transfer_surcharge;
                env.charge_weight(base_weight.saturating_add(extension_overhead))?;

                let (transfer_amount, recipient_account, currency_id): (
                    Balance,
                    T::AccountId,
                    CurrencyId,
                ) = env.read_as()?;
                let recipient = T::Lookup::unlookup(recipient_account);
                let caller = env.ext().caller().clone();

                // pallet_balances::Pallet::<T>::transfer(
                //     RawOrigin::Signed(caller).into(),
                //     recipient,
                //     transfer_amount,
                // )
                // .map_err(|d| d.error)?;
                sample_extension::Pallet::<T>::transfer(
                    RawOrigin::Signed(caller).into(),
                    recipient,
                    currency_id,
                    transfer_amount,
                );
            }
            3 | 4 => {
                let base_weight = RocksDbWeight::get().reads(1);
                env.charge_weight(base_weight.saturating_add(extension_overhead))?;

                match func_id {
                    // do_get_balance
                    3 => {
                        let account: T::AccountId = env.read_as()?;
                        let result = pallet_balances::Pallet::<T>::free_balance(account).encode();

                        env.write(&result, false, None)
                            .map_err(|_| "Encountered an error when querying balance.")?;
                    }
                    // do_get_from_runtime
                    4 => {
                        let result = sample_extension::Pallet::<T>::get_value().encode();
                        env.write(&result, false, None).map_err(|_| {
                            "Encountered an error when retrieving runtime storage value."
                        })?;
                    }
                    _ => unreachable!(),
                }
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
