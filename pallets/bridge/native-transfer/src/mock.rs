use crate::{self as bholdus_bridge_native_transfer, pallet};
use frame_support::{parameter_types, traits::ExistenceRequirement};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use system::EnsureRoot;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;
pub type Balance = u128;
pub type AccountId = u32;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        BridgeNativeTransfer: bholdus_bridge_native_transfer::{Pallet, Call, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>, Config<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u16 = 2207;
}

impl system::Config for Runtime {
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
    type AccountId = AccountId;
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
    pub const ExistentialDeposit: u64 = 10;
}
impl pallet_balances::Config for Runtime {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl bholdus_bridge_native_transfer::Config for Runtime {
    type Event = Event;
    type AdminOrigin = EnsureRoot<Self::AccountId>;
    type Currency = Balances;
    type MinimumDeposit = ExistentialDeposit;
    type WeightInfo = ();
}

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;

pub struct ExtBuilder {
    balances: Vec<(AccountId, Balance)>,
}
impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            balances: vec![(ALICE, 100_000u128), (BOB, 100_000u128)],
        }
    }
}

impl ExtBuilder {
    pub fn with_balances(self, balances: Vec<(AccountId, Balance)>) -> Self {
        Self { balances }
    }

    pub fn build(&self) -> sp_io::TestExternalities {
        let mut t = system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap()
            .into();

        pallet_balances::GenesisConfig::<Runtime> {
            balances: self.balances.iter().cloned().collect(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        sp_io::TestExternalities::from(t)
    }
}
