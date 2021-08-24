use crate as dex;
use bholdus_currencies::{BasicCurrencyAdapter, NativeCurrencyOf};
use bholdus_primitives::{CurrencyId, TokenInfo, TokenSymbol, TradingPair};
use bholdus_support::parameter_type_with_key;
use dex::*;
use frame_support::{parameter_types, PalletId};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
};
use system::EnsureRoot;

type AccountId = u32;
type Balance = u128;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u16 = 2207;
}
impl system::Config for Runtime {
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
    pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Config for Runtime {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Pallet<Runtime>;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        Default::default()
    };
}
parameter_types! {
    pub DustAccount: AccountId = PalletId(*b"orml/dst").into_account();
    pub MaxLocks: u32 = 100_000;
}
parameter_types! {
    pub const BasicDeposit: u64 = 10;
    pub const FieldDeposit: u64 = 10;
    pub const SubAccountDeposit: u64 = 10;
    pub const MaxSubAccounts: u32 = 2;
    pub const MaxAdditionalFields: u32 = 2;
    pub const MaxRegistrars: u32 = 20;
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
    type AssetId = CurrencyId;
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
}

parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = NATIVE_CURRENCY_ID;
}
impl bholdus_currencies::Config for Runtime {
    type Event = Event;
    type MultiCurrency = BholdusTokens;
    type NativeCurrency = AdaptedBasicCurrency;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}
pub type NativeCurrency = NativeCurrencyOf<Runtime>;
pub type AdaptedBasicCurrency = BasicCurrencyAdapter<Runtime, Balances, i64, u64>;

parameter_types! {
    pub const DexPalletId: PalletId = PalletId(*b"bhod/dex");
    pub const ExchangeFee: (u32, u32) = (10,100); // 10$ fee
    pub const TradingPathLimit: u32 = 3;

}
impl Config for Runtime {
    type Event = Event;
    type ListingOrigin = EnsureRoot<Self::AccountId>;
    type PalletId = DexPalletId;
    type ExchangeFee = ExchangeFee;
    type TradingPathLimit = TradingPathLimit;
    type Currency = Currencies;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        BholdusTokens: bholdus_tokens::{Pallet, Storage, Event<T>},
        Currencies: bholdus_currencies::{Pallet, Call, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Dex: dex::{Pallet, Call, Storage, Config<T>, Event<T>},
    }
);

// Mock Currencies
pub const BHO: CurrencyId = CurrencyId::Token(TokenSymbol::Native);
pub const BNB: CurrencyId = CurrencyId::Token(TokenSymbol::Token(TokenInfo { id: 1 }));
pub const DOT: CurrencyId = CurrencyId::Token(TokenSymbol::Token(TokenInfo { id: 2 }));
pub const BTC: CurrencyId = CurrencyId::Token(TokenSymbol::Token(TokenInfo { id: 3 }));
pub const NATIVE_CURRENCY_ID: CurrencyId = CurrencyId::Token(TokenSymbol::Native);

// Mock TradingPairs
parameter_types! {
    pub static BHOBNBPair: TradingPair = TradingPair::from_currency_ids(BHO, BNB).unwrap();
    pub static BNBDOTPair: TradingPair = TradingPair::from_currency_ids(BNB, DOT).unwrap();
    pub static DOTBTCPair: TradingPair = TradingPair::from_currency_ids(DOT,BTC).unwrap();
}

// Mock Accounts
pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const BNB_ADMIN: AccountId = 3;
pub const DOT_ADMIN: AccountId = 4;

pub struct ExtBuilder {
    pub balances: Vec<(CurrencyId, AccountId, Balance)>,
    pub initial_provisioning_trading_pairs:
        Vec<(TradingPair, (Balance, Balance), (Balance, Balance))>,
    pub initial_enabled_trading_pairs: Vec<TradingPair>,
    pub initial_added_liquidity_pools: Vec<(
        <Runtime as frame_system::Config>::AccountId,
        Vec<(TradingPair, (Balance, Balance))>,
    )>,
}

#[cfg(feature = "std")]
impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            balances: vec![
                (BHO, ALICE, 1_000_000_000u128),
                (BHO, BOB, 1_000_000_000u128),
                (BNB, ALICE, 1_000_000_000u128),
                (BNB, BOB, 1_000_000_000u128),
                (DOT, ALICE, 1_000_000_000u128),
                (DOT, BOB, 1_000_000_000u128),
                (BTC, ALICE, 1_000_000_000u128),
                (BTC, BOB, 1_000_000_000u128),
            ],
            initial_provisioning_trading_pairs: vec![],
            initial_added_liquidity_pools: vec![],
            initial_enabled_trading_pairs: vec![],
        }
    }
}

impl ExtBuilder {
    fn initialize_enabled_trading_pairs(mut self) -> Self {
        self.initial_enabled_trading_pairs = vec![BHOBNBPair::get()];
        self
    }

    fn initialize_added_liquidity(
        mut self,
        who: <Runtime as frame_system::Config>::AccountId,
    ) -> Self {
        self.initial_added_liquidity_pools = vec![(
            who,
            vec![
                (BHOBNBPair::get(), (1_000_000u128, 2_000_000u128)),
                (BNBDOTPair::get(), (1_000_000u128, 2_000_000u128)),
            ],
        )];
        self
    }

    pub fn build(&self) -> sp_io::TestExternalities {
        let mut t = system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap()
            .into();

        pallet_balances::GenesisConfig::<Runtime> {
            balances: self
                .balances
                .clone()
                .into_iter()
                .filter_map(|(currency_id, account_id, balance)| {
                    if currency_id == NATIVE_CURRENCY_ID {
                        Some((account_id.clone(), balance))
                    } else {
                        None
                    }
                })
                .collect(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        dex::GenesisConfig::<Runtime> {
            initial_added_liquidity_pools: self.initial_added_liquidity_pools.clone(),
            initial_enabled_trading_pairs: self.initial_enabled_trading_pairs.clone(),
            initial_provisioning_trading_pairs: self.initial_provisioning_trading_pairs.clone(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        sp_io::TestExternalities::from(t)
    }
}
