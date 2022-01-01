#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, HasCompact};
use frame_support::traits::Currency;
use frame_system::{self as system};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

use bholdus_primitives::{EraIndex, TokenId};

pub mod pallet;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

pub use pallet::pallet::*;
pub use weights::WeightInfo;

pub type CurrencyId = TokenId;

/// Mode of era-forcing
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum Forcing {
    /// Not forcing anything - just let whatever happen.
    NotForcing,
    /// Force a new era, then reset to `NotForcing` as soon as it is done.
    ForceNew,
    /// Avoid a new era indefinitely.
    ForceNone,
    /// Force a new era at the end of all sessions indefinitely.
    ForceAlways,
}

impl Default for Forcing {
    fn default() -> Self {
        Forcing::NotForcing
    }
}

/// PoolId for various rewards pools
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum PoolId {
    Token(CurrencyId),

    /// Rewards and shares pool
    Dex(CurrencyId),
}
