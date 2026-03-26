use crate::api::NamespaceId;
use crate::store::audit::AppendOnlyAuditLog;
use crate::store::cache::{CacheManager, PrefetchTrigger};
use crate::types::{MemoryId, SessionId};
use std::collections::HashMap;

/// Hard limits for learned recall-sequence tracking and speculative prewarm.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PredictiveRecallConfig {
    /// Minimum observed transitions required before one prediction is eligible.
    pub min_transition_support: u32,
    /// Minimum confidence for the strongest transition before prewarm may queue.
    pub min_prediction_confidence: f32,
    /// Maximum learned next-step branches preserved for one source memory.
    pub max_predictions_per_source: usize,
    /// Maximum predictions submitted to bounded speculative prewarm.
    pub max_prefetch_ids: usize,
    /// Fraction of prefetch capacity that predictive work may occupy before it observes only.
    pub max_queue_fill_ratio: f32,
    /// Whether session-local history is preferred over namespace-wide history when present.
    pub prefer_session_local: bool,
}

impl Default for PredictiveRecallConfig {
    fn default() -> Self {
        Self {
            min_transition_support: 2,
            min_prediction_confidence: 0.60,
            max_predictions_per_source: 4,
            max_prefetch_ids: 3,
            max_queue_fill_ratio: 0.75,
            prefer_session_local: true,
        }
    }
}

/// One learned next-step transition candidate.
#[derive(Debug, Clone, PartialEq)]
pub struct PredictedRecallCandidate {
    pub predicted_memory_id: MemoryId,
    pub support: u32,
    pub confidence: f32,
}

/// Stable explain surface for whether predictive work queued or observed only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredictivePrewarmAction {
    ObserveOnly,
    QueuePrefetch,
}

impl PredictivePrewarmAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ObserveOnly => "observe_only",
            Self::QueuePrefetch => "queue_prefetch",
        }
    }
}

/// One bounded speculative-prewarm decision.
#[derive(Debug, Clone, PartialEq)]
pub struct PredictivePrewarmDecision {
    pub action: PredictivePrewarmAction,
    pub trigger: &'static str,
    pub reason: &'static str,
    pub strongest_confidence: Option<f32>,
    pub submitted_predictions: Vec<PredictedRecallCandidate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TransitionKey {
    session_id: Option<SessionId>,
    from_memory_id: MemoryId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LearnedTransition {
    to_memory_id: MemoryId,
    support: u32,
}

/// Learned recall-sequence tracker derived from bounded audit history.
#[derive(Debug, Clone, PartialEq)]
pub struct PredictiveRecallEngine {
    config: PredictiveRecallConfig,
    namespace: NamespaceId,
    transitions: HashMap<TransitionKey, Vec<LearnedTransition>>,
}

impl PredictiveRecallEngine {
    /// Learns bounded recall-to-recall transitions from retained audit history for one namespace.
    pub fn from_audit(
        audit_log: &AppendOnlyAuditLog,
        namespace: NamespaceId,
        config: PredictiveRecallConfig,
    ) -> Self {
        let mut counts = HashMap::<TransitionKey, HashMap<MemoryId, u32>>::new();
        let mut previous_by_session = HashMap::<Option<SessionId>, MemoryId>::new();
        let mut namespace_previous = None::<MemoryId>;

        for entry in audit_log.entries_for_namespace(&namespace) {
            if entry.kind != crate::observability::AuditEventKind::RecallServed {
                continue;
            }
            let Some(memory_id) = entry.memory_id else {
                continue;
            };
            let session_id = entry.session_id;

            if let Some(previous) = previous_by_session.insert(session_id, memory_id) {
                let key = TransitionKey {
                    session_id,
                    from_memory_id: previous,
                };
                *counts.entry(key).or_default().entry(memory_id).or_default() += 1;
            }

            if let Some(previous) = namespace_previous.replace(memory_id) {
                let key = TransitionKey {
                    session_id: None,
                    from_memory_id: previous,
                };
                *counts.entry(key).or_default().entry(memory_id).or_default() += 1;
            }
        }

        let transitions = counts
            .into_iter()
            .map(|(key, by_target)| {
                let mut learned = by_target
                    .into_iter()
                    .map(|(to_memory_id, support)| LearnedTransition {
                        to_memory_id,
                        support,
                    })
                    .collect::<Vec<_>>();
                learned.sort_by(|left, right| {
                    right
                        .support
                        .cmp(&left.support)
                        .then_with(|| left.to_memory_id.0.cmp(&right.to_memory_id.0))
                });
                learned.truncate(config.max_predictions_per_source.max(1));
                (key, learned)
            })
            .collect();

        Self {
            config,
            namespace,
            transitions,
        }
    }

    /// Returns bounded next-step predictions for one prior recalled memory.
    pub fn predict_next(
        &self,
        previous_memory_id: MemoryId,
        session_id: Option<SessionId>,
    ) -> Vec<PredictedRecallCandidate> {
        let scoped = self
            .config
            .prefer_session_local
            .then(|| TransitionKey {
                session_id,
                from_memory_id: previous_memory_id,
            })
            .and_then(|key| self.transitions.get(&key));
        let fallback = self.transitions.get(&TransitionKey {
            session_id: None,
            from_memory_id: previous_memory_id,
        });

        let learned = scoped.or(fallback).cloned().unwrap_or_default();
        let total_support = learned.iter().map(|item| item.support).sum::<u32>().max(1);

        learned
            .into_iter()
            .map(|item| PredictedRecallCandidate {
                predicted_memory_id: item.to_memory_id,
                support: item.support,
                confidence: item.support as f32 / total_support as f32,
            })
            .collect()
    }

    /// Queues one bounded speculative prewarm based on learned recall transitions.
    pub fn speculative_prewarm(
        &self,
        cache: &mut CacheManager,
        previous_memory_id: MemoryId,
        session_id: Option<SessionId>,
    ) -> PredictivePrewarmDecision {
        let predicted = self.predict_next(previous_memory_id, session_id);
        if predicted.is_empty() {
            return PredictivePrewarmDecision {
                action: PredictivePrewarmAction::ObserveOnly,
                trigger: "session_recency",
                reason: "no_transition_history",
                strongest_confidence: None,
                submitted_predictions: Vec::new(),
            };
        }

        let strongest = predicted.first().map(|candidate| candidate.confidence);
        let strongest_support = predicted
            .first()
            .map(|candidate| candidate.support)
            .unwrap_or(0);
        if strongest_support < self.config.min_transition_support {
            return PredictivePrewarmDecision {
                action: PredictivePrewarmAction::ObserveOnly,
                trigger: "session_recency",
                reason: "insufficient_transition_support",
                strongest_confidence: strongest,
                submitted_predictions: Vec::new(),
            };
        }

        if strongest.unwrap_or_default() < self.config.min_prediction_confidence {
            return PredictivePrewarmDecision {
                action: PredictivePrewarmAction::ObserveOnly,
                trigger: "session_recency",
                reason: "confidence_below_threshold",
                strongest_confidence: strongest,
                submitted_predictions: Vec::new(),
            };
        }

        if !cache.prefetch.is_enabled() {
            return PredictivePrewarmDecision {
                action: PredictivePrewarmAction::ObserveOnly,
                trigger: "session_recency",
                reason: "prefetch_disabled",
                strongest_confidence: strongest,
                submitted_predictions: Vec::new(),
            };
        }

        let capacity = cache.prefetch.capacity().max(1);
        let queue_depth = cache.prefetch.queue_depth();
        let fill_ratio = queue_depth as f32 / capacity as f32;
        if fill_ratio >= self.config.max_queue_fill_ratio {
            return PredictivePrewarmDecision {
                action: PredictivePrewarmAction::ObserveOnly,
                trigger: "session_recency",
                reason: "predictive_budget_constrained",
                strongest_confidence: strongest,
                submitted_predictions: Vec::new(),
            };
        }

        let submitted_predictions = predicted
            .into_iter()
            .take(self.config.max_prefetch_ids.max(1))
            .collect::<Vec<_>>();
        let predicted_ids = submitted_predictions
            .iter()
            .map(|candidate| candidate.predicted_memory_id)
            .collect::<Vec<_>>();
        if predicted_ids.is_empty() {
            return PredictivePrewarmDecision {
                action: PredictivePrewarmAction::ObserveOnly,
                trigger: "session_recency",
                reason: "no_eligible_predictions",
                strongest_confidence: strongest,
                submitted_predictions: Vec::new(),
            };
        }

        let submitted = cache.prefetch.submit_hint(
            self.namespace.clone(),
            PrefetchTrigger::SessionRecency,
            predicted_ids,
        );
        if !submitted {
            return PredictivePrewarmDecision {
                action: PredictivePrewarmAction::ObserveOnly,
                trigger: "session_recency",
                reason: "prefetch_submit_rejected",
                strongest_confidence: strongest,
                submitted_predictions: Vec::new(),
            };
        }

        PredictivePrewarmDecision {
            action: PredictivePrewarmAction::QueuePrefetch,
            trigger: "session_recency",
            reason: "bounded_predictive_sequence_hit",
            strongest_confidence: strongest,
            submitted_predictions,
        }
    }

    /// Returns how many learned source states remain after bounded compaction.
    pub fn learned_source_count(&self) -> usize {
        self.transitions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::{PredictivePrewarmAction, PredictiveRecallConfig, PredictiveRecallEngine};
    use crate::api::NamespaceId;
    use crate::observability::{AuditEventCategory, AuditEventKind};
    use crate::store::audit::{AppendOnlyAuditLog, AuditLogEntry};
    use crate::store::cache::CacheManager;
    use crate::types::{MemoryId, SessionId};

    fn ns(value: &str) -> NamespaceId {
        NamespaceId::new(value).unwrap()
    }

    fn recall_entry(
        namespace: &NamespaceId,
        memory_id: u64,
        session_id: Option<u64>,
        tick: u64,
    ) -> AuditLogEntry {
        AuditLogEntry::new(
            AuditEventCategory::Recall,
            AuditEventKind::RecallServed,
            namespace.clone(),
            "predictive_test",
            format!("recall memory={memory_id}"),
        )
        .with_memory_id(MemoryId(memory_id))
        .with_session_id(session_id.map(SessionId))
        .with_tick(tick)
    }

    #[test]
    fn learns_bounded_session_local_transition_sequences() {
        let namespace = ns("team.alpha");
        let mut audit = AppendOnlyAuditLog::new(64);
        audit.append(recall_entry(&namespace, 10, Some(7), 1));
        audit.append(recall_entry(&namespace, 11, Some(7), 2));
        audit.append(recall_entry(&namespace, 10, Some(7), 3));
        audit.append(recall_entry(&namespace, 11, Some(7), 4));
        audit.append(recall_entry(&namespace, 10, Some(7), 5));
        audit.append(recall_entry(&namespace, 12, Some(7), 6));
        audit.append(recall_entry(&namespace, 99, Some(9), 7));

        let engine = PredictiveRecallEngine::from_audit(
            &audit,
            namespace,
            PredictiveRecallConfig {
                max_predictions_per_source: 2,
                ..PredictiveRecallConfig::default()
            },
        );
        let predictions = engine.predict_next(MemoryId(10), Some(SessionId(7)));

        assert_eq!(predictions.len(), 2);
        assert_eq!(predictions[0].predicted_memory_id, MemoryId(11));
        assert_eq!(predictions[0].support, 2);
        assert!(predictions[0].confidence > predictions[1].confidence);
        assert_eq!(predictions[1].predicted_memory_id, MemoryId(12));
        assert_eq!(engine.learned_source_count(), 5);
    }

    #[test]
    fn falls_back_to_namespace_history_when_session_specific_history_is_missing() {
        let namespace = ns("team.beta");
        let mut audit = AppendOnlyAuditLog::new(64);
        audit.append(recall_entry(&namespace, 20, Some(1), 1));
        audit.append(recall_entry(&namespace, 21, Some(1), 2));
        audit.append(recall_entry(&namespace, 20, Some(2), 3));
        audit.append(recall_entry(&namespace, 21, Some(2), 4));

        let engine = PredictiveRecallEngine::from_audit(
            &audit,
            namespace,
            PredictiveRecallConfig::default(),
        );
        let predictions = engine.predict_next(MemoryId(20), Some(SessionId(99)));

        assert_eq!(predictions.len(), 1);
        assert_eq!(predictions[0].predicted_memory_id, MemoryId(21));
        assert_eq!(predictions[0].support, 2);
        assert!((predictions[0].confidence - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn queues_bounded_prefetch_when_predictions_clear_support_and_confidence_thresholds() {
        let namespace = ns("team.gamma");
        let mut audit = AppendOnlyAuditLog::new(64);
        audit.append(recall_entry(&namespace, 30, Some(5), 1));
        audit.append(recall_entry(&namespace, 31, Some(5), 2));
        audit.append(recall_entry(&namespace, 30, Some(5), 3));
        audit.append(recall_entry(&namespace, 31, Some(5), 4));
        audit.append(recall_entry(&namespace, 30, Some(5), 5));
        audit.append(recall_entry(&namespace, 31, Some(5), 6));
        audit.append(recall_entry(&namespace, 30, Some(5), 7));
        audit.append(recall_entry(&namespace, 32, Some(5), 8));

        let engine = PredictiveRecallEngine::from_audit(
            &audit,
            namespace.clone(),
            PredictiveRecallConfig {
                min_transition_support: 2,
                min_prediction_confidence: 0.60,
                max_prefetch_ids: 2,
                ..PredictiveRecallConfig::default()
            },
        );
        let mut cache = CacheManager::new(4, 8);

        let decision = engine.speculative_prewarm(&mut cache, MemoryId(30), Some(SessionId(5)));

        assert_eq!(decision.action, PredictivePrewarmAction::QueuePrefetch);
        assert_eq!(decision.reason, "bounded_predictive_sequence_hit");
        assert_eq!(cache.prefetch.queue_depth(), 1);
        let consumed = cache
            .prefetch
            .consume_hint(&namespace)
            .expect("prefetch hint");
        assert_eq!(consumed.predicted_ids, vec![MemoryId(31), MemoryId(32)]);
    }

    #[test]
    fn stays_observe_only_when_queue_is_already_constrained() {
        let namespace = ns("team.delta");
        let mut audit = AppendOnlyAuditLog::new(64);
        audit.append(recall_entry(&namespace, 40, Some(2), 1));
        audit.append(recall_entry(&namespace, 41, Some(2), 2));
        audit.append(recall_entry(&namespace, 40, Some(2), 3));
        audit.append(recall_entry(&namespace, 41, Some(2), 4));

        let engine = PredictiveRecallEngine::from_audit(
            &audit,
            namespace.clone(),
            PredictiveRecallConfig {
                min_transition_support: 2,
                min_prediction_confidence: 0.75,
                max_queue_fill_ratio: 0.50,
                ..PredictiveRecallConfig::default()
            },
        );
        let mut cache = CacheManager::new(4, 2);
        assert!(cache.prefetch.submit_hint(
            namespace.clone(),
            crate::store::cache::PrefetchTrigger::TaskIntent,
            vec![MemoryId(99)]
        ));

        let decision = engine.speculative_prewarm(&mut cache, MemoryId(40), Some(SessionId(2)));

        assert_eq!(decision.action, PredictivePrewarmAction::ObserveOnly);
        assert_eq!(decision.reason, "predictive_budget_constrained");
        assert_eq!(cache.prefetch.queue_depth(), 1);
    }
}
