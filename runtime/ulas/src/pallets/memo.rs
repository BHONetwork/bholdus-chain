#![allow(unused_imports)]
use frame_support::parameter_types;

use crate::*;

parameter_types! {
	pub const ContentLimit: u32 = 320;
}

impl bholdus_memo::Config for Runtime {
	type Event = Event;
	type UnixTime = Timestamp;
	type Currency = Balances;
	type WeightInfo = bholdus_memo::weights::SubstrateWeight<Runtime>;
	type ContentLimit = ContentLimit;
	type AdminOrigin = EnsureRoot<Self::AccountId>;
}
