use sc_cli::{KeySubcommand, SignCmd, VanityCmd, VerifyCmd};
use structopt::StructOpt;

/// An overarching CLI command definition.
#[derive(Debug, StructOpt)]
pub struct Cli {
    /// Possible subcommand with parameters.
    #[structopt(subcommand)]
    pub subcommand: Option<Subcommand>,
    #[allow(missing_docs)]
    #[structopt(flatten)]
    pub run: RunCmd,
}

#[allow(missing_docs)]
#[derive(Debug, StructOpt)]
pub struct RunCmd {
    #[allow(missing_docs)]
    #[structopt(flatten)]
    pub base: sc_cli::RunCmd,

    /// Enable EVM tracing module on a non-authority node.
    #[structopt(long, conflicts_with = "validator", require_delimiter = true)]
    pub ethapi: Vec<service::EthApiCmd>,

    /// Number of concurrent tracing tasks. Meant to be shared by both "debug" and "trace" modules.
    #[structopt(long, default_value = "10")]
    pub ethapi_max_permits: u32,

    /// Maximum number of trace entries a single request of `trace_filter` is allowed to return.
    /// A request asking for more or an unbounded one going over this limit will both return an
    /// error.
    #[structopt(long, default_value = "500")]
    pub ethapi_trace_max_count: u32,

    /// Duration (in seconds) after which the cache of `trace_filter` for a given block will be
    /// discarded.
    #[structopt(long, default_value = "300")]
    pub ethapi_trace_cache_duration: u64,

    /// Maximum number of logs in a query.
    #[structopt(long, default_value = "10000")]
    pub max_past_logs: u32,

    /// Maximum fee history cache size.
    #[structopt(long, default_value = "2048")]
    pub fee_history_limit: u64,

    /// The dynamic-fee pallet target gas price set by block author
    #[structopt(long, default_value = "1")]
    pub target_gas_price: u64,

    /// Size of the LRU cache for block data and their transaction statuses.
    #[structopt(long, default_value = "3000")]
    pub eth_log_block_cache: usize,
}

/// Possible subcommands of the main binary.
#[derive(Debug, StructOpt)]
pub enum Subcommand {
    /// Key management cli utilities
    Key(KeySubcommand),

    /// The custom inspect subcommmand for decoding blocks and extrinsics.
    #[structopt(
        name = "inspect",
        about = "Decode given block or extrinsic using current native runtime."
    )]
    Inspect(node_inspect::cli::InspectCmd),

    /// The custom benchmark subcommmand benchmarking runtime pallets.
    #[structopt(name = "benchmark", about = "Benchmark runtime pallets.")]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// Try some command against runtime state.
    #[cfg(feature = "try-runtime")]
    TryRuntime(try_runtime_cli::TryRuntimeCmd),

    /// Verify a signature for a message, provided on STDIN, with a given (public or secret) key.
    Verify(VerifyCmd),

    /// Generate a seed that provides a vanity address.
    Vanity(VanityCmd),

    /// Sign a message, with a given (secret) key.
    Sign(SignCmd),

    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),
}
