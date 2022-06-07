#![allow(unused_imports)]
pub use beefy_primitives::crypto::AuthorityId as BeefyId;
use beefy_primitives::mmr::{BeefyDataProvider, MmrLeafVersion};
use frame_support::parameter_types;

use crate::*;

impl pallet_beefy::Config for Runtime {
	type BeefyId = BeefyId;
}

impl pallet_mmr::Config for Runtime {
	const INDEXING_PREFIX: &'static [u8] = b"mmr";
	type Hashing = Keccak256;
	type Hash = <Keccak256 as traits::Hash>::Output;
	type OnNewRoot = pallet_beefy_mmr::DepositBeefyDigest<Runtime>;
	type LeafData = pallet_beefy_mmr::Pallet<Runtime>;
	type WeightInfo = ();
}

parameter_types! {
	/// Version of the produced MMR leaf.
	///
	/// The version consists of two parts;
	/// - `major` (3 bits)
	/// - `minor` (5 bits)
	///
	/// `major` should be updated only if decoding the previous MMR Leaf format from the payload
	/// is not possible (i.e. backward incompatible change).
	/// `minor` should be updated if fields are added to the previous MMR Leaf, which given SCALE
	/// encoding does not prevent old leafs from being decoded.
	///
	/// Hence we expect `major` to be changed really rarely (think never).
	/// See [`MmrLeafVersion`] type documentation for more details.
	pub LeafVersion: MmrLeafVersion = MmrLeafVersion::new(0, 0);
}

struct CustomBeefyDataProvider {}

impl BeefyDataProvider<()> for CustomBeefyDataProvider {
	fn extra_data() -> () {}
}

impl pallet_beefy_mmr::Config for Runtime {
	type LeafVersion = LeafVersion;
	type BeefyAuthorityToMerkleLeaf = pallet_beefy_mmr::BeefyEcdsaToEthereum;
	type LeafExtra = ();
	type BeefyDataProvider = CustomBeefyDataProvider;
}
