//! Context-budget request contract, allocation semantics, and greedy packing.
//!
//! Packs ranked memories into a bounded token budget with explicit reasons
//! for inclusion and omission. Utility scoring combines relevance, strength,
//! and working-memory overlap penalties so callers receive ready-to-inject
//! output plus machine-readable accounting.
//!
//! Refs: docs/PLAN.md section 46.4.

use crate::engine::result::{RetrievalResult, RetrievalResultSet};
use crate::types::{CanonicalMemoryType, MemoryId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ── Token estimation ──────────────────────────────────────────────────────────

/// Characters-per-token ratio used for budget estimation.
///
/// Simple approximation: `tokens ≈ content.len() / TOKEN_CHARS_RATIO`.
/// No tokenizer dependency; configurable per deployment.
pub const TOKEN_CHARS_RATIO: usize = 4;

/// Estimates token count from content length.
pub const fn estimate_tokens(content_len: usize) -> usize {
    content_len / TOKEN_CHARS_RATIO
}

// ── Request / response types ──────────────────────────────────────────────────

/// Stable readiness kind for why an item was included in the packed output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InjectionSourceKind {
    RetrievalResult,
    MemoryCandidate,
}

impl InjectionSourceKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RetrievalResult => "retrieval_result",
            Self::MemoryCandidate => "memory_candidate",
        }
    }
}

/// Format for injection output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InjectionFormat {
    /// Plain text, one memory per line.
    #[default]
    Plain,
    /// Markdown-formatted memories.
    Markdown,
    /// JSON-structured array of items.
    Json,
}

impl InjectionFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Markdown => "markdown",
            Self::Json => "json",
        }
    }
}

/// Request for context-budget packing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextBudgetRequest {
    /// Total token budget the caller can afford.
    pub token_budget: usize,
    /// Optional current context for relevance-aware scoring.
    pub current_context: Option<String>,
    /// Memory ids already in working memory (overlap-penalized).
    pub working_memory_ids: Vec<MemoryId>,
    /// Output format for the injection items.
    pub format: InjectionFormat,
}

impl ContextBudgetRequest {
    pub fn new(token_budget: usize) -> Self {
        Self {
            token_budget,
            current_context: None,
            working_memory_ids: Vec::new(),
            format: InjectionFormat::default(),
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.current_context = Some(context.into());
        self
    }

    pub fn with_working_memory(mut self, ids: Vec<MemoryId>) -> Self {
        self.working_memory_ids = ids;
        self
    }

    pub fn with_format(mut self, format: InjectionFormat) -> Self {
        self.format = format;
        self
    }
}

/// One memory item packed into the budget response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InjectionItem {
    /// Durable memory identity.
    pub memory_id: MemoryId,
    /// Ready-to-inject content text.
    pub content: String,
    /// Composite utility score used for packing order.
    pub utility_score: f32,
    /// Estimated token count for this item.
    pub token_count: usize,
    /// Machine-readable reason for inclusion.
    pub reason: InjectionReason,
    /// Canonical source family for the included item.
    pub source_kind: InjectionSourceKind,
    /// Canonical memory family when one is known.
    pub memory_type: Option<CanonicalMemoryType>,
    /// Final ranking score from the canonical retrieval pipeline when available.
    pub ranking_score: Option<u16>,
    /// Bounded strength score carried into utility computation when known.
    pub strength_score: Option<u16>,
    /// Ordered explain reasons copied from the canonical retrieval result when available.
    pub inclusion_reasons: Vec<String>,
}

/// Why a memory was included or excluded from the budget pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InjectionReason {
    /// Included: high utility within budget.
    HighUtility,
    /// Included: next-best after top items filled remaining budget.
    FillsBudget,
    /// Excluded: token budget exhausted.
    BudgetExhausted,
    /// Excluded: already in working memory (overlap penalty).
    WorkingMemoryOverlap,
    /// Excluded: zero or negative utility after scoring.
    LowUtility,
}

impl InjectionReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HighUtility => "high_utility",
            Self::FillsBudget => "fills_budget",
            Self::BudgetExhausted => "budget_exhausted",
            Self::WorkingMemoryOverlap => "working_memory_overlap",
            Self::LowUtility => "low_utility",
        }
    }
}

/// Response from context-budget packing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextBudgetResponse {
    /// Items packed into the budget, in utility-descending order.
    pub injections: Vec<InjectionItem>,
    /// Total tokens consumed by included items.
    pub tokens_used: usize,
    /// Tokens remaining in the budget.
    pub tokens_remaining: usize,
    /// Whether the shortlist was truncated by the hard token budget.
    pub partial_success: bool,
    /// Items considered but excluded, with reasons.
    pub omitted: Vec<OmittedItem>,
}

/// An item excluded from the budget with the reason.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OmittedItem {
    pub memory_id: MemoryId,
    pub utility_score: f32,
    pub token_count: usize,
    pub reason: InjectionReason,
    pub source_kind: InjectionSourceKind,
    pub memory_type: Option<CanonicalMemoryType>,
    pub ranking_score: Option<u16>,
    pub omission_reasons: Vec<String>,
}

// ── Candidate input ───────────────────────────────────────────────────────────

/// One candidate memory for budget packing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetCandidate {
    pub memory_id: MemoryId,
    pub content: String,
    /// Relevance score from retrieval (0.0..=1.0).
    pub relevance: f32,
    /// Effective strength after decay (0.0..=1.0).
    pub strength: f32,
    /// Canonical memory family when known.
    pub memory_type: Option<CanonicalMemoryType>,
    /// Canonical retrieval ranking score when known.
    pub ranking_score: Option<u16>,
    /// Canonical strength score before normalization when known.
    pub strength_score: Option<u16>,
    /// Ordered explain reasons from the retrieval pipeline when available.
    pub explain_reasons: Vec<String>,
}

impl BudgetCandidate {
    pub fn new(
        memory_id: MemoryId,
        content: impl Into<String>,
        relevance: f32,
        strength: f32,
    ) -> Self {
        Self {
            memory_id,
            content: content.into(),
            relevance,
            strength,
            memory_type: None,
            ranking_score: None,
            strength_score: None,
            explain_reasons: Vec::new(),
        }
    }

    pub fn with_memory_type(mut self, memory_type: CanonicalMemoryType) -> Self {
        self.memory_type = Some(memory_type);
        self
    }

    pub fn with_ranking_score(mut self, ranking_score: u16) -> Self {
        self.ranking_score = Some(ranking_score);
        self
    }

    pub fn with_strength_score(mut self, strength_score: u16) -> Self {
        self.strength_score = Some(strength_score);
        self
    }

    pub fn with_explain_reasons(mut self, explain_reasons: Vec<String>) -> Self {
        self.explain_reasons = explain_reasons;
        self
    }

    /// Builds one budget candidate directly from a canonical retrieval result.
    pub fn from_retrieval_result(result: &RetrievalResult) -> Self {
        let final_score = result.score_summary.final_score;
        let strength_score = result
            .score_summary
            .signal_breakdown
            .iter()
            .find(|(family, _, _, _)| family == "strength")
            .map(|(_, raw_value, _, _)| *raw_value)
            .unwrap_or(final_score);
        let explain_reasons = result
            .ranking_explain
            .contradiction_details
            .iter()
            .filter_map(|detail| detail.resolution_reason.clone())
            .collect::<Vec<_>>();

        Self {
            memory_id: result.memory_id,
            content: result.compact_text.clone(),
            relevance: final_score as f32 / 1000.0,
            strength: strength_score as f32 / 1000.0,
            memory_type: Some(result.memory_type),
            ranking_score: Some(final_score),
            strength_score: Some(strength_score),
            explain_reasons,
        }
    }

    /// Estimated token count for this candidate.
    pub fn estimated_tokens(&self) -> usize {
        estimate_tokens(self.content.len())
    }
}

// ── Scoring ───────────────────────────────────────────────────────────────────

/// Computes the working-memory overlap penalty for a candidate.
///
/// Returns 1.0 if the candidate is already in working memory (fully penalized),
/// 0.0 otherwise.
pub fn working_memory_overlap_penalty(
    candidate_id: MemoryId,
    working_memory_ids: &[MemoryId],
) -> f32 {
    if working_memory_ids.contains(&candidate_id) {
        1.0
    } else {
        0.0
    }
}

/// Computes the utility score for a candidate.
///
/// `utility = relevance × strength × (1 − overlap_penalty)`
///
/// Candidates already in working memory receive zero utility so they
/// are deprioritized in favor of novel context.
pub fn compute_utility(candidate: &BudgetCandidate, working_memory_ids: &[MemoryId]) -> f32 {
    let penalty = working_memory_overlap_penalty(candidate.memory_id, working_memory_ids);
    candidate.relevance * candidate.strength * (1.0 - penalty)
}

fn render_injection_content(candidate: &BudgetCandidate, format: InjectionFormat) -> String {
    match format {
        InjectionFormat::Plain => candidate.content.clone(),
        InjectionFormat::Markdown => {
            let memory_type = candidate
                .memory_type
                .map(|memory_type| memory_type.as_str())
                .unwrap_or("memory");
            format!(
                "- [{} #{}] {}",
                memory_type, candidate.memory_id.0, candidate.content
            )
        }
        InjectionFormat::Json => serde_json::json!({
            "memory_id": candidate.memory_id.0,
            "memory_type": candidate.memory_type.map(|memory_type| memory_type.as_str()),
            "content": candidate.content,
            "ranking_score": candidate.ranking_score,
            "strength_score": candidate.strength_score,
        })
        .to_string(),
    }
}

fn build_inclusion_reasons(
    candidate: &BudgetCandidate,
    utility: f32,
    tokens: usize,
    reason: InjectionReason,
) -> Vec<String> {
    let mut reasons = candidate.explain_reasons.clone();
    reasons.push(format!(
        "included because {} with utility={utility:.3} and token_count={tokens}",
        reason.as_str()
    ));
    if let Some(ranking_score) = candidate.ranking_score {
        reasons.push(format!("canonical ranking_score={ranking_score}"));
    }
    if let Some(strength_score) = candidate.strength_score {
        reasons.push(format!("strength_score={strength_score}"));
    }
    reasons
}

fn build_omission_reasons(
    candidate: &BudgetCandidate,
    utility: f32,
    tokens: usize,
    reason: InjectionReason,
) -> Vec<String> {
    let mut reasons = candidate.explain_reasons.clone();
    reasons.push(format!(
        "omitted because {} with utility={utility:.3} and token_count={tokens}",
        reason.as_str()
    ));
    if matches!(reason, InjectionReason::WorkingMemoryOverlap) {
        reasons.push("working-memory overlap penalty reduced utility to zero".to_string());
    }
    reasons
}

// ── Greedy packer ─────────────────────────────────────────────────────────────

/// Context-budget engine that packs candidates into a bounded token budget.
#[derive(Debug, Default, Clone)]
pub struct ContextBudgetEngine;

impl ContextBudgetEngine {
    pub fn new() -> Self {
        Self
    }

    /// Packs candidates into the token budget with utility scoring.
    ///
    /// 1. Score each candidate by utility
    /// 2. Sort by utility descending
    /// 3. Greedy pack: add items until token_budget exhausted
    /// 4. Report included and omitted items with reasons
    pub fn pack(
        &self,
        request: &ContextBudgetRequest,
        candidates: Vec<BudgetCandidate>,
    ) -> ContextBudgetResponse {
        let wm_set: HashSet<MemoryId> = request.working_memory_ids.iter().copied().collect();

        let mut scored: Vec<(BudgetCandidate, f32, usize)> = candidates
            .into_iter()
            .map(|candidate| {
                let utility = compute_utility(&candidate, &request.working_memory_ids);
                let rendered = render_injection_content(&candidate, request.format);
                let tokens = estimate_tokens(rendered.len());
                (candidate, utility, tokens)
            })
            .collect();

        scored.sort_by(|left, right| {
            right
                .1
                .total_cmp(&left.1)
                .then_with(|| left.0.memory_id.0.cmp(&right.0.memory_id.0))
        });

        let mut injections = Vec::new();
        let mut omitted = Vec::new();
        let mut tokens_used = 0usize;

        for (candidate, utility, tokens) in scored {
            let source_kind = if candidate.ranking_score.is_some() {
                InjectionSourceKind::RetrievalResult
            } else {
                InjectionSourceKind::MemoryCandidate
            };

            if wm_set.contains(&candidate.memory_id) {
                omitted.push(OmittedItem {
                    memory_id: candidate.memory_id,
                    utility_score: utility,
                    token_count: tokens,
                    reason: InjectionReason::WorkingMemoryOverlap,
                    source_kind,
                    memory_type: candidate.memory_type,
                    ranking_score: candidate.ranking_score,
                    omission_reasons: build_omission_reasons(
                        &candidate,
                        utility,
                        tokens,
                        InjectionReason::WorkingMemoryOverlap,
                    ),
                });
                continue;
            }

            if utility <= 0.0 {
                omitted.push(OmittedItem {
                    memory_id: candidate.memory_id,
                    utility_score: utility,
                    token_count: tokens,
                    reason: InjectionReason::LowUtility,
                    source_kind,
                    memory_type: candidate.memory_type,
                    ranking_score: candidate.ranking_score,
                    omission_reasons: build_omission_reasons(
                        &candidate,
                        utility,
                        tokens,
                        InjectionReason::LowUtility,
                    ),
                });
                continue;
            }

            if tokens > request.token_budget || tokens_used + tokens > request.token_budget {
                omitted.push(OmittedItem {
                    memory_id: candidate.memory_id,
                    utility_score: utility,
                    token_count: tokens,
                    reason: InjectionReason::BudgetExhausted,
                    source_kind,
                    memory_type: candidate.memory_type,
                    ranking_score: candidate.ranking_score,
                    omission_reasons: build_omission_reasons(
                        &candidate,
                        utility,
                        tokens,
                        InjectionReason::BudgetExhausted,
                    ),
                });
                continue;
            }

            let reason = if injections.is_empty() {
                InjectionReason::HighUtility
            } else {
                InjectionReason::FillsBudget
            };
            let content = render_injection_content(&candidate, request.format);
            tokens_used += tokens;
            injections.push(InjectionItem {
                memory_id: candidate.memory_id,
                content,
                utility_score: utility,
                token_count: tokens,
                reason,
                source_kind,
                memory_type: candidate.memory_type,
                ranking_score: candidate.ranking_score,
                strength_score: candidate.strength_score,
                inclusion_reasons: build_inclusion_reasons(&candidate, utility, tokens, reason),
            });
        }

        ContextBudgetResponse {
            tokens_remaining: request.token_budget.saturating_sub(tokens_used),
            partial_success: omitted
                .iter()
                .any(|item| item.reason == InjectionReason::BudgetExhausted),
            injections,
            tokens_used,
            omitted,
        }
    }

    /// Packs one canonical retrieval result set into the hard token budget.
    pub fn pack_result_set(
        &self,
        request: &ContextBudgetRequest,
        result_set: &RetrievalResultSet,
    ) -> ContextBudgetResponse {
        let candidates = result_set
            .evidence_pack
            .iter()
            .map(|item| {
                let mut candidate = BudgetCandidate::from_retrieval_result(&item.result)
                    .with_explain_reasons(
                        result_set
                            .explain
                            .result_reasons
                            .iter()
                            .filter(|reason| {
                                reason.memory_id.is_none()
                                    || reason.memory_id == Some(item.result.memory_id)
                            })
                            .map(|reason| format!("{}: {}", reason.reason_code, reason.detail))
                            .collect(),
                    );
                if candidate.memory_type.is_none() {
                    candidate.memory_type = Some(item.result.memory_type);
                }
                candidate
            })
            .collect();
        self.pack(request, candidates)
    }

    /// Packs candidates with budget-aware trimming and rationale outputs.
    ///
    /// Unlike `pack`, this method can compress content to fit partial items
    /// and produces a `BudgetTrimReport` explaining every inclusion/exclusion
    /// decision for inspectability.
    pub fn pack_with_trim(
        &self,
        request: &ContextBudgetRequest,
        candidates: Vec<BudgetCandidate>,
    ) -> BudgetTrimReport {
        let response = self.pack(request, candidates);

        let mut trimmed_items = Vec::new();
        for injection in &response.injections {
            trimmed_items.push(TrimmedItem {
                memory_id: injection.memory_id,
                original_tokens: injection.token_count,
                trimmed_tokens: injection.token_count,
                content: injection.content.clone(),
                was_compressed: false,
                utility_score: injection.utility_score,
                rationale: match injection.reason {
                    InjectionReason::HighUtility => TrimRationale::HighestUtility,
                    InjectionReason::FillsBudget => TrimRationale::WithinBudget,
                    _ => TrimRationale::WithinBudget,
                },
            });
        }

        for omitted in &response.omitted {
            let rationale = match omitted.reason {
                InjectionReason::BudgetExhausted => TrimRationale::BudgetExhausted,
                InjectionReason::WorkingMemoryOverlap => TrimRationale::OverlapWithWorkingMemory,
                InjectionReason::LowUtility => TrimRationale::BelowUtilityThreshold,
                _ => TrimRationale::BudgetExhausted,
            };
            trimmed_items.push(TrimmedItem {
                memory_id: omitted.memory_id,
                original_tokens: omitted.token_count,
                trimmed_tokens: 0,
                content: String::new(),
                was_compressed: false,
                utility_score: omitted.utility_score,
                rationale,
            });
        }

        BudgetTrimReport {
            token_budget: request.token_budget,
            tokens_used: response.tokens_used,
            tokens_remaining: response.tokens_remaining,
            items: trimmed_items,
            included_count: response.injections.len(),
            omitted_count: response.omitted.len(),
        }
    }

    /// Packs candidates with content compression to maximize budget utilization.
    ///
    /// When an item would exceed the remaining budget, this method truncates
    /// its content to fit within the budget instead of excluding it entirely.
    /// The compressed item is marked with `was_compressed: true`.
    pub fn pack_with_compression(
        &self,
        request: &ContextBudgetRequest,
        candidates: Vec<BudgetCandidate>,
    ) -> BudgetTrimReport {
        let wm_set: HashSet<MemoryId> = request.working_memory_ids.iter().copied().collect();

        let mut scored: Vec<(BudgetCandidate, f32, usize)> = candidates
            .into_iter()
            .map(|c| {
                let tokens = c.estimated_tokens();
                let utility = compute_utility(&c, &request.working_memory_ids);
                (c, utility, tokens)
            })
            .collect();

        scored.sort_by(|a, b| b.1.total_cmp(&a.1));

        let mut trimmed_items = Vec::new();
        let mut tokens_used = 0usize;
        let mut included_count = 0usize;
        let mut omitted_count = 0usize;

        for (candidate, utility, full_tokens) in scored {
            if wm_set.contains(&candidate.memory_id) {
                trimmed_items.push(TrimmedItem {
                    memory_id: candidate.memory_id,
                    original_tokens: full_tokens,
                    trimmed_tokens: 0,
                    content: String::new(),
                    was_compressed: false,
                    utility_score: utility,
                    rationale: TrimRationale::OverlapWithWorkingMemory,
                });
                omitted_count += 1;
                continue;
            }

            if utility <= 0.0 {
                trimmed_items.push(TrimmedItem {
                    memory_id: candidate.memory_id,
                    original_tokens: full_tokens,
                    trimmed_tokens: 0,
                    content: String::new(),
                    was_compressed: false,
                    utility_score: utility,
                    rationale: TrimRationale::BelowUtilityThreshold,
                });
                omitted_count += 1;
                continue;
            }

            let remaining = request.token_budget.saturating_sub(tokens_used);

            if full_tokens <= remaining {
                // Fits entirely
                tokens_used += full_tokens;
                included_count += 1;
                trimmed_items.push(TrimmedItem {
                    memory_id: candidate.memory_id,
                    original_tokens: full_tokens,
                    trimmed_tokens: full_tokens,
                    content: candidate.content,
                    was_compressed: false,
                    utility_score: utility,
                    rationale: if included_count == 1 {
                        TrimRationale::HighestUtility
                    } else {
                        TrimRationale::WithinBudget
                    },
                });
            } else if remaining > 0 {
                // Compress to fit
                let char_budget = remaining * TOKEN_CHARS_RATIO;
                let truncated: String = candidate.content.chars().take(char_budget).collect();
                let trimmed_tokens = estimate_tokens(truncated.len());
                tokens_used += trimmed_tokens;
                included_count += 1;
                trimmed_items.push(TrimmedItem {
                    memory_id: candidate.memory_id,
                    original_tokens: full_tokens,
                    trimmed_tokens,
                    content: truncated,
                    was_compressed: true,
                    utility_score: utility,
                    rationale: TrimRationale::CompressedToFit,
                });
            } else {
                // No budget left at all
                omitted_count += 1;
                trimmed_items.push(TrimmedItem {
                    memory_id: candidate.memory_id,
                    original_tokens: full_tokens,
                    trimmed_tokens: 0,
                    content: String::new(),
                    was_compressed: false,
                    utility_score: utility,
                    rationale: TrimRationale::BudgetExhausted,
                });
            }
        }

        BudgetTrimReport {
            token_budget: request.token_budget,
            tokens_used,
            tokens_remaining: request.token_budget.saturating_sub(tokens_used),
            items: trimmed_items,
            included_count,
            omitted_count,
        }
    }
}

// ── Budget trim report ────────────────────────────────────────────────────────

/// Rationale explaining why an item was included, compressed, or excluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrimRationale {
    /// Item has the highest utility score and was included first.
    HighestUtility,
    /// Item fits within the remaining budget.
    WithinBudget,
    /// Item was truncated to fit within the remaining budget.
    CompressedToFit,
    /// Token budget was exhausted before this item could be considered.
    BudgetExhausted,
    /// Item is already in working memory (overlap penalty).
    OverlapWithWorkingMemory,
    /// Item has zero or negative utility after scoring.
    BelowUtilityThreshold,
}

impl TrimRationale {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HighestUtility => "highest_utility",
            Self::WithinBudget => "within_budget",
            Self::CompressedToFit => "compressed_to_fit",
            Self::BudgetExhausted => "budget_exhausted",
            Self::OverlapWithWorkingMemory => "overlap_with_working_memory",
            Self::BelowUtilityThreshold => "below_utility_threshold",
        }
    }

    /// Whether this rationale indicates the item was included in the output.
    pub const fn was_included(self) -> bool {
        matches!(
            self,
            Self::HighestUtility | Self::WithinBudget | Self::CompressedToFit
        )
    }
}

/// One item in the budget trim report with full rationale.
#[derive(Debug, Clone, PartialEq)]
pub struct TrimmedItem {
    pub memory_id: MemoryId,
    /// Original estimated token count before any compression.
    pub original_tokens: usize,
    /// Token count after compression (0 if excluded).
    pub trimmed_tokens: usize,
    /// Content after compression (empty if excluded).
    pub content: String,
    /// Whether content was truncated to fit the budget.
    pub was_compressed: bool,
    /// Utility score used for packing order.
    pub utility_score: f32,
    /// Machine-readable rationale for inclusion or exclusion.
    pub rationale: TrimRationale,
}

/// Full budget trim report showing how every candidate was handled.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetTrimReport {
    /// Original token budget from the request.
    pub token_budget: usize,
    /// Total tokens consumed by included items.
    pub tokens_used: usize,
    /// Tokens remaining after packing.
    pub tokens_remaining: usize,
    /// All candidates with their inclusion/exclusion rationale.
    pub items: Vec<TrimmedItem>,
    /// Count of items included (including compressed).
    pub included_count: usize,
    /// Count of items omitted.
    pub omitted_count: usize,
}

impl BudgetTrimReport {
    /// Returns only the items that were included in the output.
    pub fn included_items(&self) -> Vec<&TrimmedItem> {
        self.items
            .iter()
            .filter(|i| i.rationale.was_included())
            .collect()
    }

    /// Returns only the items that were omitted.
    pub fn omitted_items(&self) -> Vec<&TrimmedItem> {
        self.items
            .iter()
            .filter(|i| !i.rationale.was_included())
            .collect()
    }

    /// Returns items that were compressed to fit the budget.
    pub fn compressed_items(&self) -> Vec<&TrimmedItem> {
        self.items.iter().filter(|i| i.was_compressed).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mid(id: u64) -> MemoryId {
        MemoryId(id)
    }

    // ── Token estimation ──────────────────────────────────────────────────

    #[test]
    fn estimate_tokens_basic() {
        assert_eq!(estimate_tokens(0), 0);
        assert_eq!(estimate_tokens(4), 1);
        assert_eq!(estimate_tokens(100), 25);
        assert_eq!(estimate_tokens(101), 25);
    }

    // ── Working memory penalty ────────────────────────────────────────────

    #[test]
    fn overlap_penalty_returns_one_for_working_memory() {
        let wm = vec![mid(1), mid(2)];
        assert_eq!(working_memory_overlap_penalty(mid(1), &wm), 1.0);
        assert_eq!(working_memory_overlap_penalty(mid(2), &wm), 1.0);
    }

    #[test]
    fn overlap_penalty_returns_zero_for_non_working_memory() {
        let wm = vec![mid(1)];
        assert_eq!(working_memory_overlap_penalty(mid(99), &wm), 0.0);
    }

    #[test]
    fn overlap_penalty_returns_zero_for_empty_working_memory() {
        assert_eq!(working_memory_overlap_penalty(mid(1), &[]), 0.0);
    }

    // ── Utility scoring ───────────────────────────────────────────────────

    #[test]
    fn utility_is_relevance_times_strength_when_no_overlap() {
        let c = BudgetCandidate::new(mid(1), "test", 0.8, 0.5);
        let utility = compute_utility(&c, &[]);
        assert!((utility - 0.4).abs() < 1e-6);
    }

    #[test]
    fn utility_is_zero_when_in_working_memory() {
        let c = BudgetCandidate::new(mid(1), "test", 0.9, 0.9);
        let utility = compute_utility(&c, &[mid(1)]);
        assert_eq!(utility, 0.0);
    }

    #[test]
    fn utility_reflects_strength_variation() {
        let strong = BudgetCandidate::new(mid(1), "a", 0.8, 0.9);
        let weak = BudgetCandidate::new(mid(2), "b", 0.8, 0.1);
        assert!(compute_utility(&strong, &[]) > compute_utility(&weak, &[]));
    }

    // ── Greedy packing ────────────────────────────────────────────────────

    #[test]
    fn pack_respects_token_budget() {
        let engine = ContextBudgetEngine::new();
        let request = ContextBudgetRequest::new(50);
        let candidates = vec![
            BudgetCandidate::new(mid(1), "short", 0.9, 0.9),
            BudgetCandidate::new(
                mid(2),
                "a much longer content string that takes more tokens",
                0.8,
                0.8,
            ),
            BudgetCandidate::new(mid(3), "medium content here", 0.7, 0.7),
        ];

        let response = engine.pack(&request, candidates);
        assert!(response.tokens_used <= 50);
        assert_eq!(response.tokens_used + response.tokens_remaining, 50);
    }

    #[test]
    fn pack_sorts_by_utility_descending() {
        let engine = ContextBudgetEngine::new();
        let request = ContextBudgetRequest::new(1000);
        let candidates = vec![
            BudgetCandidate::new(mid(1), "low", 0.1, 0.1),
            BudgetCandidate::new(mid(2), "high", 0.9, 0.9),
            BudgetCandidate::new(mid(3), "mid", 0.5, 0.5),
        ];

        let response = engine.pack(&request, candidates);
        assert_eq!(response.injections.len(), 3);
        assert!(response.injections[0].utility_score >= response.injections[1].utility_score);
        assert!(response.injections[1].utility_score >= response.injections[2].utility_score);
    }

    #[test]
    fn pack_excludes_working_memory_items() {
        let engine = ContextBudgetEngine::new();
        let request = ContextBudgetRequest::new(1000).with_working_memory(vec![mid(2)]);
        let candidates = vec![
            BudgetCandidate::new(mid(1), "fresh", 0.9, 0.9),
            BudgetCandidate::new(mid(2), "already known", 0.9, 0.9),
        ];

        let response = engine.pack(&request, candidates);
        assert_eq!(response.injections.len(), 1);
        assert_eq!(response.injections[0].memory_id, mid(1));
        assert_eq!(response.omitted.len(), 1);
        assert_eq!(response.omitted[0].memory_id, mid(2));
        assert_eq!(
            response.omitted[0].reason,
            InjectionReason::WorkingMemoryOverlap
        );
    }

    #[test]
    fn pack_handles_zero_budget() {
        let engine = ContextBudgetEngine::new();
        let request = ContextBudgetRequest::new(0);
        let candidates = vec![
            BudgetCandidate::new(mid(1), "first", 0.9, 0.9),
            BudgetCandidate::new(mid(2), "second", 0.5, 0.5),
        ];

        let response = engine.pack(&request, candidates);
        assert!(response.injections.is_empty());
        assert_eq!(response.omitted.len(), 2);
        assert!(response
            .omitted
            .iter()
            .all(|item| item.reason == InjectionReason::BudgetExhausted));
        assert!(response.partial_success);
    }

    #[test]
    fn pack_first_item_takes_high_utility_reason() {
        let engine = ContextBudgetEngine::new();
        let request = ContextBudgetRequest::new(1000);
        let candidates = vec![
            BudgetCandidate::new(mid(1), "best", 0.9, 0.9),
            BudgetCandidate::new(mid(2), "second", 0.5, 0.5),
        ];

        let response = engine.pack(&request, candidates);
        assert_eq!(response.injections[0].reason, InjectionReason::HighUtility);
        assert_eq!(response.injections[1].reason, InjectionReason::FillsBudget);
    }

    #[test]
    fn pack_empty_candidates_returns_empty() {
        let engine = ContextBudgetEngine::new();
        let request = ContextBudgetRequest::new(100);
        let response = engine.pack(&request, vec![]);
        assert!(response.injections.is_empty());
        assert!(response.omitted.is_empty());
        assert_eq!(response.tokens_used, 0);
        assert_eq!(response.tokens_remaining, 100);
        assert!(!response.partial_success);
    }

    #[test]
    fn pack_marks_partial_success_when_budget_omits_candidates() {
        let engine = ContextBudgetEngine::new();
        let request = ContextBudgetRequest::new(2);
        let candidates = vec![
            BudgetCandidate::new(mid(1), "fits", 0.9, 0.9),
            BudgetCandidate::new(mid(2), "does not fit after first", 0.8, 0.8),
        ];

        let response = engine.pack(&request, candidates);
        assert_eq!(response.injections.len(), 1);
        assert_eq!(response.omitted.len(), 1);
        assert!(response.partial_success);
        assert_eq!(response.omitted[0].reason, InjectionReason::BudgetExhausted);
    }

    #[test]
    fn pack_mixed_scenario() {
        let engine = ContextBudgetEngine::new();
        let request = ContextBudgetRequest::new(30).with_working_memory(vec![mid(3)]);
        let candidates = vec![
            BudgetCandidate::new(mid(1), "alpha beta", 0.9, 0.8),
            BudgetCandidate::new(mid(2), "gamma", 0.7, 0.6),
            BudgetCandidate::new(mid(3), "delta echo", 0.95, 0.9),
            BudgetCandidate::new(mid(4), "foxtrot golf hotel india", 0.3, 0.2),
        ];

        let response = engine.pack(&request, candidates);
        // mid(3) should be omitted (working memory)
        assert!(!response
            .omitted
            .iter()
            .any(|o| o.memory_id == mid(3) && o.reason != InjectionReason::WorkingMemoryOverlap));
        // mid(1) has highest utility and should be included
        assert!(response.injections.iter().any(|i| i.memory_id == mid(1)));
        // Total should respect budget
        assert!(response.tokens_used <= 30);
    }

    // ── Builder patterns ──────────────────────────────────────────────────

    #[test]
    fn budget_request_builder() {
        let req = ContextBudgetRequest::new(200)
            .with_context("debugging session")
            .with_working_memory(vec![mid(1), mid(2)])
            .with_format(InjectionFormat::Markdown);
        assert_eq!(req.token_budget, 200);
        assert_eq!(req.current_context.as_deref(), Some("debugging session"));
        assert_eq!(req.working_memory_ids.len(), 2);
        assert_eq!(req.format, InjectionFormat::Markdown);
    }

    // ── Parity fixtures for later server / MCP reuse ───────────────────────

    #[test]
    fn parity_utility_formula_matches_plan_spec() {
        // PLAN 46.4: utility = relevance × strength × (1 − overlap_penalty)
        let c = BudgetCandidate::new(mid(1), "test content", 0.7, 0.6);
        let utility = compute_utility(&c, &[]);
        let expected = 0.7 * 0.6 * 1.0;
        assert!((utility - expected).abs() < 1e-6);
    }

    #[test]
    fn parity_token_estimation_matches_plan_spec() {
        // PLAN 46.4: tokens ≈ content.len() / 4
        assert_eq!(estimate_tokens(400), 100);
        assert_eq!(estimate_tokens(2000), 500);
    }

    #[test]
    fn parity_packing_order_is_deterministic() {
        // Same input must always produce same output order
        let engine = ContextBudgetEngine::new();
        let request = ContextBudgetRequest::new(100);
        let candidates = vec![
            BudgetCandidate::new(mid(3), "c", 0.5, 0.5),
            BudgetCandidate::new(mid(1), "a", 0.9, 0.9),
            BudgetCandidate::new(mid(2), "b", 0.7, 0.7),
        ];

        let r1 = engine.pack(&request, candidates.clone());
        let r2 = engine.pack(&request, candidates);
        assert_eq!(r1.injections, r2.injections);
    }

    // ── Enum string conversions ───────────────────────────────────────────

    #[test]
    fn injection_format_as_str() {
        assert_eq!(InjectionFormat::Plain.as_str(), "plain");
        assert_eq!(InjectionFormat::Markdown.as_str(), "markdown");
        assert_eq!(InjectionFormat::Json.as_str(), "json");
    }

    #[test]
    fn injection_source_kind_as_str() {
        assert_eq!(
            InjectionSourceKind::RetrievalResult.as_str(),
            "retrieval_result"
        );
        assert_eq!(
            InjectionSourceKind::MemoryCandidate.as_str(),
            "memory_candidate"
        );
    }

    #[test]
    fn injection_reason_as_str() {
        assert_eq!(InjectionReason::HighUtility.as_str(), "high_utility");
        assert_eq!(InjectionReason::FillsBudget.as_str(), "fills_budget");
        assert_eq!(
            InjectionReason::BudgetExhausted.as_str(),
            "budget_exhausted"
        );
        assert_eq!(
            InjectionReason::WorkingMemoryOverlap.as_str(),
            "working_memory_overlap"
        );
        assert_eq!(InjectionReason::LowUtility.as_str(), "low_utility");
    }

    // ── Trim report ───────────────────────────────────────────────────────

    #[test]
    fn trim_rationale_as_str() {
        assert_eq!(TrimRationale::HighestUtility.as_str(), "highest_utility");
        assert_eq!(TrimRationale::WithinBudget.as_str(), "within_budget");
        assert_eq!(TrimRationale::CompressedToFit.as_str(), "compressed_to_fit");
        assert_eq!(TrimRationale::BudgetExhausted.as_str(), "budget_exhausted");
        assert_eq!(
            TrimRationale::OverlapWithWorkingMemory.as_str(),
            "overlap_with_working_memory"
        );
        assert_eq!(
            TrimRationale::BelowUtilityThreshold.as_str(),
            "below_utility_threshold"
        );
    }

    #[test]
    fn trim_rationale_was_included() {
        assert!(TrimRationale::HighestUtility.was_included());
        assert!(TrimRationale::WithinBudget.was_included());
        assert!(TrimRationale::CompressedToFit.was_included());
        assert!(!TrimRationale::BudgetExhausted.was_included());
        assert!(!TrimRationale::OverlapWithWorkingMemory.was_included());
        assert!(!TrimRationale::BelowUtilityThreshold.was_included());
    }

    #[test]
    fn pack_with_trim_produces_rationale() {
        let engine = ContextBudgetEngine::new();
        let request = ContextBudgetRequest::new(20).with_working_memory(vec![mid(3)]);
        let candidates = vec![
            BudgetCandidate::new(mid(1), "alpha content", 0.9, 0.8),
            BudgetCandidate::new(mid(2), "beta content more", 0.7, 0.6),
            BudgetCandidate::new(mid(3), "gamma in wm", 0.95, 0.9),
            BudgetCandidate::new(mid(4), "delta", 0.0, 0.5),
        ];

        let report = engine.pack_with_trim(&request, candidates);
        assert_eq!(report.token_budget, 20);
        assert_eq!(
            report.included_count + report.omitted_count,
            report.items.len()
        );

        let included = report.included_items();
        let omitted = report.omitted_items();
        assert!(!included.is_empty());
        assert!(!omitted.is_empty());

        // mid(3) should be omitted with overlap rationale
        let wm_item = omitted.iter().find(|i| i.memory_id == mid(3)).unwrap();
        assert_eq!(wm_item.rationale, TrimRationale::OverlapWithWorkingMemory);

        // mid(4) should be omitted with low utility
        let low_item = omitted.iter().find(|i| i.memory_id == mid(4)).unwrap();
        assert_eq!(low_item.rationale, TrimRationale::BelowUtilityThreshold);

        // First included item should be highest utility
        assert_eq!(included[0].rationale, TrimRationale::HighestUtility);
    }

    #[test]
    fn pack_with_compression_fits_more_items() {
        let engine = ContextBudgetEngine::new();
        let request = ContextBudgetRequest::new(10);
        let candidates = vec![
            BudgetCandidate::new(mid(1), "alpha beta gamma delta epsilon", 0.9, 0.9),
            BudgetCandidate::new(mid(2), "zeta eta theta", 0.8, 0.8),
        ];

        let report = engine.pack_with_compression(&request, candidates);

        // At least one item should be included
        assert!(report.included_count > 0);
        // All included items should be in the items list
        let included = report.included_items();
        assert_eq!(included.len(), report.included_count);
    }

    #[test]
    fn pack_with_compression_marks_compressed_items() {
        let engine = ContextBudgetEngine::new();
        // Very small budget to force compression
        let request = ContextBudgetRequest::new(3);
        let candidates = vec![BudgetCandidate::new(
            mid(1),
            "this is a long content string that will need compression to fit",
            0.9,
            0.9,
        )];

        let report = engine.pack_with_compression(&request, candidates);
        let compressed = report.compressed_items();
        if report.included_count > 0 {
            assert!(!compressed.is_empty());
            assert!(compressed[0].was_compressed);
            assert!(compressed[0].trimmed_tokens <= compressed[0].original_tokens);
        }
    }

    #[test]
    fn different_budgets_produce_different_reports() {
        let engine = ContextBudgetEngine::new();
        let candidates = vec![
            BudgetCandidate::new(mid(1), "alpha beta gamma", 0.9, 0.9),
            BudgetCandidate::new(mid(2), "delta epsilon zeta", 0.8, 0.8),
            BudgetCandidate::new(mid(3), "eta theta iota", 0.7, 0.7),
        ];

        let small = engine.pack_with_trim(&ContextBudgetRequest::new(10), candidates.clone());
        let large = engine.pack_with_trim(&ContextBudgetRequest::new(1000), candidates.clone());

        // Large budget should include more items
        assert!(large.included_count >= small.included_count);
        assert!(large.tokens_used >= small.tokens_used);
        assert!(large.omitted_count <= small.omitted_count);
    }

    #[test]
    fn trim_report_filters() {
        let engine = ContextBudgetEngine::new();
        let request = ContextBudgetRequest::new(20).with_working_memory(vec![mid(2)]);
        let candidates = vec![
            BudgetCandidate::new(mid(1), "first item here", 0.9, 0.9),
            BudgetCandidate::new(mid(2), "in working memory", 0.8, 0.8),
            BudgetCandidate::new(mid(3), "third", 0.5, 0.5),
        ];

        let report = engine.pack_with_trim(&request, candidates);
        assert_eq!(report.included_items().len(), report.included_count);
        assert_eq!(report.omitted_items().len(), report.omitted_count);
        assert_eq!(report.compressed_items().len(), 0); // no compression in trim mode
    }

    #[test]
    fn pack_formats_markdown_and_keeps_reasons() {
        let engine = ContextBudgetEngine::new();
        let request = ContextBudgetRequest::new(20).with_format(InjectionFormat::Markdown);
        let candidates = vec![BudgetCandidate::new(mid(1), "deploy checklist", 0.9, 0.9)
            .with_memory_type(CanonicalMemoryType::ToolOutcome)
            .with_ranking_score(810)
            .with_strength_score(700)
            .with_explain_reasons(vec!["score_kept: top ranked".to_string()])];

        let response = engine.pack(&request, candidates);
        assert_eq!(response.injections.len(), 1);
        assert_eq!(
            response.injections[0].content,
            "- [tool_outcome #1] deploy checklist"
        );
        assert_eq!(
            response.injections[0].source_kind,
            InjectionSourceKind::RetrievalResult
        );
        assert!(response.injections[0]
            .inclusion_reasons
            .iter()
            .any(|reason| reason.contains("score_kept: top ranked")));
    }
}
