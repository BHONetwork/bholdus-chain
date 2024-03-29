//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![allow(missing_docs)]

use crate::{client::RuntimeApiCollection, Block, BlockNumber, Hash, TransactionConverters};
use fc_rpc::{
	EthBlockDataCacheTask, OverrideHandle, RuntimeApiStorageOverride, SchemaV1Override,
	SchemaV2Override, SchemaV3Override, StorageOverride,
};
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use fp_rpc::EthereumRuntimeRPCApi;
use fp_storage::EthereumStorageSchema;
use futures::prelude::*;
use jsonrpsee::RpcModule;
use sc_client_api::{
	backend::{Backend, StateBackend, StorageProvider},
	client::BlockchainEvents,
	AuxStore, BlockOf,
};
#[cfg(feature = "with-hyper-runtime")]
use sc_consensus_manual_seal::rpc::{ManualSeal, ManualSealApiServer};
use sc_finality_grandpa::{
	FinalityProofProvider, GrandpaJustificationStream, SharedAuthoritySet, SharedVoterState,
};
use sc_network::NetworkService;
pub use sc_rpc::SubscriptionTaskExecutor;
pub use sc_rpc_api::DenyUnsafe;
use sc_service::TaskManager;
use sc_transaction_pool::{ChainApi, Pool};
use sc_transaction_pool_api::TransactionPool;
use sp_api::{HeaderT, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{
	Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
use std::{collections::BTreeMap, str::FromStr, sync::Arc, time::Duration};

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpsee::RpcModule<()>;
/// RPC result.
pub type RpcResult = Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>;

/// Ethereum Tracing
// pub mod tracing;

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

use beefy_gadget::notification::{BeefyBestBlockStream, BeefySignedCommitmentStream};
/// Extra dependencies for BEEFY
pub struct BeefyDeps {
	/// Receives notifications about signed commitment events from BEEFY.
	pub beefy_commitment_stream: BeefySignedCommitmentStream<Block>,
	/// Receives notifications about best block events from BEEFY.
	pub beefy_best_block_stream: BeefyBestBlockStream<Block>,
	/// Executor to drive the subscription manager in the BEEFY RPC handler.
	pub subscription_executor: sc_rpc::SubscriptionTaskExecutor,
}

/// Full client dependencies.
pub struct FullDeps<C, P, SC, B, A: ChainApi> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Graph pool instance.
	pub graph: Arc<Pool<A>>,
	/// The SelectChain Strategy
	pub select_chain: SC,
	/// A copy of the chain spec.
	pub chain_spec: Box<dyn sc_chain_spec::ChainSpec>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// GRANDPA specific dependencies.
	#[cfg(not(feature = "with-hyper-runtime"))]
	pub grandpa: GrandpaDeps<B>,
	/// BEEFY specific dependencies
	#[cfg(not(feature = "with-hyper-runtime"))]
	pub beefy: BeefyDeps,
	/// The Node authority flag
	pub is_authority: bool,
	/// Network service
	pub network: Arc<NetworkService<Block, Hash>>,
	/// Frontier Backend.
	#[cfg(not(feature = "with-hyper-runtime"))]
	pub frontier_backend: Arc<fc_db::Backend<Block>>,
	/// RPC Config
	pub rpc_config: RpcConfig,
	/// Fee History Cache
	#[cfg(not(feature = "with-hyper-runtime"))]
	pub fee_history_cache: FeeHistoryCache,
	/// Ethereum Schema Overrides
	#[cfg(not(feature = "with-hyper-runtime"))]
	pub overrides: Arc<OverrideHandle<Block>>,
	/// EthFilterApi pool.
	#[cfg(not(feature = "with-hyper-runtime"))]
	pub filter_pool: Option<FilterPool>,
	/// Subscription Task Executor
	pub subscription_executor: SubscriptionTaskExecutor,
	/// Cache for Ethereum block data.
	#[cfg(not(feature = "with-hyper-runtime"))]
	pub block_data_cache: Arc<EthBlockDataCacheTask<Block>>,
	/// Manual seal command sink
	#[cfg(feature = "with-hyper-runtime")]
	pub command_sink:
		Option<futures::channel::mpsc::Sender<sc_consensus_manual_seal::rpc::EngineCommand<Hash>>>,
	#[cfg(feature = "with-hyper-runtime")]
	pub sealing: Sealing,
	/// Used to bypass type parameter `B` of FullDeps when compiles with `manual-seal` feature.
	#[cfg(feature = "with-hyper-runtime")]
	pub _phantom: std::marker::PhantomData<B>,
}

/// Ethereum Api Command
#[derive(Debug, PartialEq, Clone)]
pub enum EthApiCmd {
	/// Enable Ethereum Debug module
	Debug,
	/// Enable Ethereum Tracing module
	Trace,
}

impl FromStr for EthApiCmd {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			| "debug" => Self::Debug,
			| "trace" => Self::Trace,
			| _ => return Err(format!("`{}` is not recognized as a supported Ethereum Api", s)),
		})
	}
}

/// Available Sealing methods.
#[cfg(feature = "with-hyper-runtime")]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Sealing {
	// Seal using rpc method.
	Manual,
	// Seal when transaction is executed.
	Instant,
}

#[cfg(feature = "with-hyper-runtime")]
impl FromStr for Sealing {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			| "manual" => Self::Manual,
			| "instant" => Self::Instant,
			| _ => return Err(format!("`{}` is not recognized as a supported sealing method", s)),
		})
	}
}

/// Configurations of RPC
#[derive(Clone)]
pub struct RpcConfig {
	#[cfg(feature = "with-hyper-runtime")]
	pub sealing: Sealing,
	pub ethapi: Vec<EthApiCmd>,
	pub ethapi_max_permits: u32,
	pub ethapi_trace_max_count: u32,
	pub ethapi_trace_cache_duration: u64,
	/// Ethereum log block cache
	pub eth_log_block_cache: usize,
	pub eth_statuses_cache: usize,
	/// Maximum number of logs in a query.
	pub max_past_logs: u32,
	/// Maximum fee history cache size.
	pub fee_history_limit: u64,
}

pub struct SpawnTasksParams<'a, B: BlockT, C, BE> {
	pub task_manager: &'a TaskManager,
	pub client: Arc<C>,
	pub substrate_backend: Arc<BE>,
	pub frontier_backend: Arc<fc_db::Backend<B>>,
	pub filter_pool: Option<FilterPool>,
	pub overrides: Arc<OverrideHandle<B>>,
	pub fee_history_limit: u64,
	pub fee_history_cache: FeeHistoryCache,
}

/// Ethereum Storage Schema overrides
#[cfg(not(feature = "with-hyper-runtime"))]
pub fn overrides_handle<C, BE>(client: Arc<C>) -> Arc<OverrideHandle<Block>>
where
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<sp_runtime::traits::HashFor<Block>>,
	BE::Blockchain: BlockchainBackend<Block>,
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
	C: BlockchainEvents<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
	C: Send + Sync + 'static,
	C::Api: RuntimeApiCollection<StateBackend = BE::State>,
{
	let mut overrides_map = BTreeMap::new();
	overrides_map.insert(
		EthereumStorageSchema::V1,
		Box::new(SchemaV1Override::new(client.clone()))
			as Box<dyn StorageOverride<_> + Send + Sync>,
	);
	overrides_map.insert(
		EthereumStorageSchema::V2,
		Box::new(SchemaV2Override::new(client.clone()))
			as Box<dyn StorageOverride<_> + Send + Sync>,
	);
	overrides_map.insert(
		EthereumStorageSchema::V3,
		Box::new(SchemaV3Override::new(client.clone()))
			as Box<dyn StorageOverride<_> + Send + Sync>,
	);

	Arc::new(OverrideHandle {
		schemas: overrides_map,
		fallback: Box::new(RuntimeApiStorageOverride::new(client.clone())),
	})
}

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, SC, BE, A>(deps: FullDeps<C, P, SC, BE, A>) -> RpcResult
where
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<sp_runtime::traits::HashFor<Block>>,
	BE::Blockchain: BlockchainBackend<Block>,
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
	C: BlockchainEvents<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
	C: Send + Sync + 'static,
	C::Api: RuntimeApiCollection<StateBackend = BE::State>,
	P: TransactionPool<Block = Block> + 'static,
	A: ChainApi<Block = Block> + 'static,
{
	use beefy_gadget_rpc::{Beefy, BeefyApiServer};
	use fc_rpc::{
		Eth, EthApiServer, EthDevSigner, EthFilter, EthFilterApiServer, EthPubSub,
		EthPubSubApiServer, EthSigner, Net, NetApiServer, Web3, Web3ApiServer,
	};
	use pallet_contracts_rpc::{Contracts, ContractsApiServer};
	use pallet_mmr_rpc::{Mmr, MmrApiServer};
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use sc_finality_grandpa_rpc::{Grandpa, GrandpaApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};

	let mut io = RpcModule::new(());

	#[cfg(not(feature = "with-hyper-runtime"))]
	let FullDeps {
		client,
		pool,
		graph,
		deny_unsafe,
		grandpa,
		beefy,
		is_authority,
		network,
		frontier_backend,
		fee_history_cache,
		rpc_config,
		overrides,
		filter_pool,
		block_data_cache,
		..
	} = deps;

	#[cfg(feature = "with-hyper-runtime")]
	let FullDeps {
		client,
		pool,
		graph,
		deny_unsafe,
		is_authority,
		network,
		rpc_config,
		command_sink,
		sealing,
		..
	} = deps;

	#[cfg(not(feature = "with-hyper-runtime"))]
	let GrandpaDeps {
		shared_voter_state,
		shared_authority_set,
		justification_stream,
		subscription_executor,
		finality_provider,
	} = grandpa;

	io.merge(System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
	io.merge(Contracts::new(client.clone()).into_rpc())?;
	#[cfg(not(feature = "with-hyper-runtime"))]
	io.merge(Mmr::new(client.clone()).into_rpc())?;
	io.merge(TransactionPayment::new(client.clone()).into_rpc())?;

	#[cfg(not(feature = "with-hyper-runtime"))]
	{
		io.merge(
			Grandpa::new(
				subscription_executor,
				shared_authority_set.clone(),
				shared_voter_state,
				justification_stream,
				finality_provider,
			)
			.into_rpc(),
		)?;

		io.merge(
			Beefy::<Block>::new(
				beefy.beefy_commitment_stream,
				beefy.beefy_best_block_stream,
				beefy.subscription_executor,
			)?
			.into_rpc(),
		)?;

		// Nor any signers
		let signers = Vec::new();

		let no_tx_converter: Option<fp_rpc::NoTransactionConverter> = None;

		io.merge(
			Eth::new(
				client.clone(),
				pool.clone(),
				graph,
				no_tx_converter,
				network.clone(),
				signers,
				overrides.clone(),
				frontier_backend.clone(),
				is_authority,
				block_data_cache.clone(),
				fee_history_cache,
				rpc_config.fee_history_limit,
			)
			.into_rpc(),
		)?;

		if let Some(filter_pool) = filter_pool {
			io.merge(
				EthFilter::new(
					client.clone(),
					frontier_backend.clone(),
					filter_pool.clone(),
					500_usize, // max stored filters
					rpc_config.max_past_logs,
					block_data_cache.clone(),
				)
				.into_rpc(),
			)?;
		}

		io.merge(Web3::new(client.clone()).into_rpc())?;

		io.merge(
			EthPubSub::new(
				pool.clone(),
				client.clone(),
				network.clone(),
				deps.subscription_executor.clone(),
				overrides,
			)
			.into_rpc(),
		)?;

		io.merge(
			Net::new(
				client.clone(),
				network.clone(),
				// Whether to format the `peer_count` response as Hex (default) or not.
				true,
			)
			.into_rpc(),
		)?;
	}

	#[cfg(feature = "with-hyper-runtime")]
	if let Some(command_sink) = command_sink {
		if sealing == Sealing::Manual {
			io.merge(
				// We provide the rpc handler with the sending end of the channel to allow the rpc
				// send EngineCommands to the background block authorship task.
				ManualSeal::new(command_sink).into_rpc(),
			);
		}
	}

	Ok(io)
}
