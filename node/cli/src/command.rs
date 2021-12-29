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
use sc_service::PartialComponents;
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

            #[cfg(feature = "with-cygnus-runtime")]
            "cygnus-dev" => Box::new(chain_spec::cygnus::development_config()?),
            #[cfg(feature = "with-cygnus-runtime")]
            "cygnus-local" => Box::new(chain_spec::cygnus::local_testnet_config()?),
            #[cfg(feature = "with-cygnus-runtime")]
            "cygnus-prod-sample" => Box::new(chain_spec::cygnus::production_sample_config()?),
            #[cfg(feature = "with-cygnus-runtime")]
            "cygnus" => Box::new(chain_spec::cygnus::config()?),

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
                } else if chain_spec.is_cygnus() {
                    #[cfg(feature = "with-cygnus-runtime")]
                    {
                        Box::new(chain_spec::cygnus::ChainSpec::from_json_file(path)?)
                    }

                    #[cfg(not(feature = "with-cygnus-runtime"))]
                    return Err(service::CYGNUS_RUNTIME_NOT_AVAILABLE.into());
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
        } else if spec.is_cygnus() {
            #[cfg(feature = "with-cygnus-runtime")]
            return &service::cygnus_runtime::VERSION;
            #[cfg(not(feature = "with-cygnus-runtime"))]
            panic!("{}", service::CYGNUS_RUNTIME_NOT_AVAILABLE);
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
            let runner = cli.create_runner(&cli.run)?;
            runner.run_node_until_exit(|config| async move {
                let chain_spec = &config.chain_spec;

                if chain_spec.is_ulas() {
                    #[cfg(feature = "with-ulas-runtime")]
                    {
                        return service::service::ulas::new_full(config)
                            .map_err(sc_cli::Error::Service);
                    }
                    #[cfg(not(feature = "with-ulas-runtime"))]
                    return Err(service::ULAS_RUNTIME_NOT_AVAILABLE.into());
                } else if chain_spec.is_cygnus() {
                    #[cfg(feature = "with-cygnus-runtime")]
                    {
                        return service::service::cygnus::new_full(config)
                            .map_err(sc_cli::Error::Service);
                    }
                    #[cfg(not(feature = "with-cygnus-runtime"))]
                    return Err(service::CYGNUS_RUNTIME_NOT_AVAILABLE.into());
                } else {
                    #[cfg(feature = "with-phoenix-runtime")]
                    {
                        return service::service::phoenix::new_full(config)
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
                        return cmd.run::<service::ulas_runtime::Block, service::ulas_runtime::RuntimeApi, service::service::ulas::ExecutorDispatch>(
                            config,
                        )
                    }
                    #[cfg(not(feature = "with-ulas-runtime"))]
                    return Err(service::ULAS_RUNTIME_NOT_AVAILABLE.into());
                } else if chain_spec.is_cygnus() {
                    #[cfg(feature = "with-cygnus-runtime")]
                    {
                        return cmd.run::<service::cygnus_runtime::Block, service::cygnus_runtime::RuntimeApi, service::service::cygnus::ExecutorDispatch>(
                            config,
                        )
                    }
                    #[cfg(not(feature = "with-cygnus-runtime"))]
                    return Err(service::CYGNUS_RUNTIME_NOT_AVAILABLE.into());
                } else {
                    #[cfg(feature = "with-phoenix-runtime")]
                    {
                        return cmd.run::<service::phoenix_runtime::Block, service::phoenix_runtime::RuntimeApi, service::service::phoenix::ExecutorDispatch>(
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
                            return cmd.run::<service::ulas_runtime::Block, service::service::ulas::ExecutorDispatch>(config)
                        }
                        #[cfg(not(feature = "with-ulas-runtime"))]
                        return Err(service::ULAS_RUNTIME_NOT_AVAILABLE.into());
                    } else if chain_spec.is_cygnus() {
                        #[cfg(feature = "with-cygnus-runtime")]
                        {
                            return cmd.run::<service::cygnus_runtime::Block, service::service::cygnus::ExecutorDispatch>(config)
                        }
                        #[cfg(not(feature = "with-cygnus-runtime"))]
                        return Err(service::CYGNUS_RUNTIME_NOT_AVAILABLE.into());
                    } else {
                        #[cfg(feature = "with-phoenix-runtime")]
                        {
                            return cmd.run::<service::phoenix_runtime::Block, service::service::phoenix::ExecutorDispatch>(config)
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
            let chain_spec = &runner.config().chain_spec;
            if chain_spec.is_ulas() {
                #[cfg(feature = "with-ulas-runtime")]
                {
                    return runner.async_run(|config| {
                        let PartialComponents {
                            client,
                            task_manager,
                            import_queue,
                            ..
                        } = service::service::ulas::new_partial(&config)?;
                        return Ok((cmd.run(client, import_queue), task_manager));
                    });
                }
                #[cfg(not(feature = "with-ulas-runtime"))]
                return Err(service::ULAS_RUNTIME_NOT_AVAILABLE.into());
            } else if chain_spec.is_cygnus() {
                #[cfg(feature = "with-cygnus-runtime")]
                {
                    return runner.async_run(|config| {
                        let PartialComponents {
                            client,
                            task_manager,
                            import_queue,
                            ..
                        } = service::service::cygnus::new_partial(&config)?;
                        return Ok((cmd.run(client, import_queue), task_manager));
                    });
                }
                #[cfg(not(feature = "with-cygnus-runtime"))]
                return Err(service::CYGNUS_RUNTIME_NOT_AVAILABLE.into());
            } else {
                #[cfg(feature = "with-phoenix-runtime")]
                #[allow(unused_imports)]
                {
                    return runner.async_run(|config| {
                        let PartialComponents {
                            client,
                            task_manager,
                            import_queue,
                            ..
                        } = service::service::phoenix::new_partial(&config)?;
                        return Ok((cmd.run(client, import_queue), task_manager));
                    });
                }
                #[cfg(not(feature = "with-phoenix-runtime"))]
                return Err(service::PHOENIX_RUNTIME_NOT_AVAILABLE.into());
            }
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;
            if chain_spec.is_ulas() {
                #[cfg(feature = "with-ulas-runtime")]
                {
                    return runner.async_run(|config| {
                        let PartialComponents {
                            client,
                            task_manager,
                            ..
                        } = service::service::ulas::new_partial(&config)?;
                        Ok((cmd.run(client, config.database), task_manager))
                    });
                }
                #[cfg(not(feature = "with-ulas-runtime"))]
                return Err(service::ULAS_RUNTIME_NOT_AVAILABLE.into());
            } else if chain_spec.is_cygnus() {
                #[cfg(feature = "with-cygnus-runtime")]
                {
                    return runner.async_run(|config| {
                        let PartialComponents {
                            client,
                            task_manager,
                            ..
                        } = service::service::cygnus::new_partial(&config)?;
                        Ok((cmd.run(client, config.database), task_manager))
                    });
                }
                #[cfg(not(feature = "with-cygnus-runtime"))]
                return Err(service::CYGNUS_RUNTIME_NOT_AVAILABLE.into());
            } else {
                #[cfg(feature = "with-phoenix-runtime")]
                #[allow(unused_imports)]
                {
                    return runner.async_run(|config| {
                        let PartialComponents {
                            client,
                            task_manager,
                            ..
                        } = service::service::phoenix::new_partial(&config)?;
                        Ok((cmd.run(client, config.database), task_manager))
                    });
                }
                #[cfg(not(feature = "with-phoenix-runtime"))]
                return Err(service::PHOENIX_RUNTIME_NOT_AVAILABLE.into());
            }
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            if chain_spec.is_ulas() {
                #[cfg(feature = "with-ulas-runtime")]
                {
                    return runner.async_run(|config| {
                        let PartialComponents {
                            client,
                            task_manager,
                            ..
                        } = service::service::ulas::new_partial(&config)?;
                        Ok((cmd.run(client, config.chain_spec), task_manager))
                    });
                }
                #[cfg(not(feature = "with-ulas-runtime"))]
                return Err(service::ULAS_RUNTIME_NOT_AVAILABLE.into());
            } else if chain_spec.is_cygnus() {
                #[cfg(feature = "with-cygnus-runtime")]
                {
                    return runner.async_run(|config| {
                        let PartialComponents {
                            client,
                            task_manager,
                            ..
                        } = service::service::cygnus::new_partial(&config)?;
                        Ok((cmd.run(client, config.chain_spec), task_manager))
                    });
                }
                #[cfg(not(feature = "with-cygnus-runtime"))]
                return Err(service::CYGNUS_RUNTIME_NOT_AVAILABLE.into());
            } else {
                #[cfg(feature = "with-phoenix-runtime")]
                {
                    return runner.async_run(|config| {
                        let PartialComponents {
                            client,
                            task_manager,
                            ..
                        } = service::service::phoenix::new_partial(&config)?;
                        Ok((cmd.run(client, config.chain_spec), task_manager))
                    });
                }
                #[cfg(not(feature = "with-phoenix-runtime"))]
                return Err(service::PHOENIX_RUNTIME_NOT_AVAILABLE.into());
            }
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            if chain_spec.is_ulas() {
                #[cfg(feature = "with-ulas-runtime")]
                {
                    return runner.async_run(|config| {
                        let PartialComponents {
                            client,
                            task_manager,
                            import_queue,
                            ..
                        } = service::service::ulas::new_partial(&config)?;
                        Ok((cmd.run(client, import_queue), task_manager))
                    });
                }
                #[cfg(not(feature = "with-ulas-runtime"))]
                return Err(service::ULAS_RUNTIME_NOT_AVAILABLE.into());
            } else if chain_spec.is_cygnus() {
                #[cfg(feature = "with-cygnus-runtime")]
                {
                    return runner.async_run(|config| {
                        let PartialComponents {
                            client,
                            task_manager,
                            import_queue,
                            ..
                        } = service::service::cygnus::new_partial(&config)?;
                        Ok((cmd.run(client, import_queue), task_manager))
                    });
                }
                #[cfg(not(feature = "with-cygnus-runtime"))]
                return Err(service::CYGNUS_RUNTIME_NOT_AVAILABLE.into());
            } else {
                #[cfg(feature = "with-phoenix-runtime")]
                {
                    return runner.async_run(|config| {
                        let PartialComponents {
                            client,
                            task_manager,
                            import_queue,
                            ..
                        } = service::service::phoenix::new_partial(&config)?;
                        Ok((cmd.run(client, import_queue), task_manager))
                    });
                }
                #[cfg(not(feature = "with-phoenix-runtime"))]
                return Err(service::PHOENIX_RUNTIME_NOT_AVAILABLE.into());
            }
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            if chain_spec.is_ulas() {
                #[cfg(feature = "with-ulas-runtime")]
                {
                    return runner.async_run(|config| {
                        let PartialComponents {
                            client,
                            task_manager,
                            backend,
                            ..
                        } = service::service::ulas::new_partial(&config)?;
                        Ok((cmd.run(client, backend), task_manager))
                    });
                }
                #[cfg(not(feature = "with-ulas-runtime"))]
                return Err(service::ULAS_RUNTIME_NOT_AVAILABLE.into());
            } else if chain_spec.is_cygnus() {
                #[cfg(feature = "with-cygnus-runtime")]
                {
                    return runner.async_run(|config| {
                        let PartialComponents {
                            client,
                            task_manager,
                            backend,
                            ..
                        } = service::service::cygnus::new_partial(&config)?;
                        Ok((cmd.run(client, backend), task_manager))
                    });
                }
                #[cfg(not(feature = "with-cygnus-runtime"))]
                return Err(service::CYGNUS_RUNTIME_NOT_AVAILABLE.into());
            } else {
                #[cfg(feature = "with-phoenix-runtime")]
                {
                    return runner.async_run(|config| {
                        let PartialComponents {
                            client,
                            task_manager,
                            backend,
                            ..
                        } = service::service::phoenix::new_partial(&config)?;
                        Ok((cmd.run(client, backend), task_manager))
                    });
                }
                #[cfg(not(feature = "with-phoenix-runtime"))]
                return Err(service::PHOENIX_RUNTIME_NOT_AVAILABLE.into());
            }
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

                        Ok((cmd.run::<service::ulas_runtime::Block, service::service::ulas::ExecutorDispatch>(config), task_manager))
                    });
                }
                #[cfg(not(feature = "with-ulas-runtime"))]
                return Err(service::ULAS_RUNTIME_NOT_AVAILABLE.into());
            } else if chain_spec.is_cygnus() {
                #[cfg(feature = "with-cygnus-runtime")]
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

                        Ok((cmd.run::<service::cygnus_runtime::Block, service::service::cygnus::ExecutorDispatch>(config), task_manager))});
                }
                #[cfg(not(feature = "with-cygnus-runtime"))]
                return Err(service::CYGNUS_RUNTIME_NOT_AVAILABLE.into());
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

                        Ok((cmd.run::<service::phoenix_runtime::Block, service::service::phoenix::ExecutorDispatch>(config), task_manager))});
                }
                #[cfg(not(feature = "with-phoenix-runtime"))]
                return Err(service::PHOENIX_RUNTIME_NOT_AVAILABLE.into());
            }
        }
    }
}
