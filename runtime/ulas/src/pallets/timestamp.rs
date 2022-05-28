#![allow(unused_imports)]

use crate::*;

#[cfg(not(feature = "manual-seal"))]
parameter_types! {
	pub const MinimumPeriod: Moment = SLOT_DURATION / 2;
}

#[cfg(feature = "manual-seal")]
parameter_types! {
	pub const MinimumPeriod: Moment = 5;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = Moment;
	#[cfg(not(feature = "manual-seal"))]
	type OnTimestampSet = Aura;
	#[cfg(feature = "manual-seal")]
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
}
