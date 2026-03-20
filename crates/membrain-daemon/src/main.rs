use clap::Parser;
use membrain_daemon::daemon::{DaemonRuntime, DaemonRuntimeConfig};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "membrain-daemon", version, about = "Membrain local daemon")]
struct Cli {
    /// Unix socket path to bind.
    #[arg(long, default_value = "/tmp/membrain.sock")]
    socket_path: PathBuf,

    /// Maximum number of concurrent request handlers.
    #[arg(long, default_value_t = 8)]
    request_concurrency: usize,

    /// Maximum queued requests before new requests are rejected.
    #[arg(long, default_value_t = 32)]
    max_queue_depth: usize,

    /// Background maintenance interval in seconds.
    #[arg(long, default_value_t = 60)]
    maintenance_interval_secs: u64,

    /// Poll budget for each maintenance run.
    #[arg(long, default_value_t = 4)]
    maintenance_poll_budget: u32,

    /// Delay between maintenance poll steps in milliseconds.
    #[arg(long, default_value_t = 25)]
    maintenance_step_delay_ms: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut config = DaemonRuntimeConfig::new(&cli.socket_path);
    config.request_concurrency = cli.request_concurrency;
    config.max_queue_depth = cli.max_queue_depth;
    config.maintenance_interval = Duration::from_secs(cli.maintenance_interval_secs);
    config.maintenance_poll_budget = cli.maintenance_poll_budget;
    config.maintenance_step_delay = Duration::from_millis(cli.maintenance_step_delay_ms);

    let runtime = DaemonRuntime::with_config(config);
    runtime.run_until_stopped().await
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::Parser;

    #[test]
    fn cli_parses_runtime_maintenance_flags() {
        let cli = Cli::parse_from([
            "membrain-daemon",
            "--socket-path",
            "/tmp/custom.sock",
            "--request-concurrency",
            "3",
            "--max-queue-depth",
            "11",
            "--maintenance-interval-secs",
            "90",
            "--maintenance-poll-budget",
            "6",
            "--maintenance-step-delay-ms",
            "40",
        ]);

        assert_eq!(cli.request_concurrency, 3);
        assert_eq!(cli.max_queue_depth, 11);
        assert_eq!(cli.maintenance_interval_secs, 90);
        assert_eq!(cli.maintenance_poll_budget, 6);
        assert_eq!(cli.maintenance_step_delay_ms, 40);
    }
}
