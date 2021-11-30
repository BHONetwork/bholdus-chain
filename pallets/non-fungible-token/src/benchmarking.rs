//! Benchmarks for the nft module.

#![cfg(feature = "runtime-benchmarks")]

use sp_std::vec;

use frame_benchmarking::{account, benchmarks};
use frame_support::{dispatch::DispatchErrorWithPostInfo, traits::Get, weights::DispatchClass};
use frame_system::RawOrigin;
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};

pub use crate::*;
use bholdus_primitives::Balance;

pub struct Module<T: Config>(crate::Pallet<T>);

const SEED: u32 = 0;

fn dollar(d: u32) -> Balance {
    let d: Balance = d.into();
    d.saturating_mul(1_000_000_000_000_000_000)
}

fn test_attr() -> Attributes {
    let mut attr: Attributes = BTreeMap::new();
    for i in 0..30 {
        attr.insert(vec![i], vec![0; 64]);
    }
    attr
}

fn create_token_class<T: Config>(
    caller: T::AccountId,
) -> Result<T::AccountId, DispatchErrorWithPostInfo> {
    let module_account: T::AccountId =
        T::PalletId::get().into_sub_account(bholdus_support_nft::Pallet::<T>::next_class_id());

    crate::Pallet::<T>::create_class(RawOrigin::Signed(caller).into(), test_attr())?;
    Ok(module_account)
}

benchmarks! {
    // create NFT class
    create_class {
        let caller: T::AccountId = account("caller", 0, SEED);
    }: _(RawOrigin::Signed(caller), test_attr())

    // mint NFT token
    mint {
        let i in 1 .. 99;

        let caller: T::AccountId = account("caller", 0, SEED);
        let to: T::AccountId = account("to", 0, SEED);
        let to_lookup = T::Lookup::unlookup(to);
        let account = create_token_class::<T>(caller)?;

    }: _(RawOrigin::Signed(account), to_lookup, 0u32.into(), vec![1], test_attr(), i)

    // transfer NFT token to another account
    transfer {
        let caller: T::AccountId = account("caller", 0, SEED);
        let caller_lookup = T::Lookup::unlookup(caller.clone());
        let to: T::AccountId = account("to", 0, SEED);
        let to_lookup = T::Lookup::unlookup(to.clone());
        let account = create_token_class::<T>(caller)?;

        crate::Pallet::<T>::mint(
            RawOrigin::Signed(account).into(),
            to_lookup,
            0u32.into(),
            vec![1],
            test_attr(),
            1)?;
    }: _(RawOrigin::Signed(to), caller_lookup, (0u32.into(), 0u32.into()))

    // burn NFT token
    burn {
        let caller: T::AccountId = account("caller", 0, SEED);
        let to: T::AccountId = account("to", 0, SEED);
        let to_lookup = T::Lookup::unlookup(to.clone());
        let account = create_token_class::<T>(caller)?;
        crate::Pallet::<T>::mint(RawOrigin::Signed(account).into(), to_lookup, 0u32.into(), vec![1], test_attr(), 1)?;
    }: _(RawOrigin::Signed(to), (0u32.into(), 0u32.into()))

    // destroy NFT class
    destroy_class {
        let caller: T::AccountId = account("caller", 0, SEED);
        create_token_class::<T>(caller.clone())?;
    }: _(RawOrigin::Signed(caller.clone()), 0u32.into())
}

#[cfg(test)]
mod mock {
    use super::*;
    use crate as nft;

    use codec::{Decode, Encode};
    use frame_support::{
        parameter_types,
        traits::{Contains, InstanceFilter},
        weights::Weight,
        PalletId, RuntimeDebug,
    };
    use sp_core::{crypto::AccountId32, H256};
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        Perbill,
    };

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::one();
    }

    pub type AccountId = AccountId32;

    impl frame_system::Config for Runtime {
        type BaseCallFilter = BaseFilter;
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Call = Call;
        type Hashing = BlakeTwo256;
        type AccountId = AccountId;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type BlockWeights = ();
        type BlockLength = ();
        type DbWeight = ();
        type Version = ();
        type PalletInfo = PalletInfo;
        type AccountData = pallet_balances::AccountData<Balance>;
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
        type SS58Prefix = ();
        type OnSetCode = ();
    }
    parameter_types! {
        pub const ExistentialDeposit: u64 = 1;
        pub const MaxReserves: u32 = 50;
    }
    impl pallet_balances::Config for Runtime {
        type Balance = Balance;
        type Event = ();
        type DustRemoval = ();
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = frame_system::Pallet<Runtime>;
        type MaxLocks = ();
        type MaxReserves = MaxReserves;
        type ReserveIdentifier = ReserveIdentifier;
        type WeightInfo = ();
    }
    impl pallet_utility::Config for Runtime {
        type Event = ();
        type Call = Call;
        type WeightInfo = ();
    }
    parameter_types! {
        pub const ProxyDepositBase: u64 = 1;
        pub const ProxyDepositFactor: u64 = 1;
        pub const MaxProxies: u16 = 4;
        pub const MaxPending: u32 = 2;
        pub const AnnouncementDepositBase: u64 = 1;
        pub const AnnouncementDepositFactor: u64 = 1;
    }
    #[derive(
        Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug, MaxEncodedLen,
    )]
    pub enum ProxyType {
        Any,
        JustTransfer,
        JustUtility,
    }
    impl Default for ProxyType {
        fn default() -> Self {
            Self::Any
        }
    }
    impl InstanceFilter<Call> for ProxyType {
        fn filter(&self, c: &Call) -> bool {
            match self {
                ProxyType::Any => true,
                ProxyType::JustTransfer => {
                    matches!(c, Call::Balances(pallet_balances::Call::transfer(..)))
                }
                ProxyType::JustUtility => matches!(c, Call::Utility(..)),
            }
        }
        fn is_superset(&self, o: &Self) -> bool {
            self == &ProxyType::Any || self == o
        }
    }
    pub struct BaseFilter;
    impl Contains<Call> for BaseFilter {
        fn contains(c: &Call) -> bool {
            match *c {
                // Remark is used as a no-op call in the benchmarking
                Call::System(SystemCall::remark(_)) => true,
                Call::System(_) => false,
                _ => true,
            }
        }
    }

    parameter_types! {
        pub const CreateClassDeposit: Balance = 200;
        pub const CreateTokenDeposit: Balance = 100;
        pub const DataDepositPerByte: Balance = 10;
        pub const NftPalletId: PalletId = PalletId(*b"aca/aNFT");
        pub MaxAttributesBytes: u32 = 2048;
    }

    impl crate::Config for Runtime {
        type Event = ();
        type PalletId = NftPalletId;
        type MaxAttributesBytes = MaxAttributesBytes;
        type WeightInfo = ();
    }

    parameter_types! {
        pub const MaxClassMetadata: u32 = 1024;
        pub const MaxTokenMetadata: u32 = 1024;
    }

    impl bholdus_lib_nft::Config for Runtime {
        type ClassId = u32;
        type TokenId = u64;
        type ClassData = ClassData;
        type TokenData = TokenData;
        type MaxClassMetadata = MaxClassMetadata;
        type MaxTokenMetadata = MaxTokenMetadata;
    }

    type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
    type Block = frame_system::mocking::MockBlock<Runtime>;

    frame_support::construct_runtime!(
        pub enum Runtime where
            Block = Block,
            NodeBlock = Block,
            UncheckedExtrinsic = UncheckedExtrinsic,
        {
            System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
            Utility: pallet_utility::{Pallet, Call, Event},
            BholdusLibNFT: bholdus_lib_nft::{Pallet, Storage, Config<T>},
            NFT: nft::{Pallet, Call, Event<T>},
        }
    );

    use frame_system::Call as SystemCall;

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

#[cfg(test)]
mod tests {
    use super::mock::*;
    use super::*;
    use frame_benchmarking::impl_benchmark_test_suite;

    impl_benchmark_test_suite!(Pallet, super::new_test_ext(), super::Runtime,);
}
