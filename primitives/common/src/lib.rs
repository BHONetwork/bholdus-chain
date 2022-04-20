//! Low-level types used throughout the Substrate code.

#![cfg_attr(not(feature = "std"), no_std)]

pub use sp_runtime::OpaqueExtrinsic;
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	FixedU128, MultiSignature,
};

/// An index to a block.
/// 32-bits will allow for 136 years of blocks assuming 1 block per second.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Signed version of Balance
pub type Amount = i128;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// Hashing algorithm used by the chain.
pub type Hashing = BlakeTwo256;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Type used for expressing timestamp.
/// `u64` is enough to represent a duration of half a billion years, when the
/// time scale is milliseconds.
pub type Moment = u64;

/// Digest item type.
pub type DigestItem = generic::DigestItem;
/// Header type.
pub type Header = generic::Header<BlockNumber, Hashing>;
/// Block type.
pub type OpaqueBlock = generic::Block<Header, OpaqueExtrinsic>;
/// Block ID.
pub type BlockId = generic::BlockId<OpaqueBlock>;

// Digital Assets ID
pub type TokenId = u64;

pub type Ratio = FixedU128;
pub type Rate = FixedU128;

pub type NFTBalance = u128;
/// An insert or duration in time.
pub type EraIndex = u32;
