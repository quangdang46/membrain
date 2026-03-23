//! Confidence-interval storage and scoring inputs.
//!
//! Computes uncertainty scores from explicit evidence inputs rather than
//! treating confidence as a single opaque number. Each sub-component
//! (corroboration, freshness, contradiction, missing evidence) contributes
//! to a combined uncertainty score, with optional interval bounds for
//! high-stakes paths.

use crate::engine::contradiction::ResolutionState;

// ── Uncertainty bounds ───────────────────────────────────────────────────────

/// Confidence interval bounds for high-stakes retrieval paths.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UncertaintyBounds {
    /// Lower bound of the confidence interval (0..=1000).
    pub lower: u16,
    /// Upper bound of the confidence interval (0..=1000).
    pub upper: u16,
    /// Point estimate within the interval.
    pub point: u16,
}

impl UncertaintyBounds {
    /// Builds interval bounds from a point estimate and uncertainty spread.
    pub fn from_point_and_spread(point: u16, spread: u16) -> Self {
        let lower = point.saturating_sub(spread);
        let upper = (point + spread).min(1000);
        Self {
            lower,
            upper,
            point: point.min(1000),
        }
    }

    /// Returns whether the interval is degenerate (lower == upper).
    pub const fn is_degenerate(&self) -> bool {
        self.lower == self.upper
    }

    /// Returns the width of the interval.
    pub const fn width(&self) -> u16 {
        self.upper.saturating_sub(self.lower)
    }

    /// Returns whether the interval contains a given value.
    pub const fn contains(&self, value: u16) -> bool {
        value >= self.lower && value <= self.upper
    }
}

// ── Confidence inputs ────────────────────────────────────────────────────────

/// Evidence inputs used to compute a confidence score.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfidenceInputs {
    /// Number of memories that corroborate this belief.
    pub corroboration_count: u32,
    /// Number of ticks since last access.
    pub ticks_since_last_access: u64,
    /// Number of ticks since encoding.
    pub age_ticks: u64,
    /// Current contradiction state.
    pub resolution_state: ResolutionState,
    /// Conflict score from contradiction detection (0..1000).
    pub conflict_score: u16,
    /// Number of causal parents (provenance depth).
    pub causal_parent_count: u32,
    /// Source authoritativeness (0..1000).
    pub authoritativeness: u16,
    /// Number of times recalled.
    pub recall_count: u32,
}

impl ConfidenceInputs {
    /// Builds inputs for a fresh memory with no history.
    pub fn fresh() -> Self {
        Self {
            corroboration_count: 0,
            ticks_since_last_access: 0,
            age_ticks: 0,
            resolution_state: ResolutionState::None,
            conflict_score: 0,
            causal_parent_count: 0,
            authoritativeness: 500,
            recall_count: 0,
        }
    }
}

// ── Confidence output ────────────────────────────────────────────────────────

/// Computed confidence and uncertainty from evidence inputs.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfidenceOutput {
    /// Combined uncertainty score (0..1000, lower = more certain).
    pub uncertainty_score: u16,
    /// Uncertainty from lack of corroboration.
    pub corroboration_uncertainty: u16,
    /// Uncertainty from temporal staleness.
    pub freshness_uncertainty: u16,
    /// Uncertainty from active contradiction.
    pub contradiction_uncertainty: u16,
    /// Uncertainty from sparse causal links.
    pub missing_evidence_uncertainty: u16,
    /// Derived confidence (0..1000, higher = more confident).
    pub confidence: u16,
    /// Optional interval bounds for high-stakes paths.
    pub confidence_interval: Option<UncertaintyBounds>,
}

// ── Confidence policy ────────────────────────────────────────────────────────

/// Policy controlling how uncertainty components are weighted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfidencePolicy {
    /// Maximum ticks before freshness uncertainty reaches cap.
    pub freshness_decay_ticks: u64,
    /// Weight of corroboration in combined uncertainty (0..1).
    pub corroboration_weight: f32,
    /// Weight of freshness in combined uncertainty (0..1).
    pub freshness_weight: f32,
    /// Weight of contradiction in combined uncertainty (0..1).
    pub contradiction_weight: f32,
    /// Weight of missing evidence in combined uncertainty (0..1).
    pub missing_evidence_weight: f32,
    /// Whether to compute interval bounds (adds computation cost).
    pub compute_intervals: bool,
    /// Spread multiplier for interval computation.
    pub interval_spread_factor: f32,
}

impl Default for ConfidencePolicy {
    fn default() -> Self {
        Self {
            freshness_decay_ticks: 1000,
            corroboration_weight: 0.25,
            freshness_weight: 0.25,
            contradiction_weight: 0.30,
            missing_evidence_weight: 0.20,
            compute_intervals: true,
            interval_spread_factor: 0.15,
        }
    }
}

// ── Confidence engine ────────────────────────────────────────────────────────

/// Canonical confidence engine owned by the core crate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ConfidenceEngine;

impl ConfidenceEngine {
    /// Returns the stable component identifier.
    pub const fn component_name(&self) -> &'static str {
        "engine.confidence"
    }

    /// Computes confidence and uncertainty from evidence inputs.
    pub fn compute(
        &self,
        inputs: &ConfidenceInputs,
        policy: &ConfidencePolicy,
    ) -> ConfidenceOutput {
        let corroboration = self.corroboration_uncertainty(inputs.corroboration_count);
        let freshness = self.freshness_uncertainty(inputs.ticks_since_last_access, policy);
        let contradiction = self.contradiction_uncertainty(inputs);
        let missing_evidence = self.missing_evidence_uncertainty(inputs);

        let combined = self.combined_uncertainty(
            corroboration,
            freshness,
            contradiction,
            missing_evidence,
            policy,
        );

        let confidence = 1000u16.saturating_sub(combined);

        let interval = if policy.compute_intervals {
            let spread = (combined as f32 * policy.interval_spread_factor) as u16;
            Some(UncertaintyBounds::from_point_and_spread(confidence, spread))
        } else {
            None
        };

        ConfidenceOutput {
            uncertainty_score: combined,
            corroboration_uncertainty: corroboration,
            freshness_uncertainty: freshness,
            contradiction_uncertainty: contradiction,
            missing_evidence_uncertainty: missing_evidence,
            confidence,
            confidence_interval: interval,
        }
    }

    /// Computes corroboration uncertainty (0..1000).
    ///
    /// More corroborating memories reduce uncertainty. Caps at 1000 when
    /// there is zero corroboration.
    fn corroboration_uncertainty(&self, count: u32) -> u16 {
        match count {
            0 => 1000,
            1 => 700,
            2 => 500,
            3 => 350,
            4..=5 => 200,
            6..=10 => 100,
            _ => 50,
        }
    }

    /// Computes freshness uncertainty (0..1000).
    ///
    /// Uncertainty grows with time since last access, capped by the
    /// freshness_decay_ticks policy parameter.
    fn freshness_uncertainty(&self, ticks_since_access: u64, policy: &ConfidencePolicy) -> u16 {
        if policy.freshness_decay_ticks == 0 {
            return 0;
        }
        let ratio = ticks_since_access as f32 / policy.freshness_decay_ticks as f32;
        let capped = ratio.min(1.0);
        (capped * 1000.0) as u16
    }

    /// Computes contradiction uncertainty (0..1000).
    ///
    /// Active contradictions add uncertainty. Resolved contradictions
    /// reduce uncertainty based on confidence in the resolution.
    fn contradiction_uncertainty(&self, inputs: &ConfidenceInputs) -> u16 {
        match inputs.resolution_state {
            ResolutionState::None => 0,
            ResolutionState::Unresolved => {
                // Active contradiction: half the conflict score becomes uncertainty
                inputs.conflict_score / 2
            }
            ResolutionState::AutoResolved => {
                // Auto-resolved: low uncertainty from contradiction
                inputs.conflict_score / 10
            }
            ResolutionState::ManuallyResolved => {
                // Human resolution: very low residual uncertainty
                inputs.conflict_score / 20
            }
            ResolutionState::AuthoritativelyResolved => {
                // Authoritative: minimal residual uncertainty
                5
            }
        }
    }

    /// Computes missing evidence uncertainty (0..1000).
    ///
    /// Sparse causal links and low authoritativeness increase uncertainty.
    fn missing_evidence_uncertainty(&self, inputs: &ConfidenceInputs) -> u16 {
        let causal_uncertainty = match inputs.causal_parent_count {
            0 => 600,
            1 => 300,
            2..=3 => 150,
            _ => 50,
        };

        let auth_uncertainty = 1000u16.saturating_sub(inputs.authoritativeness);

        // Recall history reduces missing evidence uncertainty
        let recall_reduction = match inputs.recall_count {
            0 => 0,
            1..=2 => 100,
            3..=5 => 200,
            _ => 300,
        };

        let combined = (causal_uncertainty + auth_uncertainty) / 2;
        combined.saturating_sub(recall_reduction)
    }

    /// Combines uncertainty sub-components into a single score.
    fn combined_uncertainty(
        &self,
        corroboration: u16,
        freshness: u16,
        contradiction: u16,
        missing_evidence: u16,
        policy: &ConfidencePolicy,
    ) -> u16 {
        let total_weight = policy.corroboration_weight
            + policy.freshness_weight
            + policy.contradiction_weight
            + policy.missing_evidence_weight;

        if total_weight == 0.0 {
            return 0;
        }

        let weighted = corroboration as f32 * policy.corroboration_weight
            + freshness as f32 * policy.freshness_weight
            + contradiction as f32 * policy.contradiction_weight
            + missing_evidence as f32 * policy.missing_evidence_weight;

        (weighted / total_weight).min(1000.0) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_memory_has_baseline_confidence() {
        let engine = ConfidenceEngine;
        let policy = ConfidencePolicy::default();
        let inputs = ConfidenceInputs::fresh();

        let output = engine.compute(&inputs, &policy);
        // Fresh memory with no corroboration → high uncertainty
        assert!(output.uncertainty_score > 300);
        assert!(output.confidence < 700);
    }

    #[test]
    fn corroborated_memory_has_higher_confidence() {
        let engine = ConfidenceEngine;
        let policy = ConfidencePolicy::default();

        let mut inputs = ConfidenceInputs::fresh();
        inputs.corroboration_count = 0;
        let no_corrob = engine.compute(&inputs, &policy);

        inputs.corroboration_count = 5;
        let with_corrob = engine.compute(&inputs, &policy);

        assert!(with_corrob.confidence > no_corrob.confidence);
        assert!(with_corrob.corroboration_uncertainty < no_corrob.corroboration_uncertainty);
    }

    #[test]
    fn stale_memory_has_lower_confidence() {
        let engine = ConfidenceEngine;
        let policy = ConfidencePolicy::default();

        let mut inputs = ConfidenceInputs::fresh();
        inputs.ticks_since_last_access = 0;
        let fresh = engine.compute(&inputs, &policy);

        inputs.ticks_since_last_access = 500;
        let stale = engine.compute(&inputs, &policy);

        assert!(stale.freshness_uncertainty > fresh.freshness_uncertainty);
        assert!(stale.confidence < fresh.confidence);
    }

    #[test]
    fn unresolved_contradiction_adds_uncertainty() {
        let engine = ConfidenceEngine;
        let policy = ConfidencePolicy::default();

        let mut inputs = ConfidenceInputs::fresh();
        inputs.resolution_state = ResolutionState::None;
        let no_conflict = engine.compute(&inputs, &policy);

        inputs.resolution_state = ResolutionState::Unresolved;
        inputs.conflict_score = 800;
        let with_conflict = engine.compute(&inputs, &policy);

        assert!(with_conflict.contradiction_uncertainty > no_conflict.contradiction_uncertainty);
        assert!(with_conflict.uncertainty_score > no_conflict.uncertainty_score);
    }

    #[test]
    fn authoritative_resolution_minimizes_contradiction_uncertainty() {
        let engine = ConfidenceEngine;
        let policy = ConfidencePolicy::default();

        let mut inputs = ConfidenceInputs::fresh();
        inputs.resolution_state = ResolutionState::AuthoritativelyResolved;
        inputs.conflict_score = 900;

        let output = engine.compute(&inputs, &policy);
        assert_eq!(output.contradiction_uncertainty, 5);
    }

    #[test]
    fn confidence_interval_bounds_are_valid() {
        let engine = ConfidenceEngine;
        let policy = ConfidencePolicy::default();
        let inputs = ConfidenceInputs::fresh();

        let output = engine.compute(&inputs, &policy);
        let interval = output.confidence_interval.unwrap();

        assert!(interval.lower <= interval.point);
        assert!(interval.point <= interval.upper);
        assert!(interval.contains(interval.point));
        assert!(!interval.is_degenerate());
    }

    #[test]
    fn confidence_interval_disabled_by_policy() {
        let engine = ConfidenceEngine;
        let policy = ConfidencePolicy {
            compute_intervals: false,
            ..Default::default()
        };
        let inputs = ConfidenceInputs::fresh();

        let output = engine.compute(&inputs, &policy);
        assert!(output.confidence_interval.is_none());
    }

    #[test]
    fn uncertainty_bounds_from_point_and_spread() {
        let bounds = UncertaintyBounds::from_point_and_spread(500, 100);
        assert_eq!(bounds.lower, 400);
        assert_eq!(bounds.upper, 600);
        assert_eq!(bounds.point, 500);
        assert_eq!(bounds.width(), 200);
        assert!(bounds.contains(500));
        assert!(bounds.contains(400));
        assert!(bounds.contains(600));
        assert!(!bounds.contains(399));
        assert!(!bounds.contains(601));
    }

    #[test]
    fn uncertainty_bounds_saturate_at_edges() {
        let low = UncertaintyBounds::from_point_and_spread(50, 100);
        assert_eq!(low.lower, 0);
        assert_eq!(low.upper, 150);

        let high = UncertaintyBounds::from_point_and_spread(950, 100);
        assert_eq!(high.lower, 850);
        assert_eq!(high.upper, 1000);
    }

    #[test]
    fn uncertainty_bounds_degenerate_when_spread_zero() {
        let bounds = UncertaintyBounds::from_point_and_spread(500, 0);
        assert!(bounds.is_degenerate());
        assert_eq!(bounds.width(), 0);
    }

    #[test]
    fn missing_evidence_reduced_by_recall_history() {
        let engine = ConfidenceEngine;
        let policy = ConfidencePolicy::default();

        let mut inputs = ConfidenceInputs::fresh();
        inputs.recall_count = 0;
        let no_recall = engine.compute(&inputs, &policy);

        inputs.recall_count = 10;
        let recalled = engine.compute(&inputs, &policy);

        assert!(recalled.missing_evidence_uncertainty < no_recall.missing_evidence_uncertainty);
    }

    #[test]
    fn authoritativeness_reduces_missing_evidence_uncertainty() {
        let engine = ConfidenceEngine;
        let policy = ConfidencePolicy::default();

        let mut inputs = ConfidenceInputs::fresh();
        inputs.authoritativeness = 100;
        let low_auth = engine.compute(&inputs, &policy);

        inputs.authoritativeness = 900;
        let high_auth = engine.compute(&inputs, &policy);

        assert!(high_auth.confidence > low_auth.confidence);
    }

    #[test]
    fn component_name_is_stable() {
        let engine = ConfidenceEngine;
        assert_eq!(engine.component_name(), "engine.confidence");
    }
}
