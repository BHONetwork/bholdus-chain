// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::cli::{Cli, Subcommand};
use sc_cli::{ChainSpec, RuntimeVersion, SubstrateCli};
use service::{chain_spec, IdentifyVariant};

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "Bholdus Node".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "support.anonymous.an".into()
    }

    fn copyright_start_year() -> i32 {
        2017
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(match id {
            #[cfg(feature = "with-ulas-runtime")]
            "ulas-dev" => Box::new(chain_spec::ulas::development_config()?),
            #[cfg(feature = "with-ulas-runtime")]
            "ulas-local" => Box::new(chain_spec::ulas::local_testnet_config()?),
            #[cfg(feature = "with-ulas-runtime")]
            "ulas-prod-sample" => Box::new(chain_spec::ulas::production_sample_config()?),
            #[cfg(feature = "with-ulas-runtime")]
            "ulas" => Box::new(chain_spec::ulas::config()?),

            #[cfg(feature = "with-phoenix-runtime")]
            "phoenix-dev" => Box::new(chain_spec::phoenix::development_config()?),
            #[cfg(feature = "with-phoenix-runtime")]
            "phoenix-local" => Box::new(chain_spec::phoenix::local_testnet_config()?),
            #[cfg(feature = "with-phoenix-runtime")]
            "phoenix-prod-sample" => Box::new(chain_spec::phoenix::production_sample_config()?),
            #[cfg(feature = "with-phoenix-runtime")]
            "phoenix" => Box::new(chain_spec::phoenix::config()?),

            path => {
                let path = std::path::PathBuf::from(path);
                let chain_spec = Box::new(service::chain_spec::DummyChainSpec::from_json_file(
                    path.clone(),
                )?) as Box<dyn sc_service::ChainSpec>;

                if chain_spec.is_ulas() {
                    #[cfg(feature = "with-ulas-runtime")]
                    {
                        Box::new(chain_spec::ulas::ChainSpec::from_json_file(path)?)
                    }

                    #[cfg(not(feature = "with-ulas-runtime"))]
                    return Err(service::ULAS_RUNTIME_NOT_AVAILABLE.into());
                } else {
                    #[cfg(feature = "with-phoenix-runtime")]
                    {
                        Box::new(chain_spec::phoenix::ChainSpec::from_json_file(path)?)
                    }

                    #[cfg(not(feature = "with-phoenix-runtime"))]
                    return Err(service::PHOENIX_RUNTIME_NOT_AVAILABLE.into());
                }
            }
        })
    }

    fn native_runtime_version(spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        if spec.is_ulas() {
            #[cfg(feature = "with-ulas-runtime")]
            return &service::ulas_runtime::VERSION;
            #[cfg(not(feature = "with-ulas-runtime"))]
            panic!("{}", service::ULAS_RUNTIME_NOT_AVAILABLE);
        } else {
            #[cfg(feature = "with-phoenix-runtime")]
            return &service::phoenix_runtime::VERSION;
            #[cfg(not(feature = "with-phoenix-runtime"))]
            panic!("{}", service::PHOENIX_RUNTIME_NOT_AVAILABLE);
        }
    }
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
    let cli = Cli::from_args();

    match &cli.subcommand {
        None => {
            let runner = cli.create_runner(&cli.run.base)?;
            runner.run_node_until_exit(|config| async move {
                let chain_spec = &config.chain_spec;
                let rpc_config = service::rpc::RpcConfig {
                    eth_log_block_cache: cli.run.eth_log_block_cache,
                    eth_statuses_cache: cli.run.eth_statuses_cache,
                    fee_history_limit: cli.run.fee_history_limit,
                    max_past_logs: cli.run.max_past_logs,
                    ethapi: cli.run.ethapi,
                    ethapi_max_permits: cli.run.ethapi_max_permits,
                    ethapi_trace_cache_duration: cli.run.ethapi_trace_cache_duration,
                    ethapi_trace_max_count: cli.run.ethapi_trace_max_count,
                };

                if chain_spec.is_ulas() {
                    #[cfg(feature = "with-ulas-runtime")]
                    {
                        return service::new_full::<
                            service::ulas_runtime::RuntimeApi,
                            service::UlasExecutor,
                        >(config, rpc_config)
                        .map_err(sc_cli::Error::Service);
                    }
                    #[cfg(not(feature = "with-ulas-runtime"))]
                    return Err(service::ULAS_RUNTIME_NOT_AVAILABLE.into());
                } else {
                    #[cfg(feature = "with-phoenix-runtime")]
                    {
                        return service::new_full::<
                            service::phoenix_runtime::RuntimeApi,
                            service::PhoenixExecutor,
                        >(config, rpc_config)
                        .map_err(sc_cli::Error::Service);
                    }
                    #[cfg(not(feature = "with-phoenix-runtime"))]
                    return Err(service::PHOENIX_RUNTIME_NOT_AVAILABLE.into());
                }
            })
        }
        Some(Subcommand::Inspect(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| {
                let chain_spec = &config.chain_spec;

                if chain_spec.is_ulas() {
                    #[cfg(feature = "with-ulas-runtime")]
                    {
                        return cmd.run::<service::ulas_runtime::Block, service::ulas_runtime::RuntimeApi, service::UlasExecutor>(
                            config,
                        )
                    }
                    #[cfg(not(feature = "with-ulas-runtime"))]
                    return Err(service::ULAS_RUNTIME_NOT_AVAILABLE.into());
                }  else {
                    #[cfg(feature = "with-phoenix-runtime")]
                    {
                        return cmd.run::<service::phoenix_runtime::Block, service::phoenix_runtime::RuntimeApi, service::PhoenixExecutor>(
                            config,
                        )
                    }
                    #[cfg(not(feature = "with-phoenix-runtime"))]
                    return Err(service::PHOENIX_RUNTIME_NOT_AVAILABLE.into());
                }
            })
        }
        Some(Subcommand::Benchmark(cmd)) => {
            if cfg!(feature = "runtime-benchmarks") {
                let runner = cli.create_runner(cmd)?;
                runner.sync_run(|config| {
                    let chain_spec = &config.chain_spec;
                    if chain_spec.is_ulas() {
                        #[cfg(feature = "with-ulas-runtime")]
                        {
                            return cmd.run::<service::ulas_runtime::Block, service::UlasExecutor>(
                                config,
                            );
                        }
                        #[cfg(not(feature = "with-ulas-runtime"))]
                        return Err(service::ULAS_RUNTIME_NOT_AVAILABLE.into());
                    } else {
                        #[cfg(feature = "with-phoenix-runtime")]
                        {
                            return cmd
                                .run::<service::phoenix_runtime::Block, service::PhoenixExecutor>(
                                    config,
                                );
                        }
                        #[cfg(not(feature = "with-phoenix-runtime"))]
                        return Err(service::PHOENIX_RUNTIME_NOT_AVAILABLE.into());
                    }
                })
            } else {
                Err("Benchmarking wasn't enabled when building the node. \
				You can enable it with `--features runtime-benchmarks`."
                    .into())
            }
        }
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::Sign(cmd)) => cmd.run(),
        Some(Subcommand::Verify(cmd)) => cmd.run(),
        Some(Subcommand::Vanity(cmd)) => cmd.run(),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, import_queue, task_manager) = service::new_chain_ops(&mut config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, _, task_manager) = service::new_chain_ops(&mut config)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, _, task_manager) = service::new_chain_ops(&mut config)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, import_queue, task_manager) = service::new_chain_ops(&mut config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| {
                // Remove Frontier offchain db
                let frontier_database_config = sc_service::DatabaseSource::RocksDb {
                    path: service::frontier_database_dir(&config),
                    cache_size: 0,
                };

                cmd.run(frontier_database_config)?;
                cmd.run(config.database)
            })
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, backend, _, task_manager) = service::new_chain_ops(&mut config)?;
                Ok((cmd.run(client, backend), task_manager))
            })
        }
        #[cfg(feature = "try-runtime")]
        Some(Subcommand::TryRuntime(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            if chain_spec.is_ulas() {
                #[cfg(feature = "with-ulas-runtime")]
                {
                    return runner.async_run(|config| {
                        // we don't need any of the components of new_partial, just a runtime, or a task
                        // manager to do `async_run`.
                        let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
                        let task_manager =
                            sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
                                .map_err(|e| {
                                    sc_cli::Error::Service(sc_service::Error::Prometheus(e))
                                })?;

                        Ok((
                            cmd.run::<service::ulas_runtime::Block, service::UlasExecutor>(config),
                            task_manager,
                        ))
                    });
                }
                #[cfg(not(feature = "with-ulas-runtime"))]
                return Err(service::ULAS_RUNTIME_NOT_AVAILABLE.into());
            } else {
                #[cfg(feature = "with-phoenix-runtime")]
                {
                    return runner.async_run(|config| {
                        // we don't need any of the components of new_partial, just a runtime, or a task
                        // manager to do `async_run`.
                        let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
                        let task_manager =
                            sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
                                .map_err(|e| {
                                    sc_cli::Error::Service(sc_service::Error::Prometheus(e))
                                })?;

                        Ok((
                            cmd.run::<service::phoenix_runtime::Block, service::PhoenixExecutor>(
                                config,
                            ),
                            task_manager,
                        ))
                    });
                }
                #[cfg(not(feature = "with-phoenix-runtime"))]
                return Err(service::PHOENIX_RUNTIME_NOT_AVAILABLE.into());
            }
        }
        #[cfg(not(feature = "try-runtime"))]
        Some(Subcommand::TryRuntime) => Err("TryRuntime wasn't enabled when building the node. \
					You can enable it with `--features try-runtime`."
            .into()),
    }
}
