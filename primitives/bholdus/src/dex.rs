#![allow(missing_docs)]

use crate::*;
use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{FixedU128, RuntimeDebug};
use sp_std::prelude::*;

#[derive(Encode, PartialEq, Eq, Clone, Copy, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct TradingPair(CurrencyId, CurrencyId);

impl TradingPair {
    pub fn from_currency_ids(currency_id_a: CurrencyId, currency_id_b: CurrencyId) -> Option<Self> {
        if currency_id_a.is_token_currency_id()
            && currency_id_b.is_token_currency_id()
            && currency_id_a != currency_id_b
        {
            if currency_id_a > currency_id_b {
                Some(TradingPair(currency_id_a, currency_id_b))
            } else {
                Some(TradingPair(currency_id_b, currency_id_a))
            }
        } else {
            None
        }
    }

    pub fn first(&self) -> CurrencyId {
        self.0.clone()
    }

    pub fn second(&self) -> CurrencyId {
        self.1.clone()
    }

    pub fn dex_share_currency_id(&self) -> CurrencyId {
        CurrencyId::join_dex_share_currency_id(self.first(), self.second())
            .expect("shouldn't be invalid! guaranteed by construction")
    }
}

impl Decode for TradingPair {
    fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
        let (first, second): (CurrencyId, CurrencyId) = Decode::decode(input)?;
        TradingPair::from_currency_ids(first, second)
            .ok_or_else(|| codec::Error::from("invalid currency"))
    }
}

pub type ExchangeRate = FixedU128;
