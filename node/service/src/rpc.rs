//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use crate::client::RuntimeApiCollection;
use crate::{AccountId, Balance, Block, BlockNumber, Hash, Index};
use fc_mapping_sync::{MappingSyncWorker, SyncStrategy};
use fc_rpc::{
    EthBlockDataCache, EthTask, OverrideHandle, RuntimeApiStorageOverride, SchemaV1Override,
    SchemaV2Override, StorageOverride,
};
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use fp_rpc::EthereumRuntimeRPCApi;
use futures::prelude::*;
use jsonrpc_pubsub::manager::SubscriptionManager;
use pallet_ethereum::EthereumStorageSchema;
use sc_client_api::backend::{Backend, StateBackend, StorageProvider};
use sc_client_api::client::BlockchainEvents;
use sc_client_api::AuxStore;
use sc_client_api::BlockOf;
use sc_finality_grandpa::{
    FinalityProofProvider, GrandpaJustificationStream, SharedAuthoritySet, SharedVoterState,
};
use sc_finality_grandpa_rpc::GrandpaRpcHandler;
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
use sp_consensus::SelectChain;
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
use std::collections::BTreeMap;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

/// A IO handler that uses all Full RPC extensions.
pub type IoHandler = jsonrpc_core::IoHandler<sc_rpc::Metadata>;
/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpc_core::IoHandler<sc_rpc::Metadata>;
/// RPC result.
pub type RpcResult = Result<RpcExtension, Box<dyn Error + Send + Sync>>;

pub mod tracing;

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
    pub grandpa: GrandpaDeps<B>,
    /// BEEFY specific dependencies
    pub beefy: BeefyDeps,
    /// The Node authority flag
    pub is_authority: bool,
    /// Network service
    pub network: Arc<NetworkService<Block, Hash>>,
    /// Frontier Backend.
    pub frontier_backend: Arc<fc_db::Backend<Block>>,
    /// RPC Config
    pub rpc_config: RpcConfig,
    /// Fee History Cache
    pub fee_history_cache: FeeHistoryCache,
    /// Ethereum Schema Overrides
    pub overrides: Arc<OverrideHandle<Block>>,
    /// EthFilterApi pool.
    pub filter_pool: Option<FilterPool>,
    /// Subscription Task Executor
    pub subscription_executor: SubscriptionTaskExecutor,
}

/// Light client extra dependencies.
pub struct LightDeps<C, F, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Remote access to the blockchain (async).
    pub remote_blockchain: Arc<dyn sc_client_api::light::RemoteBlockchain<Block>>,
    /// Fetcher instance.
    pub fetcher: Arc<F>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum EthApiCmd {
    Debug,
    Trace,
}

impl FromStr for EthApiCmd {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "debug" => Self::Debug,
            "trace" => Self::Trace,
            _ => {
                return Err(format!(
                    "`{}` is not recognized as a supported Ethereum Api",
                    s
                ))
            }
        })
    }
}

/// Configurations of RPC
#[derive(Clone)]
pub struct RpcConfig {
    pub ethapi: Vec<EthApiCmd>,
    pub ethapi_max_permits: u32,
    pub ethapi_trace_max_count: u32,
    pub ethapi_trace_cache_duration: u64,
    /// Ethereum log block cache
    pub eth_log_block_cache: usize,
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

/// Spawn the tasks that are required to run Bholdus.
pub fn spawn_essential_tasks<B, C, BE>(params: SpawnTasksParams<B, C, BE>)
where
    C: ProvideRuntimeApi<B> + BlockOf,
    C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError> + 'static,
    C: BlockchainEvents<B> + StorageProvider<B, BE>,
    C: Send + Sync + 'static,
    C::Api: EthereumRuntimeRPCApi<B>,
    C::Api: BlockBuilder<B>,
    B: BlockT<Hash = H256> + Send + Sync + 'static,
    B::Header: HeaderT<Number = u32>,
    BE: Backend<B> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
{
    // Frontier offchain DB task. Essential.
    // Maps emulated ethereum data to substrate native data.
    params.task_manager.spawn_essential_handle().spawn(
        "frontier-mapping-sync-worker",
        MappingSyncWorker::new(
            params.client.import_notification_stream(),
            Duration::new(6, 0),
            params.client.clone(),
            params.substrate_backend.clone(),
            params.frontier_backend.clone(),
            SyncStrategy::Normal,
        )
        .for_each(|()| futures::future::ready(())),
    );

    // Frontier `EthFilterApi` maintenance.
    // Manages the pool of user-created Filters.
    if let Some(filter_pool) = params.filter_pool {
        // Each filter is allowed to stay in the pool for 100 blocks.
        const FILTER_RETAIN_THRESHOLD: u64 = 100;
        params.task_manager.spawn_essential_handle().spawn(
            "frontier-filter-pool",
            EthTask::filter_pool_task(
                Arc::clone(&params.client),
                filter_pool,
                FILTER_RETAIN_THRESHOLD,
            ),
        );
    }

    params.task_manager.spawn_essential_handle().spawn(
        "frontier-schema-cache-task",
        EthTask::ethereum_schema_cache_task(
            Arc::clone(&params.client),
            Arc::clone(&params.frontier_backend),
        ),
    );

    // Spawn Frontier FeeHistory cache maintenance task.
    params.task_manager.spawn_essential_handle().spawn(
        "frontier-fee-history",
        EthTask::fee_history_task(
            Arc::clone(&params.client),
            Arc::clone(&params.overrides),
            params.fee_history_cache,
            params.fee_history_limit,
        ),
    );
}

pub fn overrides_handle<C, BE>(client: Arc<C>) -> Arc<OverrideHandle<Block>>
where
    C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
    C: Send + Sync + 'static,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
    BE: Backend<Block> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
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

    Arc::new(OverrideHandle {
        schemas: overrides_map,
        fallback: Box::new(RuntimeApiStorageOverride::new(client.clone())),
    })
}

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, SC, B, A>(deps: FullDeps<C, P, SC, B, A>) -> IoHandler
where
    C: ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + AuxStore
        + HeaderMetadata<Block, Error = BlockChainError>
        + Sync
        + Send
        + 'static,
    C: StorageProvider<Block, B>,
    C: BlockchainEvents<Block>,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_contracts_rpc::ContractsRuntimeApi<Block, AccountId, Balance, BlockNumber, Hash>,
    C::Api: pallet_mmr_rpc::MmrRuntimeApi<Block, <Block as sp_runtime::traits::Block>::Hash>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: BlockBuilder<Block>,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
    P: TransactionPool<Block = Block> + 'static,
    SC: SelectChain<Block> + 'static,
    B: sc_client_api::Backend<Block> + Send + Sync + 'static,
    B::State: sc_client_api::backend::StateBackend<sp_runtime::traits::HashFor<Block>>,
    A: ChainApi<Block = Block> + 'static,
{
    use fc_rpc::{
        EthApi, EthApiServer, EthDevSigner, EthFilterApi, EthFilterApiServer, EthPubSubApi,
        EthPubSubApiServer, EthSigner, HexEncodedIdProvider, NetApi, NetApiServer, Web3Api,
        Web3ApiServer,
    };
    use pallet_contracts_rpc::{Contracts, ContractsApi};
    use pallet_mmr_rpc::{Mmr, MmrApi};
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use substrate_frame_rpc_system::{FullSystem, SystemApi};

    let mut io = jsonrpc_core::IoHandler::default();
    let FullDeps {
        client,
        pool,
        graph,
        select_chain,
        chain_spec,
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
        ..
    } = deps;

    let GrandpaDeps {
        shared_voter_state,
        shared_authority_set,
        justification_stream,
        subscription_executor,
        finality_provider,
    } = grandpa;

    io.extend_with(SystemApi::to_delegate(FullSystem::new(
        client.clone(),
        pool.clone(),
        deny_unsafe,
    )));

    // Making synchronous calls in light client freezes the browser currently,
    // more context: https://github.com/paritytech/substrate/pull/3480
    // These RPCs should use an asynchronous caller instead.
    io.extend_with(ContractsApi::to_delegate(Contracts::new(client.clone())));
    io.extend_with(MmrApi::to_delegate(Mmr::new(client.clone())));
    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        client.clone(),
    )));

    io.extend_with(sc_finality_grandpa_rpc::GrandpaApi::to_delegate(
        GrandpaRpcHandler::new(
            shared_authority_set.clone(),
            shared_voter_state,
            justification_stream,
            subscription_executor,
            finality_provider,
        ),
    ));

    io.extend_with(beefy_gadget_rpc::BeefyApi::to_delegate(
        beefy_gadget_rpc::BeefyRpcHandler::new(
            beefy.beefy_commitment_stream,
            beefy.subscription_executor,
        ),
    ));

    io.extend_with(NetApiServer::to_delegate(NetApi::new(
        client.clone(),
        network.clone(),
        // Whether to format the `peer_count` response as Hex (default) or not.
        true,
    )));

    // Nor any signers
    let signers = Vec::new();

    // Reasonable default caching inspired by the frontier template
    let block_data_cache = Arc::new(EthBlockDataCache::new(
        rpc_config.eth_log_block_cache,
        rpc_config.eth_log_block_cache,
    ));

    io.extend_with(EthApiServer::to_delegate(EthApi::new(
        client.clone(),
        pool.clone(),
        graph,
        phoenix_runtime::TransactionConverter,
        network.clone(),
        signers,
        overrides.clone(),
        frontier_backend.clone(),
        is_authority,
        rpc_config.max_past_logs,
        block_data_cache.clone(),
        rpc_config.fee_history_limit,
        fee_history_cache,
    )));

    if let Some(filter_pool) = filter_pool {
        io.extend_with(EthFilterApiServer::to_delegate(EthFilterApi::new(
            client.clone(),
            frontier_backend.clone(),
            filter_pool.clone(),
            500 as usize, // max stored filters
            overrides.clone(),
            rpc_config.max_past_logs,
            block_data_cache.clone(),
        )));
    }

    io.extend_with(Web3ApiServer::to_delegate(Web3Api::new(client.clone())));

    io.extend_with(EthPubSubApiServer::to_delegate(EthPubSubApi::new(
        pool.clone(),
        client.clone(),
        network.clone(),
        SubscriptionManager::<HexEncodedIdProvider>::with_id_provider(
            HexEncodedIdProvider::default(),
            Arc::new(deps.subscription_executor.clone()),
        ),
        overrides,
    )));

    io
}

/// Instantiate all Light RPC extensions.
pub fn create_light<C, P, M, F>(deps: LightDeps<C, F, P>) -> jsonrpc_core::IoHandler<M>
where
    C: sp_blockchain::HeaderBackend<Block>,
    C: Send + Sync + 'static,
    F: sc_client_api::light::Fetcher<Block> + 'static,
    P: TransactionPool + 'static,
    M: jsonrpc_core::Metadata + Default,
{
    use substrate_frame_rpc_system::{LightSystem, SystemApi};

    let LightDeps {
        client,
        pool,
        remote_blockchain,
        fetcher,
    } = deps;
    let mut io = jsonrpc_core::IoHandler::default();
    io.extend_with(SystemApi::<Hash, AccountId, Index>::to_delegate(
        LightSystem::new(client, remote_blockchain, fetcher, pool),
    ));

    io
}
