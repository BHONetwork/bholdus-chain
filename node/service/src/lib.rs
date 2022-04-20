// Copyright 2019-2021 Bholdus Inc.
// This file is part of Bholdus.

// Bholdus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Bholdus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Bholdus.  If not, see <http://www.gnu.org/licenses/>.

//! This module assembles the Bholdus service components, executes them, and manages communication
//! between them. This is the backbone of the client-side node implementation.
//!
//! This module can assemble:
//! PartialComponents: For maintence tasks without a complete node (eg import/export blocks, purge)
//! Full Service: A complete parachain node including the pool, rpc, network

use beefy_gadget::notification::{
    BeefyBestBlockSender, BeefyBestBlockStream, BeefySignedCommitmentSender,
    BeefySignedCommitmentStream,
};
pub use common_primitives::{
    AccountId, Balance, BlockNumber, Hash, Header, Nonce, OpaqueBlock as Block,
};
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use futures::prelude::*;
use sc_client_api::{BlockBackend, ExecutorProvider};
use sc_consensus_aura::{self, ImportQueueParams, SlotProportion, StartAuraParams};
use sc_executor::{NativeElseWasmExecutor, NativeExecutionDispatch};
use sc_finality_grandpa::{self as grandpa};
use sc_keystore::LocalKeystore;
use sc_network::{Event, NetworkService};
use sc_service::{
    error::Error as ServiceError, BasePath, ChainSpec, Configuration, PartialComponents,
    TFullBackend, TFullClient, TaskManager,
};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sp_api::ConstructRuntimeApi;
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
use sp_trie::PrefixedMemoryDB;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub mod chain_spec;
pub mod client;
pub mod rpc;

pub use rpc::EthApiCmd;

pub use client::*;

#[cfg(feature = "with-ulas-runtime")]
pub use ulas_runtime;

#[cfg(feature = "with-phoenix-runtime")]
pub use phoenix_runtime;

type FullClient<RuntimeApi, Executor> =
    TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<Executor>>;
type FullBackend = TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

/// The transaction pool type defintion.
pub type TransactionPool<RuntimeApi, Executor> =
    sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>;

#[cfg(feature = "with-ulas-runtime")]
pub struct UlasExecutor;

#[cfg(feature = "with-ulas-runtime")]
impl sc_executor::NativeExecutionDispatch for UlasExecutor {
    /// Only enable the benchmarking host functions when we actually want to benchmark.
    #[cfg(feature = "runtime-benchmarks")]
    type ExtendHostFunctions = (
        frame_benchmarking::benchmarking::HostFunctions,
        bholdus_evm_primitives_ext::bholdus_ext::HostFunctions,
    );
    /// Otherwise we only use the default Substrate host functions.
    #[cfg(not(feature = "runtime-benchmarks"))]
    type ExtendHostFunctions = (bholdus_evm_primitives_ext::bholdus_ext::HostFunctions,);

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        ulas_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        ulas_runtime::native_version()
    }
}

#[cfg(feature = "with-phoenix-runtime")]
pub struct PhoenixExecutor;

#[cfg(feature = "with-phoenix-runtime")]
impl sc_executor::NativeExecutionDispatch for PhoenixExecutor {
    /// Only enable the benchmarking host functions when we actually want to benchmark.
    #[cfg(feature = "runtime-benchmarks")]
    type ExtendHostFunctions = (
        frame_benchmarking::benchmarking::HostFunctions,
        bholdus_evm_primitives_ext::bholdus_ext::HostFunctions,
    );
    /// Otherwise we only use the default Substrate host functions.
    #[cfg(not(feature = "runtime-benchmarks"))]
    type ExtendHostFunctions = (bholdus_evm_primitives_ext::bholdus_ext::HostFunctions,);

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        phoenix_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        phoenix_runtime::native_version()
    }
}

pub trait IdentifyVariant {
    fn is_ulas(&self) -> bool;
    fn is_phoenix(&self) -> bool;
}

impl IdentifyVariant for Box<dyn ChainSpec> {
    fn is_ulas(&self) -> bool {
        self.id().starts_with("ulas")
    }

    fn is_phoenix(&self) -> bool {
        self.id().starts_with("phoenix")
    }
}

pub const ULAS_RUNTIME_NOT_AVAILABLE: &str = "Ulas runtime is not available. Please compile the node with `--features with-ulas-runtime` to enable it.";
pub const PHOENIX_RUNTIME_NOT_AVAILABLE: &str = "Phoenix runtime is not available. Please compile the node with `--features with-phoenix-runtime` to enable it.";

pub fn frontier_database_dir(config: &Configuration) -> std::path::PathBuf {
    let config_dir = config
        .base_path
        .as_ref()
        .map(|base_path| base_path.config_dir(config.chain_spec.id()))
        .unwrap_or_else(|| {
            BasePath::from_project("", "", "phoenix").config_dir(config.chain_spec.id())
        });
    config_dir.join("frontier").join("db")
}

pub fn open_frontier_backend(config: &Configuration) -> Result<Arc<fc_db::Backend<Block>>, String> {
    Ok(Arc::new(fc_db::Backend::<Block>::new(
        &fc_db::DatabaseSettings {
            source: fc_db::DatabaseSettingsSrc::RocksDb {
                path: frontier_database_dir(&config),
                cache_size: 0,
            },
        },
    )?))
}

/// Builds a new object suitable for chain operations.
#[allow(clippy::type_complexity)]
pub fn new_chain_ops(
    config: &mut Configuration,
) -> Result<
    (
        Arc<Client>,
        Arc<FullBackend>,
        sc_consensus::BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
        TaskManager,
    ),
    ServiceError,
> {
    match &config.chain_spec {
        #[cfg(feature = "with-ulas-runtime")]
        spec if spec.is_ulas() => {
            new_chain_ops_inner::<ulas_runtime::RuntimeApi, UlasExecutor>(config)
        }
        #[cfg(feature = "with-phoenix-runtime")]
        _ => new_chain_ops_inner::<phoenix_runtime::RuntimeApi, PhoenixExecutor>(config),
        #[cfg(not(feature = "with-phoenix-runtime"))]
        _ => panic!("invalid chain spec"),
    }
}

#[allow(clippy::type_complexity)]
fn new_chain_ops_inner<RuntimeApi, Executor>(
    mut config: &mut Configuration,
) -> Result<
    (
        Arc<Client>,
        Arc<FullBackend>,
        sc_consensus::BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
        TaskManager,
    ),
    ServiceError,
>
where
    Client: From<Arc<crate::FullClient<RuntimeApi, Executor>>>,
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
    Executor: NativeExecutionDispatch + 'static,
{
    config.keystore = sc_service::config::KeystoreConfig::InMemory;
    let PartialComponents {
        client,
        backend,
        import_queue,
        task_manager,
        ..
    } = new_partial::<RuntimeApi, Executor>(config)?;
    Ok((
        Arc::new(Client::from(client)),
        backend,
        import_queue,
        task_manager,
    ))
}

/// `fp_rpc::ConvertTransaction` is implemented for an arbitrary struct that lives in each runtime.
/// It receives a ethereum::Transaction and returns a pallet-ethereum transact Call wrapped in an
/// UncheckedExtrinsic.
///
/// Although the implementation should be the same in each runtime, this might change at some point.
/// `TransactionConverters` is just a `fp_rpc::ConvertTransaction` implementor that proxies calls to
/// each runtime implementation.
pub enum TransactionConverters {
    #[cfg(feature = "with-ulas-runtime")]
    Ulas(ulas_runtime::TransactionConverter),
    #[cfg(feature = "with-phoenix-runtime")]
    Phoenix(phoenix_runtime::TransactionConverter),
}

impl TransactionConverters {
    #[cfg(feature = "with-ulas-runtime")]
    fn ulas() -> Self {
        TransactionConverters::Ulas(ulas_runtime::TransactionConverter)
    }
    #[cfg(not(feature = "with-ulas-runtime"))]
    fn ulas() -> Self {
        unimplemented!()
    }
    #[cfg(feature = "with-phoenix-runtime")]
    fn phoenix() -> Self {
        TransactionConverters::Phoenix(phoenix_runtime::TransactionConverter)
    }
    #[cfg(not(feature = "with-phoenix-runtime"))]
    fn phoenix() -> Self {
        unimplemented!()
    }
}

impl fp_rpc::ConvertTransaction<common_primitives::OpaqueExtrinsic> for TransactionConverters {
    fn convert_transaction(
        &self,
        transaction: pallet_ethereum::Transaction,
    ) -> common_primitives::OpaqueExtrinsic {
        match &self {
            #[cfg(feature = "with-ulas-runtime")]
            Self::Ulas(inner) => inner.convert_transaction(transaction),
            #[cfg(feature = "with-phoenix-runtime")]
            Self::Phoenix(inner) => inner.convert_transaction(transaction),
        }
    }
}

pub fn new_partial<RuntimeApi, Executor>(
    config: &Configuration,
) -> Result<
    sc_service::PartialComponents<
        FullClient<RuntimeApi, Executor>,
        FullBackend,
        FullSelectChain,
        sc_consensus::DefaultImportQueue<Block, FullClient<RuntimeApi, Executor>>,
        TransactionPool<RuntimeApi, Executor>,
        (
            (
                sc_finality_grandpa::GrandpaBlockImport<
                    FullBackend,
                    Block,
                    FullClient<RuntimeApi, Executor>,
                    FullSelectChain,
                >,
                sc_finality_grandpa::LinkHalf<
                    Block,
                    FullClient<RuntimeApi, Executor>,
                    FullSelectChain,
                >,
                BeefySignedCommitmentSender<Block>,
                BeefySignedCommitmentStream<Block>,
                BeefyBestBlockSender<Block>,
                BeefyBestBlockStream<Block>,
            ),
            Arc<fc_db::Backend<Block>>,
            Option<Telemetry>,
        ),
    >,
    ServiceError,
>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
    Executor: NativeExecutionDispatch + 'static,
{
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = NativeElseWasmExecutor::<Executor>::new(
        config.wasm_method,
        config.default_heap_pages,
        config.max_runtime_instances,
        config.runtime_cache_size,
    );

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            &config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );

    let frontier_backend = open_frontier_backend(config)?;

    let (grandpa_block_import, grandpa_link) = grandpa::block_import(
        client.clone(),
        &(client.clone() as Arc<_>),
        select_chain.clone(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;

    let slot_duration = sc_consensus_aura::slot_duration(&*client)?;

    let import_queue = sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _, _>(
        ImportQueueParams {
            block_import: grandpa_block_import.clone(),
            justification_import: Some(Box::new(grandpa_block_import.clone())),
            client: client.clone(),
            create_inherent_data_providers: move |_, ()| async move {
                let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                let slot =
                    sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                        *timestamp,
                        slot_duration,
                    );

                Ok((timestamp, slot))
            },
            spawner: &task_manager.spawn_essential_handle(),
            can_author_with: sp_consensus::CanAuthorWithNativeVersion::new(
                client.executor().clone(),
            ),
            registry: config.prometheus_registry(),
            check_for_equivocation: Default::default(),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
        },
    )?;

    let (beefy_link, beefy_commitment_stream) =
        beefy_gadget::notification::BeefySignedCommitmentStream::<Block>::channel();
    let (beefy_best_block_link, beefy_best_block_stream) =
        beefy_gadget::notification::BeefyBestBlockStream::<Block>::channel();

    let import_setup = (
        grandpa_block_import.clone(),
        grandpa_link,
        beefy_link,
        beefy_commitment_stream,
        beefy_best_block_link,
        beefy_best_block_stream,
    );

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (import_setup, frontier_backend, telemetry),
    })
}

pub struct NewFullBase<RuntimeApi, Executor>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
    Executor: NativeExecutionDispatch + 'static,
{
    pub task_manager: TaskManager,
    pub client: Arc<FullClient<RuntimeApi, Executor>>,
    pub network: Arc<NetworkService<Block, <Block as BlockT>::Hash>>,
    pub transaction_pool: Arc<TransactionPool<RuntimeApi, Executor>>,
}

fn remote_keystore(_url: &String) -> Result<Arc<LocalKeystore>, &'static str> {
    // FIXME: here would the concrete keystore be built,
    //        must return a concrete type (NOT `LocalKeystore`) that
    //        implements `CryptoStore` and `SyncCryptoStore`
    Err("Remote Keystore not supported.")
}

/// Creates a full service from the configuration.
pub fn new_full_base<RuntimeApi, Executor>(
    mut config: Configuration,
    rpc_config: rpc::RpcConfig,
) -> Result<NewFullBase<RuntimeApi, Executor>, ServiceError>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
    Executor: NativeExecutionDispatch + 'static,
{
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        mut keystore_container,
        select_chain,
        transaction_pool,
        other: (import_setup, frontier_backend, mut telemetry),
    } = new_partial(&config)?;

    let auth_disc_publish_non_global_ips = config.network.allow_non_globals_in_dht;
    if let Some(url) = &config.keystore_remote {
        match remote_keystore(url) {
            Ok(k) => keystore_container.set_remote_keystore(k),
            Err(e) => {
                return Err(ServiceError::Other(format!(
                    "Error hooking up remote keystore for {}: {}",
                    url, e
                )))
            }
        };
    }

    let grandpa_protocol_name = sc_finality_grandpa::protocol_standard_name(
        &client
            .block_hash(0)
            .ok()
            .flatten()
            .expect("Genesis block exists; qed"),
        &config.chain_spec,
    );
    config
        .network
        .extra_sets
        .push(grandpa::grandpa_peers_set_config(
            grandpa_protocol_name.clone(),
        ));

    let beefy_protocol_name = beefy_gadget::protocol_standard_name(
        &client
            .block_hash(0)
            .ok()
            .flatten()
            .expect("Genesis block exists; qed"),
        &config.chain_spec,
    );
    config
        .network
        .extra_sets
        .push(beefy_gadget::beefy_peers_set_config(
            beefy_protocol_name.clone(),
        ));

    let (
        grandpa_block_import,
        grandpa_link,
        beefy_signed_commitment_sender,
        beefy_commitment_stream,
        beefy_best_block_sender,
        beefy_best_block_stream,
    ) = import_setup;

    let warp_sync = Arc::new(grandpa::warp_proof::NetworkProvider::new(
        backend.clone(),
        grandpa_link.shared_authority_set().clone(),
        Vec::default(),
    ));

    let (network, system_rpc_tx, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync: Some(warp_sync),
        })?;

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &config,
            task_manager.spawn_handle(),
            client.clone(),
            network.clone(),
        );
    }

    let role = config.role.clone();
    let is_authority = role.is_authority();
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks: Option<()> = None;
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa;
    let prometheus_registry = config.prometheus_registry().cloned();

    let fee_history_cache: FeeHistoryCache = Arc::new(Mutex::new(BTreeMap::new()));
    let overrides = rpc::overrides_handle(client.clone());
    let filter_pool: Option<FilterPool> = Some(Arc::new(Mutex::new(BTreeMap::new())));

    // Spawning Ethereum tasks
    rpc::spawn_essential_tasks(rpc::SpawnTasksParams {
        task_manager: &task_manager,
        client: client.clone(),
        substrate_backend: backend.clone(),
        frontier_backend: frontier_backend.clone(),
        filter_pool: filter_pool.clone(),
        overrides: overrides.clone(),
        fee_history_limit: rpc_config.fee_history_limit,
        fee_history_cache: fee_history_cache.clone(),
    });

    let ethapi_cmd = rpc_config.ethapi.clone();
    let tracing_requesters =
        if ethapi_cmd.contains(&EthApiCmd::Debug) || ethapi_cmd.contains(&EthApiCmd::Trace) {
            rpc::tracing::spawn_tracing_tasks(
                &rpc_config,
                rpc::SpawnTasksParams {
                    task_manager: &task_manager,
                    client: client.clone(),
                    substrate_backend: backend.clone(),
                    frontier_backend: frontier_backend.clone(),
                    filter_pool: filter_pool.clone(),
                    overrides: overrides.clone(),
                    fee_history_limit: rpc_config.fee_history_limit,
                    fee_history_cache: fee_history_cache.clone(),
                },
            )
        } else {
            rpc::tracing::RpcRequesters {
                debug: None,
                trace: None,
            }
        };

    let (rpc_extensions_builder, rpc_setup) = {
        let justification_stream = grandpa_link.justification_stream();
        let shared_authority_set = grandpa_link.shared_authority_set().clone();
        let shared_voter_state = sc_finality_grandpa::SharedVoterState::empty();
        let finality_proof_provider = sc_finality_grandpa::FinalityProofProvider::new_for_service(
            backend.clone(),
            Some(shared_authority_set.clone()),
        );

        let rpc_setup = (shared_voter_state.clone(),);
        let client = client.clone();
        let pool = transaction_pool.clone();
        let select_chain = select_chain.clone();
        let _keystore = keystore_container.sync_keystore();
        let chain_spec = config.chain_spec.cloned_box();
        let frontier_backend = frontier_backend.clone();
        let network = network.clone();
        let fee_history_cache = fee_history_cache.clone();
        let overrides = overrides.clone();
        let rpc_config = rpc_config.clone();
        let filter_pool = filter_pool.clone();
        let block_data_cache = Arc::new(fc_rpc::EthBlockDataCache::new(
            task_manager.spawn_handle(),
            overrides.clone(),
            rpc_config.eth_log_block_cache,
            rpc_config.eth_statuses_cache,
        ));

        let is_ulas = config.chain_spec.is_ulas();

        let rpc_extensions_builder =
            move |deny_unsafe, subscription_executor: rpc::SubscriptionTaskExecutor| {
                let transaction_converter: TransactionConverters = if is_ulas {
                    TransactionConverters::ulas()
                } else {
                    TransactionConverters::phoenix()
                };

                let deps = rpc::FullDeps {
                    client: client.clone(),
                    pool: pool.clone(),
                    graph: pool.pool().clone(),
                    select_chain: select_chain.clone(),
                    deny_unsafe,
                    chain_spec: chain_spec.cloned_box(),
                    is_authority,
                    network: network.clone(),
                    transaction_converter,
                    // Grandpa
                    grandpa: rpc::GrandpaDeps {
                        shared_voter_state: shared_voter_state.clone(),
                        shared_authority_set: shared_authority_set.clone(),
                        justification_stream: justification_stream.clone(),
                        subscription_executor: subscription_executor.clone(),
                        finality_provider: finality_proof_provider.clone(),
                    },
                    beefy: rpc::BeefyDeps {
                        beefy_commitment_stream: beefy_commitment_stream.clone(),
                        beefy_best_block_stream: beefy_best_block_stream.clone(),
                        subscription_executor: subscription_executor.clone(),
                    },
                    frontier_backend: frontier_backend.clone(),
                    rpc_config: rpc_config.clone(),
                    fee_history_cache: fee_history_cache.clone(),
                    overrides: overrides.clone(),
                    filter_pool: filter_pool.clone(),
                    subscription_executor: subscription_executor.clone(),
                    block_data_cache: block_data_cache.clone(),
                };

                let mut io = rpc::create_full(deps)?;

                // Ethereum Tracing RPC
                if ethapi_cmd.contains(&EthApiCmd::Debug) || ethapi_cmd.contains(&EthApiCmd::Trace)
                {
                    rpc::tracing::extend_with_tracing(
                        client.clone(),
                        tracing_requesters.clone(),
                        rpc_config.ethapi_trace_max_count,
                        &mut io,
                    );
                }
                Ok(io)
            };

        (rpc_extensions_builder, rpc_setup)
    };

    let (shared_voter_state,) = rpc_setup;

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network: network.clone(),
        client: client.clone(),
        keystore: keystore_container.sync_keystore(),
        task_manager: &mut task_manager,
        transaction_pool: transaction_pool.clone(),
        rpc_extensions_builder: Box::new(rpc_extensions_builder),
        backend: backend.clone(),
        system_rpc_tx,
        config,
        telemetry: telemetry.as_mut(),
    })?;

    if let sc_service::config::Role::Authority { .. } = &role {
        let proposer_factory = sc_basic_authorship::ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool.clone(),
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );

        let can_author_with =
            sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());

        let slot_duration = sc_consensus_aura::slot_duration(&*client)?;

        let aura = sc_consensus_aura::start_aura::<AuraPair, _, _, _, _, _, _, _, _, _, _, _>(
            StartAuraParams {
                slot_duration,
                client: client.clone(),
                select_chain,
                block_import: grandpa_block_import.clone(),
                proposer_factory,
                create_inherent_data_providers: move |_, ()| async move {
                    let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                    let slot =
                    sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                        *timestamp,
                        slot_duration,
                    );

                    Ok((timestamp, slot))
                },
                force_authoring,
                backoff_authoring_blocks,
                keystore: keystore_container.sync_keystore(),
                can_author_with,
                sync_oracle: network.clone(),
                justification_sync_link: network.clone(),
                block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
                max_block_proposal_slot_portion: None,
                telemetry: telemetry.as_ref().map(|x| x.handle()),
            },
        )?;

        // the AURA authoring task is considered essential, i.e. if it
        // fails we take down the service with it.
        task_manager
            .spawn_essential_handle()
            .spawn_blocking("aura", Some("block-authoring"), aura);
    }

    // Spawn authority discovery module.
    if role.is_authority() {
        let authority_discovery_role =
            sc_authority_discovery::Role::PublishAndDiscover(keystore_container.keystore());
        let dht_event_stream =
            network
                .event_stream("authority-discovery")
                .filter_map(|e| async move {
                    match e {
                        Event::Dht(e) => Some(e),
                        _ => None,
                    }
                });
        let (authority_discovery_worker, _service) =
            sc_authority_discovery::new_worker_and_service_with_config(
                sc_authority_discovery::WorkerConfig {
                    publish_non_global_ips: auth_disc_publish_non_global_ips,
                    ..Default::default()
                },
                client.clone(),
                network.clone(),
                Box::pin(dht_event_stream),
                authority_discovery_role,
                prometheus_registry.clone(),
            );

        task_manager.spawn_handle().spawn(
            "authority-discovery-worker",
            Some("networking"),
            authority_discovery_worker.run(),
        );
    }

    // if the node isn't actively participating in consensus then it doesn't
    // need a keystore, regardless of which protocol we use below.
    let keystore = if role.is_authority() {
        Some(keystore_container.sync_keystore())
    } else {
        None
    };

    let beefy_params = beefy_gadget::BeefyParams {
        client: client.clone(),
        backend,
        key_store: keystore.clone(),
        network: network.clone(),
        signed_commitment_sender: beefy_signed_commitment_sender,
        beefy_best_block_sender: beefy_best_block_sender,
        min_block_delta: 4,
        prometheus_registry: prometheus_registry.clone(),
        protocol_name: beefy_protocol_name,
    };

    // Start BEEFY bridge gadget
    task_manager.spawn_essential_handle().spawn_blocking(
        "beefy-gadget",
        None,
        beefy_gadget::start_beefy_gadget::<_, _, _, _>(beefy_params),
    );

    let config = grandpa::Config {
        // FIXME #1578 make this available through chainspec
        gossip_duration: Duration::from_millis(333),
        justification_period: 512,
        name: Some(name),
        observer_enabled: false,
        keystore,
        local_role: role,
        telemetry: telemetry.as_ref().map(|x| x.handle()),
        protocol_name: grandpa_protocol_name,
    };

    if enable_grandpa {
        // start the full GRANDPA voter
        // NOTE: non-authorities could run the GRANDPA observer protocol, but at
        // this point the full voter should provide better guarantees of block
        // and vote data availability than the observer. The observer has not
        // been tested extensively yet and having most nodes in a network run it
        // could lead to finality stalls.
        let grandpa_config = grandpa::GrandpaParams {
            config,
            link: grandpa_link,
            network: network.clone(),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
            voting_rule: grandpa::VotingRulesBuilder::default().build(),
            prometheus_registry,
            shared_voter_state,
        };

        // the GRANDPA voter task is considered infallible, i.e.
        // if it fails we take down the service with it.
        task_manager.spawn_essential_handle().spawn_blocking(
            "grandpa-voter",
            None,
            grandpa::run_grandpa_voter(grandpa_config)?,
        );
    }

    network_starter.start_network();
    Ok(NewFullBase {
        task_manager,
        client: client.clone(),
        network,
        transaction_pool,
    })
}

/// Builds a new service for a full client.
pub fn new_full<RuntimeApi, Executor>(
    config: Configuration,
    rpc_config: rpc::RpcConfig,
) -> Result<TaskManager, ServiceError>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
    Executor: NativeExecutionDispatch + 'static,
{
    new_full_base::<RuntimeApi, Executor>(config, rpc_config)
        .map(|NewFullBase { task_manager, .. }| task_manager)
}
