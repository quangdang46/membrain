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
            return Err(EmbedError::LocalBackendUnavailable(
                "single failed".to_string(),
            ));
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
            return Err(EmbedError::LocalBackendUnavailable(
                "batch failed".to_string(),
            ));
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
    let records = vec![record(
        1,
        "deploy pipeline rollback",
        "deploy pipeline rollback",
    )];
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
    assert_eq!(
        result.candidates[0].ranking_score,
        result.candidates[1].ranking_score
    );
    assert_eq!(
        result.candidates[0].semantic_score,
        result.candidates[1].semantic_score
    );
    assert_eq!(
        result.candidates[0].lexical_score,
        result.candidates[1].lexical_score
    );
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
        record(
            1,
            "deploy pipeline rollback after outage",
            "deploy pipeline rollback after outage",
        ),
        record(
            2,
            "deploy pipeline canary validation checklist",
            "deploy pipeline canary validation checklist",
        ),
        record(
            3,
            "deploy pipeline smoke test notes",
            "deploy pipeline smoke test notes",
        ),
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
