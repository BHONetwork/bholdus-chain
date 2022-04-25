//! Test environment for Tokens pallet.

use super::*;
use crate as bholdus_tokens;

use bholdus_support::parameter_type_with_key;
use frame_support::{construct_runtime, parameter_types};

use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
	AccountId32,
};

// pub type AccountId = AccountId32;
pub type AssetId = u32;
pub type Balance = u64;
pub type AccountId = u64;

pub const BUSD: AssetId = 1;
pub const ALICE: AccountId = 0;
pub const BOB: AccountId = 1;
pub const EVE: AccountId = 2;
pub const ASSET_ID: AssetId = 0;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
	Block = Block,
	NodeBlock = Block,
	UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		BholdusTokens: bholdus_tokens::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Runtime {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Runtime {
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const AssetDeposit: u64 = 1;
	pub const ApprovalDeposit: u64 = 1;
	pub const StringLimit: u32 = 50;
	pub const MaxDecimals: u8 = 18;
	pub const MetadataDepositBase: u64 = 1;
	pub const MetadataDepositPerByte: u64 = 1;
}

parameter_types! {
	pub const BasicDeposit: u64 = 10;
	pub const FieldDeposit: u64 = 10;
	pub const MaxAdditionalFields: u32 = 2;
	pub const MaxRegistrars: u32 = 20;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |currency_id: AssetId| -> Balance {
		0
	};
}

parameter_types! {
	pub DustAccount: AccountId = PalletId(*b"bhol/dst").into_account();
	pub MaxLocks: u32 = 2;
}

impl Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = i64;
	type AssetId = AssetId;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<u64>;
	type AssetDeposit = AssetDeposit;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type MaxDecimals = MaxDecimals;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Freezer = TestFreezer;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type Extra = ();
}

use std::{cell::RefCell, collections::HashMap};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum Hook {
	Died(u32, u64),
}

thread_local! {
	static FROZEN: RefCell<HashMap<(u32, u64), u64>> = RefCell::new(Default::default());
	static HOOKS: RefCell<Vec<Hook>> = RefCell::new(Default::default());
}

pub struct TestFreezer;
impl FrozenBalance<u32, u64, u64> for TestFreezer {
	fn frozen_balance(asset: u32, who: &u64) -> Option<u64> {
		FROZEN.with(|f| f.borrow().get(&(asset, who.clone())).cloned())
	}

	fn died(asset: u32, who: &u64) {
		HOOKS.with(|h| h.borrow_mut().push(Hook::Died(asset, who.clone())));
	}
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn ten() -> AssetIdentity {
	AssetIdentity {
		basic_information: BasicInformation {
			project_name: Data::Raw(b"Bholdus".to_vec()),
			..Default::default()
		},
		social_profiles: SocialProfile {
			github: Data::Raw(b"https://apps.bholdus.com".to_vec()),
			..Default::default()
		},
		//official_project_website: Data::Raw(b"https://apps.bholdus.com".to_vec()),
		..Default::default()
	}
}

pub struct ExtBuilder {
	balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self { balances: vec![(DustAccount::get(), ExistentialDeposits::get(&BUSD))] }
	}
}

impl ExtBuilder {
	pub fn balances(mut self, mut balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances.append(&mut balances);
		self
	}

	pub fn one_hundred_for_alice(self) -> Self {
		self.balances(vec![(ALICE, 100)])
	}

	pub fn one_hundred_for_alice_n_bob(self) -> Self {
		self.balances(vec![(ALICE, 100), (BOB, 100)])
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
		bholdus_tokens::GenesisConfig::<Runtime> { balances: self.balances }
			.assimilate_storage(&mut t)
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
