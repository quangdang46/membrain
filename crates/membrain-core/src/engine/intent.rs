use crate::engine::ranking::RankingProfile;
use crate::engine::retrieval_planner::QueryPath;

/// Stable visible query-intent taxonomy for ask-style routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryIntent {
    SemanticBroad,
    ExistenceCheck,
    RecentFirst,
    StrengthWeighted,
    UncertaintyFocused,
    CausalTrace,
    TemporalAnchor,
    DiverseSample,
    ProceduralLookup,
    EmotionalFilter,
}

impl QueryIntent {
    /// Returns the stable machine-readable intent class name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SemanticBroad => "semantic_broad",
            Self::ExistenceCheck => "existence_check",
            Self::RecentFirst => "recent_first",
            Self::StrengthWeighted => "strength_weighted",
            Self::UncertaintyFocused => "uncertainty_focused",
            Self::CausalTrace => "causal_trace",
            Self::TemporalAnchor => "temporal_anchor",
            Self::DiverseSample => "diverse_sample",
            Self::ProceduralLookup => "procedural_lookup",
            Self::EmotionalFilter => "emotional_filter",
        }
    }
}

/// Stable machine-readable signal families consulted during shallow intent classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntentSignalKind {
    Phrase,
    Token,
    Temporal,
    Emotional,
    QuestionForm,
    Fallback,
}

impl IntentSignalKind {
    /// Returns the stable machine-readable signal family name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Phrase => "phrase",
            Self::Token => "token",
            Self::Temporal => "temporal",
            Self::Emotional => "emotional",
            Self::QuestionForm => "question_form",
            Self::Fallback => "fallback",
        }
    }
}

/// One matched signal preserved for inspect and explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentSignalMatch {
    pub kind: IntentSignalKind,
    pub pattern: &'static str,
    pub matched_text: String,
    pub weight: u16,
}

/// Stable ranking posture names exposed to later routing layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntentRankingProfile {
    Balanced,
    RecencyBiased,
    RelevanceBiased,
}

impl IntentRankingProfile {
    /// Returns the stable machine-readable profile name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::RecencyBiased => "recency_biased",
            Self::RelevanceBiased => "relevance_biased",
        }
    }

    /// Returns the concrete ranking profile for this machine-readable posture.
    pub const fn ranking_profile(self) -> RankingProfile {
        match self {
            Self::Balanced => RankingProfile::balanced(),
            Self::RecencyBiased => RankingProfile::recency_biased(),
            Self::RelevanceBiased => RankingProfile::relevance_biased(),
        }
    }
}

/// Explicit routing-classification inputs emitted before any later auto-routing decision layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntentRouteInputs {
    pub query_path: QueryPath,
    pub ranking_profile: IntentRankingProfile,
    pub prefer_small_lookup: bool,
    pub prefer_preview_only_on_low_confidence: bool,
    pub high_stakes: bool,
}

/// Inspectable classification result for one query.
#[derive(Debug, Clone, PartialEq)]
pub struct IntentClassification {
    pub query: String,
    pub normalized_query: String,
    pub intent: QueryIntent,
    pub confidence: f32,
    pub route_inputs: IntentRouteInputs,
    pub matched_signals: Vec<IntentSignalMatch>,
    pub low_confidence_fallback: bool,
}

impl IntentClassification {
    /// Returns a stable log-friendly payload for sample classification traces.
    pub fn log_record(&self) -> IntentClassificationLog {
        IntentClassificationLog {
            normalized_query: self.normalized_query.clone(),
            intent: self.intent.as_str(),
            confidence: self.confidence,
            query_path: self.route_inputs.query_path.as_str(),
            ranking_profile: self.route_inputs.ranking_profile.as_str(),
            prefer_small_lookup: self.route_inputs.prefer_small_lookup,
            prefer_preview_only_on_low_confidence: self
                .route_inputs
                .prefer_preview_only_on_low_confidence,
            high_stakes: self.route_inputs.high_stakes,
            low_confidence_fallback: self.low_confidence_fallback,
            matched_signal_kinds: self
                .matched_signals
                .iter()
                .map(|signal| signal.kind.as_str())
                .collect(),
            matched_patterns: self
                .matched_signals
                .iter()
                .map(|signal| signal.pattern)
                .collect(),
        }
    }
}

/// Stable log payload showing how one query was classified.
#[derive(Debug, Clone, PartialEq)]
pub struct IntentClassificationLog {
    pub normalized_query: String,
    pub intent: &'static str,
    pub confidence: f32,
    pub query_path: &'static str,
    pub ranking_profile: &'static str,
    pub prefer_small_lookup: bool,
    pub prefer_preview_only_on_low_confidence: bool,
    pub high_stakes: bool,
    pub low_confidence_fallback: bool,
    pub matched_signal_kinds: Vec<&'static str>,
    pub matched_patterns: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IntentRule {
    intent: QueryIntent,
    kind: IntentSignalKind,
    pattern: &'static str,
    weight: u16,
}

impl IntentRule {
    const fn new(
        intent: QueryIntent,
        kind: IntentSignalKind,
        pattern: &'static str,
        weight: u16,
    ) -> Self {
        Self {
            intent,
            kind,
            pattern,
            weight,
        }
    }
}

const INTENT_RULES: &[IntentRule] = &[
    IntentRule::new(
        QueryIntent::SemanticBroad,
        IntentSignalKind::Phrase,
        "what do i know about",
        990,
    ),
    IntentRule::new(
        QueryIntent::ExistenceCheck,
        IntentSignalKind::Phrase,
        "did i",
        950,
    ),
    IntentRule::new(
        QueryIntent::ExistenceCheck,
        IntentSignalKind::Phrase,
        "have i",
        950,
    ),
    IntentRule::new(
        QueryIntent::ExistenceCheck,
        IntentSignalKind::Phrase,
        "do i know",
        950,
    ),
    IntentRule::new(
        QueryIntent::CausalTrace,
        IntentSignalKind::Phrase,
        "why do i believe",
        980,
    ),
    IntentRule::new(
        QueryIntent::CausalTrace,
        IntentSignalKind::Phrase,
        "how did i learn",
        980,
    ),
    IntentRule::new(
        QueryIntent::CausalTrace,
        IntentSignalKind::Phrase,
        "origin of",
        900,
    ),
    IntentRule::new(
        QueryIntent::ProceduralLookup,
        IntentSignalKind::Phrase,
        "how to",
        980,
    ),
    IntentRule::new(
        QueryIntent::ProceduralLookup,
        IntentSignalKind::Phrase,
        "steps for",
        960,
    ),
    IntentRule::new(
        QueryIntent::ProceduralLookup,
        IntentSignalKind::Phrase,
        "procedure for",
        960,
    ),
    IntentRule::new(
        QueryIntent::RecentFirst,
        IntentSignalKind::Temporal,
        "recently",
        940,
    ),
    IntentRule::new(
        QueryIntent::RecentFirst,
        IntentSignalKind::Temporal,
        "lately",
        940,
    ),
    IntentRule::new(QueryIntent::RecentFirst, IntentSignalKind::Temporal, "today", 900),
    IntentRule::new(
        QueryIntent::RecentFirst,
        IntentSignalKind::Temporal,
        "last time",
        930,
    ),
    IntentRule::new(
        QueryIntent::RecentFirst,
        IntentSignalKind::Temporal,
        "most recent",
        930,
    ),
    IntentRule::new(
        QueryIntent::StrengthWeighted,
        IntentSignalKind::Token,
        "important",
        900,
    ),
    IntentRule::new(
        QueryIntent::StrengthWeighted,
        IntentSignalKind::Token,
        "critical",
        920,
    ),
    IntentRule::new(QueryIntent::StrengthWeighted, IntentSignalKind::Token, "key", 850),
    IntentRule::new(
        QueryIntent::StrengthWeighted,
        IntentSignalKind::Token,
        "essential",
        900,
    ),
    IntentRule::new(
        QueryIntent::StrengthWeighted,
        IntentSignalKind::Phrase,
        "must know",
        940,
    ),
    IntentRule::new(
        QueryIntent::UncertaintyFocused,
        IntentSignalKind::Token,
        "uncertain",
        930,
    ),
    IntentRule::new(
        QueryIntent::UncertaintyFocused,
        IntentSignalKind::Phrase,
        "not sure",
        940,
    ),
    IntentRule::new(QueryIntent::UncertaintyFocused, IntentSignalKind::Token, "might", 860),
    IntentRule::new(
        QueryIntent::UncertaintyFocused,
        IntentSignalKind::Token,
        "possibly",
        860,
    ),
    IntentRule::new(QueryIntent::UncertaintyFocused, IntentSignalKind::Token, "unsure", 930),
    IntentRule::new(
        QueryIntent::TemporalAnchor,
        IntentSignalKind::Temporal,
        "before",
        860,
    ),
    IntentRule::new(
        QueryIntent::TemporalAnchor,
        IntentSignalKind::Temporal,
        "after",
        860,
    ),
    IntentRule::new(
        QueryIntent::TemporalAnchor,
        IntentSignalKind::Phrase,
        "when did",
        940,
    ),
    IntentRule::new(
        QueryIntent::TemporalAnchor,
        IntentSignalKind::Temporal,
        "timeline",
        920,
    ),
    IntentRule::new(QueryIntent::TemporalAnchor, IntentSignalKind::Temporal, "era", 900),
    IntentRule::new(
        QueryIntent::DiverseSample,
        IntentSignalKind::Token,
        "different",
        900,
    ),
    IntentRule::new(
        QueryIntent::DiverseSample,
        IntentSignalKind::Token,
        "varied",
        900,
    ),
    IntentRule::new(
        QueryIntent::DiverseSample,
        IntentSignalKind::Token,
        "alternative",
        920,
    ),
    IntentRule::new(
        QueryIntent::DiverseSample,
        IntentSignalKind::Token,
        "alternatives",
        920,
    ),
    IntentRule::new(
        QueryIntent::DiverseSample,
        IntentSignalKind::Token,
        "counterexample",
        940,
    ),
    IntentRule::new(
        QueryIntent::EmotionalFilter,
        IntentSignalKind::Emotional,
        "worried",
        950,
    ),
    IntentRule::new(
        QueryIntent::EmotionalFilter,
        IntentSignalKind::Emotional,
        "frustrated",
        950,
    ),
    IntentRule::new(
        QueryIntent::EmotionalFilter,
        IntentSignalKind::Emotional,
        "stressed",
        950,
    ),
    IntentRule::new(
        QueryIntent::EmotionalFilter,
        IntentSignalKind::Emotional,
        "excited",
        920,
    ),
    IntentRule::new(
        QueryIntent::EmotionalFilter,
        IntentSignalKind::Emotional,
        "anxious",
        950,
    ),
];

/// Canonical query-intent classifier used by future ask-style wrappers.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct IntentEngine;

impl IntentEngine {
    /// Classifies one query into an explicit intent class plus inspectable routing inputs.
    pub fn classify(&self, query: &str) -> IntentClassification {
        let normalized_query = normalize_query(query);
        let mut matched_signals = Vec::new();

        for rule in INTENT_RULES {
            if normalized_query.contains(rule.pattern) {
                matched_signals.push(IntentSignalMatch {
                    kind: rule.kind,
                    pattern: rule.pattern,
                    matched_text: rule.pattern.to_string(),
                    weight: rule.weight,
                });
            }
        }

        if normalized_query.starts_with("what ") || normalized_query.starts_with("which ") {
            matched_signals.push(IntentSignalMatch {
                kind: IntentSignalKind::QuestionForm,
                pattern: "what|which",
                matched_text: normalized_query
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                weight: 250,
            });
        }

        let selected_intent = matched_signals
            .iter()
            .max_by_key(|signal| signal.weight)
            .and_then(|signal| {
                INTENT_RULES
                    .iter()
                    .find(|rule| rule.pattern == signal.pattern)
                    .map(|rule| rule.intent)
            })
            .unwrap_or(QueryIntent::SemanticBroad);

        let low_confidence_fallback = matched_signals.is_empty();
        if low_confidence_fallback {
            matched_signals.push(IntentSignalMatch {
                kind: IntentSignalKind::Fallback,
                pattern: "default_semantic_broad",
                matched_text: normalized_query.clone(),
                weight: 350,
            });
        }

        let strongest_weight = matched_signals
            .iter()
            .map(|signal| signal.weight)
            .max()
            .unwrap_or(350);
        let confidence = strongest_weight as f32 / 1000.0;

        IntentClassification {
            query: query.to_string(),
            normalized_query,
            intent: selected_intent,
            confidence,
            route_inputs: route_inputs_for(selected_intent),
            matched_signals,
            low_confidence_fallback,
        }
    }
}

fn route_inputs_for(intent: QueryIntent) -> IntentRouteInputs {
    match intent {
        QueryIntent::SemanticBroad => IntentRouteInputs {
            query_path: QueryPath::Hybrid,
            ranking_profile: IntentRankingProfile::RelevanceBiased,
            prefer_small_lookup: false,
            prefer_preview_only_on_low_confidence: false,
            high_stakes: false,
        },
        QueryIntent::ExistenceCheck => IntentRouteInputs {
            query_path: QueryPath::ExactId,
            ranking_profile: IntentRankingProfile::Balanced,
            prefer_small_lookup: true,
            prefer_preview_only_on_low_confidence: false,
            high_stakes: false,
        },
        QueryIntent::RecentFirst => IntentRouteInputs {
            query_path: QueryPath::Temporal,
            ranking_profile: IntentRankingProfile::RecencyBiased,
            prefer_small_lookup: true,
            prefer_preview_only_on_low_confidence: false,
            high_stakes: false,
        },
        QueryIntent::StrengthWeighted => IntentRouteInputs {
            query_path: QueryPath::Hybrid,
            ranking_profile: IntentRankingProfile::Balanced,
            prefer_small_lookup: true,
            prefer_preview_only_on_low_confidence: false,
            high_stakes: true,
        },
        QueryIntent::UncertaintyFocused => IntentRouteInputs {
            query_path: QueryPath::PartialCue,
            ranking_profile: IntentRankingProfile::Balanced,
            prefer_small_lookup: false,
            prefer_preview_only_on_low_confidence: true,
            high_stakes: true,
        },
        QueryIntent::CausalTrace => IntentRouteInputs {
            query_path: QueryPath::EntityHeavy,
            ranking_profile: IntentRankingProfile::Balanced,
            prefer_small_lookup: true,
            prefer_preview_only_on_low_confidence: true,
            high_stakes: false,
        },
        QueryIntent::TemporalAnchor => IntentRouteInputs {
            query_path: QueryPath::Temporal,
            ranking_profile: IntentRankingProfile::RecencyBiased,
            prefer_small_lookup: false,
            prefer_preview_only_on_low_confidence: false,
            high_stakes: false,
        },
        QueryIntent::DiverseSample => IntentRouteInputs {
            query_path: QueryPath::Hybrid,
            ranking_profile: IntentRankingProfile::RelevanceBiased,
            prefer_small_lookup: false,
            prefer_preview_only_on_low_confidence: false,
            high_stakes: false,
        },
        QueryIntent::ProceduralLookup => IntentRouteInputs {
            query_path: QueryPath::EntityHeavy,
            ranking_profile: IntentRankingProfile::Balanced,
            prefer_small_lookup: true,
            prefer_preview_only_on_low_confidence: true,
            high_stakes: true,
        },
        QueryIntent::EmotionalFilter => IntentRouteInputs {
            query_path: QueryPath::SemanticOnly,
            ranking_profile: IntentRankingProfile::Balanced,
            prefer_small_lookup: false,
            prefer_preview_only_on_low_confidence: true,
            high_stakes: true,
        },
    }
}

fn normalize_query(query: &str) -> String {
    query.split_whitespace()
        .map(|part| {
            part.trim_matches(|c: char| !c.is_ascii_alphanumeric())
                .to_ascii_lowercase()
        })
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::{IntentEngine, IntentRankingProfile, IntentSignalKind, QueryIntent};
    use crate::engine::retrieval_planner::QueryPath;

    #[test]
    fn classifies_representative_queries_across_the_full_taxonomy() {
        let engine = IntentEngine;
        let fixtures = [
            (
                "what do I know about rust lifetimes?",
                QueryIntent::SemanticBroad,
                QueryPath::Hybrid,
            ),
            (
                "did I ever hit this deploy issue before?",
                QueryIntent::ExistenceCheck,
                QueryPath::ExactId,
            ),
            (
                "what happened recently with the release pipeline?",
                QueryIntent::RecentFirst,
                QueryPath::Temporal,
            ),
            (
                "what is most important about the rollback plan?",
                QueryIntent::StrengthWeighted,
                QueryPath::Hybrid,
            ),
            (
                "what might be wrong with the new cache behavior?",
                QueryIntent::UncertaintyFocused,
                QueryPath::PartialCue,
            ),
            (
                "why do I believe this service should stay split?",
                QueryIntent::CausalTrace,
                QueryPath::EntityHeavy,
            ),
            (
                "what changed before the march deploy?",
                QueryIntent::TemporalAnchor,
                QueryPath::Temporal,
            ),
            (
                "show different alternatives for the retry policy",
                QueryIntent::DiverseSample,
                QueryPath::Hybrid,
            ),
            (
                "how to rotate the credentials safely",
                QueryIntent::ProceduralLookup,
                QueryPath::EntityHeavy,
            ),
            (
                "what am I worried about in this migration?",
                QueryIntent::EmotionalFilter,
                QueryPath::SemanticOnly,
            ),
        ];

        for (query, expected_intent, expected_path) in fixtures {
            let classification = engine.classify(query);
            assert_eq!(classification.intent, expected_intent, "query: {query}");
            assert_eq!(classification.route_inputs.query_path, expected_path);
            assert!(!classification.matched_signals.is_empty());
            assert!(classification.confidence >= 0.85, "query: {query}");
        }
    }

    #[test]
    fn default_semantic_broad_fallback_stays_explicit_and_low_confidence() {
        let classification = IntentEngine.classify("rust borrow checker notes");

        assert_eq!(classification.intent, QueryIntent::SemanticBroad);
        assert!(classification.low_confidence_fallback);
        assert_eq!(classification.route_inputs.query_path, QueryPath::Hybrid);
        assert_eq!(
            classification.route_inputs.ranking_profile,
            IntentRankingProfile::RelevanceBiased,
        );
        assert_eq!(classification.matched_signals.len(), 1);
        assert_eq!(classification.matched_signals[0].kind, IntentSignalKind::Fallback);
        assert_eq!(classification.matched_signals[0].pattern, "default_semantic_broad");
        assert_eq!(classification.confidence, 0.35);
    }

    #[test]
    fn log_record_preserves_how_the_sample_query_was_classified() {
        let classification =
            IntentEngine.classify("how to deploy the service after the last incident?");
        let log = classification.log_record();

        assert_eq!(log.intent, "procedural_lookup");
        assert_eq!(log.query_path, "entity_heavy");
        assert_eq!(log.ranking_profile, "balanced");
        assert!(log.prefer_small_lookup);
        assert!(log.prefer_preview_only_on_low_confidence);
        assert!(log.high_stakes);
        assert!(!log.low_confidence_fallback);
        assert!(log.matched_signal_kinds.contains(&"phrase"));
        assert!(log.matched_patterns.contains(&"how to"));
    }

    #[test]
    fn explicit_route_inputs_remain_stable_for_high_stakes_uncertainty_queries() {
        let classification = IntentEngine.classify("I am not sure whether this repair is safe");

        assert_eq!(classification.intent, QueryIntent::UncertaintyFocused);
        assert_eq!(classification.route_inputs.query_path, QueryPath::PartialCue);
        assert_eq!(
            classification.route_inputs.ranking_profile,
            IntentRankingProfile::Balanced,
        );
        assert!(classification.route_inputs.prefer_preview_only_on_low_confidence);
        assert!(classification.route_inputs.high_stakes);
    }
}
