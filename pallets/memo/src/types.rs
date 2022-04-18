use super::*;
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
pub struct MemoInfo<AccountId, BoundedString> {
    pub(super) content: BoundedString,
    pub(super) sender: Vec<u8>,
    pub(super) receiver: Vec<u8>,
    pub(super) operator: AccountId,
    pub(super) time: u64,
}

pub type ChainId = u16;

pub type TxnHash = Vec<u8>;
