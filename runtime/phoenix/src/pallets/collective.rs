#![allow(unused_imports)]

use frame_support::{parameter_types, traits::EnsureOneOf};
use frame_system::EnsureRoot;
use pallet_collective::EnsureProportionAtLeast;
pub use pallet_collective::Instance1 as CouncilCollective;

use crate::*;

pub type EnsureRootOrHalfCouncil =
    EnsureOneOf<EnsureRoot<AccountId>, EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>>;

parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
}

impl pallet_collective::Config<CouncilCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = CouncilMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}
