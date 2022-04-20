#![allow(unused_imports)]

pub use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;

use crate::*;

impl pallet_authority_discovery::Config for Runtime {
	type MaxAuthorities = MaxAuthorities;
}
