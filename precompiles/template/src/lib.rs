// Copyright 2019-2021 Bholdus Inc.
// This file is part of Bholdus.

// Bholdus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Bholdus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Bholdus. If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(test, feature(assert_matches))]

use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::OriginTrait,
};
use pallet_evm::{AddressMapping, ExitSucceed, PrecompileSet};
use precompile_utils::{
	keccak256, revert, succeed, EvmDataReader, EvmDataWriter, EvmResult, FunctionModifier,
	LogsBuilder, PrecompileHandleExt, RuntimeHelper,
};

use fp_evm::{PrecompileHandle, PrecompileOutput};

use sp_core::H160;
use sp_std::{fmt::Debug, marker::PhantomData};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Solidity selector of the Something log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_SOMETHING: [u8; 32] = keccak256!("Something(uint256)");

/// Each variant represents a method that is exposed in the public Solidity interface
/// The function selectors will be automatically generated at compile-time by the macros
#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	DoSomething = "do_something(uint256)",
	GetValue = "get_value()",
}

pub struct PalletTemplatePrecompileSet<Runtime, Instance: 'static = ()>(
	PhantomData<(Runtime, Instance)>,
);

impl<Runtime, Instance> PrecompileSet for PalletTemplatePrecompileSet<Runtime, Instance>
where
	Instance: 'static,
	Runtime: pallet_template::Config + pallet_evm::Config + frame_system::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_template::Call<Runtime>>,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin: OriginTrait,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		let result = {
			let selector = match handle.read_selector() {
				Ok(selector) => selector,
				Err(e) => return Some(Err(e)),
			};

			if let Err(err) = handle.check_function_modifier(match selector {
				Action::DoSomething => FunctionModifier::NonPayable,
				_ => FunctionModifier::View,
			}) {
				return Some(Err(err));
			}

			match selector {
				// Check for accessor methods first. These return results immediately
				Action::DoSomething => Self::do_something(handle),
				Action::GetValue => Self::get_value(handle),
			}
		};

		return Some(result);
	}

	fn is_precompile(&self, _address: H160) -> bool {
		false
	}
}

impl<Runtime, Instance> PalletTemplatePrecompileSet<Runtime, Instance> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<Runtime, Instance> PalletTemplatePrecompileSet<Runtime, Instance>
where
	Instance: 'static,
	Runtime: pallet_template::Config + pallet_evm::Config + frame_system::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_template::Call<Runtime>>,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin: OriginTrait,
{
	fn do_something(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Read input.
		let mut input = handle.read_input()?;
		input.expect_arguments(1)?;

		// // Parse the u32 value that will be dispatched to the pallet.
		// let value = input.read::<u32>(gasometer)?.into();

		// // Use pallet-evm's account mapping to determine what AccountId to dispatch from.
		// let origin = Runtime::AddressMapping::into_account_id(context.caller);
		// let call = pallet_template::Call::<Runtime>::do_something { something: value };

		// // Dispatch the call into the runtime.
		// // The RuntimeHelper tells how much gas was actually used.
		// RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(succeed(EvmDataWriter::new().write(1_u32).build()))
	}

	fn get_value(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Read input.
		let mut input = handle.read_input()?;
		input.expect_arguments(0)?;

		// fetch data from pallet
		let stored_value = pallet_template::Something::<Runtime>::get().unwrap_or_default();

		// Record one storage red worth of gas.
		// The utility internally uses pallet_evm's GasWeightMapping.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		Ok(succeed(EvmDataWriter::new().write(stored_value).build()))
	}
}
