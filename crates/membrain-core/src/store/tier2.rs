use crate::api::NamespaceId;
use crate::observability::{Tier2PrefilterOutcome, Tier2PrefilterTrace};
use crate::store::Tier2StoreApi;
use crate::types::{
    CanonicalMemoryType, FastPathRouteFamily, LandmarkMetadata, MemoryId, NormalizedMemoryEnvelope,
    SessionId,
};

/// Durable Tier2 indexed store boundary owned by `membrain-core`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Tier2Store;

impl Tier2Store {
    /// Builds a durable Tier2 item layout that keeps prefilter-safe metadata separate from the
    /// heavyweight payload body.
    pub fn layout_item(
        &self,
        namespace: NamespaceId,
        memory_id: MemoryId,
        session_id: SessionId,
        fingerprint: u64,
        envelope: &NormalizedMemoryEnvelope,
    ) -> Tier2DurableItemLayout {
        let payload_locator = Tier2PayloadLocator::for_memory(&namespace, memory_id);

        Tier2DurableItemLayout {
            metadata: Tier2MetadataRecord {
                namespace: namespace.clone(),
                memory_id,
                session_id,
                memory_type: envelope.memory_type,
                route_family: FastPathRouteFamily::from_memory_type(envelope.memory_type),
                compact_text: envelope.compact_text.clone(),
                fingerprint,
                normalization_generation: envelope.normalization_generation,
                payload_size_bytes: envelope.payload_size_bytes,
                landmark: envelope.landmark.clone(),
                observation_source: envelope.observation_source.clone(),
                observation_chunk_id: envelope.observation_chunk_id.clone(),
                payload_locator: payload_locator.clone(),
            },
            payload: Tier2PayloadRecord {
                namespace,
                memory_id,
                payload_locator,
                raw_text: envelope.raw_text.clone(),
                raw_size_bytes: envelope.raw_text.len(),
            },
        }
    }
}

impl Tier2StoreApi for Tier2Store {
    fn component_name(&self) -> &'static str {
        "store.tier2"
    }
}

/// Durable Tier2 item split into prefilter-safe metadata plus heavyweight payload storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tier2DurableItemLayout {
    pub metadata: Tier2MetadataRecord,
    pub payload: Tier2PayloadRecord,
}

impl Tier2DurableItemLayout {
    /// Returns the metadata-only projection safe for Tier2 prefilter/index work.
    pub fn prefilter_view(&self) -> Tier2PrefilterView<'_> {
        Tier2PrefilterView {
            namespace: &self.metadata.namespace,
            memory_id: self.metadata.memory_id,
            session_id: self.metadata.session_id,
            memory_type: self.metadata.memory_type,
            route_family: self.metadata.route_family,
            compact_text: &self.metadata.compact_text,
            fingerprint: self.metadata.fingerprint,
            normalization_generation: self.metadata.normalization_generation,
            payload_size_bytes: self.metadata.payload_size_bytes,
            landmark: &self.metadata.landmark,
            observation_source: self.metadata.observation_source.as_deref(),
            observation_chunk_id: self.metadata.observation_chunk_id.as_deref(),
            payload_locator: self.metadata.payload_locator.clone(),
        }
    }

    /// Returns the deterministic key surface suitable for Tier2 filter and index maintenance.
    pub fn metadata_index_key(&self) -> Tier2MetadataIndexKey<'_> {
        Tier2MetadataIndexKey {
            namespace: &self.metadata.namespace,
            memory_id: self.metadata.memory_id,
            session_id: self.metadata.session_id,
            memory_type: self.metadata.memory_type,
            route_family: self.metadata.route_family,
            fingerprint: self.metadata.fingerprint,
            compact_text: &self.metadata.compact_text,
            normalization_generation: self.metadata.normalization_generation,
            landmark: &self.metadata.landmark,
            payload_locator: self.metadata.payload_locator.clone(),
        }
    }

    /// Returns whether the prefilter path stays on metadata-only state before the final cut.
    pub const fn prefilter_stays_metadata_only(&self) -> bool {
        true
    }

    /// Returns whether index-safe metadata keys avoid materializing heavyweight payload content.
    pub const fn index_key_stays_metadata_only(&self) -> bool {
        true
    }

    /// Returns the inspectable metadata-first trace for Tier2 planners.
    pub fn prefilter_trace(&self) -> Tier2PrefilterTrace {
        Tier2PrefilterTrace {
            outcome: Tier2PrefilterOutcome::Ready,
            metadata_candidate_count: 1,
            payload_fetch_count: 0,
        }
    }

    /// Returns a stable reference to the separated heavyweight payload body.
    pub fn payload_record(&self) -> &Tier2PayloadRecord {
        &self.payload
    }

    /// Returns the stable hydration path for fetching the deferred heavyweight payload body.
    pub fn payload_hydration_path(&self) -> String {
        self.metadata.payload_locator.hydration_path()
    }
}

/// Stable locator for the heavyweight Tier2 payload body stored outside metadata rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tier2PayloadLocator {
    pub namespace: NamespaceId,
    pub shard: u16,
    pub slot: u64,
}

impl Tier2PayloadLocator {
    /// Derives a deterministic locator from the durable namespace and memory id.
    pub fn for_memory(namespace: &NamespaceId, memory_id: MemoryId) -> Self {
        Self {
            namespace: namespace.clone(),
            shard: (memory_id.0 % 1024) as u16,
            slot: memory_id.0,
        }
    }

    /// Returns the stable hydration path used to materialize the deferred Tier2 payload.
    pub fn hydration_path(&self) -> String {
        format!(
            "tier2://{}/payload/{:04x}/{}",
            self.namespace_segment(),
            self.shard,
            self.slot
        )
    }

    fn namespace_segment(&self) -> String {
        self.namespace.as_str().replace('/', "%2F")
    }
}

/// Durable Tier2 metadata row kept hot-path-safe for filtering and recall planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tier2MetadataRecord {
    pub namespace: NamespaceId,
    pub memory_id: MemoryId,
    pub session_id: SessionId,
    pub memory_type: CanonicalMemoryType,
    pub route_family: FastPathRouteFamily,
    pub compact_text: String,
    pub fingerprint: u64,
    pub normalization_generation: &'static str,
    pub payload_size_bytes: usize,
    pub landmark: LandmarkMetadata,
    pub observation_source: Option<String>,
    pub observation_chunk_id: Option<String>,
    pub payload_locator: Tier2PayloadLocator,
}

/// Durable Tier2 payload body stored separately from metadata and prefilter indexes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tier2PayloadRecord {
    pub namespace: NamespaceId,
    pub memory_id: MemoryId,
    pub payload_locator: Tier2PayloadLocator,
    pub raw_text: String,
    pub raw_size_bytes: usize,
}

/// Borrowed metadata-only view used by Tier2 prefilter/index planners.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tier2PrefilterView<'a> {
    pub namespace: &'a NamespaceId,
    pub memory_id: MemoryId,
    pub session_id: SessionId,
    pub memory_type: CanonicalMemoryType,
    pub route_family: FastPathRouteFamily,
    pub compact_text: &'a str,
    pub fingerprint: u64,
    pub normalization_generation: &'static str,
    pub payload_size_bytes: usize,
    pub landmark: &'a LandmarkMetadata,
    pub observation_source: Option<&'a str>,
    pub observation_chunk_id: Option<&'a str>,
    pub payload_locator: Tier2PayloadLocator,
}

impl Tier2PrefilterView<'_> {
    /// Returns the landmark label visible to metadata-first temporal consumers.
    pub fn landmark_label(&self) -> Option<&str> {
        self.landmark.landmark_label.as_deref()
    }

    /// Returns the era identifier visible to metadata-first temporal consumers.
    pub fn era_id(&self) -> Option<&str> {
        self.landmark.era_id.as_deref()
    }
}

/// Borrowed deterministic metadata key used by Tier2 filter/index maintenance surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tier2MetadataIndexKey<'a> {
    pub namespace: &'a NamespaceId,
    pub memory_id: MemoryId,
    pub session_id: SessionId,
    pub memory_type: CanonicalMemoryType,
    pub route_family: FastPathRouteFamily,
    pub fingerprint: u64,
    pub compact_text: &'a str,
    pub normalization_generation: &'static str,
    pub landmark: &'a LandmarkMetadata,
    pub payload_locator: Tier2PayloadLocator,
}

impl Tier2MetadataIndexKey<'_> {
    /// Returns the landmark label preserved on the metadata-only index key.
    pub fn landmark_label(&self) -> Option<&str> {
        self.landmark.landmark_label.as_deref()
    }

    /// Returns the era identifier preserved on the metadata-only index key.
    pub fn era_id(&self) -> Option<&str> {
        self.landmark.era_id.as_deref()
    }
}
