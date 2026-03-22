use clap::Parser;
use membrain_daemon::daemon::{DaemonRuntime, DaemonRuntimeConfig};
use std::path::PathBuf;
use std::time::Duration;

fn parse_positive_usize(value: &str) -> Result<usize, String> {
    let parsed: usize = value
        .parse()
        .map_err(|_| format!("invalid integer value: {value}"))?;
    if parsed == 0 {
        Err("value must be at least 1".to_string())
    } else {
        Ok(parsed)
    }
}

fn parse_positive_u64(value: &str) -> Result<u64, String> {
    let parsed: u64 = value
        .parse()
        .map_err(|_| format!("invalid integer value: {value}"))?;
    if parsed == 0 {
        Err("value must be at least 1".to_string())
    } else {
        Ok(parsed)
    }
}

fn parse_positive_u32(value: &str) -> Result<u32, String> {
    let parsed: u32 = value
        .parse()
        .map_err(|_| format!("invalid integer value: {value}"))?;
    if parsed == 0 {
        Err("value must be at least 1".to_string())
    } else {
        Ok(parsed)
    }
}

#[derive(Parser, Debug)]
#[command(name = "membrain-daemon", version, about = "Membrain local daemon")]
struct Cli {
    /// Unix socket path to bind.
    #[arg(long, default_value = "/tmp/membrain.sock")]
    socket_path: PathBuf,

    /// Maximum number of concurrent request handlers.
    #[arg(long, default_value_t = 8, value_parser = parse_positive_usize)]
    request_concurrency: usize,

    /// Maximum queued requests before new requests are rejected.
    #[arg(long, default_value_t = 32, value_parser = parse_positive_usize)]
    max_queue_depth: usize,

    /// Background maintenance interval in seconds.
    #[arg(long, default_value_t = 60, value_parser = parse_positive_u64)]
    maintenance_interval_secs: u64,

    /// Poll budget for each maintenance run.
    #[arg(long, default_value_t = 4, value_parser = parse_positive_u32)]
    maintenance_poll_budget: u32,

    /// Delay between maintenance poll steps in milliseconds.
    #[arg(long, default_value_t = 25, value_parser = parse_positive_u64)]
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

    #[test]
    fn cli_rejects_zero_runtime_maintenance_flags() {
        for args in [
            ["membrain-daemon", "--request-concurrency", "0"],
            ["membrain-daemon", "--max-queue-depth", "0"],
            ["membrain-daemon", "--maintenance-interval-secs", "0"],
            ["membrain-daemon", "--maintenance-poll-budget", "0"],
            ["membrain-daemon", "--maintenance-step-delay-ms", "0"],
        ] {
            let error = Cli::try_parse_from(args).expect_err("zero value should be rejected");
            let rendered = error.to_string();
            assert!(
                rendered.contains("value must be at least 1"),
                "unexpected clap error: {rendered}"
            );
        }
    }

    #[test]
    fn cli_rejects_non_numeric_runtime_maintenance_flags() {
        for args in [
            ["membrain-daemon", "--request-concurrency", "abc"],
            ["membrain-daemon", "--max-queue-depth", "abc"],
            ["membrain-daemon", "--maintenance-interval-secs", "abc"],
            ["membrain-daemon", "--maintenance-poll-budget", "abc"],
            ["membrain-daemon", "--maintenance-step-delay-ms", "abc"],
        ] {
            let error =
                Cli::try_parse_from(args).expect_err("non-numeric value should be rejected");
            let rendered = error.to_string();
            assert!(
                rendered.contains("invalid integer value: abc"),
                "unexpected clap error: {rendered}"
            );
        }
    }
}
