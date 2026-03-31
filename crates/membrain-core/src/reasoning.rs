use crate::api::NamespaceId;
use crate::embed::{CachedTextEmbedder, LocalTextEmbedder};
use crate::engine::semantic_retrieval::{
    HydratedMemoryRecord, SemanticExecutorConfig, SemanticRetrievalCandidate,
    SemanticRetrievalResult, SemanticRetrievalTrace, SharedSemanticRetrievalExecutor,
};
use crate::types::MemoryId;
use llama_gguf::engine::{Engine, EngineConfig};
use llama_gguf::HfClient;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Instant;

const DEFAULT_REASONING_MODEL_REPO: &str = "Qwen/Qwen2.5-0.5B-Instruct-GGUF";
const DEFAULT_REASONING_MODEL_FILENAME: &str = "qwen2.5-0.5b-instruct-q4_k_m.gguf";
const DEFAULT_REASONING_MODEL_LOCAL_NAME: &str = "Qwen2.5-0.5B-Instruct-Q4_K_M.gguf";
const DEFAULT_REASONING_MAX_CONTEXT_TOKENS: usize = 512;
const DEFAULT_REASONING_MAX_OUTPUT_TOKENS: usize = 64;
const DEFAULT_REASONING_TIMEOUT_MS: u64 = 4_000;
const DEFAULT_REASONING_MAX_REWRITES: usize = 2;
const MAX_ALLOWED_REWRITES: usize = 4;
const REWRITE_HIT_BONUS: u16 = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasoningState {
    Disabled,
    NotLoaded,
    Loaded,
    Warm,
    Degraded,
    Unavailable,
}

impl ReasoningState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::NotLoaded => "not_loaded",
            Self::Loaded => "loaded",
            Self::Warm => "warm",
            Self::Degraded => "degraded",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReasoningStatus {
    pub state: ReasoningState,
    pub enabled: bool,
    pub model_path: String,
    pub auto_download: bool,
    pub detail: Option<String>,
}

impl ReasoningStatus {
    pub fn operator_note(&self) -> String {
        let mut details = vec![
            format!("state={}", self.state.as_str()),
            format!("enabled={}", self.enabled),
            format!("auto_download={}", self.auto_download),
            format!("model_path={}", self.model_path),
        ];
        if let Some(detail) = self.detail.as_deref() {
            details.push(format!("detail={detail}"));
        }
        details.join(" ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryRewriteOutcome {
    pub rewritten_queries: Vec<String>,
    pub trace_note: String,
    pub degraded_reason: Option<String>,
}

#[derive(Debug, Clone)]
struct ReasoningConfig {
    enabled: bool,
    auto_download: bool,
    model_path: PathBuf,
    max_context_tokens: usize,
    max_output_tokens: usize,
    timeout_ms: u64,
    max_rewrites: usize,
}

impl ReasoningConfig {
    fn from_env() -> Self {
        Self {
            enabled: env_bool("MEMBRAIN_REASONING_ENABLED", true),
            auto_download: env_bool(
                "MEMBRAIN_REASONING_AUTO_DOWNLOAD",
                !running_under_test_harness(),
            ),
            model_path: env::var_os("MEMBRAIN_REASONING_MODEL_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(default_model_path),
            max_context_tokens: env_usize(
                "MEMBRAIN_REASONING_MAX_CONTEXT_TOKENS",
                DEFAULT_REASONING_MAX_CONTEXT_TOKENS,
            )
            .max(128),
            max_output_tokens: env_usize(
                "MEMBRAIN_REASONING_MAX_OUTPUT_TOKENS",
                DEFAULT_REASONING_MAX_OUTPUT_TOKENS,
            )
            .clamp(16, 256),
            timeout_ms: env_u64(
                "MEMBRAIN_REASONING_TIMEOUT_MS",
                DEFAULT_REASONING_TIMEOUT_MS,
            )
            .max(500),
            max_rewrites: env_usize(
                "MEMBRAIN_REASONING_MAX_REWRITES",
                DEFAULT_REASONING_MAX_REWRITES,
            )
            .clamp(1, MAX_ALLOWED_REWRITES),
        }
    }
}

#[derive(Default)]
struct ReasoningRuntime {
    engine: Option<Engine>,
    loaded_model_path: Option<PathBuf>,
    warm_generations: u64,
    last_error: Option<String>,
    last_event: Option<String>,
}

#[derive(Debug, Clone)]
struct PendingQueryResult {
    source_label: String,
    query_text: String,
    result: SemanticRetrievalResult,
}

#[derive(Debug, Clone)]
struct MergedCandidateAccumulator {
    candidate: SemanticRetrievalCandidate,
    hit_count: usize,
    source_labels: Vec<String>,
    source_queries: Vec<String>,
}

static REASONING_RUNTIME: OnceLock<Mutex<ReasoningRuntime>> = OnceLock::new();

fn runtime() -> &'static Mutex<ReasoningRuntime> {
    REASONING_RUNTIME.get_or_init(|| Mutex::new(ReasoningRuntime::default()))
}

pub fn status_snapshot() -> ReasoningStatus {
    let config = ReasoningConfig::from_env();
    if !config.enabled {
        return ReasoningStatus {
            state: ReasoningState::Disabled,
            enabled: false,
            model_path: config.model_path.display().to_string(),
            auto_download: config.auto_download,
            detail: Some("reasoning explicitly disabled by configuration".to_string()),
        };
    }

    let runtime = runtime()
        .lock()
        .expect("reasoning runtime lock should be available");
    let (state, detail) = if runtime.engine.is_some() {
        let state = if runtime.warm_generations > 0 {
            ReasoningState::Warm
        } else {
            ReasoningState::Loaded
        };
        let detail = runtime
            .last_event
            .clone()
            .or_else(|| Some("native GGUF runtime is loaded".to_string()));
        (state, detail)
    } else if let Some(error) = runtime.last_error.clone() {
        let state = if config.model_path.exists() || config.auto_download {
            ReasoningState::Degraded
        } else {
            ReasoningState::Unavailable
        };
        (state, Some(error))
    } else if config.model_path.exists() {
        (
            ReasoningState::NotLoaded,
            Some("model file present; runtime will lazy-load on first reasoning query".to_string()),
        )
    } else if config.auto_download {
        (
            ReasoningState::NotLoaded,
            Some(
                "model file absent; runtime will auto-download on first reasoning query"
                    .to_string(),
            ),
        )
    } else {
        (
            ReasoningState::Unavailable,
            Some("model file absent and auto-download disabled".to_string()),
        )
    };

    ReasoningStatus {
        state,
        enabled: config.enabled,
        model_path: config.model_path.display().to_string(),
        auto_download: config.auto_download,
        detail,
    }
}

pub fn maybe_execute_with_query_rewrites<E: LocalTextEmbedder>(
    records: &[HydratedMemoryRecord],
    namespace: &NamespaceId,
    query: &str,
    exact_memory_id: Option<MemoryId>,
    config: SemanticExecutorConfig,
    embedder: &mut CachedTextEmbedder<E>,
) -> (
    SemanticRetrievalResult,
    Option<QueryRewriteOutcome>,
    Vec<SemanticRetrievalTrace>,
) {
    let base_result = SharedSemanticRetrievalExecutor.execute(
        records,
        namespace,
        query,
        exact_memory_id,
        config,
        embedder,
    );
    let mut traces = vec![base_result.trace.clone()];

    let reasoning_config = ReasoningConfig::from_env();
    if !reasoning_config.enabled || exact_memory_id.is_some() || !should_attempt_reasoning(query) {
        return (base_result, None, traces);
    }

    let rewrite_outcome = execute_query_rewrite(query, &reasoning_config);
    let outcome = match rewrite_outcome {
        Ok(outcome) => outcome,
        Err(reason) => {
            return (
                base_result,
                Some(QueryRewriteOutcome {
                    rewritten_queries: Vec::new(),
                    trace_note: "reasoning_query_rewrite_skipped".to_string(),
                    degraded_reason: Some(reason),
                }),
                traces,
            );
        }
    };

    if outcome.rewritten_queries.is_empty() {
        return (base_result, Some(outcome), traces);
    }

    let mut pending_results = Vec::with_capacity(outcome.rewritten_queries.len() + 1);
    pending_results.push(PendingQueryResult {
        source_label: "original".to_string(),
        query_text: query.trim().to_string(),
        result: base_result,
    });

    for (index, rewritten_query) in outcome.rewritten_queries.iter().enumerate() {
        let result = SharedSemanticRetrievalExecutor.execute(
            records,
            namespace,
            rewritten_query,
            exact_memory_id,
            config,
            embedder,
        );
        traces.push(result.trace.clone());
        pending_results.push(PendingQueryResult {
            source_label: format!("rewrite_{}", index + 1),
            query_text: rewritten_query.clone(),
            result,
        });
    }

    let merged_result = merge_semantic_results(config, pending_results);
    (merged_result, Some(outcome), traces)
}

fn should_attempt_reasoning(query: &str) -> bool {
    let trimmed = query.trim();
    if trimmed.len() < 8 {
        return false;
    }
    trimmed.split_whitespace().take(2).count() >= 2
}

fn execute_query_rewrite(
    query: &str,
    config: &ReasoningConfig,
) -> Result<QueryRewriteOutcome, String> {
    let started_at = Instant::now();
    let prompt = rewrite_prompt(query, config.max_rewrites);
    let raw_output = {
        let mut runtime = runtime()
            .lock()
            .expect("reasoning runtime lock should be available");
        let engine = ensure_engine_loaded(&mut runtime, config)?;
        match engine.generate(&prompt, config.max_output_tokens) {
            Ok(output) => output,
            Err(error) => {
                let message = format!("reasoning_generation_failed:{error}");
                runtime.last_error = Some(message.clone());
                runtime.last_event = None;
                return Err(message);
            }
        }
    };

    let elapsed_ms = started_at.elapsed().as_millis() as u64;
    if elapsed_ms > config.timeout_ms {
        let message = format!(
            "reasoning_soft_timeout_exceeded:elapsed_ms={elapsed_ms} timeout_ms={}",
            config.timeout_ms
        );
        let mut runtime = runtime()
            .lock()
            .expect("reasoning runtime lock should be available");
        runtime.last_error = Some(message.clone());
        runtime.last_event = None;
        return Err(message);
    }

    let rewritten_queries = parse_rewritten_queries(&raw_output, query, config.max_rewrites);
    let mut runtime = runtime()
        .lock()
        .expect("reasoning runtime lock should be available");
    runtime.warm_generations = runtime.warm_generations.saturating_add(1);
    runtime.last_error = None;
    runtime.last_event = Some(format!(
        "rewrite_generated:{} elapsed_ms={elapsed_ms}",
        rewritten_queries.len()
    ));

    Ok(QueryRewriteOutcome {
        trace_note: format!(
            "reasoning_query_rewrite_applied:generated={} elapsed_ms={elapsed_ms}",
            rewritten_queries.len()
        ),
        rewritten_queries,
        degraded_reason: None,
    })
}

fn ensure_engine_loaded<'a>(
    runtime: &'a mut ReasoningRuntime,
    config: &ReasoningConfig,
) -> Result<&'a Engine, String> {
    let model_path = ensure_model_available(config)?;
    let already_loaded = runtime.engine.is_some()
        && runtime
            .loaded_model_path
            .as_ref()
            .is_some_and(|loaded| loaded == &model_path);
    if !already_loaded {
        let engine = Engine::load(EngineConfig {
            model_path: model_path.display().to_string(),
            temperature: 0.0,
            top_k: 1,
            top_p: 1.0,
            repeat_penalty: 1.0,
            max_tokens: config.max_output_tokens,
            seed: Some(0),
            use_gpu: false,
            max_context_len: Some(config.max_context_tokens),
            ..Default::default()
        })
        .map_err(|error| format!("reasoning_model_load_failed:{error}"))?;
        runtime.engine = Some(engine);
        runtime.loaded_model_path = Some(model_path.clone());
        runtime.last_error = None;
        runtime.last_event = Some(format!(
            "native_reasoning_model_loaded:path={}",
            model_path.display()
        ));
    }

    runtime
        .engine
        .as_ref()
        .ok_or_else(|| "reasoning_engine_uninitialized".to_string())
}

fn ensure_model_available(config: &ReasoningConfig) -> Result<PathBuf, String> {
    if config.model_path.is_file() {
        return Ok(config.model_path.clone());
    }

    if !config.auto_download {
        return Err(format!(
            "reasoning_model_missing:model_path={} auto_download=false",
            config.model_path.display()
        ));
    }

    download_default_model(config)
}

fn download_default_model(config: &ReasoningConfig) -> Result<PathBuf, String> {
    let model_path = config.model_path.clone();
    let handle = thread::spawn(move || {
        let model_parent = model_path
            .parent()
            .ok_or_else(|| {
                format!(
                    "reasoning_model_parent_missing:model_path={}",
                    model_path.display()
                )
            })?
            .to_path_buf();
        fs::create_dir_all(&model_parent).map_err(|error| {
            format!(
                "reasoning_model_dir_create_failed:path={} error={error}",
                model_parent.display()
            )
        })?;

        let cache_dir = model_parent.join(".hf-cache");
        fs::create_dir_all(&cache_dir).map_err(|error| {
            format!(
                "reasoning_model_cache_dir_create_failed:path={} error={error}",
                cache_dir.display()
            )
        })?;

        let client = HfClient::with_cache_dir(cache_dir);
        let downloaded = client
            .download_file(
                DEFAULT_REASONING_MODEL_REPO,
                DEFAULT_REASONING_MODEL_FILENAME,
                false,
            )
            .map_err(|error| format!("reasoning_model_download_failed:{error}"))?;

        if downloaded != model_path {
            fs::copy(&downloaded, &model_path).map_err(|error| {
                format!(
                    "reasoning_model_copy_failed:from={} to={} error={error}",
                    downloaded.display(),
                    model_path.display(),
                )
            })?;
        }

        Ok::<PathBuf, String>(model_path)
    });

    handle
        .join()
        .map_err(|_| "reasoning_model_download_thread_panicked".to_string())?
}

fn merge_semantic_results(
    config: SemanticExecutorConfig,
    results: Vec<PendingQueryResult>,
) -> SemanticRetrievalResult {
    let mut merged = HashMap::<u64, MergedCandidateAccumulator>::new();
    let mut any_bounded_shortlist_truncated = false;
    let mut degraded_reasons = Vec::new();
    let mut namespace_candidate_count = 0usize;
    let mut lexical_prefilter_count = 0usize;

    for pending in &results {
        let trace = &pending.result.trace;
        namespace_candidate_count = namespace_candidate_count.max(trace.namespace_candidate_count);
        lexical_prefilter_count = lexical_prefilter_count.max(trace.lexical_prefilter_count);
        any_bounded_shortlist_truncated |= trace.bounded_shortlist_truncated;
        if let Some(reason) = trace.degraded_reason.as_deref() {
            degraded_reasons.push(reason.to_string());
        }

        for candidate in &pending.result.candidates {
            let entry = merged
                .entry(candidate.record.memory_id.0)
                .or_insert_with(|| MergedCandidateAccumulator {
                    candidate: candidate.clone(),
                    hit_count: 0,
                    source_labels: Vec::new(),
                    source_queries: Vec::new(),
                });
            entry.hit_count += 1;
            if !entry
                .source_labels
                .iter()
                .any(|label| label == &pending.source_label)
            {
                entry.source_labels.push(pending.source_label.clone());
            }
            if !entry
                .source_queries
                .iter()
                .any(|query_text| query_text == &pending.query_text)
            {
                entry.source_queries.push(pending.query_text.clone());
            }

            if candidate.ranking_score > entry.candidate.ranking_score
                || (candidate.ranking_score == entry.candidate.ranking_score
                    && candidate.semantic_score > entry.candidate.semantic_score)
                || (candidate.ranking_score == entry.candidate.ranking_score
                    && candidate.semantic_score == entry.candidate.semantic_score
                    && candidate.lexical_score > entry.candidate.lexical_score)
            {
                entry.candidate.record = candidate.record.clone();
            }
            entry.candidate.lexical_score =
                entry.candidate.lexical_score.max(candidate.lexical_score);
            entry.candidate.semantic_score =
                entry.candidate.semantic_score.max(candidate.semantic_score);
            entry.candidate.blended_score =
                entry.candidate.blended_score.max(candidate.blended_score);
            entry.candidate.ranking_score =
                entry.candidate.ranking_score.max(candidate.ranking_score);
        }
    }

    let mut candidates = merged
        .into_values()
        .map(|entry| {
            let bonus =
                REWRITE_HIT_BONUS.saturating_mul((entry.hit_count.saturating_sub(1)) as u16);
            let mut candidate = entry.candidate;
            candidate.ranking_score = candidate.ranking_score.saturating_add(bonus);
            candidate.blended_score = (candidate.blended_score
                + 0.03 * entry.hit_count.saturating_sub(1) as f32)
                .clamp(0.0, 1.0);
            (candidate, entry.hit_count)
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| right.0.ranking_score.cmp(&left.0.ranking_score))
            .then_with(|| right.0.semantic_score.total_cmp(&left.0.semantic_score))
            .then_with(|| right.0.lexical_score.total_cmp(&left.0.lexical_score))
            .then_with(|| left.0.record.memory_id.0.cmp(&right.0.record.memory_id.0))
    });
    candidates.truncate(config.result_limit);

    let mut semantic_trace = results
        .first()
        .map(|result| result.result.trace.clone())
        .unwrap_or(SemanticRetrievalTrace {
            exact_match_used: false,
            namespace_candidate_count: 0,
            lexical_prefilter_count: 0,
            semantic_candidate_count: 0,
            result_limit: config.result_limit,
            bounded_shortlist_truncated: false,
            degraded_reason: Some("reasoning_merge_without_results".to_string()),
            query_trace: None,
            batch_trace: None,
        });
    semantic_trace.exact_match_used = false;
    semantic_trace.namespace_candidate_count = namespace_candidate_count;
    semantic_trace.lexical_prefilter_count = lexical_prefilter_count;
    semantic_trace.semantic_candidate_count = candidates.len();
    semantic_trace.result_limit = config.result_limit;
    semantic_trace.bounded_shortlist_truncated = any_bounded_shortlist_truncated;
    semantic_trace.degraded_reason = if degraded_reasons.is_empty() {
        None
    } else {
        Some(deduplicate_strings(degraded_reasons).join("|"))
    };
    semantic_trace.query_trace = None;
    semantic_trace.batch_trace = None;

    SemanticRetrievalResult {
        candidates: candidates
            .into_iter()
            .map(|(candidate, _)| candidate)
            .collect(),
        trace: semantic_trace,
    }
}

fn rewrite_prompt(query: &str, max_rewrites: usize) -> String {
    format!(
        "Rewrite the query for local memory retrieval.\n\
Return at most {max_rewrites} alternate queries.\n\
Rules:\n\
- Output only lines that start with QUERY:\n\
- Keep each rewrite short, concrete, and retrieval-focused.\n\
- Preserve entities, errors, filenames, libraries, and technical terms.\n\
- Prefer likely synonyms and debugging wording.\n\
- Do not explain.\n\
Original query:\n\
{query}\n"
    )
}

fn parse_rewritten_queries(
    raw_output: &str,
    original_query: &str,
    max_rewrites: usize,
) -> Vec<String> {
    let normalized_original = normalize_query(original_query);
    let mut parsed = Vec::new();
    for line in raw_output.lines() {
        let candidate = line
            .trim()
            .strip_prefix("QUERY:")
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let Some(candidate) = candidate else {
            continue;
        };
        let normalized_candidate = normalize_query(candidate);
        if normalized_candidate.is_empty() || normalized_candidate == normalized_original {
            continue;
        }
        if parsed
            .iter()
            .any(|existing: &String| normalize_query(existing) == normalized_candidate)
        {
            continue;
        }
        parsed.push(candidate.to_string());
        if parsed.len() >= max_rewrites {
            break;
        }
    }
    parsed
}

fn normalize_query(text: &str) -> String {
    text.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn deduplicate_strings(values: Vec<String>) -> Vec<String> {
    let mut unique = Vec::new();
    for value in values {
        if !unique.iter().any(|existing| existing == &value) {
            unique.push(value);
        }
    }
    unique
}

fn env_bool(name: &str, default: bool) -> bool {
    match env::var(name) {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => default,
        },
        Err(_) => default,
    }
}

fn env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(default)
}

fn env_u64(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(default)
}

fn default_model_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".membrain")
        .join("models")
        .join(DEFAULT_REASONING_MODEL_LOCAL_NAME)
}

fn running_under_test_harness() -> bool {
    env::var_os("RUST_TEST_THREADS").is_some()
        || env::var_os("NEXTEST").is_some()
        || env::var_os("CARGO_TARGET_TMPDIR").is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rewritten_queries_and_deduplicates() {
        let raw = "QUERY: rust ownership borrow checker fix\nQUERY: rust ownership borrow checker fix\nQUERY: lifetime reference mismatch";
        let parsed = parse_rewritten_queries(
            raw,
            "Rust borrow checker problem",
            DEFAULT_REASONING_MAX_REWRITES,
        );
        assert_eq!(
            parsed,
            vec![
                "rust ownership borrow checker fix".to_string(),
                "lifetime reference mismatch".to_string()
            ]
        );
    }

    #[test]
    fn skips_short_queries() {
        assert!(!should_attempt_reasoning("cache"));
        assert!(should_attempt_reasoning("cache invalidation bug"));
    }
}
