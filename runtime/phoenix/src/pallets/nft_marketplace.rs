#![allow(unused_imports)]

use crate::*;

parameter_types! {
	pub const RoyaltyValue: (u32, u32) = (10_000, 10_000);
}

impl bholdus_support_nft_marketplace::Config for Runtime {
	type GetRoyaltyValue = RoyaltyValue;
	type Time = Timestamp;
	type Currency = Currencies;
}

impl bholdus_nft_marketplace::Config for Runtime {
	type Event = Event;
}
