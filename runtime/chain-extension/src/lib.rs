#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Get;

mod chain_extension;
pub use chain_extension::*;
