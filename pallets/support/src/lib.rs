#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use impl_trait_for_tuples::impl_for_tuples;
use sp_runtime::{DispatchResult, RuntimeDebug};
use sp_std::{
    cmp::{Eq, PartialEq},
    prelude::Vec,
};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub use currency::{
    BalanceStatus, BasicCurrency, BasicCurrencyExtended, BasicLockableCurrency,
    BasicReservableCurrency, LockIdentifier, MultiCurrency, MultiCurrencyExtended,
    MultiLockableCurrency, MultiReservableCurrency, OnDust,
};

pub use get_by_key::GetByKey;

pub mod arithmetic;
pub mod currency;
pub mod get_by_key;
