//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

#[cfg(feature = "with-bholdus-runtime")]
pub use bholdus_runtime;
#[cfg(feature = "with-bholdus-runtime")]
use bholdus_runtime::RuntimeApi;

use bholdus_primitives::Block;
use futures::prelude::*;
use sc_client_api::{ExecutorProvider, RemoteBackend};
use sc_consensus_aura::{self, ImportQueueParams, SlotProportion, StartAuraParams};
use sc_executor::native_executor_instance;
use sc_finality_grandpa::{self as grandpa};
use sc_keystore::LocalKeystore;
use sc_network::{Event, NetworkService};
use sc_service::{config::Configuration, error::Error as ServiceError, RpcHandlers, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sp_consensus::SlotData;
use sp_consensus_aura::ed25519::AuthorityPair as AuraPair;
use sp_inherents::InherentDataProvider;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

use bholdus_rpc;

pub mod chain_spec;
mod client;

// Our native executor instance.
#[cfg(feature = "with-bholdus-runtime")]
native_executor_instance!(
    pub Executor,
    bholdus_runtime::api::dispatch,
    bholdus_runtime::native_version,
    frame_benchmarking::benchmarking::HostFunctions,
);

type FullClient = sc_service::TFullClient<Block, RuntimeApi, Executor>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
type FullGrandpaBlockImport =
    sc_finality_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>;
type LightClient = sc_service::TLightClient<Block, RuntimeApi, Executor>;

pub fn new_partial(
    config: &Configuration,
) -> Result<
    sc_service::PartialComponents<
        FullClient,
        FullBackend,
        FullSelectChain,
        sp_consensus::DefaultImportQueue<Block, FullClient>,
        sc_transaction_pool::FullPool<Block, FullClient>,
        (
            sc_finality_grandpa::GrandpaBlockImport<
                FullBackend,
                Block,
                FullClient,
                FullSelectChain,
            >,
            sc_finality_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
            Option<Telemetry>,
        ),
    >,
    ServiceError,
> {
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

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, Executor>(
            &config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager.spawn_handle().spawn("telemetry", worker.run());
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

    let (grandpa_block_import, grandpa_link) = grandpa::block_import(
        client.clone(),
        &(client.clone() as Arc<_>),
        select_chain.clone(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;

    let slot_duration = sc_consensus_aura::slot_duration(&*client)?.slot_duration();

    let import_queue =
        sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _, _>(ImportQueueParams {
            block_import: grandpa_block_import.clone(),
            justification_import: Some(Box::new(grandpa_block_import.clone())),
            client: client.clone(),
            create_inherent_data_providers: move |_, ()| async move {
                let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                let slot =
                    sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
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
        })?;

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (grandpa_block_import, grandpa_link, telemetry),
    })
}

pub struct NewFullBase {
    pub task_manager: TaskManager,
    pub client: Arc<FullClient>,
    pub network: Arc<NetworkService<Block, <Block as BlockT>::Hash>>,
    pub transaction_pool: Arc<sc_transaction_pool::FullPool<Block, FullClient>>,
}

fn remote_keystore(_url: &String) -> Result<Arc<LocalKeystore>, &'static str> {
    // FIXME: here would the concrete keystore be built,
    //        must return a concrete type (NOT `LocalKeystore`) that
    //        implements `CryptoStore` and `SyncCryptoStore`
    Err("Remote Keystore not supported.")
}

/// Creates a full service from the configuration.
pub fn new_full_base(mut config: Configuration) -> Result<NewFullBase, ServiceError> {
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        mut keystore_container,
        select_chain,
        transaction_pool,
        other: (block_import, grandpa_link, mut telemetry),
    } = new_partial(&config)?;

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

    config
        .network
        .extra_sets
        .push(grandpa::grandpa_peers_set_config());

    #[cfg(feature = "cli")]
    config.network.request_response_protocols.push(
        sc_finality_grandpa_warp_sync::request_response_config_for_chain(
            &config,
            task_manager.spawn_handle(),
            backend.clone(),
            import_setup.1.shared_authority_set().clone(),
        ),
    );

    let (network, system_rpc_tx, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            on_demand: None,
            block_announce_validator_builder: None,
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
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks: Option<()> = None;
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa;
    let prometheus_registry = config.prometheus_registry().cloned();
    let chain_spec = config.chain_spec.cloned_box();

    let (rpc_extensions_builder, rpc_setup) = {
        let justification_stream = grandpa_link.justification_stream();
        let shared_authority_set = grandpa_link.shared_authority_set().clone();
        let shared_voter_state = sc_finality_grandpa::SharedVoterState::empty();
        let finality_proof_provider = sc_finality_grandpa::FinalityProofProvider::new_for_service(
            backend.clone(),
            Some(shared_authority_set.clone()),
        );

        let rpc_setup = (shared_voter_state.clone(), finality_proof_provider.clone());
        let client = client.clone();
        let pool = transaction_pool.clone();
        let select_chain = select_chain.clone();
        let network = network.clone();
        let is_authority = config.role.clone().is_authority();
        let _keystore = keystore_container.sync_keystore();
        let subscription_executor =
            sc_rpc::SubscriptionTaskExecutor::new(task_manager.spawn_handle());

        let rpc_extensions_builder = move |deny_unsafe, _| {
            let deps = bholdus_rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                select_chain: select_chain.clone(),
                deny_unsafe,
                chain_spec: chain_spec.cloned_box(),
                // Grandpa
                grandpa: bholdus_rpc::GrandpaDeps {
                    shared_voter_state: shared_voter_state.clone(),
                    shared_authority_set: shared_authority_set.clone(),
                    justification_stream: justification_stream.clone(),
                    subscription_executor: subscription_executor.clone(),
                    finality_provider: finality_proof_provider.clone(),
                },
            };

            bholdus_rpc::create_full(deps)
        };

        (rpc_extensions_builder, rpc_setup)
    };

    let (shared_voter_state, _finality_proof_provider) = rpc_setup;

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        config,
        backend: backend.clone(),
        client: client.clone(),
        keystore: keystore_container.sync_keystore(),
        network: network.clone(),
        rpc_extensions_builder: Box::new(rpc_extensions_builder),
        transaction_pool: transaction_pool.clone(),
        task_manager: &mut task_manager,
        on_demand: None,
        remote_blockchain: None,
        system_rpc_tx,
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
        let raw_duration = slot_duration.slot_duration();

        let aura = sc_consensus_aura::start_aura::<
            sp_consensus_aura::ed25519::AuthorityPair,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
        >(StartAuraParams {
            slot_duration,
            client: client.clone(),
            select_chain,
            block_import,
            proposer_factory,
            create_inherent_data_providers: move |_, ()| async move {
                let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                let slot =
                    sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
                        *timestamp,
                        raw_duration,
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
        })?;

        task_manager
            .spawn_essential_handle()
            .spawn_blocking("aura-proposer", aura);
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
        let (authority_discovery_worker, _service) = sc_authority_discovery::new_worker_and_service(
            client.clone(),
            network.clone(),
            Box::pin(dht_event_stream),
            authority_discovery_role,
            prometheus_registry.clone(),
        );

        task_manager.spawn_handle().spawn(
            "authority-discovery-worker",
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

    let config = grandpa::Config {
        // FIXME #1578 make this available through chainspec
        gossip_duration: std::time::Duration::from_millis(333),
        justification_period: 512,
        name: Some(name),
        observer_enabled: false,
        keystore,
        local_role: role,
        telemetry: telemetry.as_ref().map(|x| x.handle()),
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
        task_manager
            .spawn_essential_handle()
            .spawn_blocking("grandpa-voter", grandpa::run_grandpa_voter(grandpa_config)?);
    }

    network_starter.start_network();
    Ok(NewFullBase {
        task_manager,
        client,
        network,
        transaction_pool,
    })
}

/// Builds a new service for a full client.
pub fn new_full(config: Configuration) -> Result<TaskManager, ServiceError> {
    new_full_base(config).map(|NewFullBase { task_manager, .. }| task_manager)
}

pub fn new_light_base(
    mut config: Configuration,
) -> Result<
    (
        TaskManager,
        RpcHandlers,
        Arc<LightClient>,
        Arc<NetworkService<Block, <Block as BlockT>::Hash>>,
        Arc<
            sc_transaction_pool::LightPool<Block, LightClient, sc_network::config::OnDemand<Block>>,
        >,
    ),
    ServiceError,
> {
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            #[cfg(feature = "browser")]
            let transport = Some(sc_telemetry::ExtTransport::new(
                libp2p_wasm_ext::ffi::websocket_transport(),
            ));
            #[cfg(not(feature = "browser"))]
            let transport = None;

            let worker = TelemetryWorker::with_transport(16, transport)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let (client, backend, keystore_container, mut task_manager, on_demand) =
        sc_service::new_light_parts::<Block, RuntimeApi, Executor>(
            &config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
        )?;

    let mut telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager.spawn_handle().spawn("telemetry", worker.run());
        telemetry
    });

    config
        .network
        .extra_sets
        .push(grandpa::grandpa_peers_set_config());

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = Arc::new(sc_transaction_pool::BasicPool::new_light(
        config.transaction_pool.clone(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
        on_demand.clone(),
    ));

    let (grandpa_block_import, _) = sc_finality_grandpa::block_import(
        client.clone(),
        &(client.clone() as Arc<_>),
        select_chain.clone(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;

    let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
    let raw_duration = slot_duration.slot_duration();

    let import_queue = sc_consensus_aura::import_queue::<
        sp_consensus_aura::ed25519::AuthorityPair,
        _,
        _,
        _,
        _,
        _,
        _,
    >(ImportQueueParams {
        block_import: grandpa_block_import.clone(),
        justification_import: Some(Box::new(grandpa_block_import.clone())),
        client: client.clone(),
        create_inherent_data_providers: move |_, ()| async move {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

            let slot =
                sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
                    *timestamp,
                    raw_duration,
                );

            Ok((timestamp, slot))
        },
        spawner: &task_manager.spawn_essential_handle(),
        can_author_with: sp_consensus::NeverCanAuthor,
        registry: config.prometheus_registry(),
        check_for_equivocation: Default::default(),
        telemetry: telemetry.as_ref().map(|x| x.handle()),
    })?;

    let (network, system_rpc_tx, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            on_demand: Some(on_demand.clone()),
            block_announce_validator_builder: None,
        })?;

    network_starter.start_network();

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &config,
            task_manager.spawn_handle(),
            client.clone(),
            network.clone(),
        );
    }

    let light_deps = bholdus_rpc::LightDeps {
        remote_blockchain: backend.remote_blockchain(),
        fetcher: on_demand.clone(),
        client: client.clone(),
        pool: transaction_pool.clone(),
    };

    let rpc_extensions = bholdus_rpc::create_light(light_deps);

    let rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        on_demand: Some(on_demand),
        remote_blockchain: Some(backend.remote_blockchain()),
        rpc_extensions_builder: Box::new(sc_service::NoopRpcExtensionBuilder(rpc_extensions)),
        client: client.clone(),
        transaction_pool: transaction_pool.clone(),
        keystore: keystore_container.sync_keystore(),
        config,
        backend,
        system_rpc_tx,
        network: network.clone(),
        task_manager: &mut task_manager,
        telemetry: telemetry.as_mut(),
    })?;

    Ok((
        task_manager,
        rpc_handlers,
        client,
        network,
        transaction_pool,
    ))
}

/// Builds a new service for a light client.
pub fn new_light(config: Configuration) -> Result<TaskManager, ServiceError> {
    new_light_base(config).map(|(task_manager, _, _, _, _)| task_manager)
}
