use super::*;
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MemoInfo<AccountId> {
    pub(super) content: Vec<u8>,
    pub(super) sender: Vec<u8>,
    pub(super) receiver: Vec<u8>,
    pub(super) operator: AccountId,
    pub(super) time: u64,
}

pub type ChainId = u16;

pub type TxnHash = Vec<u8>;
