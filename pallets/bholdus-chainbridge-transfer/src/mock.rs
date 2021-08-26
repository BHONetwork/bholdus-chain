use crate as chainbridge_transfer;
use bholdus_chainbridge as chainbridge;
use frame_support::{parameter_types, traits::GenesisBuild};
use frame_system as system;
use hex::FromHex;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};
use system::EnsureRoot;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
        ChainBridge: chainbridge::{Pallet, Call, Storage, Event<T>},
        ChainBridgeTransfer: chainbridge_transfer::{Pallet, Call, Config, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u16 = 2207;
}

impl system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
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
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Config for Test {
    type Balance = u128;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Pallet<Test>;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

parameter_types! {
    pub const ProposalLifetime: u32 = 100;
    pub const ChainId_: chainbridge::ChainId = 0;
}

impl chainbridge::Config for Test {
    type Event = Event;
    type Proposal = Call;
    type AdminOrigin = EnsureRoot<Self::AccountId>;
    type ProposalLifetime = ProposalLifetime;
    type ChainId = ChainId_;
}

impl chainbridge_transfer::Config for Test {
    type Event = Event;
    type BridgeOrigin = chainbridge::EnsureBridge<Test>;
    type Currency = Balances;
    type AdminOrigin = EnsureRoot<Self::AccountId>;
}

pub const USER_ID: u64 = 1;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(USER_ID, 100)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    chainbridge_transfer::GenesisConfig {
        native_resource_id: <[u8; 32]>::from_hex(
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap(),
    }
    .assimilate_storage::<Test>(&mut t)
    .unwrap();

    sp_io::TestExternalities::new(t)
}
