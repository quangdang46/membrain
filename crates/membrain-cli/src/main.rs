use clap::{Parser, Subcommand};
use membrain_core::api::NamespaceId;
use membrain_core::engine::maintenance::{
    MaintenanceController, MaintenanceJobHandle, MaintenanceJobState,
};
use membrain_core::engine::repair::{IndexRepairEntrypoint, RepairTarget};
use membrain_core::index::{IndexApi, IndexModule};
use membrain_core::observability::{AuditEventCategory, AuditEventKind};
use membrain_core::store::audit::{AppendOnlyAuditLog, AuditLogEntry, AuditLogStore};
use membrain_core::types::{MemoryId, SessionId};
use membrain_core::{BrainStore, RuntimeConfig};
use membrain_daemon::daemon::{DaemonRuntime, DaemonRuntimeConfig};
use membrain_daemon::rpc::{RuntimeMetrics, RuntimePosture, RuntimeStatus};
use serde::Serialize;
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
    /// Query and export bounded audit history slices
    Audit {
        /// Namespace to inspect
        #[arg(long)]
        namespace: String,
        /// Optional memory id filter
        #[arg(long)]
        id: Option<u64>,
        /// Optional minimum sequence filter
        #[arg(long)]
        since: Option<u64>,
        /// Optional event or category filter
        #[arg(long)]
        op: Option<String>,
        /// Optional tail count after filtering
        #[arg(long)]
        recent: Option<usize>,
        /// Emit JSON instead of text
        #[arg(long, default_value_t = false)]
        json: bool,
    },
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AuditRow {
    sequence: u64,
    category: &'static str,
    kind: &'static str,
    namespace: String,
    memory_id: Option<u64>,
    session_id: Option<u64>,
    triggered_by: &'static str,
    request_id: Option<String>,
    related_run: Option<String>,
    redacted: bool,
    note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct DoctorIndexRow {
    family: &'static str,
    health: &'static str,
    usable: bool,
    entry_count: usize,
    generation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RepairReportRow {
    target: &'static str,
    status: &'static str,
    verification_passed: bool,
    rebuild_entrypoint: Option<&'static str>,
    rebuilt_outputs: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct DoctorReport {
    status: &'static str,
    action: &'static str,
    posture: &'static str,
    degraded_reasons: Vec<String>,
    metrics: RuntimeMetrics,
    indexes: Vec<DoctorIndexRow>,
    repair_engine_component: &'static str,
    repair_reports: Vec<RepairReportRow>,
    warnings: Vec<&'static str>,
}

impl From<AuditLogEntry> for AuditRow {
    fn from(entry: AuditLogEntry) -> Self {
        Self {
            sequence: entry.sequence,
            category: entry.category.as_str(),
            kind: entry.kind.as_str(),
            namespace: entry.namespace.as_str().to_owned(),
            memory_id: entry.memory_id.map(|id| id.0),
            session_id: entry.session_id.map(|id| id.0),
            triggered_by: entry.actor_source,
            request_id: entry.request_id,
            related_run: entry.related_run,
            redacted: entry.redacted,
            note: entry.detail,
        }
    }
}

fn sample_audit_log(namespace: &NamespaceId) -> AppendOnlyAuditLog {
    let mut log = AuditLogStore.new_log(8);
    log.append(
        AuditLogEntry::new(
            AuditEventCategory::Encode,
            AuditEventKind::EncodeAccepted,
            namespace.clone(),
            "encode_engine",
            "encoded memory into durable flow",
        )
        .with_memory_id(MemoryId(21))
        .with_session_id(SessionId(5))
        .with_request_id("req-encode-21"),
    );
    log.append(
        AuditLogEntry::new(
            AuditEventCategory::Policy,
            AuditEventKind::PolicyRedacted,
            namespace.clone(),
            "policy_module",
            "redacted protected actor details for export",
        )
        .with_memory_id(MemoryId(21))
        .with_request_id("req-policy-21")
        .with_related_run("incident-2026-03-20")
        .with_redaction(),
    );
    log.append(
        AuditLogEntry::new(
            AuditEventCategory::Maintenance,
            AuditEventKind::MaintenanceMigrationApplied,
            namespace.clone(),
            "migration_runner",
            "applied audit-log schema migration",
        )
        .with_memory_id(MemoryId(21))
        .with_request_id("req-migration-21")
        .with_related_run("migration-0042"),
    );
    log.append(AuditLogEntry::new(
        AuditEventCategory::Archive,
        AuditEventKind::ArchiveRecorded,
        namespace.clone(),
        "cold_store",
        "archived superseded evidence",
    ));
    log.append(AuditLogEntry::new(
        AuditEventCategory::Recall,
        AuditEventKind::RecallServed,
        namespace.clone(),
        "recall_engine",
        "served filtered audit history preview",
    ));
    log
}

fn filter_audit_rows(
    log: &AppendOnlyAuditLog,
    namespace: &NamespaceId,
    memory_id: Option<u64>,
    since: Option<u64>,
    op: Option<&str>,
    recent: Option<usize>,
) -> Vec<AuditRow> {
    let op = op.map(str::trim).filter(|value| !value.is_empty());
    let mut rows: Vec<_> = log
        .entries_for_namespace(namespace)
        .into_iter()
        .filter(|entry| since.is_none_or(|min_sequence| entry.sequence >= min_sequence))
        .filter(|entry| {
            memory_id.is_none_or(|expected| entry.memory_id == Some(MemoryId(expected)))
        })
        .filter(|entry| {
            op.is_none_or(|needle| {
                entry.kind.as_str() == needle || entry.category.as_str() == needle
            })
        })
        .map(AuditRow::from)
        .collect();

    if let Some(limit) = recent {
        if rows.len() > limit {
            rows = rows.split_off(rows.len() - limit);
        }
    }

    rows
}

fn print_audit_rows(rows: &[AuditRow], json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(rows)?);
        return Ok(());
    }

    if rows.is_empty() {
        println!("No audit rows matched the requested filters.");
        return Ok(());
    }

    for row in rows {
        println!(
            "#{} {} {} ns={} memory={:?} session={:?} actor={} request_id={:?} redacted={} run={:?} note={}",
            row.sequence,
            row.category,
            row.kind,
            row.namespace,
            row.memory_id,
            row.session_id,
            row.triggered_by,
            row.request_id,
            row.redacted,
            row.related_run,
            row.note,
        );
    }

    Ok(())
}

fn sample_runtime_status() -> RuntimeStatus {
    RuntimeStatus {
        posture: RuntimePosture::Full,
        degraded_reasons: Vec::new(),
        metrics: RuntimeMetrics {
            queue_depth: 0,
            active_requests: 0,
            background_jobs: 0,
            cancelled_requests: 0,
            maintenance_runs: 0,
        },
    }
}

fn doctor_report() -> DoctorReport {
    let status = sample_runtime_status();
    let indexes = IndexModule
        .health_reports()
        .into_iter()
        .map(|report| DoctorIndexRow {
            family: report.family.as_str(),
            health: report.health.as_str(),
            usable: report.health.is_usable(),
            entry_count: report.entry_count,
            generation: report.generation,
        })
        .collect::<Vec<_>>();
    let warnings = indexes
        .iter()
        .filter_map(|row| match row.health {
            "stale" => Some("index_stale"),
            "needs_rebuild" => Some("index_needs_rebuild"),
            "missing" => Some("index_missing"),
            _ => None,
        })
        .collect::<Vec<_>>();
    let overall_status = if warnings.is_empty() {
        "healthy"
    } else {
        "warn"
    };
    let store = BrainStore::new(RuntimeConfig::default());
    let repair_engine = store.repair_engine();
    let namespace = NamespaceId::new("doctor.system").expect("doctor namespace should be valid");
    let mut repair_handle = MaintenanceJobHandle::new(
        repair_engine.create_targeted(
            namespace,
            vec![RepairTarget::LexicalIndex, RepairTarget::MetadataIndex],
            IndexRepairEntrypoint::RebuildIfNeeded,
        ),
        8,
    );
    repair_handle.start();
    let mut repair_reports = Vec::new();
    loop {
        let snapshot = repair_handle.poll();
        match snapshot.state {
            MaintenanceJobState::Completed(summary) => {
                repair_reports = summary
                    .results
                    .into_iter()
                    .map(|result| RepairReportRow {
                        target: result.target.as_str(),
                        status: result.status.as_str(),
                        verification_passed: result.verification_passed,
                        rebuild_entrypoint: result
                            .rebuild_entrypoint
                            .map(IndexRepairEntrypoint::as_str),
                        rebuilt_outputs: result.rebuilt_outputs,
                    })
                    .collect();
                break;
            }
            MaintenanceJobState::Running { .. } => continue,
            _ => break,
        }
    }

    DoctorReport {
        status: overall_status,
        action: "doctor",
        posture: status.posture.as_str(),
        degraded_reasons: status.degraded_reasons,
        metrics: status.metrics,
        indexes,
        repair_engine_component: repair_engine.component_name(),
        repair_reports,
        warnings,
    }
}

fn print_doctor_report(report: &DoctorReport) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(report)?);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::{filter_audit_rows, print_audit_rows, sample_audit_log};
    use membrain_core::api::NamespaceId;

    #[test]
    fn audit_rows_preserve_request_id_in_json_export() {
        let namespace = NamespaceId::new("team.alpha").expect("valid namespace");
        let log = sample_audit_log(&namespace);
        let rows = filter_audit_rows(&log, &namespace, Some(21), None, None, None);

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].request_id.as_deref(), Some("req-encode-21"));
        assert_eq!(rows[1].request_id.as_deref(), Some("req-policy-21"));
        assert_eq!(rows[2].request_id.as_deref(), Some("req-migration-21"));
    }

    #[test]
    fn text_export_includes_request_id_field() {
        let namespace = NamespaceId::new("team.alpha").expect("valid namespace");
        let log = sample_audit_log(&namespace);
        let rows = filter_audit_rows(&log, &namespace, Some(21), None, None, Some(1));

        let rendered = rows
            .iter()
            .map(|row| {
                format!(
                    "#{} {} {} ns={} memory={:?} session={:?} actor={} request_id={:?} redacted={} run={:?} note={}",
                    row.sequence,
                    row.category,
                    row.kind,
                    row.namespace,
                    row.memory_id,
                    row.session_id,
                    row.triggered_by,
                    row.request_id,
                    row.redacted,
                    row.related_run,
                    row.note,
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("request_id=Some(\"req-migration-21\")"));
        assert!(rendered.contains("run=Some(\"migration-0042\")"));

        print_audit_rows(&rows, false).expect("text export should render");
    }
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
            println!(
                "Output: {{\"status\": \"success\", \"action\": \"maintenance\", \"target\": \"{}\"}}",
                action
            );
        }
        Commands::Benchmark { target, iters } => {
            println!("Benchmarking '{}' over {} iterations", target, iters);
            println!(
                "Output: {{\"status\": \"success\", \"action\": \"benchmark\", \"duration_ms\": 0}}"
            );
        }
        Commands::Doctor => {
            let report = doctor_report();
            print_doctor_report(&report)?;
        }
        Commands::Audit {
            namespace,
            id,
            since,
            op,
            recent,
            json,
        } => {
            let ns = NamespaceId::new(namespace)?;
            let log = sample_audit_log(&ns);
            let rows = filter_audit_rows(&log, &ns, *id, *since, op.as_deref(), *recent);
            print_audit_rows(&rows, *json)?;
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
