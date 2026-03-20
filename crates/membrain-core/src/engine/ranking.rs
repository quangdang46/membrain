//! Ranking, score-fusion, and explainability surfaces.
//!
//! This module owns the scoring formulas that combine recency, salience,
//! relevance, and conflict signals into a final retrieval score. It
//! produces explain payloads that downstream surfaces can inspect.

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
    /// Query-match quality from retrieval (lexical + semantic).
    Relevance,
    /// Contradiction-aware penalty or boost.
    ConflictAdjustment,
    /// Access-frequency signal from recall history.
    AccessFrequency,
}

impl ScoreFamily {
    /// Stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Recency => "recency",
            Self::Salience => "salience",
            Self::Relevance => "relevance",
            Self::ConflictAdjustment => "conflict_adjustment",
            Self::AccessFrequency => "access_frequency",
        }
    }

    /// Default weight for this signal family (0..100).
    pub const fn default_weight(self) -> u8 {
        match self {
            Self::Relevance => 40,
            Self::Recency => 25,
            Self::Salience => 20,
            Self::ConflictAdjustment => 10,
            Self::AccessFrequency => 5,
        }
    }
}

// ── Ranking profile ──────────────────────────────────────────────────────────

/// Configurable weight profile for the score fusion formula.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RankingProfile {
    pub recency_weight: u8,
    pub salience_weight: u8,
    pub relevance_weight: u8,
    pub conflict_weight: u8,
    pub access_weight: u8,
}

impl RankingProfile {
    /// Default balanced profile summing to 100.
    pub const fn balanced() -> Self {
        Self {
            recency_weight: ScoreFamily::Recency.default_weight(),
            salience_weight: ScoreFamily::Salience.default_weight(),
            relevance_weight: ScoreFamily::Relevance.default_weight(),
            conflict_weight: ScoreFamily::ConflictAdjustment.default_weight(),
            access_weight: ScoreFamily::AccessFrequency.default_weight(),
        }
    }

    /// Recency-biased profile for "what happened recently" queries.
    pub const fn recency_biased() -> Self {
        Self {
            recency_weight: 50,
            salience_weight: 15,
            relevance_weight: 25,
            conflict_weight: 5,
            access_weight: 5,
        }
    }

    /// Relevance-biased profile for semantic search queries.
    pub const fn relevance_biased() -> Self {
        Self {
            recency_weight: 10,
            salience_weight: 15,
            relevance_weight: 60,
            conflict_weight: 10,
            access_weight: 5,
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
    /// Query relevance from retrieval matching (0..1000).
    pub relevance: u16,
    /// Conflict adjustment: positive boosts preferred, negative penalizes superseded.
    /// Stored as 500 = neutral, 0 = max penalty, 1000 = max boost.
    pub conflict: u16,
    /// Access frequency signal (0..1000).
    pub access: u16,
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
}

/// Fuses ranking signals into a final score using the given profile.
pub fn fuse_scores(input: RankingInput, profile: RankingProfile) -> RankingResult {
    let signals = vec![
        ScoreSignal::new("recency", input.recency, profile.recency_weight),
        ScoreSignal::new("salience", input.salience, profile.salience_weight),
        ScoreSignal::new("relevance", input.relevance, profile.relevance_weight),
        ScoreSignal::new(
            "conflict_adjustment",
            input.conflict,
            profile.conflict_weight,
        ),
        ScoreSignal::new("access_frequency", input.access, profile.access_weight),
    ];

    let total_weighted: u32 = signals.iter().map(|s| s.weighted_value).sum();
    // Normalize back to 0..1000 range
    let final_score = (total_weighted).min(1000) as u16;

    let profile_name = if profile == RankingProfile::balanced() {
        "balanced"
    } else if profile == RankingProfile::recency_biased() {
        "recency_biased"
    } else if profile == RankingProfile::relevance_biased() {
        "relevance_biased"
    } else {
        "custom"
    };

    RankingResult {
        final_score,
        signals,
        profile_name,
        contradiction_explains: Vec::new(),
    }
}

// ── Explain payload ──────────────────────────────────────────────────────────

/// Machine-readable ranking explain payload for retrieval result envelopes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RankingExplain {
    /// Final fused score.
    pub final_score: u16,
    /// Named signal breakdown.
    pub signal_breakdown: Vec<(ScoreFamily, u16, u8)>,
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
        let signal_breakdown = vec![
            (
                ScoreFamily::Recency,
                result.signals[0].raw_value,
                result.signals[0].weight,
            ),
            (
                ScoreFamily::Salience,
                result.signals[1].raw_value,
                result.signals[1].weight,
            ),
            (
                ScoreFamily::Relevance,
                result.signals[2].raw_value,
                result.signals[2].weight,
            ),
            (
                ScoreFamily::ConflictAdjustment,
                result.signals[3].raw_value,
                result.signals[3].weight,
            ),
            (
                ScoreFamily::AccessFrequency,
                result.signals[4].raw_value,
                result.signals[4].weight,
            ),
        ];

        Self {
            final_score: result.final_score,
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
            + p.relevance_weight as u16
            + p.conflict_weight as u16
            + p.access_weight as u16;
        assert_eq!(total, 100);
    }

    #[test]
    fn recency_biased_profile_weights_sum_to_100() {
        let p = RankingProfile::recency_biased();
        let total = p.recency_weight as u16
            + p.salience_weight as u16
            + p.relevance_weight as u16
            + p.conflict_weight as u16
            + p.access_weight as u16;
        assert_eq!(total, 100);
    }

    #[test]
    fn max_signals_produce_max_score() {
        let input = RankingInput {
            recency: 1000,
            salience: 1000,
            relevance: 1000,
            conflict: 1000,
            access: 1000,
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
            relevance: 0,
            conflict: 0,
            access: 0,
        };
        let result = fuse_scores(input, RankingProfile::balanced());
        assert_eq!(result.final_score, 0);
    }

    #[test]
    fn relevance_biased_profile_ranks_relevance_higher() {
        let high_relevance = RankingInput {
            recency: 200,
            salience: 200,
            relevance: 1000,
            conflict: 500,
            access: 100,
        };
        let high_recency = RankingInput {
            recency: 1000,
            salience: 200,
            relevance: 200,
            conflict: 500,
            access: 100,
        };

        let profile = RankingProfile::relevance_biased();
        let score_relevance = fuse_scores(high_relevance, profile).final_score;
        let score_recency = fuse_scores(high_recency, profile).final_score;

        assert!(score_relevance > score_recency);
    }

    #[test]
    fn explain_payload_reflects_ranking_result() {
        let input = RankingInput {
            recency: 800,
            salience: 600,
            relevance: 900,
            conflict: 500,
            access: 300,
        };
        let result = fuse_scores(input, RankingProfile::balanced());
        let explain = RankingExplain::from_result(&result);

        assert_eq!(explain.final_score, result.final_score);
        assert_eq!(explain.profile, "balanced");
        assert!(!explain.has_conflict);
        assert_eq!(explain.signal_breakdown.len(), 5);
        assert_eq!(explain.signal_breakdown[0].0, ScoreFamily::Recency);
    }

    #[test]
    fn ranking_engine_implements_runtime_trait() {
        let engine = RankingEngine;
        assert!(engine.packages_explainable_results());

        let input = RankingInput {
            recency: 500,
            salience: 500,
            relevance: 500,
            conflict: 500,
            access: 500,
        };
        let result = engine.rank_candidate(input, RankingProfile::balanced());
        assert_eq!(result.final_score, 500);
    }
}
