use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
pub use bholdus_primitives::{AccountId};

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PoolInfo<AccountId> {
    pub(super) rate: Rate,
    pub(super) account_pool: AccountId,
    pub(super) partner_admin: AccountId,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Rate {
    pub(super) numerator: u128,
    pub(super) denominator: u128,
}

