#![cfg(test)]

use super::*;

use crate as nft;
use codec::{Decode, Encode};
use frame_support::{
    construct_runtime, parameter_types,
    traits::{Everything, Filter, InstanceFilter},
    RuntimeDebug,
};

use bholdus_primitives::{Amount, Balance, BlockNumber, CurrencyId, TokenSymbol};
use bholdus_support::parameter_type_with_key;
use sp_core::{crypto::AccountId32, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

pub type AccountId = AccountId32;

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
}

// parameter_types! {
//     pub const ExistentialDeposit: u64 = 1;
//     pub const MaxReserves: u32 = 50;
// }
// impl pallet_balances::Config for Runtime {
//     type Balance = Balance;
//     type Event = Event;
//     type DustRemoval = ();
//     type ExistentialDeposit = ExistentialDeposit;
//     type AccountStore = frame_system::Pallet<Runtime>;
//     type MaxLocks = ();
//     type MaxReserves = MaxReserves;
//     type ReserveIdentifier = ReserveIdentifier;
//     type WeightInfo = ();
// }

impl pallet_utility::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type WeightInfo = ();
}

parameter_types! {
    pub const NftPalletId: PalletId = PalletId(*b"bho/bNFT");
    pub MaxAttributesBytes: u32 = 10;
    pub MaxQuantity: u32 = 100;
}

impl Config for Runtime {
    type Event = Event;
    type PalletId = NftPalletId;
    type MaxAttributesBytes = MaxAttributesBytes;
    type MaxQuantity = MaxQuantity;
    type WeightInfo = ();
}

parameter_types! {
    pub const MaxClassMetadata: u32 = 1024;
    pub const MaxTokenMetadata: u32 = 1024;
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
        NFTModule: nft::{Pallet, Call, Event<T>},
        BNFT: bholdus_support_nft::{Pallet, Storage},
        // Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Utility: pallet_utility::{Pallet, Call, Event},
    }
);

pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const CLASS_ID: <Runtime as bholdus_support_nft::Config>::ClassId = 0;
pub const GROUP_ID: <Runtime as bholdus_support_nft::Config>::GroupId = 0;
pub const CLASS_ID_NOT_EXIST: <Runtime as bholdus_support_nft::Config>::ClassId = 1;
pub const TOKEN_ID: <Runtime as bholdus_support_nft::Config>::TokenId = 0;
pub const TOKEN_ID_NOT_EXIST: <Runtime as bholdus_support_nft::Config>::TokenId = 1;

pub struct ExtBuilder;
impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        // pallet_balances::GenesisConfig::<Runtime> {
        //     balances: vec![(ALICE, 100000)],
        // }
        // .assimilate_storage(&mut t)
        // .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}
