use frame_support::{
	log::error,
	weights::{constants::RocksDbWeight, Weight},
};
use frame_system::RawOrigin;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::{traits::StaticLookup, DispatchError};
use crate::{
	sp_api_hidden_includes_construct_runtime::hidden_include::traits::Get, Encode, TemplateModule,
};

pub struct ExampleExtension;

impl<T> ChainExtension<T> for ExampleExtension
where
	T: SysConfig + pallet_contracts::Config + pallet_template::Config + pallet_balances::Config,
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
		let extension_overhead =
			<T as pallet_contracts::Config>::Schedule::get().host_fn_weights.debug_message;

		// Match on function id assigned in the contract
		match func_id {
			// do_store_in_runtime
			1 => {
				use pallet_template::WeightInfo;
				// retrieve argument that was passed in smart contract invocation
				let value: u32 = env.read_as()?;
				// Capture weight for the main action being performed by the extrinsic
				let base_weight: Weight =
					<T as pallet_template::Config>::WeightInfo::insert_number(value);
				env.charge_weight(base_weight.saturating_add(extension_overhead))?;
				let caller = env.ext().caller().clone();

				crate::pallet_template::Pallet::<T>::insert_number(
					RawOrigin::Signed(caller).into(),
					value,
				)?;
			},
			// do_balance_transfer
			2 => {
				// Retrieve arguments
				let base_weight = <T as pallet_contracts::Config>::Schedule::get()
					.host_fn_weights
					.call_transfer_surcharge;
				env.charge_weight(base_weight.saturating_add(extension_overhead))?;

				let (transfer_amount, recipient_account): (u32, T::AccountId) = env.read_as()?;
				let recipient = T::Lookup::unlookup(recipient_account);
				let caller = env.ext().caller().clone();

				pallet_balances::Pallet::<T>::transfer(
					RawOrigin::Signed(caller).into(),
					recipient,
					transfer_amount.into(),
				)
				.map_err(|d| d.error)?;
			},
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
					},
					// do_get_from_runtime
					4 => {
						let result = TemplateModule::get_value().encode();
						env.write(&result, false, None).map_err(|_| {
							"Encountered an error when retrieving runtime storage value."
						})?;
					},
					_ => unreachable!(),
				}
			},
			_ => {
				error!("Called an unregistered `func_id`: {:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"))
			},
		}
		// No error, return status code `0`, indicating `Ok(())`
		Ok(RetVal::Converging(0))
	}
}
