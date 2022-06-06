// Copyright 2022 SmallWorld Selendra.
// This file is part of Selendra.

// Selendra is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Selendra is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Selendra.  If not, see <http://www.gnu.org/licenses/>.

// Disable the following lints
#![allow(clippy::borrowed_box)]

use crate::cli::{Cli, RelayChainCli, Subcommand};
use codec::Encode;
use cumulus_client_service::genesis::generate_genesis_block;
use cumulus_primitives_core::ParaId;
use service::{chain_spec, IdentifyVariant};

use log::info;
use sc_cli::{
	ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams,
	NetworkParams, Result, RuntimeVersion, SharedParams, SubstrateCli,
};
use sc_service::config::{BasePath, PrometheusConfig};
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::traits::Block as BlockT;
use std::{io::Write, net::SocketAddr};

fn chain_name() -> String {
	"Bitriel".into()
}

fn get_exec_name() -> Option<String> {
	std::env::current_exe()
		.ok()
		.and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
		.and_then(|s| s.into_string().ok())
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		format!("{} Node", chain_name())
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
		"https://github.com/bitriel/bitriel-finance/issues".into()
	}

	fn copyright_start_year() -> i32 {
		2022
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		let id = if id == "" {
			let n = get_exec_name().unwrap_or_default();
			["bitriel", "rieltest"]
				.iter()
				.cloned()
				.find(|&chain| n.starts_with(chain))
				.unwrap_or("bitriel")
		} else {
			id
		};

		Ok(match id {
			#[cfg(feature = "with-rieltest-runtime")]
			"rieltest" => Box::new(chain_spec::rieltest::rieltest_config()?),
			#[cfg(feature = "with-rieltest-runtime")]
			"rieltest-dev" => Box::new(chain_spec::rieltest::rieltest_dev_config()?),
			#[cfg(feature = "with-rieltest-runtime")]
			"rieltest-staging" => Box::new(chain_spec::rieltest::rieltest_staging_config()?),

			#[cfg(feature = "with-bitriel-runtime")]
			"bitriel" => Box::new(chain_spec::bitriel::bitriel_config()?),
			#[cfg(feature = "with-bitriel-runtime")]
			"bitriel-selendra" => Box::new(chain_spec::bitriel::bitriel_staging_config()?),
			#[cfg(feature = "with-bitriel-runtime")]
			"bitriel-staging" => Box::new(chain_spec::bitriel::bitriel_dev_config()?),
			path => {
				let path = std::path::PathBuf::from(path);

				let chain_spec =
					Box::new(service::chain_spec::DummyChainSpec::from_json_file(path.clone())?)
						as Box<dyn service::ChainSpec>;

				if chain_spec.is_bitriel() {
					#[cfg(feature = "with-bitriel-runtime")]
					{
						Box::new(chain_spec::bitriel::ChainSpec::from_json_file(path)?)
					}
					#[cfg(not(feature = "with-bitriel-runtime"))]
					return Err(service::BITRIEL_RUNTIME_NOT_AVAILABLE.into());
				} else {
					#[cfg(feature = "with-rieltest-runtime")]
					{
						Box::new(chain_spec::rieltest::ChainSpec::from_json_file(path)?)
					}
					#[cfg(not(feature = "with-rieltest-runtime"))]
					return Err(service::RIELTEST_RUNTIME_NOT_AVAILABLE.into());
				}
			},
		})
	}

	fn native_runtime_version(spec: &Box<dyn sc_service::ChainSpec>) -> &'static RuntimeVersion {
		if spec.is_bitriel() {
			#[cfg(feature = "with-bitriel-runtime")]
			return &service::bitriel_runtime::VERSION;
			#[cfg(not(feature = "with-bitriel-runtime"))]
			panic!("{}", service::BITRIEL_RUNTIME_NOT_AVAILABLE);
		} else {
			#[cfg(feature = "with-rieltest-runtime")]
			return &service::rieltest_runtime::VERSION;
			#[cfg(not(feature = "with-rieltest-runtime"))]
			panic!("{}", service::RIELTEST_RUNTIME_NOT_AVAILABLE);
		}
	}
}

impl SubstrateCli for RelayChainCli {
	fn impl_name() -> String {
		format!("{} Parachain Collator", chain_name())
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		format!(
			"{} parachain collator\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relaychain node.\n\n\
		cardamom-collator [parachain-args] -- [relaychain-args]",
			chain_name()
		)
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/bitriel/bitriel-finance/issues".into()
	}

	fn copyright_start_year() -> i32 {
		2022
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		selendra_cli::Cli::from_iter([RelayChainCli::executable_name()].iter()).load_spec(id)
	}

	fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		selendra_cli::Cli::native_runtime_version(chain_spec)
	}
}

fn set_default_ss58_version(spec: &Box<dyn service::ChainSpec>) {
	use sp_core::crypto::Ss58AddressFormatRegistry;

	let ss58_version = if spec.is_rieltest() {
		Ss58AddressFormatRegistry::SubstrateAccount
	} else if spec.is_bitriel() {
		Ss58AddressFormatRegistry::SubstrateAccount
	} else {
		Ss58AddressFormatRegistry::SubstrateAccount
	};

	sp_core::crypto::set_default_ss58_version(ss58_version.into());
}

#[allow(dead_code)]
const DEV_ONLY_ERROR_PATTERN: &str =
	"can only use subcommand with --chain [bitriel-dev, pc-dev, dev], got ";

#[allow(dead_code)]
fn ensure_dev(spec: &Box<dyn service::ChainSpec>) -> std::result::Result<(), String> {
	if spec.is_dev() {
		Ok(())
	} else {
		Err(format!("{}{}", DEV_ONLY_ERROR_PATTERN, spec.id()))
	}
}

fn extract_genesis_wasm(chain_spec: &Box<dyn service::ChainSpec>) -> Result<Vec<u8>> {
	let mut storage = chain_spec.build_storage()?;

	storage
		.top
		.remove(sp_core::storage::well_known_keys::CODE)
		.ok_or_else(|| "Could not find wasm file in genesis state!".into())
}

macro_rules! with_runtime_or_err {
	($chain_spec:expr, { $( $code:tt )* }) => {
		if $chain_spec.is_bitriel() {
			#[cfg(feature = "with-bitriel-runtime")]
			#[allow(unused_imports)]
			use service::{bitriel_runtime::{Block, RuntimeApi}, BitrielExecutorDispatch as Executor};
			#[cfg(feature = "with-bitriel-runtime")]
			$( $code )*

			#[cfg(not(feature = "with-bitriel-runtime"))]
			return Err(service::BITRIEL_RUNTIME_NOT_AVAILABLE.into());
		} else {
			#[cfg(feature = "with-rieltest-runtime")]
			#[allow(unused_imports)]
			use service::{rieltest_runtime::{Block, RuntimeApi}, RieltestExecutorDispatch as Executor};
			#[cfg(feature = "with-rieltest-runtime")]
			$( $code )*

			#[cfg(not(feature = "with-rieltest-runtime"))]
			return Err(service::RIELTEST_RUNTIME_NOT_AVAILABLE.into());
		}
	}
}

/// Parses bitriel specific CLI arguments and run the service.
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::Inspect(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.sync_run(|mut config| {
				let (client, _, _, _) = service::new_chain_ops(&mut config)?;
				cmd.run(client)
			})
		},

		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			with_runtime_or_err!(chain_spec, {
				return runner.sync_run(|config| cmd.run::<Block, Executor>(config));
			})
		},

		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::Sign(cmd)) => cmd.run(),
		Some(Subcommand::Verify(cmd)) => cmd.run(),
		Some(Subcommand::Vanity(cmd)) => cmd.run(),

		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},

		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},

		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		},

		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		},

		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},

		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| {
				// cumulus_client_cli::PurgeCommand fails to account for whether we are running --dev
				let is_dev = cmd.base.shared_params.is_dev();

				// uses base instead to run substrate's sc_cli::PurgeChain command
				if is_dev {
					return cmd.base.run(config.database);
				}

				let selendra_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()].iter().chain(cli.relaychain_args.iter()),
				);

				let selendra_config = SubstrateCli::create_configuration(
					&selendra_cli,
					&selendra_cli,
					config.tokio_handle.clone(),
				)
				.map_err(|err| format!("Relay chain argument error: {}", err))?;

				cmd.run(config, selendra_config)
			})
		},

		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.async_run(|mut config| {
				let (client, backend, _, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, backend), task_manager))
			})
		},

		Some(Subcommand::ExportGenesisState(params)) => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let chain_spec = cli.load_spec(&params.chain.clone().unwrap_or_default())?;
			let state_version = Cli::native_runtime_version(&chain_spec).state_version();
			let output_buf = with_runtime_or_err!(chain_spec, {
				{
					let block: Block = generate_genesis_block(&chain_spec, state_version)
						.map_err(|e| format!("{:?}", e))?;
					let raw_header = block.header().encode();
					let buf = if params.raw {
						raw_header
					} else {
						format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
					};
					buf
				}
			});

			if let Some(output) = &params.output {
				std::fs::write(output, output_buf)?;
			} else {
				std::io::stdout().write_all(&output_buf)?;
			}

			Ok(())
		},

		Some(Subcommand::ExportGenesisWasm(params)) => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let raw_wasm_blob =
				extract_genesis_wasm(&cli.load_spec(&params.chain.clone().unwrap_or_default())?)?;
			let output_buf = if params.raw {
				raw_wasm_blob
			} else {
				format!("0x{:?}", HexDisplay::from(&raw_wasm_blob)).into_bytes()
			};

			if let Some(output) = &params.output {
				std::fs::write(output, output_buf)?;
			} else {
				std::io::stdout().write_all(&output_buf)?;
			}

			Ok(())
		},

		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;
			set_default_ss58_version(chain_spec);

			ensure_dev(chain_spec).map_err(|err| format!("try-runtime error: {}", err))?;

			with_runtime_or_err!(chain_spec, {
				return runner.async_run(|config| {
					let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
					let task_manager =
						sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
							.map_err(|e| {
								sc_cli::Error::Service(sc_service::Error::Prometheus(e))
							})?;
					Ok((cmd.run::<Block, Executor>(config), task_manager))
				})
			})
		},

		None => {
			let runner = cli.create_runner(&cli.run.normalize())?;
			let chain_spec = &runner.config().chain_spec;
			let is_rieltest_dev = chain_spec.is_rieltest_dev();
			let collator_options = cli.run.collator_options();

			set_default_ss58_version(chain_spec);

			runner.run_node_until_exit(|config| async move {
				let para_id = chain_spec::Extensions::try_get(&*config.chain_spec)
					.map(|e| e.para_id)
					.ok_or("Could not find parachain extension for chain-spec.")?;

				if is_rieltest_dev {
					#[cfg(feature = "with-rieltest-runtime")]
					return service::rieltest_dev(config, cli.instant_sealing).map_err(Into::into);
					#[cfg(not(feature = "with-rieltest-runtime"))]
					return Err(service::RIELTEST_RUNTIME_NOT_AVAILABLE.into())
				} else if cli.instant_sealing {
					return Err("Instant sealing can be turned on only in `--dev` mode".into());
				}

				let selendra_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()].iter().chain(cli.relaychain_args.iter()),
				);

				let id = ParaId::from(para_id);

				let selendra_config = SubstrateCli::create_configuration(
					&selendra_cli,
					&selendra_cli,
					config.tokio_handle.clone(),
				)
				.map_err(|err| format!("Relay chain argument error: {}", err))?;

				info!("Parachain id: {:?}", id);
				info!("Is collating: {}", if config.role.is_authority() { "yes" } else { "no" });

				with_runtime_or_err!(config.chain_spec, {
					{
						service::start_node::<RuntimeApi>(
							config,
							selendra_config,
							collator_options,
							id,
						)
						.await
						.map(|r| r.0)
						.map_err(Into::into)
					}
				})
			})
		},
	}
}

impl DefaultConfigurationValues for RelayChainCli {
	fn p2p_listen_port() -> u16 {
		30334
	}

	fn rpc_ws_listen_port() -> u16 {
		9945
	}

	fn rpc_http_listen_port() -> u16 {
		9934
	}

	fn prometheus_listen_port() -> u16 {
		9616
	}
}

impl CliConfiguration<Self> for RelayChainCli {
	fn shared_params(&self) -> &SharedParams {
		self.base.base.shared_params()
	}

	fn import_params(&self) -> Option<&ImportParams> {
		self.base.base.import_params()
	}

	fn network_params(&self) -> Option<&NetworkParams> {
		self.base.base.network_params()
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		self.base.base.keystore_params()
	}

	fn base_path(&self) -> Result<Option<BasePath>> {
		Ok(self
			.shared_params()
			.base_path()
			.or_else(|| self.base_path.clone().map(Into::into)))
	}

	fn rpc_http(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_http(default_listen_port)
	}

	fn rpc_ipc(&self) -> Result<Option<String>> {
		self.base.base.rpc_ipc()
	}

	fn rpc_ws(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_ws(default_listen_port)
	}

	fn prometheus_config(
		&self,
		default_listen_port: u16,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<PrometheusConfig>> {
		self.base.base.prometheus_config(default_listen_port, chain_spec)
	}

	fn init<F>(
		&self,
		_support_url: &String,
		_impl_version: &String,
		_logger_hook: F,
		_config: &sc_service::Configuration,
	) -> Result<()>
	where
		F: FnOnce(&mut sc_cli::LoggerBuilder, &sc_service::Configuration),
	{
		unreachable!("SelendraCli is never initialized; qed");
	}

	fn chain_id(&self, is_dev: bool) -> Result<String> {
		let chain_id = self.base.base.chain_id(is_dev)?;

		Ok(if chain_id.is_empty() { self.chain_id.clone().unwrap_or_default() } else { chain_id })
	}

	fn role(&self, is_dev: bool) -> Result<sc_service::Role> {
		self.base.base.role(is_dev)
	}

	fn transaction_pool(&self) -> Result<sc_service::config::TransactionPoolOptions> {
		self.base.base.transaction_pool()
	}

	fn state_cache_child_ratio(&self) -> Result<Option<usize>> {
		self.base.base.state_cache_child_ratio()
	}

	fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
		self.base.base.rpc_methods()
	}

	fn rpc_ws_max_connections(&self) -> Result<Option<usize>> {
		self.base.base.rpc_ws_max_connections()
	}

	fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
		self.base.base.rpc_cors(is_dev)
	}

	fn default_heap_pages(&self) -> Result<Option<u64>> {
		self.base.base.default_heap_pages()
	}

	fn force_authoring(&self) -> Result<bool> {
		self.base.base.force_authoring()
	}

	fn disable_grandpa(&self) -> Result<bool> {
		self.base.base.disable_grandpa()
	}

	fn max_runtime_instances(&self) -> Result<Option<usize>> {
		self.base.base.max_runtime_instances()
	}

	fn announce_block(&self) -> Result<bool> {
		self.base.base.announce_block()
	}
}
