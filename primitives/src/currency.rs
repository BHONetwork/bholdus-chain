#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone, Default, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct DigitalTokenInfo {
    pub name: Vec<u8>,
    pub id: Vec<u8>,
    pub decimals: u8,
}

#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub enum TokenSymbol {
    Native,
    DigitalToken(DigitalTokenInfo),
}

#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub enum DexShare {
    Token(TokenSymbol),
}

#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub enum CurrencyId {
    Token(TokenSymbol),
    DexShare(DexShare, DexShare),
}

impl CurrencyId {
    pub fn is_token_currency_id(&self) -> bool {
        matches!(self, CurrencyId::Token(_))
    }

    pub fn is_dex_share_id(&self) -> bool {
        matches!(self, CurrencyId::DexShare(_, _))
    }

    pub fn split_dex_share_currency_id(&self) -> Option<(Self, Self)> {
        match self {
            CurrencyId::DexShare(dex_share_0, dex_share_1) => {
                let currency_id_0: CurrencyId = (*dex_share_0).into();
                let currency_id_1: CurrencyId = (*dex_share_1).into();
                Some((currency_id_0, currency_id_1))
            }
            _ => None,
        }
    }

    pub fn join_dex_share_currency_id(currency_id_0: Self, currency_id_1: Self) -> Option<Self> {
        let dex_share_0 = match currency_id_0 {
            CurrencyId::Token(symbol) => DexShare::Token(symbol),
            _ => return None,
        };

        let dex_share_1 = match currency_id_1 {
            CurrencyId::Token(symbol) => DexShare::Token(symbol),
            _ => return None,
        };

        Some(CurrencyId::DexShare(dex_share_0, dex_share_1))
    }
}

impl Into<CurrencyId> for DexShare {
    fn into(self) -> CurrencyId {
        match self {
            DexShare::Token(symbol) => CurrencyId::Token(symbol),
        }
    }
}
