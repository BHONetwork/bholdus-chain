//! Mocks for the staking tokens

#![cfg(test)]
use crate::{self as bholdus_tokens_staking, weights, CurrencyId, PoolId};
use frame_support::{
    construct_runtime,
    dispatch::{DispatchError, DispatchResult},
    ord_parameter_types, parameter_types,
    traits::{Currency, Everything, Nothing, OnFinalize, OnInitialize, OnUnbalanced},
    weights::constants::RocksDbWeight,
    PalletId,
};

// use bholdus_primitives::{Amount, Balance, Rate};
use bholdus_primitives::{Balance, DexShare, EraIndex, TokenId, TokenInfo, TokenSymbol};
use bholdus_support::parameter_type_with_key;
use frame_system as system;
use frame_system::EnsureSignedBy;
use sp_core::{H160, H256};
use sp_runtime::{testing::Header, traits::IdentityLookup, AccountId32};
use sp_std::cell::RefCell;
use system::EnsureRoot;

use sp_io::TestExternalities;

pub type AccountId = AccountId32;
pub type BlockNumber = u64;

/*pub const BHO: TokenId = 0;
pub const BBNB: TokenId = 1;
pub const BUSD: TokenId = 2;
pub const LBHO: TokenId = 3;
pub const BBNB_BUSD_LP: TokenId = 4;
*/

pub const ACA: TokenId = 0;
pub const AUSD: TokenId = 1;
pub const LDOT: TokenId = 2;
pub const BTC: TokenId = 3;
pub const DOT: TokenId = 4;
pub const BTC_AUSD_LP: TokenId = 5;
pub const DOT_AUSD_LP: TokenId = 6;

pub(crate) const EXISTENTIAL_DEPOSIT: Balance = 2;
pub(crate) const BLOCKS_PER_ERA: BlockNumber = 3;

pub(crate) const BLOCKS_REWARD: Balance = 1000;
pub(crate) const HISTORY_DEPTH: u32 = 30;

// pub const BBNB_BUSD_LP: CurrencyId = CurrencyId::DexShare(
//     DexShare::Token(TokenSymbol::Token(TokenInfo { id: 111 })),
//     DexShare::Token(TokenSymbol::Token(TokenInfo { id: 1 })),
// );

mod staking_tokens {
    pub use super::super::*;
}

ord_parameter_types! {
    pub const ALICE: AccountId = AccountId::from([1u8; 32]);
    pub const BOB: AccountId = AccountId::from([2u8; 32]);
    pub const VAULT: AccountId = BStakingTokens::account_id();
    pub const RewardsSource: AccountId = AccountId::from([3u8; 32]);
    pub const ROOT: AccountId = AccountId32::new([255u8; 32]);
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Runtime {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = RocksDbWeight;
    type BaseCallFilter = Everything;
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        Default::default()
    };
}

thread_local! {
    static IS_SHUTDOWN: RefCell<bool> = RefCell::new(false);
}

pub fn mock_shutdown() {
    IS_SHUTDOWN.with(|v| *v.borrow_mut() = true)
}

/* pub struct MockEmergencyShutdown;
impl EmergencyShutdown for MockEmergencyShutdown {
    fn is_shutdown() -> bool {
        IS_SHUTDOWN.with(|v| *v.borrow_mut())
    }
}
*/

impl bholdus_support_rewards::Config for Runtime {
    type Share = Balance;
    type Balance = Balance;
    type PoolId = PoolId;
    type CurrencyId = TokenId;
    type Handler = BStakingTokens;
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
    pub const MaxDecimals: u32 = 18;
    pub const MetadataDepositBase: u64 = 1;
    pub const MetadataDepositPerByte: u64 = 1;
}

impl bholdus_tokens::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type Amount = i64;
    type AssetId = TokenId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type AssetDeposit = TokenDeposit;
    type BasicDeposit = BasicDeposit;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = StringLimit;
    type MaxDecimals = MaxDecimals;
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
    pub const MaxLocks: u32 = 4;
    pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

parameter_types! {
    pub const HistoryDepth: u32 = HISTORY_DEPTH;
    pub const AccumulatePeriod: BlockNumber = 10;
    pub const StableCurrencyId: TokenId = 0;
    pub const StakingTokensPalletId: PalletId = PalletId(*b"bho/stkt");
    pub const BlockPerEra: BlockNumber = BLOCKS_PER_ERA;
}

ord_parameter_types! {
    pub const Root: AccountId = ROOT::get();
}

impl bholdus_tokens_staking::Config for Runtime {
    type Event = Event;
    type BlockPerEra = BlockPerEra;
    type StableCurrencyId = StableCurrencyId;
    type UpdateOrigin = EnsureSignedBy<ROOT, AccountId>;
    type RewardsSource = RewardsSource;
    type AccumulatePeriod = AccumulatePeriod;
    type Currency = BTokens;
    // type EmergencyShutdown = MockEmergencyShutdown;
    type HistoryDepth = HistoryDepth;
    type PalletId = StakingTokensPalletId;
    type WeightInfo = ();
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime where
    Block = Block,
    NodeBlock = Block,
    UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        BStakingTokens: staking_tokens::{Pallet, Storage, Call, Event<T>},
        BTokens: bholdus_tokens::{Pallet, Call, Storage, Event<T>, Config<T>},
        SupportReward: bholdus_support_rewards::{Pallet, Storage, Call},
    }
);

pub struct ExtBuilder {
    balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self { balances: vec![] }
    }
}

pub fn deposit_for_account(account_id: AccountId, token_id: TokenId, amount: Balance) {
    BTokens::force_create(
        Origin::root(),
        token_id.clone(),
        account_id.clone(),
        true,
        1,
    );
    BTokens::mint(
        Origin::signed(account_id.clone()),
        token_id.clone(),
        account_id.clone(),
        amount,
    );
}

impl ExtBuilder {
    pub fn balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
        self.balances = balances;
        self
    }

    pub fn deposit_for_vault(self) -> Self {
        self.balances(vec![
            (VAULT::get(), 10000),
            (VAULT::get(), 10000),
            (VAULT::get(), 10000),
            (VAULT::get(), 10000),
            (VAULT::get(), 10000),
            (VAULT::get(), 10000),
        ])
    }

    pub fn deposit_vault(self) -> Self {
        self.balances(vec![
            (ALICE::get(), 10000),
            (VAULT::get(), 10000),
            (VAULT::get(), 10000),
            (VAULT::get(), 10000),
        ])
    }

    pub fn build(self) -> TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        pallet_balances::GenesisConfig::<Runtime> {
            balances: vec![(ALICE::get(), 9000)],
        }
        .assimilate_storage(&mut t)
        .ok();

        let mut ext = TestExternalities::from(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

/// Used to run to the specific block number
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        BStakingTokens::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        // This is performed outside, but we expect it before on_initialize
        // BStakingTokens::on_unbalanced(Balances::issue(BLOCKS_REWARD));
        BStakingTokens::on_initialize(System::block_number());
    }
}

/// Used to run the specific number of blocks
pub fn run_for_blocks(n: u64) {
    run_to_block(System::block_number() + n);
}

/// Advance blocks to the beginning of an era.
///
/// Function has no effect if era is already passed.

pub fn advance_to_era(n: EraIndex) {
    while BStakingTokens::current_era() < n {
        run_for_blocks(1);
    }
}

/// Initialize first block.
/// This method should only be called, otherwise the first block will get initialized multiple
/// times.

pub fn initialize_first_block() {
    // This assert prevents method misuse
    assert_eq!(System::block_number(), 1 as BlockNumber);

    BStakingTokens::on_initialize(System::block_number());
    run_to_block(2);
}

// Clears all events
pub fn clear_all_events() {
    System::reset_events();
}

pub fn bholdus_tokens_staking_events() -> Vec<crate::Event<Runtime>> {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let Event::BStakingTokens(inner) = e {
                Some(inner)
            } else {
                None
            }
        })
        .collect()
}
