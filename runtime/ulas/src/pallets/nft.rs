#![allow(unused_imports)]
use frame_support::parameter_types;

use crate::*;

parameter_types! {
	pub const MaxAttributesBytes: u32 = 2048;
	pub const MaxQuantity: u32 = 100;
	pub const NftPalletId: PalletId = PalletId(*b"bho/bNFT");
}

impl bholdus_nft::Config for Runtime {
	type Event = Event;
	type PalletId = NftPalletId;
	type MaxAttributesBytes = MaxAttributesBytes;
	type MaxQuantity = MaxQuantity;
	type WeightInfo = bholdus_nft::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const MaxClassMetadata: u32 = 1024;
	pub const MaxTokenMetadata: u32 = 1024;
}

impl bholdus_support_nft::Config for Runtime {
	type ClassId = u32;
	type GroupId = u32;
	type TokenId = u64;
	type ClassData = bholdus_nft::ClassData;
	type TokenData = bholdus_nft::TokenData;
	type MaxClassMetadata = MaxClassMetadata;
	type MaxTokenMetadata = MaxTokenMetadata;
}
