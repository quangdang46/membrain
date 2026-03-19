use fastembed::{EmbeddingModel, InitOptions as FastembedInitOptions, TextEmbedding};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use xxhash_rust::xxh64::xxh64;

const DEFAULT_EMBEDDING_CACHE_ENTRIES: usize = 1_000;
const DEFAULT_EMBEDDING_BATCH_SIZE: usize = 32;
const DEFAULT_EMBEDDING_DIMENSIONS: usize = 384;

/// Embedding purpose keeps content and context vectors in distinct cache families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmbeddingPurpose {
    Content,
    Context,
}

/// Generation-aware cache key for normalized embedding inputs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmbedCacheKey {
    pub generation: String,
    pub dimensions: usize,
    pub purpose: EmbeddingPurpose,
    pub normalized_hash: u64,
}

impl EmbedCacheKey {
    /// Builds a generation-aware cache key from normalized input bytes.
    pub fn from_normalized_text(
        generation: impl Into<String>,
        dimensions: usize,
        purpose: EmbeddingPurpose,
        normalized_text: &str,
    ) -> Self {
        Self {
            generation: generation.into(),
            dimensions,
            purpose,
            normalized_hash: xxh64(normalized_text.as_bytes(), 0),
        }
    }
}

/// Cache outcome for one embedding request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbedCacheEvent {
    Hit,
    Miss,
}

/// Structured trace for one embedding request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbedTrace {
    pub backend_kind: &'static str,
    pub local_only: bool,
    pub remote_fallback_used: bool,
    pub cache_event: EmbedCacheEvent,
    pub generation: String,
    pub dimensions: usize,
    pub purpose: EmbeddingPurpose,
}

/// Structured trace for one batch embedding request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchEmbedTrace {
    pub backend_kind: &'static str,
    pub local_only: bool,
    pub remote_fallback_used: bool,
    pub cache_hit_count: usize,
    pub cache_miss_count: usize,
    pub generation: String,
    pub dimensions: usize,
    pub purpose: EmbeddingPurpose,
    pub batch_size: usize,
}

/// Vector plus structured trace for one embedding request.
#[derive(Debug, Clone, PartialEq)]
pub struct CachedEmbedding {
    pub vector: Vec<f32>,
    pub trace: EmbedTrace,
}

/// Vectors plus structured trace for one batch embedding request.
#[derive(Debug, Clone, PartialEq)]
pub struct CachedBatchEmbeddings {
    pub vectors: Vec<Vec<f32>>,
    pub trace: BatchEmbedTrace,
}

/// Configuration for a local text embedder and its generation-aware cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalEmbedderConfig {
    pub model: EmbeddingModel,
    pub generation: String,
    pub dimensions: usize,
    pub cache_entries: usize,
    pub batch_size: usize,
    pub cache_dir: Option<PathBuf>,
    pub show_download_progress: bool,
}

impl Default for LocalEmbedderConfig {
    fn default() -> Self {
        Self {
            model: EmbeddingModel::AllMiniLML6V2,
            generation: String::from("all-minilm-l6-v2@default"),
            dimensions: DEFAULT_EMBEDDING_DIMENSIONS,
            cache_entries: DEFAULT_EMBEDDING_CACHE_ENTRIES,
            batch_size: DEFAULT_EMBEDDING_BATCH_SIZE,
            cache_dir: None,
            show_download_progress: false,
        }
    }
}

/// Errors for local-only embedding and generation-aware cache handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmbedError {
    LocalBackendUnavailable(String),
    UnexpectedEmbeddingCount { expected: usize, actual: usize },
    MissingEmbeddingAt(usize),
}

impl fmt::Display for EmbedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocalBackendUnavailable(message) => {
                write!(f, "local embedder unavailable: {message}")
            }
            Self::UnexpectedEmbeddingCount { expected, actual } => {
                write!(
                    f,
                    "unexpected embedding count: expected {expected}, got {actual}"
                )
            }
            Self::MissingEmbeddingAt(index) => {
                write!(f, "missing embedding at batch index {index}")
            }
        }
    }
}

impl Error for EmbedError {}

/// Shared local-only embedding boundary owned by `membrain-core`.
pub trait LocalTextEmbedder {
    fn backend_kind(&self) -> &'static str;
    fn generation(&self) -> &str;
    fn dimensions(&self) -> usize;
    fn embed_text(
        &mut self,
        purpose: EmbeddingPurpose,
        normalized_text: &str,
    ) -> Result<Vec<f32>, EmbedError>;
    fn embed_texts(
        &mut self,
        purpose: EmbeddingPurpose,
        normalized_texts: &[String],
    ) -> Result<Vec<Vec<f32>>, EmbedError>;
}

/// Real local text embedder backed by `fastembed`.
pub struct FastembedTextEmbedder {
    model: EmbeddingModel,
    generation: String,
    dimensions: usize,
    batch_size: usize,
    inner: TextEmbedding,
}

impl FastembedTextEmbedder {
    /// Builds a local-only fastembed backend from the canonical config.
    pub fn try_new(config: &LocalEmbedderConfig) -> Result<Self, EmbedError> {
        let mut options = FastembedInitOptions::new(config.model.clone())
            .with_show_download_progress(config.show_download_progress);
        if let Some(cache_dir) = &config.cache_dir {
            options = options.with_cache_dir(cache_dir.clone());
        }

        let inner = TextEmbedding::try_new(options)
            .map_err(|error| EmbedError::LocalBackendUnavailable(error.to_string()))?;
        let dimensions = TextEmbedding::get_model_info(&config.model)
            .map(|model| model.dim)
            .unwrap_or(config.dimensions);

        Ok(Self {
            model: config.model.clone(),
            generation: config.generation.clone(),
            dimensions,
            batch_size: config.batch_size.max(1),
            inner,
        })
    }

    /// Returns the configured fastembed model.
    pub fn model(&self) -> &EmbeddingModel {
        &self.model
    }
}

impl LocalTextEmbedder for FastembedTextEmbedder {
    fn backend_kind(&self) -> &'static str {
        "local_fastembed"
    }

    fn generation(&self) -> &str {
        &self.generation
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn embed_text(
        &mut self,
        _purpose: EmbeddingPurpose,
        normalized_text: &str,
    ) -> Result<Vec<f32>, EmbedError> {
        let texts = [normalized_text];
        let vectors = self
            .inner
            .embed(texts, Some(1))
            .map_err(|error| EmbedError::LocalBackendUnavailable(error.to_string()))?;

        if vectors.len() != 1 {
            return Err(EmbedError::UnexpectedEmbeddingCount {
                expected: 1,
                actual: vectors.len(),
            });
        }

        vectors
            .into_iter()
            .next()
            .ok_or(EmbedError::UnexpectedEmbeddingCount {
                expected: 1,
                actual: 0,
            })
    }

    fn embed_texts(
        &mut self,
        _purpose: EmbeddingPurpose,
        normalized_texts: &[String],
    ) -> Result<Vec<Vec<f32>>, EmbedError> {
        let vectors = self
            .inner
            .embed(normalized_texts, Some(self.batch_size))
            .map_err(|error| EmbedError::LocalBackendUnavailable(error.to_string()))?;

        if vectors.len() == normalized_texts.len() {
            Ok(vectors)
        } else {
            Err(EmbedError::UnexpectedEmbeddingCount {
                expected: normalized_texts.len(),
                actual: vectors.len(),
            })
        }
    }
}

/// Small bounded LRU-like cache for generation-aware embeddings.
#[derive(Debug, Clone)]
pub struct EmbeddingCache {
    capacity: usize,
    entries: HashMap<EmbedCacheKey, Vec<f32>>,
    usage: VecDeque<EmbedCacheKey>,
}

impl EmbeddingCache {
    /// Builds a new cache with a bounded entry count.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            entries: HashMap::new(),
            usage: VecDeque::new(),
        }
    }

    /// Returns the number of cached vectors.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Removes all cached vectors.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.usage.clear();
    }

    /// Removes all cached vectors for one embedder generation.
    pub fn invalidate_generation(&mut self, generation: &str) -> usize {
        let before = self.entries.len();
        self.entries.retain(|key, _| key.generation != generation);
        self.usage.retain(|key| key.generation != generation);
        before.saturating_sub(self.entries.len())
    }

    fn get(&mut self, key: &EmbedCacheKey) -> Option<Vec<f32>> {
        let vector = self.entries.get(key).cloned()?;
        self.touch(key.clone());
        Some(vector)
    }

    fn put(&mut self, key: EmbedCacheKey, vector: Vec<f32>) {
        if self.entries.contains_key(&key) {
            self.entries.insert(key.clone(), vector);
            self.touch(key);
            return;
        }

        if self.entries.len() >= self.capacity {
            if let Some(oldest) = self.usage.pop_front() {
                self.entries.remove(&oldest);
            }
        }

        self.entries.insert(key.clone(), vector);
        self.usage.push_back(key);
    }

    fn touch(&mut self, key: EmbedCacheKey) {
        if let Some(position) = self.usage.iter().position(|entry| entry == &key) {
            self.usage.remove(position);
        }
        self.usage.push_back(key);
    }
}

/// Generation-aware cache wrapper around a local-only embedder.
#[derive(Debug)]
pub struct CachedTextEmbedder<E> {
    backend: E,
    cache: EmbeddingCache,
}

impl<E: LocalTextEmbedder> CachedTextEmbedder<E> {
    /// Builds a new cached local embedder.
    pub fn new(backend: E, cache_entries: usize) -> Self {
        Self {
            backend,
            cache: EmbeddingCache::new(cache_entries),
        }
    }

    /// Returns the wrapped backend.
    pub fn backend(&self) -> &E {
        &self.backend
    }

    /// Returns the wrapped backend mutably.
    pub fn backend_mut(&mut self) -> &mut E {
        &mut self.backend
    }

    /// Returns the bounded cache.
    pub fn cache(&self) -> &EmbeddingCache {
        &self.cache
    }

    /// Removes all cached vectors for one generation.
    pub fn invalidate_generation(&mut self, generation: &str) -> usize {
        self.cache.invalidate_generation(generation)
    }

    /// Removes all cached vectors.
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Returns one embedding using the generation-aware cache first.
    pub fn get_or_embed(
        &mut self,
        purpose: EmbeddingPurpose,
        normalized_text: &str,
    ) -> Result<CachedEmbedding, EmbedError> {
        let key = self.cache_key_for(purpose, normalized_text);
        if let Some(vector) = self.cache.get(&key) {
            return Ok(CachedEmbedding {
                vector,
                trace: self.trace_for(purpose, EmbedCacheEvent::Hit),
            });
        }

        let vector = self.backend.embed_text(purpose, normalized_text)?;
        self.cache.put(key, vector.clone());
        Ok(CachedEmbedding {
            vector,
            trace: self.trace_for(purpose, EmbedCacheEvent::Miss),
        })
    }

    /// Returns a batch of embeddings, calling the backend only for cache misses.
    pub fn get_or_embed_batch(
        &mut self,
        purpose: EmbeddingPurpose,
        normalized_texts: &[String],
    ) -> Result<CachedBatchEmbeddings, EmbedError> {
        let mut vectors: Vec<Option<Vec<f32>>> = vec![None; normalized_texts.len()];
        let mut missing_indices = Vec::new();
        let mut missing_keys = Vec::new();
        let mut missing_texts = Vec::new();
        let mut cache_hit_count = 0usize;

        for (index, normalized_text) in normalized_texts.iter().enumerate() {
            let key = self.cache_key_for(purpose, normalized_text);
            if let Some(vector) = self.cache.get(&key) {
                vectors[index] = Some(vector);
                cache_hit_count += 1;
            } else {
                missing_indices.push(index);
                missing_keys.push(key);
                missing_texts.push(normalized_text.clone());
            }
        }

        let cache_miss_count = missing_texts.len();
        if cache_miss_count > 0 {
            let generated = self.backend.embed_texts(purpose, &missing_texts)?;
            if generated.len() != cache_miss_count {
                return Err(EmbedError::UnexpectedEmbeddingCount {
                    expected: cache_miss_count,
                    actual: generated.len(),
                });
            }

            for (((index, key), _text), vector) in missing_indices
                .into_iter()
                .zip(missing_keys.into_iter())
                .zip(missing_texts.into_iter())
                .zip(generated.into_iter())
            {
                self.cache.put(key, vector.clone());
                vectors[index] = Some(vector);
            }
        }

        let mut final_vectors = Vec::with_capacity(vectors.len());
        for (index, vector) in vectors.into_iter().enumerate() {
            match vector {
                Some(vector) => final_vectors.push(vector),
                None => return Err(EmbedError::MissingEmbeddingAt(index)),
            }
        }

        Ok(CachedBatchEmbeddings {
            vectors: final_vectors,
            trace: BatchEmbedTrace {
                backend_kind: self.backend.backend_kind(),
                local_only: true,
                remote_fallback_used: false,
                cache_hit_count,
                cache_miss_count,
                generation: self.backend.generation().to_owned(),
                dimensions: self.backend.dimensions(),
                purpose,
                batch_size: normalized_texts.len(),
            },
        })
    }

    fn cache_key_for(&self, purpose: EmbeddingPurpose, normalized_text: &str) -> EmbedCacheKey {
        EmbedCacheKey::from_normalized_text(
            self.backend.generation(),
            self.backend.dimensions(),
            purpose,
            normalized_text,
        )
    }

    fn trace_for(&self, purpose: EmbeddingPurpose, cache_event: EmbedCacheEvent) -> EmbedTrace {
        EmbedTrace {
            backend_kind: self.backend.backend_kind(),
            local_only: true,
            remote_fallback_used: false,
            cache_event,
            generation: self.backend.generation().to_owned(),
            dimensions: self.backend.dimensions(),
            purpose,
        }
    }
}

/// Stable embedding boundary owned by `membrain-core`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EmbedModule;

impl EmbedModule {
    /// Returns the stable component identifier for this embedding surface.
    pub const fn component_name(&self) -> &'static str {
        "embed"
    }

    /// Returns the canonical config for a local-only text embedder.
    pub fn default_local_config(&self) -> LocalEmbedderConfig {
        LocalEmbedderConfig::default()
    }

    /// Builds a cached local text embedder backed by `fastembed`.
    pub fn new_local_text_embedder(
        &self,
        config: &LocalEmbedderConfig,
    ) -> Result<CachedTextEmbedder<FastembedTextEmbedder>, EmbedError> {
        let backend = FastembedTextEmbedder::try_new(config)?;
        Ok(CachedTextEmbedder::new(backend, config.cache_entries))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CachedTextEmbedder, EmbedCacheEvent, EmbedError, EmbeddingPurpose, LocalTextEmbedder,
    };

    #[derive(Debug)]
    struct FakeEmbedder {
        generation: String,
        dimensions: usize,
        single_calls: usize,
        batch_calls: usize,
        extra_single_result: bool,
    }

    impl FakeEmbedder {
        fn new(generation: &str, dimensions: usize) -> Self {
            Self {
                generation: generation.to_owned(),
                dimensions,
                single_calls: 0,
                batch_calls: 0,
                extra_single_result: false,
            }
        }

        fn with_extra_single_result(mut self) -> Self {
            self.extra_single_result = true;
            self
        }

        fn vector_for(&self, purpose: EmbeddingPurpose, normalized_text: &str) -> Vec<f32> {
            let bytes_sum = normalized_text
                .bytes()
                .fold(0u32, |accumulator, byte| accumulator + u32::from(byte));
            let purpose_offset = match purpose {
                EmbeddingPurpose::Content => 1.0,
                EmbeddingPurpose::Context => 2.0,
            };
            let mut vector = vec![0.0; self.dimensions];
            if self.dimensions > 0 {
                vector[0] = normalized_text.len() as f32 + purpose_offset;
            }
            if self.dimensions > 1 {
                vector[1] = bytes_sum as f32;
            }
            if self.dimensions > 2 {
                vector[2] = self.generation.len() as f32;
            }
            if self.dimensions > 3 {
                vector[3] = purpose_offset;
            }
            vector
        }
    }

    impl LocalTextEmbedder for FakeEmbedder {
        fn backend_kind(&self) -> &'static str {
            "test_double"
        }

        fn generation(&self) -> &str {
            &self.generation
        }

        fn dimensions(&self) -> usize {
            self.dimensions
        }

        fn embed_text(
            &mut self,
            purpose: EmbeddingPurpose,
            normalized_text: &str,
        ) -> Result<Vec<f32>, EmbedError> {
            self.single_calls += 1;
            let mut vectors = vec![self.vector_for(purpose, normalized_text)];
            if self.extra_single_result {
                vectors.push(self.vector_for(purpose, "unexpected extra result"));
            }
            match vectors.len() {
                1 => Ok(vectors.remove(0)),
                actual => Err(EmbedError::UnexpectedEmbeddingCount {
                    expected: 1,
                    actual,
                }),
            }
        }

        fn embed_texts(
            &mut self,
            purpose: EmbeddingPurpose,
            normalized_texts: &[String],
        ) -> Result<Vec<Vec<f32>>, EmbedError> {
            self.batch_calls += 1;
            Ok(normalized_texts
                .iter()
                .map(|text| self.vector_for(purpose, text))
                .collect())
        }
    }

    #[test]
    fn cache_miss_then_hit_returns_structured_trace() -> Result<(), EmbedError> {
        let backend = FakeEmbedder::new("generation-a", 4);
        let mut embedder = CachedTextEmbedder::new(backend, 8);

        let miss = embedder.get_or_embed(EmbeddingPurpose::Content, "normalized text")?;
        assert_eq!(miss.trace.cache_event, EmbedCacheEvent::Miss);
        assert!(miss.trace.local_only);
        assert!(!miss.trace.remote_fallback_used);
        assert_eq!(embedder.backend().single_calls, 1);

        let hit = embedder.get_or_embed(EmbeddingPurpose::Content, "normalized text")?;
        assert_eq!(hit.trace.cache_event, EmbedCacheEvent::Hit);
        assert_eq!(miss.vector, hit.vector);
        assert_eq!(embedder.backend().single_calls, 1);
        Ok(())
    }

    #[test]
    fn cache_key_keeps_content_and_context_separate() -> Result<(), EmbedError> {
        let backend = FakeEmbedder::new("generation-a", 4);
        let mut embedder = CachedTextEmbedder::new(backend, 8);

        let content = embedder.get_or_embed(EmbeddingPurpose::Content, "same normalized text")?;
        let context = embedder.get_or_embed(EmbeddingPurpose::Context, "same normalized text")?;

        assert_eq!(content.trace.cache_event, EmbedCacheEvent::Miss);
        assert_eq!(context.trace.cache_event, EmbedCacheEvent::Miss);
        assert_ne!(content.vector, context.vector);
        assert_eq!(embedder.backend().single_calls, 2);
        Ok(())
    }

    #[test]
    fn generation_change_misses_and_invalidation_clears_old_entries() -> Result<(), EmbedError> {
        let backend = FakeEmbedder::new("generation-a", 4);
        let mut embedder = CachedTextEmbedder::new(backend, 8);

        embedder.get_or_embed(EmbeddingPurpose::Content, "normalized text")?;
        assert_eq!(embedder.cache().len(), 1);

        embedder.backend_mut().generation = String::from("generation-b");
        let miss = embedder.get_or_embed(EmbeddingPurpose::Content, "normalized text")?;
        assert_eq!(miss.trace.cache_event, EmbedCacheEvent::Miss);
        assert_eq!(embedder.backend().single_calls, 2);
        assert_eq!(embedder.cache().len(), 2);

        let removed = embedder.invalidate_generation("generation-a");
        assert_eq!(removed, 1);
        assert_eq!(embedder.cache().len(), 1);
        Ok(())
    }

    #[test]
    fn batch_embedding_only_calls_backend_for_cache_misses() -> Result<(), EmbedError> {
        let backend = FakeEmbedder::new("generation-a", 4);
        let mut embedder = CachedTextEmbedder::new(backend, 8);

        embedder.get_or_embed(EmbeddingPurpose::Content, "already cached")?;
        assert_eq!(embedder.backend().single_calls, 1);

        let batch = vec![
            String::from("already cached"),
            String::from("new one"),
            String::from("new two"),
        ];
        let first = embedder.get_or_embed_batch(EmbeddingPurpose::Content, &batch)?;
        assert_eq!(first.trace.cache_hit_count, 1);
        assert_eq!(first.trace.cache_miss_count, 2);
        assert!(first.trace.local_only);
        assert!(!first.trace.remote_fallback_used);
        assert_eq!(embedder.backend().batch_calls, 1);

        let second = embedder.get_or_embed_batch(EmbeddingPurpose::Content, &batch)?;
        assert_eq!(second.trace.cache_hit_count, 3);
        assert_eq!(second.trace.cache_miss_count, 0);
        assert_eq!(embedder.backend().batch_calls, 1);
        Ok(())
    }

    #[test]
    fn single_embedding_path_rejects_multiple_results() {
        let backend = FakeEmbedder::new("generation-a", 4).with_extra_single_result();
        let mut embedder = CachedTextEmbedder::new(backend, 8);

        let error = embedder
            .get_or_embed(EmbeddingPurpose::Content, "normalized text")
            .unwrap_err();
        assert_eq!(
            error,
            EmbedError::UnexpectedEmbeddingCount {
                expected: 1,
                actual: 2,
            }
        );
        assert_eq!(embedder.backend().single_calls, 1);
        assert!(embedder.cache().is_empty());
    }
}
