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
        }
    }

    /// Default weight for this signal family (0..100).
    pub const fn default_weight(self) -> u8 {
        match self {
            Self::Recency => 30,
            Self::Salience => 30,
            Self::Strength => 25,
            Self::Provenance => 10,
            Self::ConflictAdjustment => 5,
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
        }
    }

    /// Recency-biased profile for "what happened recently" queries.
    pub const fn recency_biased() -> Self {
        Self {
            recency_weight: 45,
            salience_weight: 25,
            strength_weight: 15,
            provenance_weight: 10,
            conflict_weight: 5,
        }
    }

    /// Strength-biased profile for reinforced-memory queries.
    pub const fn strength_biased() -> Self {
        Self {
            recency_weight: 15,
            salience_weight: 20,
            strength_weight: 50,
            provenance_weight: 10,
            conflict_weight: 5,
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
}
