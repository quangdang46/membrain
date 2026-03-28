//! Shared health metrics, subsystem status, and trend aggregation.
//!
//! Reuses bounded machine-readable signals from cache, index, repair, and
//! availability surfaces so CLI, daemon, and MCP can expose one canonical
//! health report.

use crate::api::{AvailabilityPosture, AvailabilitySummary};
use crate::engine::repair::RepairSummary;
use crate::index::{IndexHealth, IndexHealthReport};
use crate::store::cache::{CacheFamily, CacheFamilyMetrics, CacheManager};

/// Stable machine-readable subsystem status for health aggregation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum SubsystemHealthState {
    Healthy,
    Degraded,
    Blocked,
    Unavailable,
}

impl SubsystemHealthState {
    /// Returns the stable machine-readable subsystem status.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Blocked => "blocked",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Stable machine-readable trend direction for one health metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Worsening,
}

impl TrendDirection {
    /// Returns the stable machine-readable trend direction.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Improving => "improving",
            Self::Stable => "stable",
            Self::Worsening => "worsening",
        }
    }
}

/// One machine-readable trend aggregate used by health surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HealthTrend {
    pub subsystem: &'static str,
    pub metric: &'static str,
    pub previous: u64,
    pub current: u64,
    pub direction: TrendDirection,
}

/// One machine-readable metric sample attached to a subsystem health row.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SubsystemMetric {
    pub metric: &'static str,
    pub value: u64,
}

/// One machine-readable trend rollup for a subsystem dashboard card.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SubsystemTrendSummary {
    pub subsystem: &'static str,
    pub state: SubsystemHealthState,
    pub trends: Vec<HealthTrend>,
}

/// One machine-readable feature-availability entry.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FeatureAvailabilityEntry {
    pub feature: String,
    pub posture: AvailabilityPosture,
    pub note: Option<String>,
}

/// Stable alert severity for operator-facing health alerts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum HealthAlertSeverity {
    Info,
    Warning,
    Critical,
}

impl HealthAlertSeverity {
    /// Returns the stable machine-readable alert severity.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }
}

/// Stable dashboard view identifier exposed by health surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardViewId {
    Overview,
    Alerts,
    Subsystems,
    Trends,
    Attention,
    AffectTrajectory,
    DegradedMode,
}

impl DashboardViewId {
    /// Returns the stable machine-readable view identifier.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Alerts => "alerts",
            Self::Subsystems => "subsystems",
            Self::Trends => "trends",
            Self::Attention => "attention",
            Self::AffectTrajectory => "affect_trajectory",
            Self::DegradedMode => "degraded_mode",
        }
    }
}

/// One canonical dashboard view definition.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DashboardView {
    pub view: DashboardViewId,
    pub title: &'static str,
    pub summary: String,
    pub alert_count: usize,
    pub drill_down_targets: Vec<&'static str>,
}

/// Explicit contribution counts that explain one attention hotspot.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AttentionSignalBreakdown {
    pub recall_count: u64,
    pub encode_count: u64,
    pub working_memory_pressure: usize,
    pub promotion_count: u64,
    pub overflow_count: u64,
}

/// One ranked hotspot entry derived from inspectable attention inputs.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AttentionHotspot {
    pub namespace: String,
    pub attention_score: u64,
    pub rank: usize,
    pub status: &'static str,
    pub dominant_signal: &'static str,
    pub heat_bucket: &'static str,
    pub heat_band: u8,
    pub prewarm_trigger: &'static str,
    pub prewarm_action: &'static str,
    pub prewarm_target_family: &'static str,
    pub contributing_signals: AttentionSignalBreakdown,
    pub sample_log: String,
}

/// Shared input row for one namespace-scoped attention aggregate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttentionNamespaceInputs {
    pub namespace: String,
    pub recall_count: u64,
    pub encode_count: u64,
    pub working_memory_pressure: usize,
    pub promotion_count: u64,
    pub overflow_count: u64,
}

/// Canonical attention aggregate shared across health and future heatmap surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AttentionAggregateReport {
    pub total_recall_count: u64,
    pub total_encode_count: u64,
    pub total_promotion_count: u64,
    pub total_overflow_count: u64,
    pub highest_namespace_pressure: usize,
    pub hotspot_count: usize,
    pub max_attention_score: u64,
    pub warming_candidate_count: usize,
    pub hot_candidate_count: usize,
    pub hotspots: Vec<AttentionHotspot>,
}

impl AttentionAggregateReport {
    /// Builds the inspectable hotspot report from explicit namespace inputs.
    pub fn from_inputs(inputs: &[AttentionNamespaceInputs]) -> Self {
        let mut hotspots = inputs
            .iter()
            .map(|row| {
                let attention_score = row.recall_count * 5
                    + row.encode_count * 3
                    + row.promotion_count * 11
                    + row.overflow_count * 13
                    + (row.working_memory_pressure as u64 * 17);
                let dominant_signal = dominant_signal(row);
                let status = hotspot_status(attention_score, row.working_memory_pressure);
                let heat_band = attention_heat_band(attention_score, row.working_memory_pressure);
                let heat_bucket = attention_heat_bucket(heat_band);
                let (prewarm_trigger, prewarm_action, prewarm_target_family) =
                    attention_prewarm_policy(status, dominant_signal);
                AttentionHotspot {
                    namespace: row.namespace.clone(),
                    attention_score,
                    rank: 0,
                    status,
                    dominant_signal,
                    heat_bucket,
                    heat_band,
                    prewarm_trigger,
                    prewarm_action,
                    prewarm_target_family,
                    contributing_signals: AttentionSignalBreakdown {
                        recall_count: row.recall_count,
                        encode_count: row.encode_count,
                        working_memory_pressure: row.working_memory_pressure,
                        promotion_count: row.promotion_count,
                        overflow_count: row.overflow_count,
                    },
                    sample_log: format!(
                        "namespace={} attention_score={} dominant_signal={} recalls={} encodes={} pressure={} promotions={} overflows={}",
                        row.namespace,
                        attention_score,
                        dominant_signal,
                        row.recall_count,
                        row.encode_count,
                        row.working_memory_pressure,
                        row.promotion_count,
                        row.overflow_count
                    ),
                }
            })
            .collect::<Vec<_>>();
        hotspots.sort_by(|left, right| {
            right
                .attention_score
                .cmp(&left.attention_score)
                .then_with(|| left.namespace.cmp(&right.namespace))
        });
        for (index, hotspot) in hotspots.iter_mut().enumerate() {
            hotspot.rank = index + 1;
        }
        let max_attention_score = hotspots
            .iter()
            .map(|hotspot| hotspot.attention_score)
            .max()
            .unwrap_or(0);
        let warming_candidate_count = hotspots
            .iter()
            .filter(|hotspot| hotspot.status == "warming")
            .count();
        let hot_candidate_count = hotspots
            .iter()
            .filter(|hotspot| hotspot.status == "hot")
            .count();
        Self {
            total_recall_count: inputs.iter().map(|row| row.recall_count).sum(),
            total_encode_count: inputs.iter().map(|row| row.encode_count).sum(),
            total_promotion_count: inputs.iter().map(|row| row.promotion_count).sum(),
            total_overflow_count: inputs.iter().map(|row| row.overflow_count).sum(),
            highest_namespace_pressure: inputs
                .iter()
                .map(|row| row.working_memory_pressure)
                .max()
                .unwrap_or(0),
            hotspot_count: hotspots.len(),
            max_attention_score,
            warming_candidate_count,
            hot_candidate_count,
            hotspots,
        }
    }
}

/// One machine-readable operator alert derived from aggregate health state.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HealthAlert {
    pub alert_id: String,
    pub subsystem: &'static str,
    pub severity: HealthAlertSeverity,
    pub summary: String,
    pub reason_codes: Vec<&'static str>,
    pub recommended_runbook: Option<&'static str>,
    pub drill_down_path: &'static str,
}

/// One machine-readable drill-down path for operator follow-up.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HealthDrillDownPath {
    pub path: &'static str,
    pub label: &'static str,
    pub target_surface: &'static str,
    pub target_ref: &'static str,
    pub summary: String,
    pub related_subsystems: Vec<&'static str>,
}

/// One machine-readable subsystem status row.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SubsystemStatus {
    pub subsystem: &'static str,
    pub state: SubsystemHealthState,
    pub detail: String,
    pub metrics: Vec<SubsystemMetric>,
    pub reasons: Vec<&'static str>,
}

/// Canonical degraded-mode status surface shared across operator interfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DegradedStatusSurface {
    pub posture: AvailabilityPosture,
    pub summary: String,
    pub affected_subsystems: Vec<&'static str>,
    pub surviving_query_capabilities: Vec<&'static str>,
    pub surviving_mutation_capabilities: Vec<&'static str>,
    pub degraded_reasons: Vec<&'static str>,
    pub remediation_steps: Vec<&'static str>,
    pub recommended_runbooks: Vec<&'static str>,
}

/// Per-cache-family health row used by aggregate health reports.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CacheFamilyHealth {
    pub family: &'static str,
    pub state: SubsystemHealthState,
    pub hit_count: u64,
    pub miss_count: u64,
    pub bypass_count: u64,
    pub invalidation_count: u64,
    pub stale_warning_count: u64,
}

/// Aggregate cache and prefetch health summary.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CacheHealthReport {
    pub state: SubsystemHealthState,
    pub family_status: Vec<CacheFamilyHealth>,
    pub total_hit_count: u64,
    pub total_miss_count: u64,
    pub total_bypass_count: u64,
    pub total_invalidation_count: u64,
    pub total_stale_warning_count: u64,
    pub prefetch_queue_depth: usize,
    pub prefetch_capacity: usize,
    pub hints_submitted: u64,
    pub hints_consumed: u64,
    pub hints_dropped: u64,
    pub drop_budget_exhausted_count: u64,
    pub drop_intent_changed_count: u64,
    pub drop_scope_changed_count: u64,
    pub drop_disabled_count: u64,
    pub adaptive_prewarm_state: &'static str,
    pub adaptive_prewarm_summary: String,
}

/// Aggregate index health summary derived from managed index reports.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct IndexHealthSummary {
    pub state: SubsystemHealthState,
    pub healthy_count: usize,
    pub stale_count: usize,
    pub needs_rebuild_count: usize,
    pub missing_count: usize,
    pub repair_backlog_total: usize,
    pub reports: Vec<IndexHealthReportEntry>,
}

/// Machine-readable subset of one index health report.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct IndexHealthReportEntry {
    pub family: &'static str,
    pub health: &'static str,
    pub entry_count: usize,
    pub generation: &'static str,
    pub repair_backlog: usize,
    pub item_count_divergence: usize,
}

/// Aggregate repair summary used by health surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RepairHealthReport {
    pub state: SubsystemHealthState,
    pub targets_checked: u32,
    pub healthy: u32,
    pub degraded: u32,
    pub corrupt: u32,
    pub rebuilt: u32,
    pub queue_depth: u64,
    pub degraded_mode: Option<&'static str>,
    pub rollback_trigger: Option<&'static str>,
    pub remediation_steps: Vec<&'static str>,
}

/// Lifecycle counters that make background memory processing observable in health output.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct LifecycleHealthReport {
    pub consolidated_to_cold_count: usize,
    pub reconsolidation_active_count: usize,
    pub forgetting_archive_count: usize,
    pub background_maintenance_runs: usize,
    pub background_maintenance_log: Vec<String>,
}

/// Inputs used to build the shared health report.
#[derive(Debug, Clone, PartialEq)]
pub struct BrainHealthInputs {
    pub hot_memories: usize,
    pub hot_capacity: usize,
    pub cold_memories: usize,
    pub avg_strength: f32,
    pub avg_confidence: f32,
    pub low_confidence_count: usize,
    pub decay_rate: f32,
    pub archive_count: usize,
    pub lifecycle: LifecycleHealthReport,
    pub total_engrams: usize,
    pub avg_cluster_size: f32,
    pub top_engrams: Vec<(String, usize)>,
    pub landmark_count: usize,
    pub unresolved_conflicts: usize,
    pub uncertain_count: usize,
    pub dream_links_total: usize,
    pub last_dream_tick: Option<u64>,
    pub affect_history_rows: usize,
    pub latest_affect_snapshot: Option<(f32, f32)>,
    pub latest_affect_tick: Option<u64>,
    pub attention_namespaces: Vec<AttentionNamespaceInputs>,
    pub total_recalls: u64,
    pub total_encodes: u64,
    pub current_tick: u64,
    pub daemon_uptime_ticks: u64,
    pub index_reports: Vec<IndexHealthReport>,
    pub availability: Option<AvailabilitySummary>,
    pub feature_availability: Vec<FeatureAvailabilityEntry>,
    pub previous_total_recalls: Option<u64>,
    pub previous_total_encodes: Option<u64>,
    pub previous_repair_queue_depth: Option<u64>,
    pub previous_hot_memories: Option<usize>,
    pub previous_low_confidence_count: Option<usize>,
    pub previous_unresolved_conflicts: Option<usize>,
    pub previous_uncertain_count: Option<usize>,
    pub previous_cache_hit_count: Option<u64>,
    pub previous_cache_miss_count: Option<u64>,
    pub previous_cache_bypass_count: Option<u64>,
    pub previous_prefetch_queue_depth: Option<usize>,
    pub previous_prefetch_drop_count: Option<u64>,
    pub previous_index_stale_count: Option<usize>,
    pub previous_index_missing_count: Option<usize>,
    pub previous_index_repair_backlog_total: Option<usize>,
    pub previous_availability_posture: Option<AvailabilityPosture>,
}

/// Canonical machine-readable operator health report.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct BrainHealthReport {
    pub hot_memories: usize,
    pub hot_capacity: usize,
    pub cold_memories: usize,
    pub hot_utilization_pct: f32,
    pub avg_strength: f32,
    pub avg_confidence: f32,
    pub low_confidence_count: usize,
    pub decay_rate: f32,
    pub archive_count: usize,
    pub lifecycle: LifecycleHealthReport,
    pub total_engrams: usize,
    pub avg_cluster_size: f32,
    pub top_engrams: Vec<(String, usize)>,
    pub landmark_count: usize,
    pub unresolved_conflicts: usize,
    pub uncertain_count: usize,
    pub dream_links_total: usize,
    pub last_dream_tick: Option<u64>,
    pub affect_history_rows: usize,
    pub latest_affect_snapshot: Option<(f32, f32)>,
    pub latest_affect_tick: Option<u64>,
    pub attention: AttentionAggregateReport,
    pub repair_queue_depth: Option<u64>,
    pub backpressure_state: Option<&'static str>,
    pub feature_availability: Vec<FeatureAvailabilityEntry>,
    pub availability_posture: Option<AvailabilityPosture>,
    pub availability_notes: Option<String>,
    pub degraded_status: Option<DegradedStatusSurface>,
    pub cache: CacheHealthReport,
    pub indexes: IndexHealthSummary,
    pub repair: Option<RepairHealthReport>,
    pub subsystem_status: Vec<SubsystemStatus>,
    pub dashboard_views: Vec<DashboardView>,
    pub alerts: Vec<HealthAlert>,
    pub drill_down_paths: Vec<HealthDrillDownPath>,
    pub trend_summary: Vec<SubsystemTrendSummary>,
    pub trends: Vec<HealthTrend>,
    pub total_recalls: u64,
    pub total_encodes: u64,
    pub current_tick: u64,
    pub daemon_uptime_ticks: u64,
}

impl BrainHealthReport {
    /// Builds a canonical health report from bounded subsystem snapshots.
    pub fn from_inputs(
        inputs: BrainHealthInputs,
        cache: &CacheManager,
        repair_summary: Option<&RepairSummary>,
    ) -> Self {
        let cache = CacheHealthReport::from_cache_manager(cache);
        let indexes = IndexHealthSummary::from_reports(inputs.index_reports);
        let repair = repair_summary.map(RepairHealthReport::from_summary);
        let attention = AttentionAggregateReport::from_inputs(&inputs.attention_namespaces);
        let repair_queue_depth = repair.as_ref().map(|report| report.queue_depth);
        let availability_posture = inputs.availability.as_ref().map(|a| a.posture);
        let availability_notes = inputs.availability.as_ref().map(availability_note);
        let degraded_status = degraded_status_surface(
            inputs.availability.as_ref(),
            &cache,
            &indexes,
            repair.as_ref(),
        );
        let backpressure_state = backpressure_state(cache.prefetch_queue_depth, repair_queue_depth);
        let hot_utilization_pct = percent(inputs.hot_memories, inputs.hot_capacity);

        let mut subsystem_status = vec![SubsystemStatus {
            subsystem: "memory",
            state: memory_state(
                inputs.low_confidence_count,
                inputs.unresolved_conflicts,
                inputs.uncertain_count,
            ),
            detail: format!(
                "hot_memories={} hot_utilization_pct={} low_confidence={} conflicts={} uncertain={} hotspots={} highest_pressure={} affect_rows={} latest_affect_tick={} consolidated_to_cold={} reconsolidation_active={} forgetting_archived={} maintenance_runs={}",
                inputs.hot_memories,
                hot_utilization_pct.round() as u64,
                inputs.low_confidence_count,
                inputs.unresolved_conflicts,
                inputs.uncertain_count,
                attention.hotspot_count,
                attention.highest_namespace_pressure,
                inputs.affect_history_rows,
                inputs.latest_affect_tick.unwrap_or(0),
                inputs.lifecycle.consolidated_to_cold_count,
                inputs.lifecycle.reconsolidation_active_count,
                inputs.lifecycle.forgetting_archive_count,
                inputs.lifecycle.background_maintenance_runs,
            ),
            metrics: vec![
                SubsystemMetric {
                    metric: "hot_memories",
                    value: inputs.hot_memories as u64,
                },
                SubsystemMetric {
                    metric: "hot_capacity",
                    value: inputs.hot_capacity as u64,
                },
                SubsystemMetric {
                    metric: "hot_utilization_pct",
                    value: hot_utilization_pct.round() as u64,
                },
                SubsystemMetric {
                    metric: "low_confidence_count",
                    value: inputs.low_confidence_count as u64,
                },
                SubsystemMetric {
                    metric: "unresolved_conflicts",
                    value: inputs.unresolved_conflicts as u64,
                },
                SubsystemMetric {
                    metric: "uncertain_count",
                    value: inputs.uncertain_count as u64,
                },
                SubsystemMetric {
                    metric: "attention_hotspot_count",
                    value: attention.hotspot_count as u64,
                },
                SubsystemMetric {
                    metric: "attention_highest_namespace_pressure",
                    value: attention.highest_namespace_pressure as u64,
                },
                SubsystemMetric {
                    metric: "affect_history_rows",
                    value: inputs.affect_history_rows as u64,
                },
                SubsystemMetric {
                    metric: "latest_affect_tick",
                    value: inputs.latest_affect_tick.unwrap_or(0),
                },
                SubsystemMetric {
                    metric: "consolidated_to_cold_count",
                    value: inputs.lifecycle.consolidated_to_cold_count as u64,
                },
                SubsystemMetric {
                    metric: "reconsolidation_active_count",
                    value: inputs.lifecycle.reconsolidation_active_count as u64,
                },
                SubsystemMetric {
                    metric: "forgetting_archive_count",
                    value: inputs.lifecycle.forgetting_archive_count as u64,
                },
                SubsystemMetric {
                    metric: "background_maintenance_runs",
                    value: inputs.lifecycle.background_maintenance_runs as u64,
                },
            ],
            reasons: memory_reasons(
                inputs.low_confidence_count,
                inputs.unresolved_conflicts,
                inputs.uncertain_count,
            ),
        }];
        subsystem_status.push(SubsystemStatus {
            subsystem: "lifecycle",
            state: if inputs.lifecycle.background_maintenance_runs == 0 {
                SubsystemHealthState::Degraded
            } else if inputs.lifecycle.reconsolidation_active_count > 0 {
                SubsystemHealthState::Degraded
            } else {
                SubsystemHealthState::Healthy
            },
            detail: format!(
                "consolidated_to_cold={} reconsolidation_active={} forgetting_archived={} maintenance_runs={} background_log={}",
                inputs.lifecycle.consolidated_to_cold_count,
                inputs.lifecycle.reconsolidation_active_count,
                inputs.lifecycle.forgetting_archive_count,
                inputs.lifecycle.background_maintenance_runs,
                join_owned_or_none(&inputs.lifecycle.background_maintenance_log),
            ),
            metrics: vec![
                SubsystemMetric {
                    metric: "consolidated_to_cold_count",
                    value: inputs.lifecycle.consolidated_to_cold_count as u64,
                },
                SubsystemMetric {
                    metric: "reconsolidation_active_count",
                    value: inputs.lifecycle.reconsolidation_active_count as u64,
                },
                SubsystemMetric {
                    metric: "forgetting_archive_count",
                    value: inputs.lifecycle.forgetting_archive_count as u64,
                },
                SubsystemMetric {
                    metric: "background_maintenance_runs",
                    value: inputs.lifecycle.background_maintenance_runs as u64,
                },
            ],
            reasons: if inputs.lifecycle.background_maintenance_runs == 0 {
                vec!["no_background_lifecycle_activity_recorded"]
            } else if inputs.lifecycle.reconsolidation_active_count > 0 {
                vec!["reconsolidation_window_open", "background_lifecycle_activity_recorded"]
            } else {
                vec!["background_lifecycle_activity_recorded"]
            },
        });
        subsystem_status.push(SubsystemStatus {
            subsystem: "cache",
            state: cache.state,
            detail: format!(
                "hits={} misses={} bypasses={} stale_warnings={} queue_depth={} hints_dropped={}",
                cache.total_hit_count,
                cache.total_miss_count,
                cache.total_bypass_count,
                cache.total_stale_warning_count,
                cache.prefetch_queue_depth,
                cache.hints_dropped
            ),
            metrics: vec![
                SubsystemMetric {
                    metric: "cache_hit_count",
                    value: cache.total_hit_count,
                },
                SubsystemMetric {
                    metric: "cache_miss_count",
                    value: cache.total_miss_count,
                },
                SubsystemMetric {
                    metric: "cache_bypass_count",
                    value: cache.total_bypass_count,
                },
                SubsystemMetric {
                    metric: "cache_stale_warning_count",
                    value: cache.total_stale_warning_count,
                },
                SubsystemMetric {
                    metric: "prefetch_queue_depth",
                    value: cache.prefetch_queue_depth as u64,
                },
                SubsystemMetric {
                    metric: "prefetch_drop_count",
                    value: cache.hints_dropped,
                },
            ],
            reasons: cache_reasons(&cache),
        });
        subsystem_status.push(SubsystemStatus {
            subsystem: "index",
            state: indexes.state,
            detail: format!(
                "healthy={} stale={} needs_rebuild={} missing={} backlog={}",
                indexes.healthy_count,
                indexes.stale_count,
                indexes.needs_rebuild_count,
                indexes.missing_count,
                indexes.repair_backlog_total
            ),
            metrics: vec![
                SubsystemMetric {
                    metric: "healthy_count",
                    value: indexes.healthy_count as u64,
                },
                SubsystemMetric {
                    metric: "stale_count",
                    value: indexes.stale_count as u64,
                },
                SubsystemMetric {
                    metric: "needs_rebuild_count",
                    value: indexes.needs_rebuild_count as u64,
                },
                SubsystemMetric {
                    metric: "missing_count",
                    value: indexes.missing_count as u64,
                },
                SubsystemMetric {
                    metric: "repair_backlog_total",
                    value: indexes.repair_backlog_total as u64,
                },
            ],
            reasons: index_reasons(&indexes),
        });
        if let Some(repair) = &repair {
            subsystem_status.push(SubsystemStatus {
                subsystem: "repair",
                state: repair.state,
                detail: format!(
                    "checked={} degraded={} corrupt={} rebuilt={} queue_depth={} degraded_mode={} rollback_trigger={} remediation_steps={}",
                    repair.targets_checked,
                    repair.degraded,
                    repair.corrupt,
                    repair.rebuilt,
                    repair.queue_depth,
                    repair.degraded_mode.unwrap_or("none"),
                    repair.rollback_trigger.unwrap_or("none"),
                    if repair.remediation_steps.is_empty() {
                        "none".to_string()
                    } else {
                        repair.remediation_steps.join(",")
                    }
                ),
                metrics: vec![
                    SubsystemMetric {
                        metric: "targets_checked",
                        value: repair.targets_checked as u64,
                    },
                    SubsystemMetric {
                        metric: "degraded_count",
                        value: repair.degraded as u64,
                    },
                    SubsystemMetric {
                        metric: "corrupt_count",
                        value: repair.corrupt as u64,
                    },
                    SubsystemMetric {
                        metric: "rebuilt_count",
                        value: repair.rebuilt as u64,
                    },
                    SubsystemMetric {
                        metric: "queue_depth",
                        value: repair.queue_depth,
                    },
                ],
                reasons: repair_reasons(repair),
            });
        }
        if let Some(availability) = &inputs.availability {
            subsystem_status.push(SubsystemStatus {
                subsystem: "availability",
                state: availability_state(availability.posture),
                detail: format!(
                    "posture={} reasons={} recovery={}",
                    availability.posture.as_str(),
                    join_or_none(&availability.reason_names()),
                    join_or_none(&availability.recovery_condition_names())
                ),
                metrics: vec![SubsystemMetric {
                    metric: "availability_posture",
                    value: availability_rank(availability.posture) as u64,
                }],
                reasons: availability.reason_names(),
            });
        }

        let mut trends = Vec::new();
        push_trend(
            &mut trends,
            "memory",
            "hot_memories",
            inputs.previous_hot_memories.map(|value| value as u64),
            Some(inputs.hot_memories as u64),
            higher_is_better,
        );
        push_trend(
            &mut trends,
            "memory",
            "low_confidence_count",
            inputs
                .previous_low_confidence_count
                .map(|value| value as u64),
            Some(inputs.low_confidence_count as u64),
            lower_is_better,
        );
        push_trend(
            &mut trends,
            "memory",
            "unresolved_conflicts",
            inputs
                .previous_unresolved_conflicts
                .map(|value| value as u64),
            Some(inputs.unresolved_conflicts as u64),
            lower_is_better,
        );
        push_trend(
            &mut trends,
            "memory",
            "uncertain_count",
            inputs.previous_uncertain_count.map(|value| value as u64),
            Some(inputs.uncertain_count as u64),
            lower_is_better,
        );
        push_trend(
            &mut trends,
            "lifecycle",
            "consolidated_to_cold_count",
            Some(0),
            Some(inputs.lifecycle.consolidated_to_cold_count as u64),
            higher_is_better,
        );
        push_trend(
            &mut trends,
            "lifecycle",
            "reconsolidation_active_count",
            Some(0),
            Some(inputs.lifecycle.reconsolidation_active_count as u64),
            lower_is_better,
        );
        push_trend(
            &mut trends,
            "lifecycle",
            "forgetting_archive_count",
            Some(0),
            Some(inputs.lifecycle.forgetting_archive_count as u64),
            higher_is_better,
        );
        push_trend(
            &mut trends,
            "lifecycle",
            "background_maintenance_runs",
            Some(0),
            Some(inputs.lifecycle.background_maintenance_runs as u64),
            higher_is_better,
        );
        push_trend(
            &mut trends,
            "activity",
            "total_recalls",
            inputs.previous_total_recalls,
            Some(inputs.total_recalls),
            higher_is_better,
        );
        push_trend(
            &mut trends,
            "activity",
            "total_encodes",
            inputs.previous_total_encodes,
            Some(inputs.total_encodes),
            higher_is_better,
        );
        push_trend(
            &mut trends,
            "cache",
            "cache_hit_count",
            inputs.previous_cache_hit_count,
            Some(cache.total_hit_count),
            higher_is_better,
        );
        push_trend(
            &mut trends,
            "cache",
            "cache_miss_count",
            inputs.previous_cache_miss_count,
            Some(cache.total_miss_count),
            lower_is_better,
        );
        push_trend(
            &mut trends,
            "cache",
            "cache_bypass_count",
            inputs.previous_cache_bypass_count,
            Some(cache.total_bypass_count),
            lower_is_better,
        );
        push_trend(
            &mut trends,
            "cache",
            "prefetch_queue_depth",
            inputs
                .previous_prefetch_queue_depth
                .map(|value| value as u64),
            Some(cache.prefetch_queue_depth as u64),
            lower_is_better,
        );
        push_trend(
            &mut trends,
            "cache",
            "prefetch_drop_count",
            inputs.previous_prefetch_drop_count,
            Some(cache.hints_dropped),
            lower_is_better,
        );
        push_trend(
            &mut trends,
            "index",
            "stale_count",
            inputs.previous_index_stale_count.map(|value| value as u64),
            Some(indexes.stale_count as u64),
            lower_is_better,
        );
        push_trend(
            &mut trends,
            "index",
            "missing_count",
            inputs
                .previous_index_missing_count
                .map(|value| value as u64),
            Some(indexes.missing_count as u64),
            lower_is_better,
        );
        push_trend(
            &mut trends,
            "index",
            "repair_backlog_total",
            inputs
                .previous_index_repair_backlog_total
                .map(|value| value as u64),
            Some(indexes.repair_backlog_total as u64),
            lower_is_better,
        );
        push_trend(
            &mut trends,
            "repair",
            "repair_queue_depth",
            inputs.previous_repair_queue_depth,
            repair_queue_depth,
            lower_is_better,
        );
        push_trend(
            &mut trends,
            "availability",
            "availability_posture",
            inputs
                .previous_availability_posture
                .map(|posture| availability_rank(posture) as u64),
            availability_posture.map(|posture| availability_rank(posture) as u64),
            lower_is_better,
        );
        let alerts = derive_health_alerts(&subsystem_status, degraded_status.as_ref());
        let dashboard_views = derive_dashboard_views(
            &subsystem_status,
            &alerts,
            &trends,
            &attention,
            &cache,
            inputs.affect_history_rows,
            inputs.latest_affect_snapshot,
            inputs.latest_affect_tick,
            degraded_status.as_ref(),
        );
        let drill_down_paths =
            derive_drill_down_paths(&subsystem_status, &alerts, degraded_status.as_ref());
        let trend_summary = summarize_trends(&trends, &subsystem_status);

        Self {
            hot_memories: inputs.hot_memories,
            hot_capacity: inputs.hot_capacity,
            cold_memories: inputs.cold_memories,
            hot_utilization_pct,
            avg_strength: inputs.avg_strength,
            avg_confidence: inputs.avg_confidence,
            low_confidence_count: inputs.low_confidence_count,
            decay_rate: inputs.decay_rate,
            archive_count: inputs.archive_count,
            lifecycle: inputs.lifecycle,
            total_engrams: inputs.total_engrams,
            avg_cluster_size: inputs.avg_cluster_size,
            top_engrams: inputs.top_engrams,
            landmark_count: inputs.landmark_count,
            unresolved_conflicts: inputs.unresolved_conflicts,
            uncertain_count: inputs.uncertain_count,
            dream_links_total: inputs.dream_links_total,
            last_dream_tick: inputs.last_dream_tick,
            attention,
            repair_queue_depth,
            backpressure_state,
            feature_availability: inputs.feature_availability,
            availability_posture,
            availability_notes,
            degraded_status,
            cache,
            indexes,
            repair,
            subsystem_status,
            dashboard_views,
            alerts,
            drill_down_paths,
            trend_summary,
            trends,
            affect_history_rows: inputs.affect_history_rows,
            latest_affect_snapshot: inputs.latest_affect_snapshot,
            latest_affect_tick: inputs.latest_affect_tick,
            total_recalls: inputs.total_recalls,
            total_encodes: inputs.total_encodes,
            current_tick: inputs.current_tick,
            daemon_uptime_ticks: inputs.daemon_uptime_ticks,
        }
    }
}

impl CacheHealthReport {
    /// Aggregates cache and prefetch health from the canonical cache manager.
    pub fn from_cache_manager(cache: &CacheManager) -> Self {
        let family_status = vec![
            cache_family_health(
                CacheFamily::Tier1Item,
                &cache.tier1_item.metrics,
                cache.tier1_item.is_disabled(),
            ),
            cache_family_health(
                CacheFamily::NegativeCache,
                &cache.negative.metrics,
                cache.negative.is_disabled(),
            ),
            cache_family_health(
                CacheFamily::ResultCache,
                &cache.result.metrics,
                cache.result.is_disabled(),
            ),
            cache_family_health(
                CacheFamily::EntityNeighborhood,
                &cache.entity_neighborhood.metrics,
                cache.entity_neighborhood.is_disabled(),
            ),
            cache_family_health(
                CacheFamily::SummaryCache,
                &cache.summary.metrics,
                cache.summary.is_disabled(),
            ),
            cache_family_health(
                CacheFamily::AnnProbeCache,
                &cache.ann_probe.metrics,
                cache.ann_probe.is_disabled(),
            ),
            prefetch_family_health(cache),
            cache_family_health(
                CacheFamily::SessionWarmup,
                &cache.session_warmup.metrics,
                cache.session_warmup.is_disabled(),
            ),
            cache_family_health(
                CacheFamily::GoalConditioned,
                &cache.goal_conditioned.metrics,
                cache.goal_conditioned.is_disabled(),
            ),
            cache_family_health(
                CacheFamily::ColdStartMitigation,
                &cache.cold_start.metrics,
                cache.cold_start.is_disabled(),
            ),
        ];
        let total_hit_count = family_status.iter().map(|item| item.hit_count).sum();
        let total_miss_count = family_status.iter().map(|item| item.miss_count).sum();
        let total_bypass_count = family_status.iter().map(|item| item.bypass_count).sum();
        let total_invalidation_count = family_status
            .iter()
            .map(|item| item.invalidation_count)
            .sum();
        let total_stale_warning_count = family_status
            .iter()
            .map(|item| item.stale_warning_count)
            .sum();
        let state = if family_status
            .iter()
            .any(|item| item.state == SubsystemHealthState::Unavailable)
        {
            SubsystemHealthState::Unavailable
        } else if total_bypass_count > 0 || total_stale_warning_count > 0 {
            SubsystemHealthState::Degraded
        } else {
            SubsystemHealthState::Healthy
        };
        let prefetch_queue_depth = cache.prefetch.queue_depth();
        let prefetch_capacity = cache.prefetch.capacity();
        let adaptive_prewarm_state = adaptive_prewarm_state(
            cache.prefetch.is_enabled(),
            prefetch_queue_depth,
            prefetch_capacity,
            cache.prefetch.metrics.hints_dropped,
        );
        let adaptive_prewarm_summary = adaptive_prewarm_summary(
            adaptive_prewarm_state,
            prefetch_queue_depth,
            prefetch_capacity,
            &cache.prefetch.metrics,
        );
        Self {
            state,
            family_status,
            total_hit_count,
            total_miss_count,
            total_bypass_count,
            total_invalidation_count,
            total_stale_warning_count,
            prefetch_queue_depth,
            prefetch_capacity,
            hints_submitted: cache.prefetch.metrics.hints_submitted,
            hints_consumed: cache.prefetch.metrics.hints_consumed,
            hints_dropped: cache.prefetch.metrics.hints_dropped,
            drop_budget_exhausted_count: cache.prefetch.metrics.dropped_budget_exhausted,
            drop_intent_changed_count: cache.prefetch.metrics.dropped_intent_changed,
            drop_scope_changed_count: cache.prefetch.metrics.dropped_scope_changed,
            drop_disabled_count: cache.prefetch.metrics.dropped_disabled,
            adaptive_prewarm_state,
            adaptive_prewarm_summary,
        }
    }
}

impl IndexHealthSummary {
    /// Aggregates managed index health reports into one machine-readable summary.
    pub fn from_reports(reports: Vec<IndexHealthReport>) -> Self {
        let healthy_count = reports
            .iter()
            .filter(|report| report.health == IndexHealth::Healthy)
            .count();
        let stale_count = reports
            .iter()
            .filter(|report| report.health == IndexHealth::Stale)
            .count();
        let needs_rebuild_count = reports
            .iter()
            .filter(|report| report.health == IndexHealth::NeedsRebuild)
            .count();
        let missing_count = reports
            .iter()
            .filter(|report| report.health == IndexHealth::Missing)
            .count();
        let repair_backlog_total = reports.iter().map(|report| report.repair_backlog).sum();
        let state = if missing_count > 0 {
            SubsystemHealthState::Unavailable
        } else if needs_rebuild_count > 0 || stale_count > 0 || repair_backlog_total > 0 {
            SubsystemHealthState::Degraded
        } else {
            SubsystemHealthState::Healthy
        };
        let reports = reports
            .into_iter()
            .map(|report| IndexHealthReportEntry {
                family: report.family.as_str(),
                health: report.health.as_str(),
                entry_count: report.entry_count,
                generation: report.generation,
                repair_backlog: report.repair_backlog,
                item_count_divergence: report.item_count_divergence,
            })
            .collect();
        Self {
            state,
            healthy_count,
            stale_count,
            needs_rebuild_count,
            missing_count,
            repair_backlog_total,
            reports,
        }
    }
}

impl RepairHealthReport {
    /// Aggregates operator-visible repair state into health-facing counters.
    pub fn from_summary(summary: &RepairSummary) -> Self {
        let queue_depth = summary.queue_report.queue_depth_after as u64;
        let state = if summary.corrupt > 0 {
            SubsystemHealthState::Blocked
        } else if summary.degraded > 0 || queue_depth > 0 {
            SubsystemHealthState::Degraded
        } else {
            SubsystemHealthState::Healthy
        };
        Self {
            state,
            targets_checked: summary.targets_checked,
            healthy: summary.healthy,
            degraded: summary.degraded,
            corrupt: summary.corrupt,
            rebuilt: summary.rebuilt,
            queue_depth,
            degraded_mode: summary.degraded_mode.map(|mode| mode.as_str()),
            rollback_trigger: summary.rollback_trigger.map(|trigger| trigger.as_str()),
            remediation_steps: summary
                .operator_reports
                .iter()
                .find(|report| report.degraded_mode.is_some() || report.rollback_trigger.is_some())
                .map(|report| {
                    report
                        .remediation_steps
                        .iter()
                        .map(|step| step.as_str())
                        .collect()
                })
                .unwrap_or_else(|| {
                    let mut steps = vec!["check_health"];
                    if summary.rollback_trigger.is_some() {
                        steps.push("rollback_recent_change");
                    }
                    if summary.degraded_mode.is_some() {
                        steps.push("run_repair");
                        steps.push("inspect_state");
                    }
                    steps
                }),
        }
    }
}

fn cache_family_health(
    family: CacheFamily,
    metrics: &CacheFamilyMetrics,
    disabled: bool,
) -> CacheFamilyHealth {
    let state = if disabled {
        SubsystemHealthState::Unavailable
    } else if metrics.bypass_count > 0 || metrics.stale_warning_count > 0 {
        SubsystemHealthState::Degraded
    } else {
        SubsystemHealthState::Healthy
    };
    CacheFamilyHealth {
        family: family.as_str(),
        state,
        hit_count: metrics.hit_count,
        miss_count: metrics.miss_count,
        bypass_count: metrics.bypass_count,
        invalidation_count: metrics.invalidation_count,
        stale_warning_count: metrics.stale_warning_count,
    }
}

fn prefetch_family_health(cache: &CacheManager) -> CacheFamilyHealth {
    let metrics = &cache.prefetch.metrics;
    let state = if !cache.prefetch.is_enabled() {
        SubsystemHealthState::Unavailable
    } else if metrics.hints_dropped > 0 {
        SubsystemHealthState::Degraded
    } else {
        SubsystemHealthState::Healthy
    };
    CacheFamilyHealth {
        family: CacheFamily::PrefetchHints.as_str(),
        state,
        hit_count: metrics.hints_consumed,
        miss_count: 0,
        bypass_count: metrics.hints_dropped,
        invalidation_count: 0,
        stale_warning_count: 0,
    }
}

fn availability_note(availability: &AvailabilitySummary) -> String {
    let reasons = availability.reason_names();
    let recovery = availability.recovery_condition_names();
    if reasons.is_empty() && recovery.is_empty() {
        availability.posture.as_str().to_string()
    } else {
        format!(
            "posture={} reasons={} recovery={}",
            availability.posture.as_str(),
            if reasons.is_empty() {
                "none".to_string()
            } else {
                reasons.join(",")
            },
            if recovery.is_empty() {
                "none".to_string()
            } else {
                recovery.join(",")
            }
        )
    }
}

fn memory_state(
    low_confidence_count: usize,
    unresolved_conflicts: usize,
    uncertain_count: usize,
) -> SubsystemHealthState {
    if unresolved_conflicts > 0 {
        SubsystemHealthState::Blocked
    } else if low_confidence_count > 0 || uncertain_count > 0 {
        SubsystemHealthState::Degraded
    } else {
        SubsystemHealthState::Healthy
    }
}

fn memory_reasons(
    low_confidence_count: usize,
    unresolved_conflicts: usize,
    uncertain_count: usize,
) -> Vec<&'static str> {
    let mut reasons = Vec::new();
    if low_confidence_count > 0 {
        reasons.push("low_confidence_present");
    }
    if unresolved_conflicts > 0 {
        reasons.push("conflicts_unresolved");
    }
    if uncertain_count > 0 {
        reasons.push("uncertain_memories_present");
    }
    reasons
}

fn cache_reasons(cache: &CacheHealthReport) -> Vec<&'static str> {
    let mut reasons = Vec::new();
    if cache
        .family_status
        .iter()
        .any(|family| family.state == SubsystemHealthState::Unavailable)
    {
        reasons.push("cache_family_unavailable");
    }
    if cache.total_bypass_count > 0 {
        reasons.push("cache_bypass_detected");
    }
    if cache.total_stale_warning_count > 0 {
        reasons.push("cache_stale_warning_detected");
    }
    if cache.hints_dropped > 0 {
        reasons.push("prefetch_drop_detected");
    }
    reasons
}

fn index_reasons(indexes: &IndexHealthSummary) -> Vec<&'static str> {
    let mut reasons = Vec::new();
    if indexes.stale_count > 0 {
        reasons.push("stale_index_present");
    }
    if indexes.needs_rebuild_count > 0 {
        reasons.push("index_rebuild_required");
    }
    if indexes.missing_count > 0 {
        reasons.push("index_missing");
    }
    if indexes.repair_backlog_total > 0 {
        reasons.push("index_repair_backlog");
    }
    reasons
}

fn repair_reasons(repair: &RepairHealthReport) -> Vec<&'static str> {
    let mut reasons = Vec::new();
    if repair.degraded > 0 {
        reasons.push("repair_degraded_targets_present");
    }
    if repair.corrupt > 0 {
        reasons.push("repair_corrupt_targets_present");
    }
    if repair.queue_depth > 0 {
        reasons.push("repair_queue_backlog");
    }
    if repair.rollback_trigger.is_some() {
        reasons.push("repair_rollback_triggered");
    }
    reasons
}

fn availability_state(posture: AvailabilityPosture) -> SubsystemHealthState {
    match posture {
        AvailabilityPosture::Full => SubsystemHealthState::Healthy,
        AvailabilityPosture::Degraded | AvailabilityPosture::ReadOnly => {
            SubsystemHealthState::Degraded
        }
        AvailabilityPosture::Offline => SubsystemHealthState::Unavailable,
    }
}

fn availability_rank(posture: AvailabilityPosture) -> u8 {
    match posture {
        AvailabilityPosture::Full => 0,
        AvailabilityPosture::Degraded => 1,
        AvailabilityPosture::ReadOnly => 2,
        AvailabilityPosture::Offline => 3,
    }
}

fn push_trend(
    trends: &mut Vec<HealthTrend>,
    subsystem: &'static str,
    metric: &'static str,
    previous: Option<u64>,
    current: Option<u64>,
    comparator: fn(u64, u64) -> TrendDirection,
) {
    if let Some((previous, current)) = previous.zip(current) {
        trends.push(HealthTrend {
            subsystem,
            metric,
            previous,
            current,
            direction: comparator(previous, current),
        });
    }
}

fn summarize_trends(
    trends: &[HealthTrend],
    subsystem_status: &[SubsystemStatus],
) -> Vec<SubsystemTrendSummary> {
    subsystem_status
        .iter()
        .map(|status| SubsystemTrendSummary {
            subsystem: status.subsystem,
            state: status.state,
            trends: trends
                .iter()
                .filter(|trend| trend.subsystem == status.subsystem)
                .cloned()
                .collect(),
        })
        .collect()
}

fn derive_health_alerts(
    subsystem_status: &[SubsystemStatus],
    degraded_status: Option<&DegradedStatusSurface>,
) -> Vec<HealthAlert> {
    let mut alerts = Vec::new();
    for status in subsystem_status {
        if status.state == SubsystemHealthState::Healthy {
            continue;
        }
        let severity = match status.state {
            SubsystemHealthState::Blocked | SubsystemHealthState::Unavailable => {
                HealthAlertSeverity::Critical
            }
            SubsystemHealthState::Degraded => HealthAlertSeverity::Warning,
            SubsystemHealthState::Healthy => HealthAlertSeverity::Info,
        };
        let recommended_runbook =
            recommended_runbook_for_subsystem(status.subsystem, degraded_status);
        alerts.push(HealthAlert {
            alert_id: format!("{}-{}", status.subsystem, severity.as_str()),
            subsystem: status.subsystem,
            severity,
            summary: format!(
                "{} [{}] {}",
                status.subsystem,
                status.state.as_str(),
                status.detail
            ),
            reason_codes: status.reasons.clone(),
            recommended_runbook,
            drill_down_path: drill_down_path_for_subsystem(status.subsystem),
        });
    }
    alerts
}

fn derive_dashboard_views(
    subsystem_status: &[SubsystemStatus],
    alerts: &[HealthAlert],
    trends: &[HealthTrend],
    attention: &AttentionAggregateReport,
    cache: &CacheHealthReport,
    affect_history_rows: usize,
    latest_affect_snapshot: Option<(f32, f32)>,
    latest_affect_tick: Option<u64>,
    degraded_status: Option<&DegradedStatusSurface>,
) -> Vec<DashboardView> {
    let degraded_count = subsystem_status
        .iter()
        .filter(|status| status.state != SubsystemHealthState::Healthy)
        .count();
    let worsening_trends = trends
        .iter()
        .filter(|trend| trend.direction == TrendDirection::Worsening)
        .count();
    let mut views = vec![
        DashboardView {
            view: DashboardViewId::Overview,
            title: "Overview",
            summary: format!(
                "subsystems={} non_healthy={} alerts={}",
                subsystem_status.len(),
                degraded_count,
                alerts.len()
            ),
            alert_count: alerts.len(),
            drill_down_targets: vec!["/health/subsystems", "/health/alerts"],
        },
        DashboardView {
            view: DashboardViewId::Alerts,
            title: "Alerts",
            summary: format!(
                "active_alerts={} critical_alerts={}",
                alerts.len(),
                alerts
                    .iter()
                    .filter(|alert| alert.severity == HealthAlertSeverity::Critical)
                    .count()
            ),
            alert_count: alerts.len(),
            drill_down_targets: vec!["/health/alerts", "/doctor", "/audit"],
        },
        DashboardView {
            view: DashboardViewId::Subsystems,
            title: "Subsystems",
            summary: format!(
                "tracked_subsystems={} non_healthy={}",
                subsystem_status.len(),
                degraded_count
            ),
            alert_count: alerts.len(),
            drill_down_targets: vec!["/health/subsystems", "/inspect"],
        },
        DashboardView {
            view: DashboardViewId::Trends,
            title: "Trends",
            summary: format!(
                "tracked_trends={} worsening={}",
                trends.len(),
                worsening_trends
            ),
            alert_count: worsening_trends,
            drill_down_targets: vec!["/health/trends", "/why"],
        },
        DashboardView {
            view: DashboardViewId::Attention,
            title: "Attention heatmap",
            summary: format!(
                "hotspots={} hot={} warming={} max_score={} prewarm={} queue={}/{}",
                attention.hotspot_count,
                attention.hot_candidate_count,
                attention.warming_candidate_count,
                attention.max_attention_score,
                cache.adaptive_prewarm_state,
                cache.prefetch_queue_depth,
                cache.prefetch_capacity,
            ),
            alert_count: attention.hot_candidate_count,
            drill_down_targets: vec!["/health/attention", "/health/subsystems/cache"],
        },
        DashboardView {
            view: DashboardViewId::AffectTrajectory,
            title: "Affect trajectory",
            summary: match latest_affect_snapshot {
                Some((valence, arousal)) => format!(
                    "rows={} latest_tick={} current_mood=({:.2},{:.2}) history=/mood_history",
                    affect_history_rows,
                    latest_affect_tick.unwrap_or(0),
                    valence,
                    arousal,
                ),
                None => format!(
                    "rows={} latest_tick={} current_mood=unavailable history=/mood_history",
                    affect_history_rows,
                    latest_affect_tick.unwrap_or(0),
                ),
            },
            alert_count: 0,
            drill_down_targets: vec!["/mood_history", "/health/subsystems/memory"],
        },
    ];
    if let Some(degraded) = degraded_status {
        views.push(DashboardView {
            view: DashboardViewId::DegradedMode,
            title: "Degraded mode",
            summary: degraded.summary.clone(),
            alert_count: alerts.len(),
            drill_down_targets: vec!["/health/degraded", "/doctor", "/audit"],
        });
    }
    views
}

fn derive_drill_down_paths(
    subsystem_status: &[SubsystemStatus],
    alerts: &[HealthAlert],
    degraded_status: Option<&DegradedStatusSurface>,
) -> Vec<HealthDrillDownPath> {
    let mut paths = subsystem_status
        .iter()
        .map(|status| HealthDrillDownPath {
            path: drill_down_path_for_subsystem(status.subsystem),
            label: drill_down_label_for_subsystem(status.subsystem),
            target_surface: drill_down_surface_for_subsystem(status.subsystem),
            target_ref: status.subsystem,
            summary: status.detail.clone(),
            related_subsystems: vec![status.subsystem],
        })
        .collect::<Vec<_>>();
    if !alerts.is_empty() {
        paths.push(HealthDrillDownPath {
            path: "/health/alerts",
            label: "Alert queue",
            target_surface: "doctor",
            target_ref: "alerts",
            summary: format!(
                "active_alerts={} inspect the alert queue before repair",
                alerts.len()
            ),
            related_subsystems: alerts.iter().map(|alert| alert.subsystem).collect(),
        });
    }
    let attention_hotspots = subsystem_status
        .iter()
        .find(|status| status.subsystem == "memory")
        .and_then(|status| {
            status
                .metrics
                .iter()
                .find(|metric| metric.metric == "attention_hotspot_count")
                .map(|metric| metric.value)
        })
        .unwrap_or(0);
    paths.push(HealthDrillDownPath {
        path: "/health/attention",
        label: "Attention heatmap",
        target_surface: "health",
        target_ref: "attention",
        summary: format!(
            "attention_hotspots={} inspect hotspot-ranked namespaces and prewarm targets",
            attention_hotspots
        ),
        related_subsystems: vec!["memory", "cache"],
    });
    paths.push(HealthDrillDownPath {
        path: "/mood_history",
        label: "Affect trajectory history",
        target_surface: "health",
        target_ref: "mood_history",
        summary:
            "inspect bounded emotional trajectory rows and the latest recall-facing mood snapshot"
                .to_string(),
        related_subsystems: vec!["memory"],
    });
    if let Some(degraded) = degraded_status {
        paths.push(HealthDrillDownPath {
            path: "/health/degraded",
            label: "Degraded-mode posture",
            target_surface: "doctor",
            target_ref: "degraded_status",
            summary: degraded.summary.clone(),
            related_subsystems: degraded.affected_subsystems.clone(),
        });
    }
    paths
}

fn recommended_runbook_for_subsystem(
    subsystem: &'static str,
    degraded_status: Option<&DegradedStatusSurface>,
) -> Option<&'static str> {
    if let Some(degraded) = degraded_status {
        if let Some(runbook) = degraded.recommended_runbooks.first().copied() {
            match subsystem {
                "availability" => return Some(runbook),
                "repair" if degraded.affected_subsystems.contains(&"repair") => {
                    return Some(runbook)
                }
                "index" if degraded.affected_subsystems.contains(&"index") => return Some(runbook),
                "cache" if degraded.affected_subsystems.contains(&"cache") => return Some(runbook),
                _ => {}
            }
        }
    }
    match subsystem {
        "memory" => Some("daily_health_review"),
        "cache" => Some("index_rebuild_operations"),
        "index" => Some("tier2_index_drift"),
        "repair" => Some("repair_backlog_growth"),
        "availability" => Some("incident_response"),
        _ => None,
    }
}

fn dominant_signal(row: &AttentionNamespaceInputs) -> &'static str {
    let candidates = [
        (
            "working_memory_pressure",
            row.working_memory_pressure as u64 * 17,
        ),
        ("overflow_count", row.overflow_count * 13),
        ("promotion_count", row.promotion_count * 11),
        ("recall_count", row.recall_count * 5),
        ("encode_count", row.encode_count * 3),
    ];
    candidates
        .into_iter()
        .max_by_key(|(_, weight)| *weight)
        .map(|(signal, _)| signal)
        .unwrap_or("recall_count")
}

fn hotspot_status(attention_score: u64, working_memory_pressure: usize) -> &'static str {
    if attention_score >= 200 || working_memory_pressure >= 8 {
        "hot"
    } else if attention_score >= 80 || working_memory_pressure >= 4 {
        "warming"
    } else {
        "stable"
    }
}

fn attention_heat_band(attention_score: u64, working_memory_pressure: usize) -> u8 {
    if attention_score >= 260 || working_memory_pressure >= 10 {
        4
    } else if attention_score >= 200 || working_memory_pressure >= 8 {
        3
    } else if attention_score >= 80 || working_memory_pressure >= 4 {
        2
    } else if attention_score > 0 || working_memory_pressure > 0 {
        1
    } else {
        0
    }
}

fn attention_heat_bucket(heat_band: u8) -> &'static str {
    match heat_band {
        4 => "critical",
        3 => "hot",
        2 => "warming",
        1 => "warm",
        _ => "idle",
    }
}

fn attention_prewarm_policy(
    status: &'static str,
    dominant_signal: &'static str,
) -> (&'static str, &'static str, &'static str) {
    match (status, dominant_signal) {
        ("hot", "working_memory_pressure") | ("hot", "overflow_count") => {
            ("task_intent", "bounded_goal_rewarm", "goal_conditioned")
        }
        ("hot", _) | ("warming", "recall_count") => (
            "session_recency",
            "bounded_session_rewarm",
            "session_warmup",
        ),
        ("warming", "promotion_count") => (
            "entity_follow",
            "bounded_neighborhood_rewarm",
            "goal_conditioned",
        ),
        ("warming", _) => ("session_recency", "queue_prefetch_hint", "prefetch_hints"),
        _ => ("none", "observe_only", "none"),
    }
}

fn adaptive_prewarm_state(
    prefetch_enabled: bool,
    prefetch_queue_depth: usize,
    prefetch_capacity: usize,
    hints_dropped: u64,
) -> &'static str {
    if !prefetch_enabled {
        "disabled"
    } else if hints_dropped > 0 {
        "constrained"
    } else if prefetch_queue_depth >= prefetch_capacity && prefetch_capacity > 0 {
        "saturated"
    } else if prefetch_queue_depth > 0 {
        "active"
    } else {
        "idle"
    }
}

fn adaptive_prewarm_summary(
    adaptive_prewarm_state: &'static str,
    prefetch_queue_depth: usize,
    prefetch_capacity: usize,
    metrics: &crate::store::cache::PrefetchMetrics,
) -> String {
    format!(
        "state={} queue_depth={}/{} hints_submitted={} hints_consumed={} hints_dropped={} drop_budget={} drop_intent={} drop_scope={} drop_disabled={}",
        adaptive_prewarm_state,
        prefetch_queue_depth,
        prefetch_capacity,
        metrics.hints_submitted,
        metrics.hints_consumed,
        metrics.hints_dropped,
        metrics.dropped_budget_exhausted,
        metrics.dropped_intent_changed,
        metrics.dropped_scope_changed,
        metrics.dropped_disabled,
    )
}

fn drill_down_path_for_subsystem(subsystem: &'static str) -> &'static str {
    match subsystem {
        "memory" => "/health/subsystems/memory",
        "lifecycle" => "/health/subsystems/lifecycle",
        "cache" => "/health/subsystems/cache",
        "index" => "/health/subsystems/index",
        "repair" => "/health/subsystems/repair",
        "availability" => "/health/subsystems/availability",
        "affect_trajectory" => "/mood_history",
        _ => "/health/subsystems",
    }
}

fn drill_down_label_for_subsystem(subsystem: &'static str) -> &'static str {
    match subsystem {
        "memory" => "Memory quality",
        "lifecycle" => "Lifecycle activity",
        "cache" => "Cache health",
        "index" => "Index health",
        "repair" => "Repair backlog",
        "availability" => "Availability posture",
        "affect_trajectory" => "Affect trajectory history",
        _ => "Subsystem health",
    }
}

fn drill_down_surface_for_subsystem(subsystem: &'static str) -> &'static str {
    match subsystem {
        "memory" => "inspect",
        "lifecycle" | "affect_trajectory" => "health",
        "cache" | "index" | "repair" | "availability" => "doctor",
        _ => "health",
    }
}

fn degraded_status_surface(
    availability: Option<&AvailabilitySummary>,
    cache: &CacheHealthReport,
    indexes: &IndexHealthSummary,
    repair: Option<&RepairHealthReport>,
) -> Option<DegradedStatusSurface> {
    let availability = availability?;
    if availability.posture == AvailabilityPosture::Full {
        return None;
    }

    let mut affected_subsystems = Vec::new();
    if cache.state != SubsystemHealthState::Healthy {
        affected_subsystems.push("cache");
    }
    if indexes.state != SubsystemHealthState::Healthy {
        affected_subsystems.push("index");
    }
    if repair.is_some_and(|report| report.state != SubsystemHealthState::Healthy) {
        affected_subsystems.push("repair");
    }
    if affected_subsystems.is_empty() {
        affected_subsystems.push("availability");
    }

    let degraded_reasons = availability.reason_names();
    let remediation_steps = availability.recovery_condition_names();
    let recommended_runbooks = recommended_runbooks(&degraded_reasons, repair);
    let summary = format!(
        "posture={} affected_subsystems={} surviving_queries={} surviving_mutations={} reasons={} remediation={} runbooks={}",
        availability.posture.as_str(),
        affected_subsystems.join(","),
        join_or_none(&availability.query_capabilities),
        join_or_none(&availability.mutation_capabilities),
        join_or_none(&degraded_reasons),
        join_or_none(&remediation_steps),
        join_or_none(&recommended_runbooks),
    );

    Some(DegradedStatusSurface {
        posture: availability.posture,
        summary,
        affected_subsystems,
        surviving_query_capabilities: availability.query_capabilities.clone(),
        surviving_mutation_capabilities: availability.mutation_capabilities.clone(),
        degraded_reasons,
        remediation_steps,
        recommended_runbooks,
    })
}

fn recommended_runbooks(
    degraded_reasons: &[&'static str],
    repair: Option<&RepairHealthReport>,
) -> Vec<&'static str> {
    let mut runbooks = Vec::new();
    if degraded_reasons
        .iter()
        .any(|reason| matches!(*reason, "index_bypassed" | "cache_invalidated"))
    {
        runbooks.push("index_rebuild_operations");
    }
    if degraded_reasons
        .iter()
        .any(|reason| *reason == "index_bypassed")
    {
        runbooks.push("tier2_index_drift");
    }
    if degraded_reasons.iter().any(|reason| {
        matches!(
            *reason,
            "repair_in_flight" | "repair_rollback_required" | "repair_rollback_in_progress"
        )
    }) || repair.is_some_and(|report| report.queue_depth > 0)
    {
        runbooks.push("repair_backlog_growth");
    }
    if degraded_reasons.iter().any(|reason| {
        matches!(
            *reason,
            "graph_unavailable" | "authoritative_input_unreadable"
        )
    }) || repair.is_some_and(|report| report.state == SubsystemHealthState::Blocked)
    {
        runbooks.push("incident_response");
    }
    if runbooks.is_empty() {
        runbooks.push("incident_response");
    }
    runbooks
}

fn join_or_none(values: &[&'static str]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(",")
    }
}

fn join_owned_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(",")
    }
}

fn backpressure_state(
    prefetch_queue_depth: usize,
    repair_queue_depth: Option<u64>,
) -> Option<&'static str> {
    match (prefetch_queue_depth, repair_queue_depth.unwrap_or(0)) {
        (0..=3, 0) => Some("normal"),
        (0..=7, 0..=1) => Some("elevated"),
        _ => Some("high"),
    }
}

fn percent(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        (numerator as f32 / denominator as f32) * 100.0
    }
}

fn higher_is_better(previous: u64, current: u64) -> TrendDirection {
    if current > previous {
        TrendDirection::Improving
    } else if current < previous {
        TrendDirection::Worsening
    } else {
        TrendDirection::Stable
    }
}

fn lower_is_better(previous: u64, current: u64) -> TrendDirection {
    if current < previous {
        TrendDirection::Improving
    } else if current > previous {
        TrendDirection::Worsening
    } else {
        TrendDirection::Stable
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AttentionNamespaceInputs, BrainHealthInputs, BrainHealthReport, FeatureAvailabilityEntry,
        LifecycleHealthReport, RepairHealthReport, SubsystemHealthState, TrendDirection,
    };
    use crate::api::{
        AvailabilityPosture, AvailabilityReason, AvailabilitySummary, NamespaceId, RemediationStep,
    };
    use crate::engine::repair::{
        IndexRepairEntrypoint, RepairCheckResult, RepairOperatorReport, RepairStatus,
        RepairSummary, RepairTarget, VerificationArtifact,
    };
    use crate::index::{IndexApi, IndexHealth, IndexHealthReport, IndexModule};
    use crate::store::cache::{CacheFamily, CacheManager};
    use std::collections::HashMap;

    #[test]
    fn repair_health_report_marks_corrupt_runs_as_blocked() {
        let report = RepairHealthReport::from_summary(&RepairSummary {
            targets_checked: 2,
            healthy: 0,
            degraded: 1,
            corrupt: 1,
            rebuilt: 0,
            affected_item_count: 5,
            error_count: 2,
            rebuild_duration_ms: 33,
            rollback_state: Some("rollback_required"),
            degraded_mode: Some(crate::engine::repair::RepairDegradedMode::EnterReadOnly),
            rollback_trigger: Some(
                crate::engine::repair::RepairRollbackTrigger::VerificationMismatch,
            ),
            queue_report: crate::observability::MaintenanceQueueReport {
                queue_family: "repair",
                queue_status: crate::observability::MaintenanceQueueStatus::Partial,
                queue_depth_before: 2,
                queue_depth_after: 2,
                jobs_processed: 0,
                affected_item_count: 5,
                duration_ms: 33,
                retry_attempts: 2,
                partial_run: true,
            },
            results: Vec::new(),
            verification_artifacts: HashMap::new(),
            operator_reports: Vec::new(),
            graph_rebuild_reports: HashMap::new(),
            cache_invalidation_reports: HashMap::new(),
            cache_warmup_reports: HashMap::new(),
        });

        assert_eq!(report.state, SubsystemHealthState::Blocked);
        assert_eq!(report.queue_depth, 2);
        assert_eq!(report.degraded_mode, Some("enter_read_only"));
        assert_eq!(report.rollback_trigger, Some("verification_mismatch"));
        assert_eq!(
            report.remediation_steps,
            vec![
                "check_health",
                "rollback_recent_change",
                "run_repair",
                "inspect_state"
            ]
        );
    }

    #[test]
    fn health_report_aggregates_subsystems_and_trends() {
        let mut cache = CacheManager::new(4, 4);
        cache.result.disable();
        cache.prefetch.submit_hint(
            NamespaceId::new("health").expect("namespace"),
            crate::store::cache::PrefetchTrigger::SessionRecency,
            vec![],
        );

        let mut index_reports = IndexModule.health_reports();
        index_reports[0] = IndexHealthReport {
            family: index_reports[0].family,
            health: IndexHealth::NeedsRebuild,
            entry_count: 8,
            generation: "v2",
            hit_rate: 82,
            miss_rate: 18,
            stale_index_ratio: 20,
            repair_backlog: 3,
            rebuild_duration_hint: "medium",
            item_count_divergence: 2,
        };

        let repair_summary = RepairSummary {
            targets_checked: 3,
            healthy: 1,
            degraded: 1,
            corrupt: 1,
            rebuilt: 0,
            affected_item_count: 10,
            error_count: 2,
            rebuild_duration_ms: 41,
            rollback_state: Some("rollback_required"),
            degraded_mode: Some(crate::engine::repair::RepairDegradedMode::EnterReadOnly),
            rollback_trigger: Some(crate::engine::repair::RepairRollbackTrigger::VerificationMismatch),
            queue_report: crate::observability::MaintenanceQueueReport {
                queue_family: "repair",
                queue_status: crate::observability::MaintenanceQueueStatus::Partial,
                queue_depth_before: 3,
                queue_depth_after: 2,
                jobs_processed: 1,
                affected_item_count: 10,
                duration_ms: 41,
                retry_attempts: 2,
                partial_run: true,
            },
            results: vec![RepairCheckResult {
                target: RepairTarget::LexicalIndex,
                status: RepairStatus::Corrupt,
                detail: "parity_failed",
                verification_passed: false,
                rebuild_entrypoint: Some(IndexRepairEntrypoint::RebuildIfNeeded),
                rebuilt_outputs: vec!["fts5_memory_index"],
                repair_hooks: Vec::new(),
            }],
            verification_artifacts: HashMap::from([(
                RepairTarget::LexicalIndex,
                VerificationArtifact {
                    artifact_name: "fts5_lexical_parity",
                    authoritative_rows: 10,
                    derived_rows: 8,
                    authoritative_generation: "durable.v2",
                    derived_generation: "durable.v1",
                    parity_check: "fts5_projection_matches_durable_truth",
                    verification_passed: false,
                    repair_hooks: Vec::new(),
                },
            )]),
            operator_reports: vec![RepairOperatorReport {
                target: RepairTarget::LexicalIndex,
                status: RepairStatus::Corrupt,
                verification_passed: false,
                rebuild_entrypoint: Some(IndexRepairEntrypoint::RebuildIfNeeded),
                rebuilt_outputs: vec!["fts5_memory_index"],
                durable_sources: vec!["durable_memory_records"],
                repair_hooks: Vec::new(),
                verification_artifact_name: "fts5_lexical_parity",
                affected_item_count: 10,
                error_count: 2,
                rebuild_duration_ms: 41,
                rollback_state: Some("rollback_required"),
                degraded_mode: Some(crate::engine::repair::RepairDegradedMode::EnterReadOnly),
                rollback_trigger: Some(crate::engine::repair::RepairRollbackTrigger::VerificationMismatch),
                remediation_steps: vec![
                    RemediationStep::CheckHealth,
                    RemediationStep::RollbackRecentChange,
                    RemediationStep::RunRepair,
                    RemediationStep::InspectState,
                ],
                queue_depth_before: 3,
                queue_depth_after: 0,
                graph_hooks: Vec::new(),
                cache_invalidation_events: Vec::new(),
                cache_warmup_events: Vec::new(),
                operator_log: "target=lexical_index status=corrupt affected_item_count=10 error_count=2 rebuild_duration_ms=41 rollback_state=rollback_required degraded_mode=enter_read_only rollback_trigger=verification_mismatch remediation_steps=check_health,rollback_recent_change,run_repair,inspect_state queue_depth_before=3 queue_depth_after=0".to_string(),
            }],
            graph_rebuild_reports: HashMap::new(),
            cache_invalidation_reports: HashMap::new(),
            cache_warmup_reports: HashMap::new(),
        };

        let report = BrainHealthReport::from_inputs(
            BrainHealthInputs {
                hot_memories: 76,
                hot_capacity: 100,
                cold_memories: 12,
                avg_strength: 0.71,
                avg_confidence: 0.84,
                low_confidence_count: 3,
                decay_rate: 0.012,
                archive_count: 5,
                lifecycle: LifecycleHealthReport {
                    consolidated_to_cold_count: 12,
                    reconsolidation_active_count: 2,
                    forgetting_archive_count: 5,
                    background_maintenance_runs: 3,
                    background_maintenance_log: vec![
                        "maintenance_consolidation_completed:cold_migration=12".to_string(),
                        "maintenance_reconsolidation_applied:volatile_results=2".to_string(),
                        "maintenance_forgetting_evaluated:archived=5".to_string(),
                    ],
                },
                total_engrams: 14,
                avg_cluster_size: 2.5,
                top_engrams: vec![("ops".to_string(), 4)],
                landmark_count: 2,
                unresolved_conflicts: 1,
                uncertain_count: 3,
                dream_links_total: 9,
                last_dream_tick: Some(42),
                affect_history_rows: 2,
                latest_affect_snapshot: Some((0.4, 0.9)),
                latest_affect_tick: Some(198),
                attention_namespaces: vec![AttentionNamespaceInputs {
                    namespace: "team.alpha".to_string(),
                    recall_count: 21,
                    encode_count: 4,
                    working_memory_pressure: 6,
                    promotion_count: 2,
                    overflow_count: 1,
                }],
                total_recalls: 55,
                total_encodes: 12,
                current_tick: 200,
                daemon_uptime_ticks: 180,
                index_reports,
                availability: Some(AvailabilitySummary::degraded(
                    vec!["recall", "health"],
                    vec!["encode"],
                    vec![AvailabilityReason::RepairRollbackRequired],
                    vec![RemediationStep::RunRepair, RemediationStep::CheckHealth],
                )),
                feature_availability: vec![FeatureAvailabilityEntry {
                    feature: "health".to_string(),
                    posture: AvailabilityPosture::Full,
                    note: None,
                }],
                previous_total_recalls: Some(44),
                previous_total_encodes: Some(10),
                previous_repair_queue_depth: Some(0),
                previous_hot_memories: Some(70),
                previous_low_confidence_count: Some(5),
                previous_unresolved_conflicts: Some(2),
                previous_uncertain_count: Some(4),
                previous_cache_hit_count: Some(0),
                previous_cache_miss_count: Some(1),
                previous_cache_bypass_count: Some(0),
                previous_prefetch_queue_depth: Some(0),
                previous_prefetch_drop_count: Some(0),
                previous_index_stale_count: Some(0),
                previous_index_missing_count: Some(0),
                previous_index_repair_backlog_total: Some(0),
                previous_availability_posture: Some(AvailabilityPosture::Full),
            },
            &cache,
            Some(&repair_summary),
        );

        assert_eq!(report.hot_utilization_pct, 76.0);
        assert_eq!(report.attention.total_recall_count, 21);
        assert_eq!(report.attention.total_encode_count, 4);
        assert_eq!(report.attention.total_promotion_count, 2);
        assert_eq!(report.attention.total_overflow_count, 1);
        assert_eq!(report.attention.highest_namespace_pressure, 6);
        assert_eq!(report.attention.hotspot_count, 1);
        assert_eq!(report.attention.hotspots[0].namespace, "team.alpha");
        assert_eq!(report.attention.hotspots[0].attention_score, 254);
        assert_eq!(report.attention.hotspots[0].status, "hot");
        assert_eq!(report.attention.hotspots[0].dominant_signal, "recall_count");
        assert_eq!(report.attention.max_attention_score, 254);
        assert_eq!(report.attention.warming_candidate_count, 0);
        assert_eq!(report.attention.hot_candidate_count, 1);
        assert_eq!(report.attention.hotspots[0].heat_bucket, "hot");
        assert_eq!(report.attention.hotspots[0].heat_band, 3);
        assert_eq!(
            report.attention.hotspots[0].prewarm_trigger,
            "session_recency"
        );
        assert_eq!(
            report.attention.hotspots[0].prewarm_action,
            "bounded_session_rewarm"
        );
        assert_eq!(
            report.attention.hotspots[0].prewarm_target_family,
            "session_warmup"
        );
        assert_eq!(report.affect_history_rows, 2);
        assert_eq!(report.latest_affect_snapshot, Some((0.4, 0.9)));
        assert_eq!(report.latest_affect_tick, Some(198));
        assert_eq!(
            report.availability_posture,
            Some(AvailabilityPosture::Degraded)
        );
        assert_eq!(report.backpressure_state, Some("high"));
        assert_eq!(report.repair_queue_depth, Some(2));
        assert_eq!(report.repair.as_ref().map(|r| r.targets_checked), Some(3));
        assert_eq!(
            report.degraded_status.as_ref().map(|status| status.posture),
            Some(AvailabilityPosture::Degraded)
        );
        assert!(report.degraded_status.as_ref().is_some_and(|status| status
            .affected_subsystems
            .contains(&"cache")
            && status.affected_subsystems.contains(&"index")
            && status.affected_subsystems.contains(&"repair")
            && status.surviving_query_capabilities == vec!["recall", "health"]
            && status.surviving_mutation_capabilities == vec!["encode"]
            && status
                .degraded_reasons
                .contains(&"repair_rollback_required")
            && status.remediation_steps.contains(&"run_repair")
            && status
                .recommended_runbooks
                .contains(&"repair_backlog_growth")));
        assert_eq!(
            report.repair.as_ref().and_then(|r| r.degraded_mode),
            Some("enter_read_only")
        );
        assert_eq!(
            report.repair.as_ref().and_then(|r| r.rollback_trigger),
            Some("verification_mismatch")
        );
        assert!(report
            .subsystem_status
            .iter()
            .find(|status| status.subsystem == "repair")
            .is_some_and(|status| {
                status.detail.contains("queue_depth=2")
                    && status.detail.contains("degraded_mode=enter_read_only")
                    && status
                        .detail
                        .contains("rollback_trigger=verification_mismatch")
                    && status.reasons.contains(&"repair_corrupt_targets_present")
                    && status
                        .metrics
                        .iter()
                        .any(|metric| metric.metric == "queue_depth" && metric.value == 2)
            }));
        assert_eq!(report.cache.state, SubsystemHealthState::Unavailable);
        assert_eq!(report.indexes.state, SubsystemHealthState::Degraded);
        assert_eq!(
            report.repair.as_ref().map(|r| r.state),
            Some(SubsystemHealthState::Blocked)
        );
        assert_eq!(report.lifecycle.consolidated_to_cold_count, 12);
        assert_eq!(report.lifecycle.reconsolidation_active_count, 2);
        assert_eq!(report.lifecycle.forgetting_archive_count, 5);
        assert_eq!(report.lifecycle.background_maintenance_runs, 3);
        assert!(report
            .lifecycle
            .background_maintenance_log
            .iter()
            .any(|entry| entry.contains("maintenance_consolidation_completed")));
        assert!(report
            .subsystem_status
            .iter()
            .find(|status| status.subsystem == "lifecycle")
            .is_some_and(|status| {
                status.detail.contains("consolidated_to_cold=12")
                    && status.detail.contains("reconsolidation_active=2")
                    && status.detail.contains("forgetting_archived=5")
                    && status.detail.contains("maintenance_runs=3")
                    && status.reasons.contains(&"reconsolidation_window_open")
            }));
        assert!(report
            .trend_summary
            .iter()
            .find(|summary| summary.subsystem == "lifecycle")
            .is_some_and(|summary| {
                summary
                    .trends
                    .iter()
                    .any(|trend| trend.metric == "consolidated_to_cold_count")
            }));
        assert!(report
            .subsystem_status
            .iter()
            .any(|row| row.subsystem == "availability"
                && row.state == SubsystemHealthState::Degraded
                && row.reasons.contains(&"repair_rollback_required")));
        assert!(report
            .subsystem_status
            .iter()
            .any(|row| row.subsystem == "memory"
                && row.state == SubsystemHealthState::Blocked
                && row.reasons.contains(&"conflicts_unresolved")));
        assert!(report
            .trends
            .iter()
            .any(|trend| trend.subsystem == "activity"
                && trend.metric == "total_recalls"
                && trend.direction == TrendDirection::Improving));
        assert!(report.trends.iter().any(|trend| trend.subsystem == "repair"
            && trend.metric == "repair_queue_depth"
            && trend.direction == TrendDirection::Worsening));
        assert!(report.trends.iter().any(|trend| trend.subsystem == "memory"
            && trend.metric == "unresolved_conflicts"
            && trend.direction == TrendDirection::Improving));
        assert!(report
            .trend_summary
            .iter()
            .find(|summary| summary.subsystem == "cache")
            .is_some_and(|summary| summary.state == SubsystemHealthState::Unavailable
                && summary
                    .trends
                    .iter()
                    .any(|trend| trend.metric == "cache_hit_count")));
        assert!(report
            .cache
            .family_status
            .iter()
            .any(|family| family.family == CacheFamily::ResultCache.as_str()
                && family.state == SubsystemHealthState::Unavailable));
        assert!(report.cache.family_status.iter().any(|family| family.family
            == CacheFamily::PrefetchHints.as_str()
            && family.state == SubsystemHealthState::Healthy
            && family.hit_count == 0
            && family.bypass_count == 0));
        assert!(report
            .dashboard_views
            .iter()
            .any(|view| view.view.as_str() == "overview"
                && view.drill_down_targets.contains(&"/health/subsystems")
                && view.drill_down_targets.contains(&"/health/alerts")));
        assert!(report
            .dashboard_views
            .iter()
            .any(|view| view.view.as_str() == "attention"
                && view.summary.contains("hotspots=1")
                && view.drill_down_targets.contains(&"/health/attention")));
        assert!(report.dashboard_views.iter().any(|view| view.view.as_str()
            == "affect_trajectory"
            && view.summary.contains("rows=2")
            && view.summary.contains("current_mood=(0.40,0.90)")
            && view.drill_down_targets.contains(&"/mood_history")));
        assert!(report
            .dashboard_views
            .iter()
            .any(|view| view.view.as_str() == "degraded_mode"));
        assert!(report.alerts.iter().any(|alert| alert.subsystem == "memory"
            && alert.severity.as_str() == "critical"
            && alert.drill_down_path == "/health/subsystems/memory"
            && alert.reason_codes.contains(&"conflicts_unresolved")));
        assert!(report.alerts.iter().any(|alert| alert.subsystem == "index"
            && alert.recommended_runbook.is_some()
            && (alert.recommended_runbook == Some("tier2_index_drift")
                || alert.recommended_runbook == Some("repair_backlog_growth"))));
        assert!(report
            .drill_down_paths
            .iter()
            .any(|path| path.path == "/health/alerts" && path.target_surface == "doctor"));
        assert!(report.drill_down_paths.iter().any(|path| path.path
            == "/health/subsystems/lifecycle"
            && path.label == "Lifecycle activity"
            && path.target_surface == "health"
            && path.target_ref == "lifecycle"
            && path.summary.contains("consolidated_to_cold=12")
            && path.summary.contains("reconsolidation_active=2")
            && path.summary.contains("forgetting_archived=5")
            && path.summary.contains("maintenance_runs=3")
            && path.related_subsystems == vec!["lifecycle"]));
        assert!(report
            .drill_down_paths
            .iter()
            .any(|path| path.path == "/health/attention"
                && path.target_surface == "health"
                && path.related_subsystems.contains(&"memory")
                && path.related_subsystems.contains(&"cache")));
        assert!(report
            .drill_down_paths
            .iter()
            .any(|path| path.path == "/mood_history"
                && path.target_surface == "health"
                && path.related_subsystems == vec!["memory"]));
        assert!(report
            .drill_down_paths
            .iter()
            .any(|path| path.path == "/health/degraded" && path.target_ref == "degraded_status"));
    }

    #[test]
    fn health_report_includes_prefetch_family_drop_metrics() {
        let mut cache = CacheManager::new(4, 4);
        cache.prefetch.submit_hint(
            NamespaceId::new("health").expect("namespace"),
            crate::store::cache::PrefetchTrigger::SessionRecency,
            vec![crate::types::MemoryId(1)],
        );
        cache.prefetch.submit_hint(
            NamespaceId::new("health").expect("namespace"),
            crate::store::cache::PrefetchTrigger::TaskIntent,
            vec![crate::types::MemoryId(2)],
        );
        let dropped = cache.prefetch.cancel_namespace(
            &NamespaceId::new("health").expect("namespace"),
            crate::store::cache::PrefetchBypassReason::NamespaceNarrowed,
        );
        assert_eq!(dropped, 2);

        let report = BrainHealthReport::from_inputs(
            BrainHealthInputs {
                hot_memories: 0,
                hot_capacity: 10,
                cold_memories: 0,
                avg_strength: 0.0,
                avg_confidence: 0.0,
                low_confidence_count: 0,
                decay_rate: 0.0,
                archive_count: 0,
                lifecycle: LifecycleHealthReport {
                    consolidated_to_cold_count: 0,
                    reconsolidation_active_count: 0,
                    forgetting_archive_count: 0,
                    background_maintenance_runs: 0,
                    background_maintenance_log: vec![
                        "no_background_lifecycle_activity_recorded".to_string()
                    ],
                },
                total_engrams: 0,
                avg_cluster_size: 0.0,
                top_engrams: Vec::new(),
                landmark_count: 0,
                unresolved_conflicts: 0,
                uncertain_count: 0,
                dream_links_total: 0,
                last_dream_tick: None,
                affect_history_rows: 0,
                latest_affect_snapshot: None,
                latest_affect_tick: None,
                attention_namespaces: vec![AttentionNamespaceInputs {
                    namespace: "health".to_string(),
                    recall_count: 2,
                    encode_count: 0,
                    working_memory_pressure: 1,
                    promotion_count: 0,
                    overflow_count: 0,
                }],
                total_recalls: 0,
                total_encodes: 0,
                current_tick: 0,
                daemon_uptime_ticks: 0,
                index_reports: IndexModule.health_reports(),
                availability: None,
                feature_availability: Vec::new(),
                previous_total_recalls: None,
                previous_total_encodes: None,
                previous_repair_queue_depth: None,
                previous_hot_memories: None,
                previous_low_confidence_count: None,
                previous_unresolved_conflicts: None,
                previous_uncertain_count: None,
                previous_cache_hit_count: None,
                previous_cache_miss_count: None,
                previous_cache_bypass_count: None,
                previous_prefetch_queue_depth: None,
                previous_prefetch_drop_count: None,
                previous_index_stale_count: None,
                previous_index_missing_count: None,
                previous_index_repair_backlog_total: None,
                previous_availability_posture: None,
            },
            &cache,
            None,
        );

        let prefetch_family = report
            .cache
            .family_status
            .iter()
            .find(|family| family.family == CacheFamily::PrefetchHints.as_str())
            .expect("prefetch family should be reported");
        assert_eq!(prefetch_family.state, SubsystemHealthState::Degraded);
        assert_eq!(report.attention.hotspot_count, 1);
        assert_eq!(report.attention.hotspots[0].namespace, "health");
        assert_eq!(report.attention.hotspots[0].attention_score, 27);
        assert_eq!(report.attention.hotspots[0].status, "stable");
        assert_eq!(report.attention.max_attention_score, 27);
        assert_eq!(report.attention.warming_candidate_count, 0);
        assert_eq!(report.attention.hot_candidate_count, 0);
        assert_eq!(report.attention.hotspots[0].heat_bucket, "warm");
        assert_eq!(report.attention.hotspots[0].heat_band, 1);
        assert_eq!(report.attention.hotspots[0].prewarm_action, "observe_only");
        assert_eq!(report.cache.prefetch_capacity, 4);
        assert_eq!(report.cache.adaptive_prewarm_state, "constrained");
        assert!(report
            .cache
            .adaptive_prewarm_summary
            .contains("state=constrained queue_depth=0/4"));
        assert!(prefetch_family.hit_count == 0);
        assert_eq!(prefetch_family.bypass_count, 2);
        assert_eq!(report.cache.hints_submitted, 2);
        assert_eq!(report.cache.hints_dropped, 2);
        assert_eq!(report.cache.drop_scope_changed_count, 2);
        assert_eq!(report.cache.total_bypass_count, 2);
        assert_eq!(report.cache.state, SubsystemHealthState::Degraded);
        assert!(report
            .subsystem_status
            .iter()
            .find(|row| row.subsystem == "cache")
            .is_some_and(|row| row.reasons.contains(&"prefetch_drop_detected")
                && row
                    .metrics
                    .iter()
                    .any(|metric| metric.metric == "prefetch_drop_count" && metric.value == 2)));
    }
}
