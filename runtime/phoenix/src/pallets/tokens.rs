#![allow(unused_imports)]
use bholdus_support::parameter_type_with_key;
use frame_support::parameter_types;

use crate::*;

parameter_types! {
	pub const AssetDeposit: Balance = 0;
	pub const ApprovalDeposit: Balance = 0;
	pub const StringLimit: u32 = 50;
	pub const MaxDecimals: u8 = 18;
	pub const MetadataDepositBase: Balance = 10;
	pub const MetadataDepositPerByte: Balance = 1;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_asset_id: TokenId| -> Balance {
		Zero::zero()
	};
}

impl bholdus_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type AssetId = TokenId;

	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type MaxRegistrars = MaxRegistrars;
	type MaxAdditionalFields = MaxAdditionalFields;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type MaxDecimals = MaxDecimals;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = bholdus_tokens::weights::SubstrateWeight<Runtime>;
	type ExistentialDeposits = ExistentialDeposits;
}
