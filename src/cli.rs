use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "omnisearch",
    about = "CLI multi-source search aggregator",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Search query (when no subcommand is used)
    pub query: Option<String>,

    /// Filter by categories (comma-separated: news,academic,code)
    #[arg(short = 'c', long, value_delimiter = ',')]
    pub categories: Option<Vec<String>>,

    /// Filter by specific sources (comma-separated: wikipedia,arxiv)
    #[arg(short = 's', long, value_delimiter = ',')]
    pub sources: Option<Vec<String>>,

    /// Output format
    #[arg(short = 'f', long, default_value = "json")]
    pub format: String,

    /// Maximum number of results
    #[arg(short = 'n', long)]
    pub max_results: Option<usize>,

    /// Timeout per source in seconds
    #[arg(short = 't', long)]
    pub timeout: Option<u64>,

    /// Verbose mode: show progress on stderr
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Disable cache, force fresh search
    #[arg(long)]
    pub no_cache: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List available sources and their status
    Sources,

    /// Start as MCP server (stdin/stdout JSON-RPC)
    Mcp,

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Set an API key for a source
    Set {
        /// Source name
        source: String,
        /// API key value
        key: String,
    },
    /// Show current configuration
    Show,
    /// Initialize default configuration file
    Init,
}
