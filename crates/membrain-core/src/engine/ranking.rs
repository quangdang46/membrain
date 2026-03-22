//! Ranking, score-fusion, and explainability surfaces.
//!
//! This module owns the scoring formulas that combine recency, salience,
//! strength, provenance, and contradiction state into a final retrieval
//! score. It produces explain payloads that downstream surfaces can inspect.

use crate::engine::contradiction::ContradictionExplain;

// ── Score components ─────────────────────────────────────────────────────────

/// Individual scoring signal carried through the ranking pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreSignal {
    /// Machine-readable signal name for explain surfaces.
    pub name: &'static str,
    /// Raw signal value (0..1000 fixed-point, 1000 = max).
    pub raw_value: u16,
    /// Weight applied to this signal in the final fusion (0..100).
    pub weight: u8,
    /// Weighted contribution after fusion (raw_value * weight / 100).
    pub weighted_value: u32,
}

impl ScoreSignal {
    /// Builds a new scoring signal and computes its weighted value.
    pub const fn new(name: &'static str, raw_value: u16, weight: u8) -> Self {
        Self {
            name,
            raw_value,
            weight,
            weighted_value: raw_value as u32 * weight as u32 / 100,
        }
    }
}

/// Stable scoring signal families used by the canonical ranking formula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ScoreFamily {
    /// Time-based decay: more recent = higher score.
    Recency,
    /// First-pass importance estimate from the encode path.
    Salience,
    /// Reinforced durability after recall and decay updates.
    Strength,
    /// Provenance confidence carried from source authoritativeness and lineage.
    Provenance,
    /// Contradiction-aware penalty or boost.
    ConflictAdjustment,
    /// Confidence-derived reliability signal from evidence inputs.
    Confidence,
}

impl ScoreFamily {
    /// Stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Recency => "recency",
            Self::Salience => "salience",
            Self::Strength => "strength",
            Self::Provenance => "provenance",
            Self::ConflictAdjustment => "conflict_adjustment",
            Self::Confidence => "confidence",
        }
    }

    /// Default weight for this signal family (0..100).
    pub const fn default_weight(self) -> u8 {
        match self {
            Self::Recency => 28,
            Self::Salience => 28,
            Self::Strength => 22,
            Self::Provenance => 8,
            Self::ConflictAdjustment => 4,
            Self::Confidence => 10,
        }
    }
}

// ── Ranking profile ──────────────────────────────────────────────────────────

/// Configurable weight profile for the score fusion formula.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RankingProfile {
    pub recency_weight: u8,
    pub salience_weight: u8,
    pub strength_weight: u8,
    pub provenance_weight: u8,
    pub conflict_weight: u8,
    pub confidence_weight: u8,
}

impl RankingProfile {
    /// Default balanced profile summing to 100.
    pub const fn balanced() -> Self {
        Self {
            recency_weight: ScoreFamily::Recency.default_weight(),
            salience_weight: ScoreFamily::Salience.default_weight(),
            strength_weight: ScoreFamily::Strength.default_weight(),
            provenance_weight: ScoreFamily::Provenance.default_weight(),
            conflict_weight: ScoreFamily::ConflictAdjustment.default_weight(),
            confidence_weight: ScoreFamily::Confidence.default_weight(),
        }
    }

    /// Recency-biased profile for "what happened recently" queries.
    pub const fn recency_biased() -> Self {
        Self {
            recency_weight: 40,
            salience_weight: 23,
            strength_weight: 15,
            provenance_weight: 8,
            conflict_weight: 4,
            confidence_weight: 10,
        }
    }

    /// Strength-biased profile for reinforced-memory queries.
    pub const fn strength_biased() -> Self {
        Self {
            recency_weight: 12,
            salience_weight: 18,
            strength_weight: 45,
            provenance_weight: 8,
            conflict_weight: 4,
            confidence_weight: 13,
        }
    }

    /// Confidence-biased profile for high-stakes queries where reliability matters most.
    pub const fn confidence_biased() -> Self {
        Self {
            recency_weight: 15,
            salience_weight: 15,
            strength_weight: 15,
            provenance_weight: 10,
            conflict_weight: 5,
            confidence_weight: 40,
        }
    }
}

impl Default for RankingProfile {
    fn default() -> Self {
        Self::balanced()
    }
}

// ── Score fusion ─────────────────────────────────────────────────────────────

/// Input signals for one candidate's ranking computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RankingInput {
    /// Recency score (0..1000, 1000 = just happened).
    pub recency: u16,
    /// Provisional salience from the encode path (0..1000).
    pub salience: u16,
    /// Reinforced durability after recall and decay updates (0..1000).
    pub strength: u16,
    /// Provenance confidence from source authoritativeness and lineage (0..1000).
    pub provenance: u16,
    /// Conflict adjustment: positive boosts preferred, negative penalizes superseded.
    /// Stored as 500 = neutral, 0 = max penalty, 1000 = max boost.
    pub conflict: u16,
    /// Confidence-derived reliability score (0..1000, higher = more certain).
    /// Computed from evidence inputs (corroboration, freshness, contradiction, provenance).
    pub confidence: u16,
}

/// Declared local reranker mode for the final bounded candidate slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LocalRerankerMode {
    /// Only the cheap float32 rescore ran.
    Disabled,
    /// A bounded local reranker is available for the final slice.
    Bounded,
}

impl LocalRerankerMode {
    /// Stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Bounded => "bounded",
        }
    }
}

/// Explainable rerank metadata preserved for inspect surfaces.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RerankMetadata {
    /// Authoritative float32 rescore candidate cap applied before the final cut.
    pub float32_rescore_limit: usize,
    /// Final candidate cut before optional local reranking.
    pub candidate_cut_limit: usize,
    /// Local reranker mode applied to the final slice.
    pub local_reranker_mode: LocalRerankerMode,
    /// Whether the optional local reranker actually ran.
    pub local_reranker_applied: bool,
    /// Bounded score delta introduced by reranking, when known.
    pub rerank_score_delta: i32,
}

impl RerankMetadata {
    /// Metadata for float32 rescore only.
    pub const fn float32_only(float32_rescore_limit: usize, candidate_cut_limit: usize) -> Self {
        Self {
            float32_rescore_limit,
            candidate_cut_limit,
            local_reranker_mode: LocalRerankerMode::Disabled,
            local_reranker_applied: false,
            rerank_score_delta: 0,
        }
    }
}

/// Fused ranking result with full explain trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankingResult {
    /// Final fused score (0..1000).
    pub final_score: u16,
    /// Individual signal contributions for explain surfaces.
    pub signals: Vec<ScoreSignal>,
    /// Profile used for this ranking.
    pub profile_name: &'static str,
    /// Contradiction explain payloads attached to this result.
    pub contradiction_explains: Vec<ContradictionExplain>,
    /// Bounded rerank metadata preserved for explain and inspect surfaces.
    pub rerank_metadata: RerankMetadata,
}

/// Fuses ranking signals into a final score using the given profile.
pub fn fuse_scores(input: RankingInput, profile: RankingProfile) -> RankingResult {
    let signals = vec![
        ScoreSignal::new("recency", input.recency, profile.recency_weight),
        ScoreSignal::new("salience", input.salience, profile.salience_weight),
        ScoreSignal::new("strength", input.strength, profile.strength_weight),
        ScoreSignal::new("provenance", input.provenance, profile.provenance_weight),
        ScoreSignal::new(
            "conflict_adjustment",
            input.conflict,
            profile.conflict_weight,
        ),
        ScoreSignal::new("confidence", input.confidence, profile.confidence_weight),
    ];

    let total_weighted: u32 = signals.iter().map(|s| s.weighted_value).sum();
    let final_score = total_weighted.min(1000) as u16;
    let rerank_limit = signals.len();

    let profile_name = if profile == RankingProfile::balanced() {
        "balanced"
    } else if profile == RankingProfile::recency_biased() {
        "recency_biased"
    } else if profile == RankingProfile::strength_biased() {
        "strength_biased"
    } else if profile == RankingProfile::confidence_biased() {
        "confidence_biased"
    } else {
        "custom"
    };

    RankingResult {
        final_score,
        signals,
        profile_name,
        contradiction_explains: Vec::new(),
        rerank_metadata: RerankMetadata::float32_only(rerank_limit, rerank_limit),
    }
}

// ── Explain payload ──────────────────────────────────────────────────────────

/// Machine-readable ranking explain payload for retrieval result envelopes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RankingExplain {
    /// Final fused score.
    pub final_score: u16,
    /// Sum of per-signal weighted contributions before final capping.
    pub total_weighted_score: u32,
    /// Named signal breakdown.
    pub signal_breakdown: Vec<(ScoreFamily, u16, u8, u32)>,
    /// Profile that was applied.
    pub profile: String,
    /// Whether the result was in a contradiction.
    pub has_conflict: bool,
    /// Contradiction-specific explanations.
    pub contradiction_details: Vec<ContradictionExplain>,
}

impl RankingExplain {
    /// Builds an explain payload from a ranking result.
    pub fn from_result(result: &RankingResult) -> Self {
        let signal_for = |family: ScoreFamily| {
            result
                .signals
                .iter()
                .find(|signal| signal.name == family.as_str())
                .map(|signal| {
                    (
                        family,
                        signal.raw_value,
                        signal.weight,
                        signal.weighted_value,
                    )
                })
                .unwrap_or((family, 0, family.default_weight(), 0))
        };

        let signal_breakdown = vec![
            signal_for(ScoreFamily::Recency),
            signal_for(ScoreFamily::Salience),
            signal_for(ScoreFamily::Strength),
            signal_for(ScoreFamily::Provenance),
            signal_for(ScoreFamily::ConflictAdjustment),
            signal_for(ScoreFamily::Confidence),
        ];

        Self {
            final_score: result.final_score,
            total_weighted_score: result
                .signals
                .iter()
                .map(|signal| signal.weighted_value)
                .sum(),
            signal_breakdown,
            profile: result.profile_name.to_string(),
            has_conflict: !result.contradiction_explains.is_empty(),
            contradiction_details: result.contradiction_explains.clone(),
        }
    }
}

// ── Confidence-aware display and filtering ────────────────────────────────────

/// Confidence interval data attached to retrieval explain outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConfidenceIntervalDisplay {
    /// Point estimate of confidence (0..1000).
    pub point: u16,
    /// Lower bound of the confidence interval.
    pub lower: u16,
    /// Upper bound of the confidence interval.
    pub upper: u16,
    /// Width of the interval (upper - lower).
    pub width: u16,
}

/// Configuration for confidence-aware retrieval filtering and display.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfidenceDisplayConfig {
    /// Minimum confidence score (0..1000) for a result to be included.
    /// Results below this threshold are suppressed from the output.
    pub min_confidence_threshold: u16,
    /// Whether to compute and display confidence intervals.
    pub show_confidence_intervals: bool,
    /// Whether to include uncertainty breakdown (corroboration, freshness, etc.) in explain.
    pub show_uncertainty_breakdown: bool,
    /// Whether to tag low-confidence results with a warning marker.
    pub tag_low_confidence: bool,
    /// Threshold below which results are tagged as "low_confidence" (0..1000).
    pub low_confidence_tag_threshold: u16,
}

impl Default for ConfidenceDisplayConfig {
    fn default() -> Self {
        Self {
            min_confidence_threshold: 100,
            show_confidence_intervals: true,
            show_uncertainty_breakdown: true,
            tag_low_confidence: true,
            low_confidence_tag_threshold: 400,
        }
    }
}

impl ConfidenceDisplayConfig {
    /// Permissive config that never filters and always shows everything.
    pub const fn permissive() -> Self {
        Self {
            min_confidence_threshold: 0,
            show_confidence_intervals: true,
            show_uncertainty_breakdown: true,
            tag_low_confidence: false,
            low_confidence_tag_threshold: 0,
        }
    }

    /// Strict config for high-stakes queries.
    pub const fn strict() -> Self {
        Self {
            min_confidence_threshold: 500,
            show_confidence_intervals: true,
            show_uncertainty_breakdown: true,
            tag_low_confidence: true,
            low_confidence_tag_threshold: 600,
        }
    }
}

// ── Ranking engine ───────────────────────────────────────────────────────────

/// Shared interface for ranking and packaging owned by `membrain-core`.
pub trait RankingRuntime {
    /// Returns whether this ranking surface packages explainable results.
    fn packages_explainable_results(&self) -> bool;

    /// Ranks a single candidate using the given profile.
    fn rank_candidate(&self, input: RankingInput, profile: RankingProfile) -> RankingResult;
}

/// Canonical ranking engine owned by the core crate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RankingEngine;

impl RankingRuntime for RankingEngine {
    fn packages_explainable_results(&self) -> bool {
        true
    }

    fn rank_candidate(&self, input: RankingInput, profile: RankingProfile) -> RankingResult {
        fuse_scores(input, profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balanced_profile_weights_sum_to_100() {
        let p = RankingProfile::balanced();
        let total = p.recency_weight as u16
            + p.salience_weight as u16
            + p.strength_weight as u16
            + p.provenance_weight as u16
            + p.conflict_weight as u16;
        assert_eq!(total, 100);
    }

    #[test]
    fn recency_biased_profile_weights_sum_to_100() {
        let p = RankingProfile::recency_biased();
        let total = p.recency_weight as u16
            + p.salience_weight as u16
            + p.strength_weight as u16
            + p.provenance_weight as u16
            + p.conflict_weight as u16;
        assert_eq!(total, 100);
    }

    #[test]
    fn strength_biased_profile_weights_sum_to_100() {
        let p = RankingProfile::strength_biased();
        let total = p.recency_weight as u16
            + p.salience_weight as u16
            + p.strength_weight as u16
            + p.provenance_weight as u16
            + p.conflict_weight as u16;
        assert_eq!(total, 100);
    }

    #[test]
    fn max_signals_produce_max_score() {
        let input = RankingInput {
            recency: 1000,
            salience: 1000,
            strength: 1000,
            provenance: 1000,
            conflict: 1000,
        };
        let result = fuse_scores(input, RankingProfile::balanced());
        assert_eq!(result.final_score, 1000);
        assert_eq!(result.profile_name, "balanced");
    }

    #[test]
    fn zero_signals_produce_zero_score() {
        let input = RankingInput {
            recency: 0,
            salience: 0,
            strength: 0,
            provenance: 0,
            conflict: 0,
        };
        let result = fuse_scores(input, RankingProfile::balanced());
        assert_eq!(result.final_score, 0);
    }

    #[test]
    fn strength_biased_profile_ranks_strength_higher() {
        let high_strength = RankingInput {
            recency: 200,
            salience: 200,
            strength: 1000,
            provenance: 300,
            conflict: 500,
        };
        let high_recency = RankingInput {
            recency: 1000,
            salience: 200,
            strength: 200,
            provenance: 300,
            conflict: 500,
        };

        let profile = RankingProfile::strength_biased();
        let score_strength = fuse_scores(high_strength, profile).final_score;
        let score_recency = fuse_scores(high_recency, profile).final_score;

        assert!(score_strength > score_recency);
    }

    #[test]
    fn explain_payload_reflects_ranking_result() {
        let input = RankingInput {
            recency: 800,
            salience: 600,
            strength: 900,
            provenance: 700,
            conflict: 500,
        };
        let result = fuse_scores(input, RankingProfile::balanced());
        let explain = RankingExplain::from_result(&result);

        assert_eq!(explain.final_score, result.final_score);
        assert_eq!(explain.profile, "balanced");
        assert!(!explain.has_conflict);
        assert_eq!(
            explain.total_weighted_score,
            result
                .signals
                .iter()
                .map(|signal| signal.weighted_value)
                .sum::<u32>()
        );
        assert_eq!(explain.signal_breakdown.len(), 5);
        assert_eq!(explain.signal_breakdown[0].0, ScoreFamily::Recency);
        assert_eq!(
            explain.signal_breakdown[0].3,
            result.signals[0].weighted_value
        );
    }

    #[test]
    fn explain_payload_maps_signals_by_name_instead_of_position() {
        let result = RankingResult {
            final_score: 777,
            signals: vec![
                ScoreSignal::new("strength", 900, 50),
                ScoreSignal::new("provenance", 300, 10),
                ScoreSignal::new("recency", 800, 15),
            ],
            profile_name: "custom",
            contradiction_explains: Vec::new(),
            rerank_metadata: RerankMetadata::float32_only(3, 3),
        };

        let explain = RankingExplain::from_result(&result);

        assert_eq!(explain.signal_breakdown.len(), 5);
        assert_eq!(explain.total_weighted_score, 600);
        assert_eq!(
            explain.signal_breakdown[0],
            (ScoreFamily::Recency, 800, 15, 120)
        );
        assert_eq!(
            explain.signal_breakdown[1],
            (ScoreFamily::Salience, 0, 30, 0)
        );
        assert_eq!(
            explain.signal_breakdown[2],
            (ScoreFamily::Strength, 900, 50, 450)
        );
        assert_eq!(
            explain.signal_breakdown[3],
            (ScoreFamily::Provenance, 300, 10, 30)
        );
        assert_eq!(
            explain.signal_breakdown[4],
            (ScoreFamily::ConflictAdjustment, 0, 5, 0)
        );
    }

    #[test]
    fn ranking_engine_implements_runtime_trait() {
        let engine = RankingEngine;
        assert!(engine.packages_explainable_results());

        let input = RankingInput {
            recency: 500,
            salience: 500,
            strength: 500,
            provenance: 500,
            conflict: 500,
        };
        let result = engine.rank_candidate(input, RankingProfile::balanced());
        assert_eq!(result.final_score, 500);
        assert_eq!(
            result.rerank_metadata.local_reranker_mode.as_str(),
            "disabled"
        );
        assert!(!result.rerank_metadata.local_reranker_applied);
    }

    #[test]
    fn float32_only_rerank_metadata_defaults_to_no_local_reranker() {
        let metadata = RerankMetadata::float32_only(20, 8);

        assert_eq!(metadata.float32_rescore_limit, 20);
        assert_eq!(metadata.candidate_cut_limit, 8);
        assert_eq!(metadata.local_reranker_mode, LocalRerankerMode::Disabled);
        assert!(!metadata.local_reranker_applied);
        assert_eq!(metadata.rerank_score_delta, 0);
    }

    // ── Calibration fixtures and score-breakdown artifacts ────────────────

    /// A single calibration input with a stable label for regression detection.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CalibrationFixture {
        /// Stable human-readable label for this fixture.
        pub label: &'static str,
        /// Ranking input signals.
        pub input: RankingInput,
    }

    /// Expected ordering invariant for a set of calibration fixtures.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct OrderingInvariant {
        /// Label of the item expected to rank higher.
        pub higher: &'static str,
        /// Label of the item expected to rank lower.
        pub lower: &'static str,
    }

    /// Score breakdown for one candidate captured in a calibration artifact.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ScoreBreakdownArtifact {
        /// Label of the candidate.
        pub label: &'static str,
        /// Final fused score.
        pub final_score: u16,
        /// Per-signal weighted contributions in order.
        pub signal_contributions: Vec<(&'static str, u16, u8, u32)>,
    }

    /// Calibration artifact produced by running fixtures through a profile.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CalibrationArtifact {
        /// Profile name used for this calibration run.
        pub profile_name: &'static str,
        /// Score breakdowns ordered by descending final score.
        pub ranked_candidates: Vec<ScoreBreakdownArtifact>,
        /// Whether all ordering invariants held.
        pub ordering_invariants_satisfied: bool,
        /// Total weighted score sum before capping.
        pub total_weighted_sum: u32,
    }

    /// Representative calibration fixtures covering distinct signal patterns.
    pub fn calibration_fixtures() -> Vec<CalibrationFixture> {
        vec![
            CalibrationFixture {
                label: "recent_low_strength",
                input: RankingInput {
                    recency: 900,
                    salience: 300,
                    strength: 200,
                    provenance: 500,
                    conflict: 500,
                },
            },
            CalibrationFixture {
                label: "strong_old",
                input: RankingInput {
                    recency: 100,
                    salience: 400,
                    strength: 950,
                    provenance: 600,
                    conflict: 500,
                },
            },
            CalibrationFixture {
                label: "high_salience_med",
                input: RankingInput {
                    recency: 500,
                    salience: 950,
                    strength: 500,
                    provenance: 400,
                    conflict: 500,
                },
            },
            CalibrationFixture {
                label: "provenance_heavy",
                input: RankingInput {
                    recency: 300,
                    salience: 300,
                    strength: 300,
                    provenance: 950,
                    conflict: 500,
                },
            },
            CalibrationFixture {
                label: "conflict_penalized",
                input: RankingInput {
                    recency: 100,
                    salience: 100,
                    strength: 100,
                    provenance: 100,
                    conflict: 0,
                },
            },
            CalibrationFixture {
                label: "all_mid",
                input: RankingInput {
                    recency: 500,
                    salience: 500,
                    strength: 500,
                    provenance: 500,
                    conflict: 500,
                },
            },
            CalibrationFixture {
                label: "zero_signals",
                input: RankingInput {
                    recency: 0,
                    salience: 0,
                    strength: 0,
                    provenance: 0,
                    conflict: 0,
                },
            },
            CalibrationFixture {
                label: "max_signals",
                input: RankingInput {
                    recency: 1000,
                    salience: 1000,
                    strength: 1000,
                    provenance: 1000,
                    conflict: 1000,
                },
            },
        ]
    }

    /// Ordering invariants that must hold for the balanced profile.
    pub fn balanced_ordering_invariants() -> Vec<OrderingInvariant> {
        vec![
            OrderingInvariant {
                higher: "max_signals",
                lower: "all_mid",
            },
            OrderingInvariant {
                higher: "all_mid",
                lower: "zero_signals",
            },
            OrderingInvariant {
                higher: "recent_low_strength",
                lower: "conflict_penalized",
            },
        ]
    }

    /// Runs calibration fixtures through a profile and produces an artifact.
    pub fn run_calibration(
        fixtures: &[CalibrationFixture],
        profile: RankingProfile,
        profile_name: &'static str,
        invariants: &[OrderingInvariant],
    ) -> CalibrationArtifact {
        let mut scored: Vec<(CalibrationFixture, RankingResult)> = fixtures
            .iter()
            .map(|f| (f.clone(), fuse_scores(f.input, profile)))
            .collect();

        scored.sort_by(|a, b| b.1.final_score.cmp(&a.1.final_score));

        let ranked_candidates = scored
            .iter()
            .map(|(fixture, result)| ScoreBreakdownArtifact {
                label: fixture.label,
                final_score: result.final_score,
                signal_contributions: result
                    .signals
                    .iter()
                    .map(|s| (s.name, s.raw_value, s.weight, s.weighted_value))
                    .collect(),
            })
            .collect();

        let label_to_score: std::collections::HashMap<&str, u16> = scored
            .iter()
            .map(|(f, r)| (f.label, r.final_score))
            .collect();

        let ordering_invariants_satisfied = invariants.iter().all(|inv| {
            let higher_score = label_to_score.get(inv.higher).copied().unwrap_or(0);
            let lower_score = label_to_score.get(inv.lower).copied().unwrap_or(0);
            higher_score >= lower_score
        });

        let total_weighted_sum = scored
            .iter()
            .map(|(_, r)| r.signals.iter().map(|s| s.weighted_value).sum::<u32>())
            .sum();

        CalibrationArtifact {
            profile_name,
            ranked_candidates,
            ordering_invariants_satisfied,
            total_weighted_sum,
        }
    }

    #[test]
    fn calibration_fixtures_cover_eight_signal_patterns() {
        let fixtures = calibration_fixtures();
        assert_eq!(fixtures.len(), 8);
        let labels: Vec<&str> = fixtures.iter().map(|f| f.label).collect();
        assert!(labels.contains(&"recent_low_strength"));
        assert!(labels.contains(&"strong_old"));
        assert!(labels.contains(&"max_signals"));
        assert!(labels.contains(&"zero_signals"));
    }

    #[test]
    fn calibration_artifact_ranked_descending() {
        let fixtures = calibration_fixtures();
        let invariants = balanced_ordering_invariants();
        let artifact = run_calibration(
            &fixtures,
            RankingProfile::balanced(),
            "balanced",
            &invariants,
        );

        assert_eq!(artifact.profile_name, "balanced");
        assert_eq!(artifact.ranked_candidates.len(), 8);
        // Verify descending order.
        for window in artifact.ranked_candidates.windows(2) {
            assert!(
                window[0].final_score >= window[1].final_score,
                "{} ({}) should >= {} ({})",
                window[0].label,
                window[0].final_score,
                window[1].label,
                window[1].final_score,
            );
        }
    }

    #[test]
    fn calibration_ordering_invariants_hold_for_balanced() {
        let fixtures = calibration_fixtures();
        let invariants = balanced_ordering_invariants();
        let artifact = run_calibration(
            &fixtures,
            RankingProfile::balanced(),
            "balanced",
            &invariants,
        );
        assert!(artifact.ordering_invariants_satisfied);
    }

    #[test]
    fn calibration_max_beats_mid_beats_zero() {
        let fixtures = calibration_fixtures();
        let invariants = balanced_ordering_invariants();
        let artifact = run_calibration(
            &fixtures,
            RankingProfile::balanced(),
            "balanced",
            &invariants,
        );

        let max = artifact
            .ranked_candidates
            .iter()
            .find(|c| c.label == "max_signals")
            .unwrap();
        let mid = artifact
            .ranked_candidates
            .iter()
            .find(|c| c.label == "all_mid")
            .unwrap();
        let zero = artifact
            .ranked_candidates
            .iter()
            .find(|c| c.label == "zero_signals")
            .unwrap();

        assert_eq!(max.final_score, 1000);
        assert_eq!(mid.final_score, 500);
        assert_eq!(zero.final_score, 0);
    }

    #[test]
    fn calibration_score_breakdown_shows_signal_contributions() {
        let fixtures = calibration_fixtures();
        let invariants = balanced_ordering_invariants();
        let artifact = run_calibration(
            &fixtures,
            RankingProfile::balanced(),
            "balanced",
            &invariants,
        );

        let all_mid = artifact
            .ranked_candidates
            .iter()
            .find(|c| c.label == "all_mid")
            .unwrap();

        assert_eq!(all_mid.signal_contributions.len(), 5);
        // Each signal: raw=500, weight varies, weighted = raw * weight / 100
        let recency_contrib = all_mid.signal_contributions[0];
        assert_eq!(recency_contrib.0, "recency");
        assert_eq!(recency_contrib.1, 500);
        assert_eq!(recency_contrib.2, 30); // default weight
        assert_eq!(recency_contrib.3, 150); // 500 * 30 / 100
    }

    #[test]
    fn calibration_recency_biased_ranks_recent_higher_than_balanced() {
        let fixtures = calibration_fixtures();
        let invariants = vec![];

        let balanced_artifact = run_calibration(
            &fixtures,
            RankingProfile::balanced(),
            "balanced",
            &invariants,
        );
        let recency_artifact = run_calibration(
            &fixtures,
            RankingProfile::recency_biased(),
            "recency_biased",
            &invariants,
        );

        let balanced_recent = balanced_artifact
            .ranked_candidates
            .iter()
            .find(|c| c.label == "recent_low_strength")
            .unwrap();
        let recency_recent = recency_artifact
            .ranked_candidates
            .iter()
            .find(|c| c.label == "recent_low_strength")
            .unwrap();

        // With recency bias, "recent_low_strength" should score higher.
        assert!(recency_recent.final_score > balanced_recent.final_score);
    }

    #[test]
    fn calibration_strength_biased_ranks_strong_higher_than_balanced() {
        let fixtures = calibration_fixtures();
        let invariants = vec![];

        let balanced_artifact = run_calibration(
            &fixtures,
            RankingProfile::balanced(),
            "balanced",
            &invariants,
        );
        let strength_artifact = run_calibration(
            &fixtures,
            RankingProfile::strength_biased(),
            "strength_biased",
            &invariants,
        );

        let balanced_strong = balanced_artifact
            .ranked_candidates
            .iter()
            .find(|c| c.label == "strong_old")
            .unwrap();
        let strength_strong = strength_artifact
            .ranked_candidates
            .iter()
            .find(|c| c.label == "strong_old")
            .unwrap();

        assert!(strength_strong.final_score > balanced_strong.final_score);
    }

    #[test]
    fn calibration_conflict_penalized_scores_below_all_mid() {
        let fixtures = calibration_fixtures();
        let invariants = balanced_ordering_invariants();
        let artifact = run_calibration(
            &fixtures,
            RankingProfile::balanced(),
            "balanced",
            &invariants,
        );

        let conflict = artifact
            .ranked_candidates
            .iter()
            .find(|c| c.label == "conflict_penalized")
            .unwrap();
        let mid = artifact
            .ranked_candidates
            .iter()
            .find(|c| c.label == "all_mid")
            .unwrap();

        // conflict_penalized has high signals but conflict=100, should score below all_mid
        assert!(conflict.final_score < mid.final_score);
    }
}
