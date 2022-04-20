#![allow(unused_imports)]

use crate::*;

pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = MaxAuthorities;
}
