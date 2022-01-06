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

use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::OriginTrait;
use pallet_evm::{AddressMapping, ExitError, ExitSucceed, PrecompileSet};
use precompile_utils::{
    keccak256, EvmDataReader, EvmDataWriter, EvmResult, Gasometer, LogsBuilder, RuntimeHelper,
};

use fp_evm::{Context, PrecompileOutput};

use sp_core::{H160, U256};
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
    SetValue = "set_value(uint256)",
    GetValue = "get_value()",
}

pub struct PalletExamplePrecompileSet<Runtime, Instance: 'static = ()>(
    PhantomData<(Runtime, Instance)>,
);

impl<Runtime, Instance> PrecompileSet for PalletExamplePrecompileSet<Runtime, Instance>
where
    Instance: 'static,
    Runtime: pallet_example::Config + pallet_evm::Config + frame_system::Config,
    Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
    <Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
    Runtime::Call: From<pallet_example::Call<Runtime>>,
    <<Runtime as frame_system::Config>::Call as Dispatchable>::Origin: OriginTrait,
{
    fn execute(
        &self,
        address: H160,
        input: &[u8], //Reminder this is big-endian
        target_gas: Option<u64>,
        context: &Context,
        _is_static: bool,
    ) -> Option<EvmResult<PrecompileOutput>> {
        let result = {
            let mut gasometer = Gasometer::new(target_gas);
            let gasometer = &mut gasometer;

            let (mut input, selector) = match EvmDataReader::new_with_selector(gasometer, input) {
                Ok((input, selector)) => (input, selector),
                Err(e) => return Some(Err(e)),
            };
            let input = &mut input;

            match selector {
                // Check for accessor methods first. These return results immediately
                Action::SetValue => Self::set_value(input, gasometer, target_gas, context),
                Action::GetValue => Self::get_value(input, gasometer, target_gas, context),
            }
        };

        return Some(result);
    }

    fn is_precompile(&self, address: H160) -> bool {
        false
    }
}

impl<Runtime, Instance> PalletExamplePrecompileSet<Runtime, Instance> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<Runtime, Instance> PalletExamplePrecompileSet<Runtime, Instance>
where
    Instance: 'static,
    Runtime: pallet_example::Config + pallet_evm::Config + frame_system::Config,
    Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
    <Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
    Runtime::Call: From<pallet_example::Call<Runtime>>,
    <<Runtime as frame_system::Config>::Call as Dispatchable>::Origin: OriginTrait,
{
    fn set_value(
        input: &mut EvmDataReader,
        gasometer: &mut Gasometer,
        target_gas: Option<u64>,
        context: &Context,
    ) -> EvmResult<PrecompileOutput> {
        gasometer.record_log_costs_manual(1, 32)?;

        // Bound check. We expect a single argument passed in.
        input.expect_arguments(gasometer, 1)?;

        // Parse the u32 value that will be dispatched to the pallet.
        let value = input.read::<u32>(gasometer)?.into();

        // Use pallet-evm's account mapping to determine what AccountId to dispatch from.
        let origin = Runtime::AddressMapping::into_account_id(context.caller);
        let call = pallet_example::Call::<Runtime>::set_value { something: value };

        // Dispatch the call into the runtime.
        // The RuntimeHelper tells how much gas was actually used.
        RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Stopped,
            cost: gasometer.used_gas(),
            output: Default::default(),
            logs: LogsBuilder::new(context.address)
                .log1(
                    SELECTOR_LOG_SOMETHING,
                    EvmDataWriter::new().write(value).build(),
                )
                .build(),
        })
    }

    fn get_value(
        input: &mut EvmDataReader,
        gasometer: &mut Gasometer,
        target_gas: Option<u64>,
        _context: &Context,
    ) -> EvmResult<PrecompileOutput> {
        // Bound check
        input.expect_arguments(gasometer, 0)?;

        // fetch data from pallet
        let stored_value = pallet_example::Something::<Runtime>::get().unwrap_or_default();

        // Record one storage red worth of gas.
        // The utility internally uses pallet_evm's GasWeightMapping.
        gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

        // Construct to Solidity-formatted output data
        let output = EvmDataWriter::new().write(stored_value).build();

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost: gasometer.used_gas(),
            output,
            logs: Default::default(),
        })
    }
}
