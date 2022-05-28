#![allow(unused_imports)]

use crate::*;

#[cfg(not(feature = "manual-seal"))]
parameter_types! {
	pub const UncleGenerations: BlockNumber = 5;
}

#[cfg(feature = "manual-seal")]
parameter_types! {
	pub const UncleGenerations: BlockNumber = 0;
}

#[cfg(not(feature = "manual-seal"))]
impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = (Staking, ImOnline);
}

#[cfg(feature = "manual-seal")]
impl pallet_authorship::Config for Runtime {
	type FindAuthor = ();
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = ();
}
