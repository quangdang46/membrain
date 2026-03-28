use crate::api::NamespaceId;
use crate::embed::{
    BatchEmbedTrace, CachedTextEmbedder, EmbedError, EmbedTrace, EmbeddingPurpose, LocalTextEmbedder,
};
use crate::engine::ranking::{fuse_scores, RankingInput, RankingProfile};
use crate::persistence::{PersistedDaemonMemoryRecord, PersistedLocalMemoryRecord};
use crate::types::{AffectSignals, CanonicalMemoryType, FastPathRouteFamily, MemoryId, SessionId};
use std::collections::HashSet;

const DEFAULT_PREFILTER_MULTIPLIER: usize = 4;

#[derive(Debug, Clone, PartialEq)]
pub struct HydratedMemoryRecord {
    pub memory_id: MemoryId,
    pub namespace: NamespaceId,
    pub session_id: SessionId,
    pub memory_type: CanonicalMemoryType,
    pub route_family: FastPathRouteFamily,
    pub compact_text: String,
    pub raw_text: String,
    pub affect: Option<AffectSignals>,
}

impl HydratedMemoryRecord {
    pub fn retrieval_text(&self) -> &str {
        if self.raw_text.trim().is_empty() {
            &self.compact_text
        } else {
            &self.raw_text
        }
    }
}

impl TryFrom<PersistedLocalMemoryRecord> for HydratedMemoryRecord {
    type Error = String;

    fn try_from(value: PersistedLocalMemoryRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            memory_id: MemoryId(value.memory_id),
            namespace: NamespaceId::new(&value.namespace).map_err(|err| err.to_string())?,
            session_id: SessionId(value.session_id),
            memory_type: value.memory_type,
            route_family: value.route_family,
            compact_text: value.compact_text,
            raw_text: value.raw_text,
            affect: value.affect,
        })
    }
}

impl TryFrom<PersistedDaemonMemoryRecord> for HydratedMemoryRecord {
    type Error = String;

    fn try_from(value: PersistedDaemonMemoryRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            memory_id: MemoryId(value.layout.memory_id),
            namespace: NamespaceId::new(&value.layout.namespace).map_err(|err| err.to_string())?,
            session_id: SessionId(value.layout.session_id),
            memory_type: value.layout.memory_type,
            route_family: value.layout.route_family,
            compact_text: value.layout.compact_text,
            raw_text: value.layout.raw_text,
            affect: value.layout.affect,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticExecutorConfig {
    pub lexical_prefilter_limit: usize,
    pub result_limit: usize,
}

impl SemanticExecutorConfig {
    pub fn bounded(result_limit: usize) -> Self {
        let bounded_result_limit = result_limit.max(1);
        Self {
            lexical_prefilter_limit: bounded_result_limit * DEFAULT_PREFILTER_MULTIPLIER,
            result_limit: bounded_result_limit,
        }
    }
}

impl Default for SemanticExecutorConfig {
    fn default() -> Self {
        Self::bounded(8)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticRetrievalCandidate {
    pub record: HydratedMemoryRecord,
    pub lexical_score: f32,
    pub semantic_score: f32,
    pub blended_score: f32,
    pub ranking_score: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticRetrievalTrace {
    pub exact_match_used: bool,
    pub namespace_candidate_count: usize,
    pub lexical_prefilter_count: usize,
    pub semantic_candidate_count: usize,
    pub result_limit: usize,
    pub bounded_shortlist_truncated: bool,
    pub degraded_reason: Option<String>,
    pub query_trace: Option<EmbedTrace>,
    pub batch_trace: Option<BatchEmbedTrace>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticRetrievalResult {
    pub candidates: Vec<SemanticRetrievalCandidate>,
    pub trace: SemanticRetrievalTrace,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SharedSemanticRetrievalExecutor;

impl SharedSemanticRetrievalExecutor {
    pub fn execute_without_embeddings(
        &self,
        records: &[HydratedMemoryRecord],
        namespace: &NamespaceId,
        query: &str,
        exact_memory_id: Option<MemoryId>,
        config: SemanticExecutorConfig,
        degraded_reason: impl Into<String>,
    ) -> SemanticRetrievalResult {
        self.execute_internal::<crate::embed::FastembedTextEmbedder>(
            records,
            namespace,
            query,
            exact_memory_id,
            config,
            None,
            Some(degraded_reason.into()),
        )
    }

    pub fn execute<E: LocalTextEmbedder>(
        &self,
        records: &[HydratedMemoryRecord],
        namespace: &NamespaceId,
        query: &str,
        exact_memory_id: Option<MemoryId>,
        config: SemanticExecutorConfig,
        embedder: &mut CachedTextEmbedder<E>,
    ) -> SemanticRetrievalResult {
        self.execute_internal(records, namespace, query, exact_memory_id, config, Some(embedder), None)
    }

    fn execute_internal<E: LocalTextEmbedder>(
        &self,
        records: &[HydratedMemoryRecord],
        namespace: &NamespaceId,
        query: &str,
        exact_memory_id: Option<MemoryId>,
        config: SemanticExecutorConfig,
        embedder: Option<&mut CachedTextEmbedder<E>>,
        forced_degraded_reason: Option<String>,
    ) -> SemanticRetrievalResult {
        let namespace_records = records
            .iter()
            .filter(|record| &record.namespace == namespace)
            .cloned()
            .collect::<Vec<_>>();

        if let Some(memory_id) = exact_memory_id {
            if let Some(record) = namespace_records
                .iter()
                .find(|record| record.memory_id == memory_id)
                .cloned()
            {
                return SemanticRetrievalResult {
                    candidates: vec![SemanticRetrievalCandidate {
                        record,
                        lexical_score: 1.0,
                        semantic_score: 1.0,
                        blended_score: 1.0,
                        ranking_score: 1000,
                    }],
                    trace: SemanticRetrievalTrace {
                        exact_match_used: true,
                        namespace_candidate_count: namespace_records.len(),
                        lexical_prefilter_count: 1,
                        semantic_candidate_count: 1,
                        result_limit: config.result_limit,
                        bounded_shortlist_truncated: false,
                        degraded_reason: None,
                        query_trace: None,
                        batch_trace: None,
                    },
                };
            }
        }

        let normalized_query = normalize_text(query);
        if normalized_query.is_empty() {
            return SemanticRetrievalResult {
                candidates: Vec::new(),
                trace: SemanticRetrievalTrace {
                    exact_match_used: false,
                    namespace_candidate_count: namespace_records.len(),
                    lexical_prefilter_count: 0,
                    semantic_candidate_count: 0,
                    result_limit: config.result_limit,
                    bounded_shortlist_truncated: false,
                    degraded_reason: Some("empty_query".to_string()),
                    query_trace: None,
                    batch_trace: None,
                },
            };
        }

        let lexical_prefilter = lexical_prefilter(
            &namespace_records,
            &normalized_query,
            config.lexical_prefilter_limit.max(config.result_limit.max(1)),
        );
        let lexical_prefilter_count = lexical_prefilter.len();

        if lexical_prefilter.is_empty() {
            return SemanticRetrievalResult {
                candidates: Vec::new(),
                trace: SemanticRetrievalTrace {
                    exact_match_used: false,
                    namespace_candidate_count: namespace_records.len(),
                    lexical_prefilter_count: 0,
                    semantic_candidate_count: 0,
                    result_limit: config.result_limit,
                    bounded_shortlist_truncated: false,
                    degraded_reason: Some("no_namespace_candidates".to_string()),
                    query_trace: None,
                    batch_trace: None,
                },
            };
        }

        let Some(embedder) = embedder else {
            let candidates = lexical_prefilter
                .into_iter()
                .take(config.result_limit)
                .map(|(record, lexical_score)| {
                    let blended_score = lexical_score * 0.35;
                    SemanticRetrievalCandidate {
                        record,
                        lexical_score,
                        semantic_score: 0.0,
                        blended_score,
                        ranking_score: lexical_only_ranking_score(lexical_score),
                    }
                })
                .collect();
            return SemanticRetrievalResult {
                candidates,
                trace: SemanticRetrievalTrace {
                    exact_match_used: false,
                    namespace_candidate_count: namespace_records.len(),
                    lexical_prefilter_count,
                    semantic_candidate_count: 0,
                    result_limit: config.result_limit,
                    bounded_shortlist_truncated: lexical_prefilter_count > config.result_limit,
                    degraded_reason: forced_degraded_reason,
                    query_trace: None,
                    batch_trace: None,
                },
            };
        };

        let query_embedding = match embedder.get_or_embed(EmbeddingPurpose::Content, &normalized_query)
        {
            Ok(embedding) => embedding,
            Err(error) => {
                let candidates = lexical_prefilter
                    .into_iter()
                    .take(config.result_limit)
                    .map(|(record, lexical_score)| {
                        let blended_score = lexical_score * 0.35;
                        SemanticRetrievalCandidate {
                            record,
                            lexical_score,
                            semantic_score: 0.0,
                            blended_score,
                            ranking_score: lexical_only_ranking_score(lexical_score),
                        }
                    })
                    .collect();
                return SemanticRetrievalResult {
                    candidates,
                    trace: SemanticRetrievalTrace {
                        exact_match_used: false,
                        namespace_candidate_count: namespace_records.len(),
                        lexical_prefilter_count,
                        semantic_candidate_count: 0,
                        result_limit: config.result_limit,
                        bounded_shortlist_truncated: lexical_prefilter_count > config.result_limit,
                        degraded_reason: Some(embed_error_reason(&error)),
                        query_trace: None,
                        batch_trace: None,
                    },
                };
            }
        };

        let candidate_texts = lexical_prefilter
            .iter()
            .map(|(record, _)| normalize_text(record.retrieval_text()))
            .collect::<Vec<_>>();
        let batch_embeddings = match embedder.get_or_embed_batch(EmbeddingPurpose::Content, &candidate_texts) {
            Ok(batch_embeddings) => batch_embeddings,
            Err(error) => {
                let candidates = lexical_prefilter
                    .into_iter()
                    .take(config.result_limit)
                    .map(|(record, lexical_score)| {
                        let blended_score = lexical_score * 0.35;
                        SemanticRetrievalCandidate {
                            record,
                            lexical_score,
                            semantic_score: 0.0,
                            blended_score,
                            ranking_score: lexical_only_ranking_score(lexical_score),
                        }
                    })
                    .collect();
                return SemanticRetrievalResult {
                    candidates,
                    trace: SemanticRetrievalTrace {
                        exact_match_used: false,
                        namespace_candidate_count: namespace_records.len(),
                        lexical_prefilter_count,
                        semantic_candidate_count: 0,
                        result_limit: config.result_limit,
                        bounded_shortlist_truncated: lexical_prefilter_count > config.result_limit,
                        degraded_reason: Some(embed_error_reason(&error)),
                        query_trace: Some(query_embedding.trace),
                        batch_trace: None,
                    },
                };
            }
        };

        let mut candidates = lexical_prefilter
            .into_iter()
            .zip(batch_embeddings.vectors)
            .map(|((record, lexical_score), vector)| {
                let semantic_score = normalized_cosine_similarity(&query_embedding.vector, &vector);
                let blended_score = semantic_score * 0.8 + lexical_score * 0.2;
                let ranking_input = RankingInput {
                    recency: to_fixed_u16(semantic_score),
                    salience: to_fixed_u16(semantic_score),
                    strength: to_fixed_u16(semantic_score * 0.7 + lexical_score * 0.3),
                    provenance: to_fixed_u16(lexical_score),
                    conflict: 500,
                    confidence: to_fixed_u16(semantic_score * 0.8 + lexical_score * 0.2),
                };
                let ranking_score =
                    fuse_scores(ranking_input, RankingProfile::balanced()).final_score;
                SemanticRetrievalCandidate {
                    record,
                    lexical_score,
                    semantic_score,
                    blended_score,
                    ranking_score,
                }
            })
            .collect::<Vec<_>>();

        candidates.sort_by(|left, right| {
            right
                .ranking_score
                .cmp(&left.ranking_score)
                .then_with(|| right.semantic_score.total_cmp(&left.semantic_score))
                .then_with(|| right.lexical_score.total_cmp(&left.lexical_score))
                .then_with(|| left.record.memory_id.0.cmp(&right.record.memory_id.0))
        });
        candidates.truncate(config.result_limit);

        SemanticRetrievalResult {
            trace: SemanticRetrievalTrace {
                exact_match_used: false,
                namespace_candidate_count: namespace_records.len(),
                lexical_prefilter_count: candidate_texts.len(),
                semantic_candidate_count: candidates.len(),
                result_limit: config.result_limit,
                bounded_shortlist_truncated: lexical_prefilter_count > config.result_limit,
                degraded_reason: None,
                query_trace: Some(query_embedding.trace),
                batch_trace: Some(batch_embeddings.trace),
            },
            candidates,
        }
    }
}

fn lexical_prefilter(
    records: &[HydratedMemoryRecord],
    normalized_query: &str,
    prefilter_limit: usize,
) -> Vec<(HydratedMemoryRecord, f32)> {
    let mut scored = records
        .iter()
        .cloned()
        .map(|record| {
            let lexical_score = lexical_score(normalized_query, &normalize_text(record.retrieval_text()));
            (record, lexical_score)
        })
        .collect::<Vec<_>>();

    scored.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.memory_id.0.cmp(&right.0.memory_id.0))
    });

    let positives = scored
        .iter()
        .filter(|(_, score)| *score > 0.0)
        .count()
        .max(1)
        .min(prefilter_limit.max(1));

    scored.into_iter().take(positives).collect()
}

fn lexical_score(normalized_query: &str, normalized_text: &str) -> f32 {
    if normalized_query.is_empty() || normalized_text.is_empty() {
        return 0.0;
    }

    if normalized_text == normalized_query {
        return 1.0;
    }

    let query_tokens = tokenize(normalized_query);
    if query_tokens.is_empty() {
        return 0.0;
    }
    let text_tokens = tokenize(normalized_text);
    let text_token_set = text_tokens.iter().cloned().collect::<HashSet<_>>();
    let overlap = query_tokens
        .iter()
        .filter(|token| text_token_set.contains(*token))
        .count();
    let token_overlap = overlap as f32 / query_tokens.len() as f32;
    let substring_bonus = if normalized_text.contains(normalized_query) {
        0.35
    } else {
        0.0
    };
    let prefix_bonus = query_tokens
        .iter()
        .filter(|token| {
            text_tokens
                .iter()
                .any(|text_token| text_token.starts_with(token.as_str()))
        })
        .count() as f32
        / query_tokens.len() as f32
        * 0.15;

    (token_overlap * 0.5 + substring_bonus + prefix_bonus).clamp(0.0, 1.0)
}

fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|token| token.trim_matches(|character: char| !character.is_alphanumeric()))
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn normalize_text(text: &str) -> String {
    text.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalized_cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0;
    let mut left_norm = 0.0;
    let mut right_norm = 0.0;
    for (lhs, rhs) in left.iter().zip(right) {
        dot += lhs * rhs;
        left_norm += lhs * lhs;
        right_norm += rhs * rhs;
    }

    if left_norm == 0.0 || right_norm == 0.0 {
        0.0
    } else {
        ((dot / (left_norm.sqrt() * right_norm.sqrt())) + 1.0) / 2.0
    }
}

fn to_fixed_u16(score: f32) -> u16 {
    (score.clamp(0.0, 1.0) * 1000.0).round() as u16
}

fn lexical_only_ranking_score(lexical_score: f32) -> u16 {
    let blended = lexical_score.clamp(0.0, 1.0);
    fuse_scores(
        RankingInput {
            recency: to_fixed_u16(blended * 0.2),
            salience: to_fixed_u16(blended),
            strength: to_fixed_u16(blended * 0.5),
            provenance: to_fixed_u16(blended),
            conflict: 500,
            confidence: to_fixed_u16(blended * 0.3),
        },
        RankingProfile::balanced(),
    )
    .final_score
}

fn embed_error_reason(error: &EmbedError) -> String {
    format!("semantic_embedding_unavailable:{error}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::{CachedTextEmbedder, EmbedError, EmbeddingPurpose, LocalTextEmbedder};
    use std::collections::HashMap;

    #[derive(Debug)]
    struct FakeEmbedder {
        fail_single: bool,
        fail_batch: bool,
        vectors: HashMap<String, Vec<f32>>,
    }

    impl FakeEmbedder {
        fn semantic() -> Self {
            let mut vectors = HashMap::new();
            vectors.insert(
                "rust borrow checker ownership lifetime fix".to_string(),
                vec![0.0, 1.0, 0.0],
            );
            vectors.insert(
                "ownership issue with references and lifetimes".to_string(),
                vec![0.0, 0.98, 0.0],
            );
            vectors.insert(
                "ownership ownership ownership unrelated distractor".to_string(),
                vec![1.0, 0.0, 0.0],
            );
            vectors.insert("alpha record".to_string(), vec![0.5, 0.5, 0.0]);
            vectors.insert("beta record".to_string(), vec![0.5, 0.5, 0.0]);
            Self {
                fail_single: false,
                fail_batch: false,
                vectors,
            }
        }

        fn failing_single() -> Self {
            let mut embedder = Self::semantic();
            embedder.fail_single = true;
            embedder
        }

        fn failing_batch() -> Self {
            let mut embedder = Self::semantic();
            embedder.fail_batch = true;
            embedder
        }
    }

    impl LocalTextEmbedder for FakeEmbedder {
        fn backend_kind(&self) -> &'static str {
            "test"
        }

        fn generation(&self) -> &str {
            "test-generation"
        }

        fn dimensions(&self) -> usize {
            3
        }

        fn embed_text(
            &mut self,
            _purpose: EmbeddingPurpose,
            normalized_text: &str,
        ) -> Result<Vec<f32>, EmbedError> {
            if self.fail_single {
                return Err(EmbedError::LocalBackendUnavailable("single failed".to_string()));
            }
            Ok(self
                .vectors
                .get(normalized_text)
                .cloned()
                .unwrap_or_else(|| vec![0.0, 0.0, 1.0]))
        }

        fn embed_texts(
            &mut self,
            _purpose: EmbeddingPurpose,
            normalized_texts: &[String],
        ) -> Result<Vec<Vec<f32>>, EmbedError> {
            if self.fail_batch {
                return Err(EmbedError::LocalBackendUnavailable("batch failed".to_string()));
            }
            Ok(normalized_texts
                .iter()
                .map(|text| {
                    self.vectors
                        .get(text)
                        .cloned()
                        .unwrap_or_else(|| vec![0.0, 0.0, 1.0])
                })
                .collect())
        }
    }

    fn record(memory_id: u64, compact_text: &str, raw_text: &str) -> HydratedMemoryRecord {
        HydratedMemoryRecord {
            memory_id: MemoryId(memory_id),
            namespace: NamespaceId::new("default").expect("namespace should parse"),
            session_id: SessionId(1),
            memory_type: CanonicalMemoryType::Observation,
            route_family: FastPathRouteFamily::Observation,
            compact_text: compact_text.to_string(),
            raw_text: raw_text.to_string(),
            affect: None,
        }
    }

    #[test]
    fn exact_memory_id_short_circuits_directly() {
        let executor = SharedSemanticRetrievalExecutor;
        let records = vec![record(7, "alpha", "alpha")];
        let mut embedder = CachedTextEmbedder::new(FakeEmbedder::semantic(), 8);

        let result = executor.execute(
            &records,
            &NamespaceId::new("default").unwrap(),
            "ignored",
            Some(MemoryId(7)),
            SemanticExecutorConfig::bounded(3),
            &mut embedder,
        );

        assert!(result.trace.exact_match_used);
        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].record.memory_id, MemoryId(7));
        assert_eq!(result.candidates[0].ranking_score, 1000);
    }

    #[test]
    fn semantic_score_beats_lexical_distractor() {
        let executor = SharedSemanticRetrievalExecutor;
        let records = vec![
            record(
                1,
                "ownership issue with references and lifetimes",
                "ownership issue with references and lifetimes",
            ),
            record(
                2,
                "ownership ownership ownership unrelated distractor",
                "ownership ownership ownership unrelated distractor",
            ),
        ];
        let mut embedder = CachedTextEmbedder::new(FakeEmbedder::semantic(), 8);

        let result = executor.execute(
            &records,
            &NamespaceId::new("default").unwrap(),
            "rust borrow checker ownership lifetime fix",
            None,
            SemanticExecutorConfig::bounded(2),
            &mut embedder,
        );

        assert_eq!(result.candidates.len(), 2);
        assert_eq!(result.candidates[0].record.memory_id, MemoryId(1));
        assert!(result.candidates[0].semantic_score > result.candidates[1].semantic_score);
        assert!(result.candidates[0].ranking_score > result.candidates[1].ranking_score);
    }

    #[test]
    fn lexical_prefilter_scores_substring_matches_higher() {
        let matching = lexical_score(
            "deploy pipeline rollback",
            "deploy pipeline rollback after outage",
        );
        let partial = lexical_score("deploy pipeline rollback", "pipeline notes only");

        assert!(matching > partial);
        assert!(matching > 0.5);
    }

    #[test]
    fn embedding_failures_degrade_to_lexical_only_results() {
        let executor = SharedSemanticRetrievalExecutor;
        let records = vec![record(1, "deploy pipeline rollback", "deploy pipeline rollback")];
        let mut embedder = CachedTextEmbedder::new(FakeEmbedder::failing_single(), 8);

        let result = executor.execute(
            &records,
            &NamespaceId::new("default").unwrap(),
            "deploy pipeline rollback",
            None,
            SemanticExecutorConfig::bounded(3),
            &mut embedder,
        );

        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].semantic_score, 0.0);
        assert!(result.trace.degraded_reason.is_some());
    }

    #[test]
    fn batch_embedding_failure_keeps_lexical_candidates() {
        let executor = SharedSemanticRetrievalExecutor;
        let records = vec![record(
            1,
            "deploy pipeline rollback after outage",
            "deploy pipeline rollback after outage",
        )];
        let mut embedder = CachedTextEmbedder::new(FakeEmbedder::failing_batch(), 8);

        let result = executor.execute(
            &records,
            &NamespaceId::new("default").unwrap(),
            "deploy pipeline rollback",
            None,
            SemanticExecutorConfig::bounded(3),
            &mut embedder,
        );

        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].semantic_score, 0.0);
        assert!(result.trace.degraded_reason.is_some());
    }

    #[test]
    fn deterministic_ties_break_by_memory_id() {
        let executor = SharedSemanticRetrievalExecutor;
        let records = vec![
            record(9, "shared tied record", "shared tied record"),
            record(3, "shared tied record", "shared tied record"),
        ];
        let mut embedder = CachedTextEmbedder::new(FakeEmbedder::semantic(), 8);

        let result = executor.execute(
            &records,
            &NamespaceId::new("default").unwrap(),
            "shared tied record",
            None,
            SemanticExecutorConfig::bounded(2),
            &mut embedder,
        );

        assert_eq!(result.candidates.len(), 2);
        assert_eq!(result.candidates[0].ranking_score, result.candidates[1].ranking_score);
        assert_eq!(result.candidates[0].semantic_score, result.candidates[1].semantic_score);
        assert_eq!(result.candidates[0].lexical_score, result.candidates[1].lexical_score);
        assert_eq!(result.candidates[0].record.memory_id, MemoryId(3));
        assert_eq!(result.candidates[1].record.memory_id, MemoryId(9));
    }

    #[test]
    fn empty_query_returns_no_candidates() {
        let executor = SharedSemanticRetrievalExecutor;
        let records = vec![record(1, "alpha", "alpha")];
        let mut embedder = CachedTextEmbedder::new(FakeEmbedder::semantic(), 8);

        let result = executor.execute(
            &records,
            &NamespaceId::new("default").unwrap(),
            "   ",
            None,
            SemanticExecutorConfig::bounded(2),
            &mut embedder,
        );

        assert!(result.candidates.is_empty());
        assert_eq!(result.trace.degraded_reason.as_deref(), Some("empty_query"));
    }

    #[test]
    fn trace_reports_when_bounded_shortlist_truncates_lower_ranked_candidates() {
        let executor = SharedSemanticRetrievalExecutor;
        let records = vec![
            record(1, "deploy pipeline rollback after outage", "deploy pipeline rollback after outage"),
            record(2, "deploy pipeline canary validation checklist", "deploy pipeline canary validation checklist"),
            record(3, "deploy pipeline smoke test notes", "deploy pipeline smoke test notes"),
        ];

        let result = executor.execute_without_embeddings(
            &records,
            &NamespaceId::new("default").unwrap(),
            "deploy pipeline",
            None,
            SemanticExecutorConfig::bounded(2),
            "semantic_embedding_unavailable:test",
        );

        assert_eq!(result.trace.lexical_prefilter_count, 3);
        assert_eq!(result.trace.result_limit, 2);
        assert_eq!(result.candidates.len(), 2);
        assert!(result.trace.bounded_shortlist_truncated);
    }
}
