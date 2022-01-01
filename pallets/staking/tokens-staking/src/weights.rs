#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for module_incentives.
pub trait WeightInfo {
	fn on_initialize(c: u32) -> Weight;
	fn deposit_dex_share() -> Weight;
	// fn withdraw_dex_share() -> Weight;
	// fn claim_rewards() -> Weight;
}


// For backwards compatibility and tests
impl WeightInfo for () {
	fn on_initialize(c: u32) -> Weight {
		(33_360_000 as Weight)
			.saturating_add((23_139_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
	}
	fn deposit_dex_share() -> Weight {
		(84_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(9 as Weight))
			.saturating_add(RocksDbWeight::get().writes(9 as Weight))
	}
}
