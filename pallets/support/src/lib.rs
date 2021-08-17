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

mod arithmetic;
mod currency;

pub mod traits {
    use super::*;
    pub use currency::{
        BalanceStatus, BasicCurrency, BasicCurrencyExtended, BasicLockableCurrency,
        BasicReservableCurrency, LockIdentifier, MultiCurrency, MultiCurrencyExtended,
        MultiLockableCurrency, MultiReservableCurrency, OnDust,
    };
}
