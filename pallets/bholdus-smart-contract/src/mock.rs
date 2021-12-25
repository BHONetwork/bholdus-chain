use crate as bholdus_smart_contract;
use frame_support::{
	parameter_types,
	weights::{IdentityFee, Weight},
};
use frame_system;
use pallet_balances;
use pallet_contracts::Schedule;
use pallet_randomness_collective_flip;
use pallet_timestamp;
use pallet_transaction_payment::CurrencyAdapter;
use pallet_utility;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};
use std::cell::RefCell;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
pub struct ExtBuilder {
	existential_deposit: u64,
}
impl Default for ExtBuilder {
	fn default() -> Self {
		Self { existential_deposit: 1 }
	}
}
impl ExtBuilder {
	pub fn set_associated_consts(&self) {
		EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
	}
	pub fn build(self) -> sp_io::TestExternalities {
		self.set_associated_consts();
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		pallet_balances::GenesisConfig::<Test> { balances: vec![] }
			.assimilate_storage(&mut t)
			.unwrap();
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{

		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		TemplateModule: bholdus_smart_contract::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Randomness: pallet_randomness_collective_flip::{Pallet, Storage},
		Contracts: pallet_contracts::{Pallet, Call, Storage, Event<T>},
		Utility: pallet_utility::{Pallet, Call, Storage, Event},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

pub type Balance = u64;

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId32;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub static ExistentialDeposit: u64 = 0;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 2;
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

impl pallet_transaction_payment::Config for Test {
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
}

thread_local! {
	static CALL_FILTER: RefCell<fn(&Call) -> bool> = RefCell::new(|_| true);
}

parameter_types! {
	pub const ContractDeposit: u64 = 16;
	pub const DeletionQueueDepth: u32 = 1024;
	pub const DeletionWeightLimit: Weight = 500_000_000_000;
	pub MySchedule: Schedule<Test> = <Schedule<Test>>::default();
}

impl pallet_randomness_collective_flip::Config for Test {}

impl pallet_contracts::Config for Test {
	type Time = Timestamp;
	type Randomness = Randomness;
	type Currency = Balances;
	type Event = Event;
	type Call = Call;
	type CallFilter = frame_support::traits::Nothing;
	type ContractDeposit = ContractDeposit;
	type CallStack = [pallet_contracts::Frame<Self>; 31];
	type WeightPrice = pallet_transaction_payment::Pallet<Self>;
	type WeightInfo = ();
	type ChainExtension = ();
	type DeletionQueueDepth = DeletionQueueDepth;
	type DeletionWeightLimit = DeletionWeightLimit;
	type Schedule = MySchedule;
}

impl bholdus_smart_contract::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = 1;
}
impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}
impl pallet_utility::Config for Test {
	type Event = Event;
	type Call = Call;
	type WeightInfo = ();
}

parameter_types! {
	pub const MaxValueSize: u32 = 16_384;
	pub const MaxCodeSize: u32 = 2 * 1024;
	pub const TransactionByteFee: u64 = 0;
}
