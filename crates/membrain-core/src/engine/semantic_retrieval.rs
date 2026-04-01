use crate::api::NamespaceId;
use crate::embed::{
    BatchEmbedTrace, CachedTextEmbedder, EmbedError, EmbedTrace, EmbeddingPurpose,
    LocalTextEmbedder,
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

struct SemanticExecutionMode<'a, E: LocalTextEmbedder> {
    embedder: Option<&'a mut CachedTextEmbedder<E>>,
    forced_degraded_reason: Option<String>,
}

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
            SemanticExecutionMode {
                embedder: None,
                forced_degraded_reason: Some(degraded_reason.into()),
            },
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
        self.execute_internal(
            records,
            namespace,
            query,
            exact_memory_id,
            config,
            SemanticExecutionMode {
                embedder: Some(embedder),
                forced_degraded_reason: None,
            },
        )
    }

    fn execute_internal<E: LocalTextEmbedder>(
        &self,
        records: &[HydratedMemoryRecord],
        namespace: &NamespaceId,
        query: &str,
        exact_memory_id: Option<MemoryId>,
        config: SemanticExecutorConfig,
        mode: SemanticExecutionMode<'_, E>,
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
            config
                .lexical_prefilter_limit
                .max(config.result_limit.max(1)),
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

        let Some(embedder) = mode.embedder else {
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
                    degraded_reason: mode.forced_degraded_reason,
                    query_trace: None,
                    batch_trace: None,
                },
            };
        };

        let query_embedding = match embedder
            .get_or_embed(EmbeddingPurpose::Content, &normalized_query)
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
        let batch_embeddings = match embedder
            .get_or_embed_batch(EmbeddingPurpose::Content, &candidate_texts)
        {
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
            let lexical_score =
                lexical_score(normalized_query, &normalize_text(record.retrieval_text()));
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
#[path = "semantic_retrieval_tests.rs"]
mod tests;
