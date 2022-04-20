#![allow(unused_imports)]
use frame_support::parameter_types;

use crate::*;

impl bholdus_bridge_native_transfer::Config for Runtime {
    type Event = Event;
    type AdminOrigin = EnsureRoot<Self::AccountId>;
    type Currency = Balances;
    type MinimumDeposit = ExistentialDeposit;
    type WeightInfo = bholdus_bridge_native_transfer::weights::SubstrateWeight<Runtime>;
}
