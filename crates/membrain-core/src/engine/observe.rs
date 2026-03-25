use crate::engine::encode::{EncodeEngine, PreparedEncodeCandidate};
use crate::policy::IngestMode;
use crate::types::{RawEncodeInput, RawIntakeKind};
use std::collections::HashSet;
use xxhash_rust::xxh64::xxh64;

const DEFAULT_SOURCE_LABEL: &str = "passive_observation";
const DEFAULT_CHUNK_SIZE_CHARS: usize = 500;
const DEFAULT_MIN_CHUNK_CHARS: usize = 50;
const DEFAULT_TOPIC_SHIFT_THRESHOLD: f32 = 0.35;

/// Bounded passive-observation configuration shared across transports.
#[derive(Debug, Clone, PartialEq)]
pub struct ObserveConfig {
    pub chunk_size_chars: usize,
    pub topic_shift_threshold: f32,
    pub min_chunk_chars: usize,
    pub context: Option<String>,
    pub source_label: Option<String>,
}

impl Default for ObserveConfig {
    fn default() -> Self {
        Self {
            chunk_size_chars: DEFAULT_CHUNK_SIZE_CHARS,
            topic_shift_threshold: DEFAULT_TOPIC_SHIFT_THRESHOLD,
            min_chunk_chars: DEFAULT_MIN_CHUNK_CHARS,
            context: None,
            source_label: None,
        }
    }
}

impl ObserveConfig {
    pub fn source_label(&self) -> String {
        self.source_label
            .clone()
            .unwrap_or_else(|| DEFAULT_SOURCE_LABEL.to_string())
    }

    fn normalized_chunk_size(&self) -> usize {
        self.chunk_size_chars.max(1)
    }

    fn normalized_min_chunk_chars(&self) -> usize {
        self.min_chunk_chars.max(1).min(self.normalized_chunk_size())
    }

    fn normalized_topic_shift_threshold(&self) -> f32 {
        self.topic_shift_threshold.clamp(0.0, 1.0)
    }
}

/// One segmented passive-observation fragment with the prepared encode candidate.
#[derive(Debug, Clone, PartialEq)]
pub struct ObserveFragment {
    pub index: usize,
    pub text: String,
    pub prepared: PreparedEncodeCandidate,
}

/// Deterministic shared observe report used by CLI and daemon surfaces.
#[derive(Debug, Clone, PartialEq)]
pub struct ObserveReport {
    pub observation_source: String,
    pub observation_chunk_id: String,
    pub bytes_processed: usize,
    pub topic_shifts_detected: usize,
    pub fragments: Vec<ObserveFragment>,
    pub memories_created: usize,
    pub suppressed: usize,
    pub denied: usize,
}

/// Shared bounded passive-observation engine.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ObserveEngine;

impl ObserveEngine {
    pub fn observe_content<F>(
        encode_engine: &EncodeEngine,
        content: &str,
        config: &ObserveConfig,
        namespace_bound: bool,
        mut duplicate_hint: F,
    ) -> ObserveReport
    where
        F: FnMut(u64) -> bool,
    {
        let source_label = config.source_label();
        let chunk_id = format!(
            "obs-{:016x}",
            xxh64(
                format!(
                    "{}\u{1f}{}\u{1f}{}\u{1f}{}",
                    source_label,
                    config.context.as_deref().unwrap_or(""),
                    config.normalized_chunk_size(),
                    content
                )
                .as_bytes(),
                0,
            )
        );
        let (segments, topic_shifts_detected) = segment_content(content, config);
        let mut fragments = Vec::with_capacity(segments.len());
        let mut memories_created = 0usize;
        let mut suppressed = 0usize;
        let mut denied = 0usize;

        for (index, segment) in segments.into_iter().enumerate() {
            let input = RawEncodeInput::new(RawIntakeKind::Observation, segment.clone());
            let initial = encode_engine.prepare_ingest_candidate(
                input.clone(),
                IngestMode::PassiveObservation,
                namespace_bound,
                false,
            );
            let has_duplicate_hint = duplicate_hint(initial.fingerprint);
            let mut prepared = if has_duplicate_hint {
                encode_engine.prepare_ingest_candidate(
                    input,
                    IngestMode::PassiveObservation,
                    namespace_bound,
                    true,
                )
            } else {
                initial
            };
            prepared.normalized.observation_source = Some(source_label.clone());
            prepared.normalized.observation_chunk_id = Some(chunk_id.clone());
            prepared.passive_observation_inspect.observation_source = Some(source_label.clone());
            prepared.passive_observation_inspect.observation_chunk_id = Some(chunk_id.clone());

            match prepared.write_decision.as_str() {
                "capture" => memories_created += 1,
                "suppress" => suppressed += 1,
                _ => denied += 1,
            }

            fragments.push(ObserveFragment {
                index,
                text: segment,
                prepared,
            });
        }

        ObserveReport {
            observation_source: source_label,
            observation_chunk_id: chunk_id,
            bytes_processed: content.len(),
            topic_shifts_detected,
            fragments,
            memories_created,
            suppressed,
            denied,
        }
    }
}

fn segment_content(content: &str, config: &ObserveConfig) -> (Vec<String>, usize) {
    let paragraphs = split_paragraphs(content);
    if paragraphs.is_empty() {
        return (Vec::new(), 0);
    }

    let chunk_size = config.normalized_chunk_size();
    let min_chunk_chars = config.normalized_min_chunk_chars();
    let topic_shift_threshold = config.normalized_topic_shift_threshold();
    let mut segments = Vec::new();
    let mut topic_shifts = 0usize;
    let mut current = String::new();
    let mut previous_piece: Option<String> = None;

    for paragraph in paragraphs {
        for piece in split_oversized_piece(&paragraph, chunk_size, min_chunk_chars) {
            if piece.trim().is_empty() {
                continue;
            }

            let would_exceed_chunk = !current.is_empty()
                && current.len().saturating_add(2).saturating_add(piece.len()) > chunk_size
                && current.len() >= min_chunk_chars;
            let shifted_topic = previous_piece
                .as_deref()
                .map(|previous| topic_similarity(previous, &piece) < topic_shift_threshold)
                .unwrap_or(false)
                && !current.is_empty()
                && current.len() >= min_chunk_chars;

            if shifted_topic {
                topic_shifts += 1;
            }

            if would_exceed_chunk || shifted_topic {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    segments.push(trimmed);
                }
                current.clear();
            }

            if !current.is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(piece.trim());
            previous_piece = Some(piece);
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        segments.push(trimmed);
    }

    (segments, topic_shifts)
}

fn split_paragraphs(content: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current = Vec::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            if !current.is_empty() {
                paragraphs.push(current.join("\n"));
                current.clear();
            }
            continue;
        }
        current.push(line.trim_end().to_string());
    }

    if !current.is_empty() {
        paragraphs.push(current.join("\n"));
    }

    if paragraphs.is_empty() {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            Vec::new()
        } else {
            vec![trimmed.to_string()]
        }
    } else {
        paragraphs
    }
}

fn split_oversized_piece(text: &str, chunk_size: usize, min_chunk_chars: usize) -> Vec<String> {
    if text.len() <= chunk_size {
        return vec![text.trim().to_string()];
    }

    let sentence_candidates = text
        .split_inclusive(['.', '!', '?'])
        .map(str::trim)
        .filter(|sentence| !sentence.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    if sentence_candidates.len() <= 1 {
        return hard_split_text(text, chunk_size);
    }

    let mut pieces = Vec::new();
    let mut current = String::new();
    for sentence in sentence_candidates {
        let would_exceed = !current.is_empty()
            && current.len().saturating_add(1).saturating_add(sentence.len()) > chunk_size
            && current.len() >= min_chunk_chars;
        if would_exceed {
            pieces.push(current.trim().to_string());
            current.clear();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(sentence.trim());
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        pieces.push(trimmed);
    }

    pieces
        .into_iter()
        .flat_map(|piece| {
            if piece.len() > chunk_size {
                hard_split_text(&piece, chunk_size)
            } else {
                vec![piece]
            }
        })
        .collect()
}

fn hard_split_text(text: &str, chunk_size: usize) -> Vec<String> {
    let mut pieces = Vec::new();
    let mut current = String::new();
    let mut count = 0usize;

    for ch in text.chars() {
        current.push(ch);
        count += 1;
        if count >= chunk_size {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                pieces.push(trimmed);
            }
            current.clear();
            count = 0;
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        pieces.push(trimmed);
    }

    pieces
}

fn topic_similarity(left: &str, right: &str) -> f32 {
    let left_tokens = topic_tokens(left);
    let right_tokens = topic_tokens(right);
    if left_tokens.is_empty() && right_tokens.is_empty() {
        return 1.0;
    }

    let intersection = left_tokens.intersection(&right_tokens).count() as f32;
    let union = left_tokens.union(&right_tokens).count() as f32;
    if union == 0.0 {
        1.0
    } else {
        intersection / union
    }
}

fn topic_tokens(text: &str) -> HashSet<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter(|token| token.len() >= 2)
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ObserveConfig, ObserveEngine};
    use crate::engine::encode::EncodeEngine;
    use crate::RuntimeConfig;
    use std::collections::HashSet;

    fn test_engine() -> EncodeEngine {
        EncodeEngine::new(RuntimeConfig::default())
    }

    #[test]
    fn observe_content_uses_deterministic_bounded_chunking() {
        let engine = test_engine();
        let config = ObserveConfig {
            chunk_size_chars: 150,
            topic_shift_threshold: 0.30,
            min_chunk_chars: 20,
            context: Some("project-x".to_string()),
            source_label: Some("watch:logs/app.jsonl".to_string()),
        };
        let content = "deploy pipeline reported a green build and stable queue depth.\n\ndeploy pipeline kept the same release train and confirmed healthy canary metrics.\n\nuser preference shifted toward dark mode and larger fonts for the dashboard.";

        let report = ObserveEngine::observe_content(&engine, content, &config, true, |_| false);

        assert_eq!(report.bytes_processed, content.len());
        assert_eq!(report.fragments.len(), 2);
        assert_eq!(report.memories_created, 2);
        assert_eq!(report.suppressed, 0);
        assert_eq!(report.denied, 0);
        assert_eq!(report.topic_shifts_detected, 1);
        assert_eq!(report.observation_source, "watch:logs/app.jsonl");
        assert!(report.fragments.iter().all(|fragment| fragment
            .prepared
            .normalized
            .observation_chunk_id
            .as_deref()
            == Some(report.observation_chunk_id.as_str())));
    }

    #[test]
    fn observe_content_shares_batch_metadata_across_all_fragments() {
        let engine = test_engine();
        let config = ObserveConfig {
            chunk_size_chars: 60,
            min_chunk_chars: 10,
            ..ObserveConfig::default()
        };
        let content = "alpha topic detail one. alpha topic detail two.\n\nbeta topic detail three. beta topic detail four.";

        let report = ObserveEngine::observe_content(&engine, content, &config, true, |_| false);

        assert!(report.fragments.len() >= 2);
        for fragment in &report.fragments {
            assert_eq!(
                fragment.prepared.normalized.observation_source.as_deref(),
                Some(report.observation_source.as_str())
            );
            assert_eq!(
                fragment.prepared.normalized.observation_chunk_id.as_deref(),
                Some(report.observation_chunk_id.as_str())
            );
            assert_eq!(
                fragment
                    .prepared
                    .passive_observation_inspect
                    .observation_source
                    .as_deref(),
                Some(report.observation_source.as_str())
            );
            assert_eq!(
                fragment
                    .prepared
                    .passive_observation_inspect
                    .observation_chunk_id
                    .as_deref(),
                Some(report.observation_chunk_id.as_str())
            );
        }
    }

    #[test]
    fn observe_content_suppresses_duplicate_fragments_without_policy_denial() {
        let engine = test_engine();
        let config = ObserveConfig {
            chunk_size_chars: 40,
            min_chunk_chars: 10,
            ..ObserveConfig::default()
        };
        let content = "duplicate fragment repeated here.\n\nduplicate fragment repeated here.";
        let mut seen = HashSet::new();

        let report = ObserveEngine::observe_content(&engine, content, &config, true, |fingerprint| {
            !seen.insert(fingerprint)
        });

        assert_eq!(report.fragments.len(), 2);
        assert_eq!(report.memories_created, 1);
        assert_eq!(report.suppressed, 1);
        assert_eq!(report.denied, 0);
        assert_eq!(report.fragments[0].prepared.write_decision.as_str(), "capture");
        assert_eq!(report.fragments[1].prepared.write_decision.as_str(), "suppress");
    }
}
