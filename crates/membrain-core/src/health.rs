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
    pub metric: &'static str,
    pub previous: u64,
    pub current: u64,
    pub direction: TrendDirection,
}

/// One machine-readable feature-availability entry.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FeatureAvailabilityEntry {
    pub feature: String,
    pub posture: AvailabilityPosture,
    pub note: Option<String>,
}

/// One machine-readable subsystem status row.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SubsystemStatus {
    pub subsystem: &'static str,
    pub state: SubsystemHealthState,
    pub detail: String,
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
    pub hints_submitted: u64,
    pub hints_consumed: u64,
    pub hints_dropped: u64,
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
    pub total_engrams: usize,
    pub avg_cluster_size: f32,
    pub top_engrams: Vec<(String, usize)>,
    pub landmark_count: usize,
    pub unresolved_conflicts: usize,
    pub uncertain_count: usize,
    pub dream_links_total: usize,
    pub last_dream_tick: Option<u64>,
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
    pub total_engrams: usize,
    pub avg_cluster_size: f32,
    pub top_engrams: Vec<(String, usize)>,
    pub landmark_count: usize,
    pub unresolved_conflicts: usize,
    pub uncertain_count: usize,
    pub dream_links_total: usize,
    pub last_dream_tick: Option<u64>,
    pub repair_queue_depth: Option<u64>,
    pub backpressure_state: Option<&'static str>,
    pub feature_availability: Vec<FeatureAvailabilityEntry>,
    pub availability_posture: Option<AvailabilityPosture>,
    pub availability_notes: Option<String>,
    pub cache: CacheHealthReport,
    pub indexes: IndexHealthSummary,
    pub repair: Option<RepairHealthReport>,
    pub subsystem_status: Vec<SubsystemStatus>,
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
        let repair_queue_depth = repair.as_ref().map(|report| report.queue_depth);
        let availability_posture = inputs.availability.as_ref().map(|a| a.posture);
        let availability_notes = inputs.availability.as_ref().map(availability_note);
        let backpressure_state = backpressure_state(cache.prefetch_queue_depth, repair_queue_depth);
        let mut subsystem_status = vec![
            SubsystemStatus {
                subsystem: "cache",
                state: cache.state,
                detail: format!(
                    "hits={} misses={} bypasses={} queue_depth={}",
                    cache.total_hit_count,
                    cache.total_miss_count,
                    cache.total_bypass_count,
                    cache.prefetch_queue_depth
                ),
            },
            SubsystemStatus {
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
            },
        ];
        if let Some(repair) = &repair {
            subsystem_status.push(SubsystemStatus {
                subsystem: "repair",
                state: repair.state,
                detail: format!(
                    "checked={} degraded={} corrupt={} rebuilt={} queue_depth={}",
                    repair.targets_checked,
                    repair.degraded,
                    repair.corrupt,
                    repair.rebuilt,
                    repair.queue_depth
                ),
            });
        }
        if let Some(availability) = &inputs.availability {
            subsystem_status.push(SubsystemStatus {
                subsystem: "availability",
                state: match availability.posture {
                    AvailabilityPosture::Full => SubsystemHealthState::Healthy,
                    AvailabilityPosture::Degraded | AvailabilityPosture::ReadOnly => {
                        SubsystemHealthState::Degraded
                    }
                    AvailabilityPosture::Offline => SubsystemHealthState::Unavailable,
                },
                detail: format!(
                    "posture={} reasons={} recovery={}",
                    availability.posture.as_str(),
                    availability.reason_names().join(","),
                    availability.recovery_condition_names().join(",")
                ),
            });
        }

        let mut trends = Vec::new();
        if let Some(previous) = inputs.previous_total_recalls {
            trends.push(HealthTrend {
                metric: "total_recalls",
                previous,
                current: inputs.total_recalls,
                direction: higher_is_better(previous, inputs.total_recalls),
            });
        }
        if let Some(previous) = inputs.previous_total_encodes {
            trends.push(HealthTrend {
                metric: "total_encodes",
                previous,
                current: inputs.total_encodes,
                direction: higher_is_better(previous, inputs.total_encodes),
            });
        }
        if let Some(previous) = inputs.previous_repair_queue_depth.zip(repair_queue_depth) {
            trends.push(HealthTrend {
                metric: "repair_queue_depth",
                previous: previous.0,
                current: previous.1,
                direction: lower_is_better(previous.0, previous.1),
            });
        }

        Self {
            hot_memories: inputs.hot_memories,
            hot_capacity: inputs.hot_capacity,
            cold_memories: inputs.cold_memories,
            hot_utilization_pct: percent(inputs.hot_memories, inputs.hot_capacity),
            avg_strength: inputs.avg_strength,
            avg_confidence: inputs.avg_confidence,
            low_confidence_count: inputs.low_confidence_count,
            decay_rate: inputs.decay_rate,
            archive_count: inputs.archive_count,
            total_engrams: inputs.total_engrams,
            avg_cluster_size: inputs.avg_cluster_size,
            top_engrams: inputs.top_engrams,
            landmark_count: inputs.landmark_count,
            unresolved_conflicts: inputs.unresolved_conflicts,
            uncertain_count: inputs.uncertain_count,
            dream_links_total: inputs.dream_links_total,
            last_dream_tick: inputs.last_dream_tick,
            repair_queue_depth,
            backpressure_state,
            feature_availability: inputs.feature_availability,
            availability_posture,
            availability_notes,
            cache,
            indexes,
            repair,
            subsystem_status,
            trends,
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
        Self {
            state,
            family_status,
            total_hit_count,
            total_miss_count,
            total_bypass_count,
            total_invalidation_count,
            total_stale_warning_count,
            prefetch_queue_depth: cache.prefetch.queue_depth(),
            hints_submitted: cache.prefetch.metrics.hints_submitted,
            hints_consumed: cache.prefetch.metrics.hints_consumed,
            hints_dropped: cache.prefetch.metrics.hints_dropped,
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
        let queue_depth = (summary.degraded + summary.corrupt) as u64;
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
        BrainHealthInputs, BrainHealthReport, FeatureAvailabilityEntry, RepairHealthReport,
        SubsystemHealthState, TrendDirection,
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
            results: Vec::new(),
            verification_artifacts: HashMap::new(),
            operator_reports: Vec::new(),
            graph_rebuild_reports: HashMap::new(),
        });

        assert_eq!(report.state, SubsystemHealthState::Blocked);
        assert_eq!(report.queue_depth, 2);
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
            results: vec![RepairCheckResult {
                target: RepairTarget::LexicalIndex,
                status: RepairStatus::Corrupt,
                detail: "parity_failed",
                verification_passed: false,
                rebuild_entrypoint: Some(IndexRepairEntrypoint::RebuildIfNeeded),
                rebuilt_outputs: vec!["fts5_memory_index"],
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
                },
            )]),
            operator_reports: vec![RepairOperatorReport {
                target: RepairTarget::LexicalIndex,
                status: RepairStatus::Corrupt,
                verification_passed: false,
                rebuild_entrypoint: Some(IndexRepairEntrypoint::RebuildIfNeeded),
                rebuilt_outputs: vec!["fts5_memory_index"],
                verification_artifact_name: "fts5_lexical_parity",
                operator_log: "target=lexical_index status=corrupt".to_string(),
            }],
            graph_rebuild_reports: HashMap::new(),
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
                total_engrams: 14,
                avg_cluster_size: 2.5,
                top_engrams: vec![("ops".to_string(), 4)],
                landmark_count: 2,
                unresolved_conflicts: 1,
                uncertain_count: 3,
                dream_links_total: 9,
                last_dream_tick: Some(42),
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
            },
            &cache,
            Some(&repair_summary),
        );

        assert_eq!(report.hot_utilization_pct, 76.0);
        assert_eq!(
            report.availability_posture,
            Some(AvailabilityPosture::Degraded)
        );
        assert_eq!(report.backpressure_state, Some("high"));
        assert_eq!(report.repair_queue_depth, Some(2));
        assert_eq!(report.cache.state, SubsystemHealthState::Unavailable);
        assert_eq!(report.indexes.state, SubsystemHealthState::Degraded);
        assert_eq!(
            report.repair.as_ref().map(|r| r.state),
            Some(SubsystemHealthState::Blocked)
        );
        assert!(report
            .subsystem_status
            .iter()
            .any(|row| row.subsystem == "availability"
                && row.state == SubsystemHealthState::Degraded));
        assert!(report
            .trends
            .iter()
            .any(|trend| trend.metric == "total_recalls"
                && trend.direction == TrendDirection::Improving));
        assert!(report
            .trends
            .iter()
            .any(|trend| trend.metric == "repair_queue_depth"
                && trend.direction == TrendDirection::Worsening));
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
                total_engrams: 0,
                avg_cluster_size: 0.0,
                top_engrams: Vec::new(),
                landmark_count: 0,
                unresolved_conflicts: 0,
                uncertain_count: 0,
                dream_links_total: 0,
                last_dream_tick: None,
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
        assert_eq!(prefetch_family.hit_count, 0);
        assert_eq!(prefetch_family.bypass_count, 2);
        assert_eq!(report.cache.hints_submitted, 2);
        assert_eq!(report.cache.hints_dropped, 2);
        assert_eq!(report.cache.total_bypass_count, 2);
        assert_eq!(report.cache.state, SubsystemHealthState::Degraded);
    }
}
