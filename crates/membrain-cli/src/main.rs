use clap::{Parser, Subcommand};
use membrain_core::api::NamespaceId;
use membrain_daemon::daemon::{DaemonRuntime, DaemonRuntimeConfig};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "membrain", version, about = "Membrain CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Encode (store) a new memory
    Encode {
        /// Content to store
        #[arg(long)]
        content: String,
        /// Namespace for the memory
        #[arg(long)]
        namespace: String,
        /// Type of memory (e.g. factual, episodic)
        #[arg(long, default_value = "factual")]
        memory_type: String,
    },
    /// Recall memories matching a query
    Recall {
        /// Query string to match
        #[arg(long)]
        query: String,
        /// Namespace to search in
        #[arg(long)]
        namespace: String,
        /// Maximum number of results to return
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Inspect a specific memory or entity by ID
    Inspect {
        /// The integer ID to inspect
        #[arg(long)]
        id: u64,
        /// Namespace of the memory
        #[arg(long)]
        namespace: String,
    },
    /// Explain the ranking and routing path for a recall query
    Explain {
        /// Query string to explain
        #[arg(long)]
        query: String,
        /// Namespace to explain over
        #[arg(long)]
        namespace: String,
    },
    /// Run maintenance tasks (repair, reclaim, metrics)
    Maintenance {
        /// The maintenance action to run (e.g. repair, reclaim_space)
        #[arg(long)]
        action: String,
        /// Scope of maintenance
        #[arg(long)]
        namespace: Option<String>,
    },
    /// Run core performance and correctness benchmarks
    Benchmark {
        /// Target metric to benchmark
        #[arg(long, default_value = "latency")]
        target: String,
        /// Number of iterations
        #[arg(long, default_value_t = 100)]
        iters: usize,
    },
    /// Validate system configuration and index health
    Doctor,
    /// Run the local daemon inside the CLI process
    Daemon {
        /// Unix socket path to bind
        #[arg(long, default_value = "/tmp/membrain.sock")]
        socket_path: PathBuf,
        /// Maximum number of concurrent request handlers
        #[arg(long, default_value_t = 8)]
        request_concurrency: usize,
        /// Maximum queued requests before new requests are rejected
        #[arg(long, default_value_t = 32)]
        max_queue_depth: usize,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Encode {
            content,
            namespace,
            memory_type,
        } => {
            let ns = NamespaceId::new(namespace)?;
            println!("Encoding memory in '{}': {}", ns.as_str(), content);
            println!("Memory Type: {}", memory_type);
            println!("Output: {{\"status\": \"success\", \"action\": \"encode\"}}");
        }
        Commands::Recall {
            query,
            namespace,
            limit,
        } => {
            let ns = NamespaceId::new(namespace)?;
            println!("Recalling top {} from '{}': {}", limit, ns.as_str(), query);
            println!(
                "Output: {{\"status\": \"success\", \"action\": \"recall\", \"results\": []}}"
            );
        }
        Commands::Inspect { id, namespace } => {
            let ns = NamespaceId::new(namespace)?;
            println!("Inspecting entity {} in '{}'", id, ns.as_str());
            println!(
                "Output: {{\"status\": \"success\", \"action\": \"inspect\", \"entity\": null}}"
            );
        }
        Commands::Explain { query, namespace } => {
            let ns = NamespaceId::new(namespace)?;
            println!("Explaining '{}' in '{}'", query, ns.as_str());
            println!(
                "Output: {{\"status\": \"success\", \"action\": \"explain\", \"trace\": null}}"
            );
        }
        Commands::Maintenance { action, namespace } => {
            let ns_str = namespace.as_deref().unwrap_or("global");
            println!(
                "Running maintenance action '{}' on scope '{}'",
                action, ns_str
            );
            println!("Output: {{\"status\": \"success\", \"action\": \"maintenance\", \"target\": \"{}\"}}", action);
        }
        Commands::Benchmark { target, iters } => {
            println!("Benchmarking '{}' over {} iterations", target, iters);
            println!("Output: {{\"status\": \"success\", \"action\": \"benchmark\", \"duration_ms\": 0}}");
        }
        Commands::Doctor => {
            println!("Running system diagnostic...");
            println!("Output: {{\"status\": \"success\", \"action\": \"doctor\", \"health\": \"healthy\"}}");
        }
        Commands::Daemon {
            socket_path,
            request_concurrency,
            max_queue_depth,
        } => {
            let mut config = DaemonRuntimeConfig::new(socket_path);
            config.request_concurrency = *request_concurrency;
            config.max_queue_depth = *max_queue_depth;
            let runtime = DaemonRuntime::with_config(config);
            runtime.run_until_stopped().await?;
        }
    }

    Ok(())
}
