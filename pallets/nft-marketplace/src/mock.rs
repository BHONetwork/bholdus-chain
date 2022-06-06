#![cfg(test)]

use super::*;
use crate as bholdus_nft_marketplace;
use codec::{Decode, Encode};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{Everything, Filter, InstanceFilter},
	PalletId, RuntimeDebug,
};
use frame_system::EnsureRoot;

use bholdus_nft::{ClassData, TokenData};
use bholdus_support::parameter_type_with_key;
use common_primitives::{Amount, Balance, BlockNumber, ReserveIdentifier};
use sp_core::{crypto::AccountId32, H256};
use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
};

use std::cell::RefCell;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

pub type AccountId = AccountId32;
// pub type CurrencyId = u64;

impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Call = Call;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Runtime>;
	type MaxLocks = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ReserveIdentifier;
	type WeightInfo = ();
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: u64| -> Balance {
		Default::default()
	};
}

parameter_types! {
	pub const BasicDeposit: u64 = 10;
	pub const FieldDeposit: u64 = 10;
	pub const SubAccountDeposit: u64 = 10;
	pub const MaxSubAccounts: u32 = 2;
	pub const MaxAdditionalFields: u32 = 2;
	pub const MaxRegistrars: u32 = 20;
	pub const MaxDecimals: u8 = 18;
}

parameter_types! {
	pub const TokenDeposit: u64 = 0;
	pub const ApprovalDeposit: u64 = 0;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: u64 = 1;
	pub const MetadataDepositPerByte: u64 = 1;
}

impl bholdus_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = i64;
	type AssetId = u64;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = TokenDeposit;
	type BasicDeposit = BasicDeposit;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type FieldDeposit = FieldDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type MaxDecimals = MaxDecimals;
}

impl bholdus_currencies::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type WeightInfo = ();
}

parameter_types! {
	pub const MaxClassMetadata: u32 = 1024;
	pub const MaxTokenMetadata: u32 = 1024;
}

parameter_types! {
	pub const NftPalletId: PalletId = PalletId(*b"bho/bNFT");
	pub MaxAttributesBytes: u32 = 10;
	pub MaxQuantity: u32 = 100;
}

impl bholdus_nft::Config for Runtime {
	type Event = Event;
	type PalletId = NftPalletId;
	type MaxAttributesBytes = MaxAttributesBytes;
	type MaxQuantity = MaxQuantity;
	type WeightInfo = ();
}

impl bholdus_support_nft::Config for Runtime {
	type ClassId = u32;
	type GroupId = u32;
	type TokenId = u64;
	type ClassData = ClassData;
	type TokenData = TokenData;
	type MaxClassMetadata = MaxClassMetadata;
	type MaxTokenMetadata = MaxTokenMetadata;
}

parameter_types! {
	pub const RoyaltyRate: (u32, u32) = (10000, 10000);
}

impl bholdus_support_nft_marketplace::Config for Runtime {
	type GetRoyaltyValue = RoyaltyRate;
	type Time = Timestamp;
	type Currency = Currencies;
}

impl Config for Runtime {
	type Event = Event;
}

use frame_system::Call as SystemCall;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
	Block = Block,
	NodeBlock = Block,
	UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Tokens: bholdus_tokens::{Pallet, Call, Storage, Config<T>, Event<T>},
	  Currencies: bholdus_currencies::{Pallet, Call, Event<T>},
	  NFT: bholdus_nft::{Pallet, Call, Event<T>},
		SupportNFT: bholdus_support_nft::{Pallet, Storage},
		SupportNFTMarketplace: bholdus_support_nft_marketplace::{Pallet, Storage},
		NFTMarketplace: bholdus_nft_marketplace::{Pallet, Call, Storage, Event<T>},
	}
);

pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const EVE: AccountId = AccountId::new([3u8; 32]);
pub const DAVE: AccountId = AccountId::new([4u8; 32]);
pub const CLASS_ID: <Runtime as bholdus_support_nft::Config>::ClassId = 0;
pub const TOKEN_ID: <Runtime as bholdus_support_nft::Config>::TokenId = 0;
pub const GROUP_ID: <Runtime as bholdus_support_nft::Config>::GroupId = 0;
pub const ASSET_ID: <Runtime as bholdus_tokens::Config>::AssetId = 0;

pub const ROYALTY_DENOMINATOR: u32 = 10_000u32;
pub const EXPIRED_TIME: MomentOf<Runtime> = 12345;
pub const PRICE: u128 = 10_000u128;
pub const ROYALTY_VALUE: (u32, u32) = (1000u32, 10_000u32);
pub const SERVICE_FEE: (u32, u32) = (1000u32, 10_000u32);

thread_local! {
	static TIME: RefCell<u32> = RefCell::new(0);
}

pub struct Timestamp;
impl Time for Timestamp {
	type Moment = u32;
	fn now() -> Self::Moment {
		TIME.with(|v| *v.borrow())
	}
}

impl Timestamp {
	pub fn set_timestamp(val: u32) {
		TIME.with(|v| *v.borrow_mut() = val);
	}
}

pub struct ExtBuilder;
impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
		pallet_balances::GenesisConfig::<Runtime> { balances: vec![(ALICE, 100_000)] }
			.assimilate_storage(&mut t)
			.unwrap();
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
