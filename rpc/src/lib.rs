//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

#[cfg(feature = "with-phoenix-runtime")]
pub mod phoenix;

#[cfg(feature = "with-cygnus-runtime")]
pub mod cygnus;

#[cfg(feature = "with-ulas-runtime")]
pub mod ulas;

#[cfg(feature = "with-phoenix-runtime")]
pub use phoenix_runtime;

use std::sync::Arc;

use bholdus_primitives::{Block, BlockNumber, Hash};
use sc_finality_grandpa::{
    FinalityProofProvider, GrandpaJustificationStream, SharedAuthoritySet, SharedVoterState,
};
pub use sc_rpc::SubscriptionTaskExecutor;
pub use sc_rpc_api::DenyUnsafe;

/// Extra dependencies for GRANDPA
pub struct GrandpaDeps<B> {
    /// Voting round info.
    pub shared_voter_state: SharedVoterState,
    /// Authority set info.
    pub shared_authority_set: SharedAuthoritySet<Hash, BlockNumber>,
    /// Receives notifications about justification events from Grandpa.
    pub justification_stream: GrandpaJustificationStream<Block>,
    /// Executor to drive the subscription manager in the Grandpa RPC handler.
    pub subscription_executor: SubscriptionTaskExecutor,
    /// Finality proof provider.
    pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}

/// Extra dependencies for BEEFY
pub struct BeefyDeps {
    /// Receives notifications about signed commitment events from BEEFY.
    pub beefy_commitment_stream: beefy_gadget::notification::BeefySignedCommitmentStream<Block>,
    /// Executor to drive the subscription manager in the BEEFY RPC handler.
    pub subscription_executor: sc_rpc::SubscriptionTaskExecutor,
}

/// Configurations of RPC
#[derive(Clone)]
pub struct RpcConfig {
    // pub ethapi: Vec<EthApiCmd>,
    // pub ethapi_max_permits: u32,
    // pub ethapi_trace_max_count: u32,
    // pub ethapi_trace_cache_duration: u64,
    /// Ethereum log block cache
    pub eth_log_block_cache: usize,
    /// Maximum number of logs in a query.
    pub max_past_logs: u32,
    /// Maximum fee history cache size.
    pub fee_history_limit: u64,
}

/// A IO handler that uses all Full RPC extensions.
pub type IoHandler = jsonrpc_core::IoHandler<sc_rpc::Metadata>;
