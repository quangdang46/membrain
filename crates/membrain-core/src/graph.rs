//! Bounded lineage and neighborhood expansion primitives.
//!
//! Owns the association graph: entities (memories and concepts),
//! edges (relationships with strengths), bounded neighborhood
//! expansion, and lineage tracing for memory provenance.

use crate::api::NamespaceId;
use crate::types::MemoryId;
use std::collections::{HashMap, HashSet, VecDeque};

// ── Entity types ─────────────────────────────────────────────────────────────

/// Stable identifier for graph nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct EntityId(pub u64);

/// Canonical entity kinds in the association graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityKind {
    /// A stored memory item.
    Memory,
    /// An extracted concept or topic.
    Concept,
    /// An agent or user identity.
    Agent,
    /// A session boundary marker.
    Session,
    /// A workspace or project scope.
    Workspace,
}

impl EntityKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Concept => "concept",
            Self::Agent => "agent",
            Self::Session => "session",
            Self::Workspace => "workspace",
        }
    }
}

/// One node in the association graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphEntity {
    pub id: EntityId,
    pub kind: EntityKind,
    pub label: String,
    pub namespace: NamespaceId,
    /// Optional link to a memory (for Memory-kind entities).
    pub memory_id: Option<MemoryId>,
}

// ── Edge types ───────────────────────────────────────────────────────────────

/// Canonical relationship kinds between entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RelationKind {
    /// Memory mentions or references a concept.
    Mentions,
    /// Memory is derived from another memory (consolidation, revision).
    DerivedFrom,
    /// Memory led to another memory through an explicit source-backed causal claim.
    Causal,
    /// Memory contradicts another memory.
    Contradicts,
    /// Memory supersedes (replaces) another memory.
    Supersedes,
    /// Memory and concept share a common topic.
    SharedTopic,
    /// Memory was created in a session.
    CreatedIn,
    /// Memory belongs to an agent's context.
    OwnedBy,
}

impl RelationKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mentions => "mentions",
            Self::DerivedFrom => "derived_from",
            Self::Causal => "causal",
            Self::Contradicts => "contradicts",
            Self::Supersedes => "supersedes",
            Self::SharedTopic => "shared_topic",
            Self::CreatedIn => "created_in",
            Self::OwnedBy => "owned_by",
        }
    }

    /// Whether this relation is directional.
    pub const fn is_directed(self) -> bool {
        matches!(
            self,
            Self::DerivedFrom
                | Self::Causal
                | Self::Contradicts
                | Self::Supersedes
                | Self::CreatedIn
                | Self::OwnedBy
        )
    }
}

/// One edge in the association graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphEdge {
    pub from: EntityId,
    pub to: EntityId,
    pub relation: RelationKind,
    /// Association strength (0..1000).
    pub strength: u16,
}

/// Canonical kinds of durable causal links between memory items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CausalLinkType {
    Derived,
    Reconsolidated,
    Extracted,
    Inferred,
}

impl CausalLinkType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Derived => "derived",
            Self::Reconsolidated => "reconsolidated",
            Self::Extracted => "extracted",
            Self::Inferred => "inferred",
        }
    }
}

/// Allowed evidence families that may support a causal claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CausalEvidenceKind {
    DurableMemory,
    ReconsolidationAudit,
    ConsolidationArtifact,
    BeliefVersionDiff,
}

impl CausalEvidenceKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DurableMemory => "durable_memory",
            Self::ReconsolidationAudit => "reconsolidation_audit",
            Self::ConsolidationArtifact => "consolidation_artifact",
            Self::BeliefVersionDiff => "belief_version_diff",
        }
    }

    pub const fn supports(self, link_type: CausalLinkType) -> bool {
        match link_type {
            CausalLinkType::Derived => {
                matches!(self, Self::DurableMemory | Self::ConsolidationArtifact)
            }
            CausalLinkType::Reconsolidated => {
                matches!(self, Self::DurableMemory | Self::ReconsolidationAudit)
            }
            CausalLinkType::Extracted => {
                matches!(self, Self::DurableMemory | Self::ConsolidationArtifact)
            }
            CausalLinkType::Inferred => matches!(
                self,
                Self::DurableMemory
                    | Self::ReconsolidationAudit
                    | Self::ConsolidationArtifact
                    | Self::BeliefVersionDiff
            ),
        }
    }
}

/// Inspectable evidence attribution carried by one causal link.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CausalEvidenceAttribution {
    pub evidence_kind: CausalEvidenceKind,
    pub source_ref: String,
    pub supporting_memory_ids: Vec<MemoryId>,
    pub confidence: u16,
}

impl CausalEvidenceAttribution {
    pub fn is_source_backed(&self) -> bool {
        !self.source_ref.is_empty() && !self.supporting_memory_ids.is_empty()
    }
}

/// Durable causal-link row kept explicit instead of hidden behind traversal-only state.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CausalLink {
    pub src_memory_id: MemoryId,
    pub dst_memory_id: MemoryId,
    pub link_type: CausalLinkType,
    pub created_at_ms: u64,
    pub agent_id: Option<String>,
    pub evidence: Vec<CausalEvidenceAttribution>,
}

impl CausalLink {
    /// Whether this causal claim is explicitly source-backed by allowed evidence.
    pub fn is_source_backed(&self) -> bool {
        !self.evidence.is_empty()
            && self.evidence.iter().all(|entry| {
                entry.is_source_backed() && entry.evidence_kind.supports(self.link_type)
            })
            && self
                .evidence
                .iter()
                .any(|entry| entry.evidence_kind == CausalEvidenceKind::DurableMemory)
    }

    /// Stable evidence log for explain and audit surfaces.
    pub fn evidence_log(&self) -> String {
        self.evidence
            .iter()
            .map(|entry| {
                format!(
                    "{}:{}@{}",
                    entry.evidence_kind.as_str(),
                    entry.source_ref,
                    entry.confidence
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    }
}

// ── Neighborhood expansion ───────────────────────────────────────────────────

/// Constraints for bounded neighborhood expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpansionConstraints {
    /// Maximum depth to expand from the seed entity.
    pub max_depth: u8,
    /// Maximum total entities to return.
    pub max_entities: usize,
    /// Minimum edge strength to follow.
    pub min_strength: u16,
    /// Whether to follow directed edges in reverse.
    pub follow_reverse: bool,
}

impl Default for ExpansionConstraints {
    fn default() -> Self {
        Self {
            max_depth: 2,
            max_entities: 50,
            min_strength: 100,
            follow_reverse: false,
        }
    }
}

/// Result of a bounded neighborhood expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Neighborhood {
    /// Seed entity the expansion started from.
    pub seed: EntityId,
    /// Entities found within the expansion bounds.
    pub entities: Vec<GraphEntity>,
    /// Edges traversed during expansion.
    pub edges: Vec<GraphEdge>,
    /// Whether the expansion hit a constraint limit.
    pub truncated: bool,
    /// Actual depth reached.
    pub depth_reached: u8,
}

// ── Graph Explanation (mb-23u.8.3) ───────────────────────────────────────────

/// Cutoff boundaries hit during bounded expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CutoffReason {
    MaxDepthReached(u8),
    MaxNodesReached(usize),
    BudgetExhausted,
    PolicyNamespaceBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FollowedEdgeSummary {
    pub from: EntityId,
    pub to: EntityId,
    pub relation: RelationKind,
    pub via_additional_seed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmittedNeighborSummary {
    pub entity_id: EntityId,
    pub reason: CutoffReason,
}

/// A traceable report showing why a graph expansion stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphExplain {
    pub seeds: Vec<EntityId>,
    pub expanded_nodes: usize,
    pub edges_followed: usize,
    pub followed_edges: Vec<FollowedEdgeSummary>,
    pub omitted_neighbors: Vec<OmittedNeighborSummary>,
    pub cutoff_reasons: Vec<CutoffReason>,
}

/// One bounded step in a causal trace from a target memory back toward its roots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalTraceStep {
    pub memory_id: MemoryId,
    pub depth: usize,
    pub link_type: Option<CausalLinkType>,
    pub source_backed: bool,
    pub evidence_log: Option<String>,
}

/// Bounded causal trace and traversal explain facts for one target memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalTrace {
    pub target_memory_id: MemoryId,
    pub steps: Vec<CausalTraceStep>,
    pub root_memory_ids: Vec<MemoryId>,
    pub depth: usize,
    pub all_roots_valid: bool,
    pub explain: GraphExplain,
}

/// One descendant penalized after invalidating one causal root.
#[derive(Debug, Clone, PartialEq)]
pub struct CausalInvalidationStep {
    pub memory_id: MemoryId,
    pub depth: usize,
    pub confidence_delta: f32,
    pub link_type: CausalLinkType,
    pub source_backed: bool,
    pub evidence_log: Option<String>,
}

/// Bounded report describing one causal invalidation cascade.
#[derive(Debug, Clone, PartialEq)]
pub struct CausalInvalidationReport {
    pub root_memory_id: MemoryId,
    pub chain_length: usize,
    pub memories_penalized: usize,
    pub avg_confidence_delta: f32,
    pub steps: Vec<CausalInvalidationStep>,
    pub explain: GraphExplain,
}

/// Returns the fixed confidence penalty applied at one causal depth.
pub const fn cascade_penalty(depth: usize) -> f32 {
    match depth {
        1 => 0.20,
        2 => 0.10,
        _ => 0.05,
    }
}

// ── Engram Clustering (mb-23u.8.5) ───────────────────────────────────────────

/// Stable identifier for Engram clusters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EngramId(pub u64);

/// Durable formation metadata carried by authoritative engram rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngramFormation {
    pub formed_at_tick: u64,
    pub seed_memory_id: MemoryId,
    pub embedding_generation: &'static str,
}

/// Explicit schema for authoritative engram rows.
#[derive(Debug, Clone, PartialEq)]
pub struct EngramCluster {
    pub id: EngramId,
    pub centroid: Vec<f32>,
    pub member_count: usize,
    pub last_activated: u64,
    pub formation: EngramFormation,
}

/// Durable membership row linking a memory to one engram cluster.
#[derive(Debug, Clone, PartialEq)]
pub struct EngramMember {
    pub engram_id: EngramId,
    pub memory_id: MemoryId,
    pub distance_to_centroid: f32,
    pub joined_at_tick: u64,
}

/// Bounded encode-time candidate discovered during similar-engram lookup.
#[derive(Debug, Clone, PartialEq)]
pub struct SimilarEngramCandidate {
    pub engram_id: EngramId,
    pub similarity: f32,
}

/// Deterministic encode-time outcome for engram assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct EngramAssignment {
    pub engram_id: EngramId,
    pub created_new_cluster: bool,
    pub similar_candidates: Vec<SimilarEngramCandidate>,
}

/// Deterministic authoritative engram store that keeps cluster membership rebuildable
/// from durable `engrams` and `engram_members` facts rather than helper-index state.
#[derive(Debug, Clone, PartialEq)]
pub struct EngramStore {
    clusters: HashMap<EngramId, EngramCluster>,
    members_by_engram: HashMap<EngramId, Vec<EngramMember>>,
    memory_to_engram: HashMap<MemoryId, EngramId>,
    memory_embeddings: HashMap<MemoryId, Vec<f32>>,
    next_engram_id: u64,
    similarity_threshold: f32,
    similar_lookup_cap: usize,
}

impl Default for EngramStore {
    fn default() -> Self {
        Self::new(0.9)
    }
}

impl EngramStore {
    pub const DEFAULT_SIMILAR_LOOKUP_CAP: usize = 3;

    pub fn new(similarity_threshold: f32) -> Self {
        Self {
            clusters: HashMap::new(),
            members_by_engram: HashMap::new(),
            memory_to_engram: HashMap::new(),
            memory_embeddings: HashMap::new(),
            next_engram_id: 1,
            similarity_threshold,
            similar_lookup_cap: Self::DEFAULT_SIMILAR_LOOKUP_CAP,
        }
    }

    pub fn with_lookup_cap(mut self, similar_lookup_cap: usize) -> Self {
        self.similar_lookup_cap = similar_lookup_cap.max(1);
        self
    }

    pub fn assign_memory(
        &mut self,
        memory_id: MemoryId,
        embedding: Vec<f32>,
        formed_at_tick: u64,
        embedding_generation: &'static str,
    ) -> EngramAssignment {
        let similar_candidates = self.similar_engrams(&embedding);
        let selected = similar_candidates
            .iter()
            .find(|candidate| candidate.similarity >= self.similarity_threshold)
            .map(|candidate| candidate.engram_id);

        let (engram_id, created_new_cluster) = if let Some(engram_id) = selected {
            (engram_id, false)
        } else {
            let engram_id = EngramId(self.next_engram_id);
            self.next_engram_id += 1;
            self.clusters.insert(
                engram_id,
                EngramCluster {
                    id: engram_id,
                    centroid: embedding.clone(),
                    member_count: 0,
                    last_activated: formed_at_tick,
                    formation: EngramFormation {
                        formed_at_tick,
                        seed_memory_id: memory_id,
                        embedding_generation,
                    },
                },
            );
            (engram_id, true)
        };

        let distance_to_centroid = self
            .clusters
            .get(&engram_id)
            .map(|cluster| cosine_distance(&embedding, &cluster.centroid))
            .unwrap_or(0.0);

        self.memory_embeddings.insert(memory_id, embedding);
        let previous_engram = self.memory_to_engram.insert(memory_id, engram_id);
        if let Some(previous_engram) = previous_engram.filter(|previous| *previous != engram_id) {
            if let Some(members) = self.members_by_engram.get_mut(&previous_engram) {
                members.retain(|member| member.memory_id != memory_id);
            }
            self.refresh_cluster(previous_engram, formed_at_tick);
        }
        self.members_by_engram
            .entry(engram_id)
            .or_default()
            .retain(|member| member.memory_id != memory_id);
        self.members_by_engram
            .entry(engram_id)
            .or_default()
            .push(EngramMember {
                engram_id,
                memory_id,
                distance_to_centroid,
                joined_at_tick: formed_at_tick,
            });
        self.refresh_cluster(engram_id, formed_at_tick);

        EngramAssignment {
            engram_id,
            created_new_cluster,
            similar_candidates,
        }
    }

    pub fn similar_engrams(&self, embedding: &[f32]) -> Vec<SimilarEngramCandidate> {
        let mut candidates = self
            .clusters
            .values()
            .map(|cluster| SimilarEngramCandidate {
                engram_id: cluster.id,
                similarity: cosine_similarity(embedding, &cluster.centroid),
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| right.similarity.total_cmp(&left.similarity));
        candidates.truncate(self.similar_lookup_cap);
        candidates
    }

    pub fn refresh_cluster(&mut self, engram_id: EngramId, activated_at_tick: u64) {
        let member_memory_ids = self
            .members_by_engram
            .get(&engram_id)
            .map(|members| {
                members
                    .iter()
                    .map(|member| member.memory_id)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let member_embeddings = member_memory_ids
            .iter()
            .filter_map(|memory_id| self.memory_embeddings.get(memory_id))
            .cloned()
            .collect::<Vec<_>>();
        let centroid = average_embedding(&member_embeddings).unwrap_or_default();
        let member_count = member_memory_ids.len();

        if let Some(cluster) = self.clusters.get_mut(&engram_id) {
            cluster.centroid = centroid.clone();
            cluster.member_count = member_count;
            cluster.last_activated = activated_at_tick;
        }

        if let Some(members) = self.members_by_engram.get_mut(&engram_id) {
            for member in members.iter_mut() {
                member.distance_to_centroid = self
                    .memory_embeddings
                    .get(&member.memory_id)
                    .map(|embedding| cosine_distance(embedding, &centroid))
                    .unwrap_or(0.0);
            }
        }
    }

    pub fn rebuild_from_memberships(&self) -> Self {
        let mut rebuilt =
            Self::new(self.similarity_threshold).with_lookup_cap(self.similar_lookup_cap);
        rebuilt.next_engram_id = self.next_engram_id;
        rebuilt.memory_embeddings = self.memory_embeddings.clone();

        for (engram_id, members) in &self.members_by_engram {
            rebuilt
                .members_by_engram
                .insert(*engram_id, members.clone());
            for member in members {
                rebuilt
                    .memory_to_engram
                    .insert(member.memory_id, *engram_id);
            }
        }

        let mut cluster_ids = self.clusters.keys().copied().collect::<Vec<_>>();
        for engram_id in self.members_by_engram.keys().copied() {
            if !cluster_ids.contains(&engram_id) {
                cluster_ids.push(engram_id);
            }
        }

        for engram_id in cluster_ids {
            let formation = self
                .clusters
                .get(&engram_id)
                .map(|cluster| cluster.formation.clone())
                .or_else(|| self.derived_formation_for_engram(engram_id));
            let last_activated = self
                .clusters
                .get(&engram_id)
                .map(|cluster| cluster.last_activated)
                .or_else(|| {
                    self.members_by_engram.get(&engram_id).and_then(|members| {
                        members.iter().map(|member| member.joined_at_tick).max()
                    })
                })
                .unwrap_or(0);

            if let Some(formation) = formation {
                rebuilt.clusters.insert(
                    engram_id,
                    EngramCluster {
                        id: engram_id,
                        centroid: Vec::new(),
                        member_count: 0,
                        last_activated,
                        formation,
                    },
                );
            }
        }

        let refresh_order = rebuilt.clusters.keys().copied().collect::<Vec<_>>();
        for engram_id in refresh_order {
            let last_activated = rebuilt
                .clusters
                .get(&engram_id)
                .map(|cluster| cluster.last_activated)
                .unwrap_or(0);
            rebuilt.refresh_cluster(engram_id, last_activated);
        }

        rebuilt
    }

    fn derived_formation_for_engram(&self, engram_id: EngramId) -> Option<EngramFormation> {
        let seed_member = self
            .members_by_engram
            .get(&engram_id)?
            .iter()
            .min_by_key(|member| (member.joined_at_tick, member.memory_id.0))?;

        Some(EngramFormation {
            formed_at_tick: seed_member.joined_at_tick,
            seed_memory_id: seed_member.memory_id,
            embedding_generation: "embed.rebuilt",
        })
    }

    pub fn lookup_for_memory(&self, memory_id: MemoryId) -> Option<EngramId> {
        self.memory_to_engram.get(&memory_id).copied()
    }

    pub fn cluster(&self, engram_id: EngramId) -> Option<&EngramCluster> {
        self.clusters.get(&engram_id)
    }

    pub fn members(&self, engram_id: EngramId) -> &[EngramMember] {
        self.members_by_engram
            .get(&engram_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

fn average_embedding(embeddings: &[Vec<f32>]) -> Option<Vec<f32>> {
    let first = embeddings.first()?;
    let mut sums = vec![0.0; first.len()];
    for embedding in embeddings {
        if embedding.len() != sums.len() {
            return None;
        }
        for (sum, value) in sums.iter_mut().zip(embedding) {
            *sum += *value;
        }
    }
    let divisor = embeddings.len() as f32;
    Some(sums.into_iter().map(|sum| sum / divisor).collect())
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0;
    let mut left_norm = 0.0;
    let mut right_norm = 0.0;
    for (l, r) in left.iter().zip(right) {
        dot += l * r;
        left_norm += l * l;
        right_norm += r * r;
    }

    if left_norm == 0.0 || right_norm == 0.0 {
        0.0
    } else {
        dot / (left_norm.sqrt() * right_norm.sqrt())
    }
}

fn cosine_distance(left: &[f32], right: &[f32]) -> f32 {
    1.0 - cosine_similarity(left, right)
}

// ── Bounded BFS Expansion Planner (mb-23u.8.2) ───────────────────────────────

/// Bounded petgraph-style BFS/DFS algorithm runner.
pub struct BoundedExpansionPlanner {
    pub constraints: ExpansionConstraints,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpansionTraversal {
    BreadthFirst,
    DepthFirst,
}

impl BoundedExpansionPlanner {
    pub fn new(constraints: ExpansionConstraints) -> Self {
        Self { constraints }
    }

    fn record_cutoff(explain: &mut GraphExplain, reason: CutoffReason) {
        if !explain.cutoff_reasons.contains(&reason) {
            explain.cutoff_reasons.push(reason);
        }
    }

    fn should_follow_edge(&self, current: EntityId, edge: &GraphEdge) -> bool {
        if edge.from == current {
            true
        } else if edge.to == current {
            !edge.relation.is_directed() || self.constraints.follow_reverse
        } else {
            false
        }
    }

    /// Run a bounded petgraph BFS from a local subgraph.
    pub fn plan_bfs<F>(&self, seed: EntityId, fetch_neighbors: F) -> (Neighborhood, GraphExplain)
    where
        F: FnMut(EntityId) -> Vec<(GraphEdge, GraphEntity)>,
    {
        self.plan_with_policy(
            seed,
            &[],
            fetch_neighbors,
            |_| true,
            ExpansionTraversal::BreadthFirst,
        )
    }

    /// Run a bounded petgraph DFS from a local subgraph.
    pub fn plan_dfs<F>(&self, seed: EntityId, fetch_neighbors: F) -> (Neighborhood, GraphExplain)
    where
        F: FnMut(EntityId) -> Vec<(GraphEdge, GraphEntity)>,
    {
        self.plan_with_policy(
            seed,
            &[],
            fetch_neighbors,
            |_| true,
            ExpansionTraversal::DepthFirst,
        )
    }

    /// Run a bounded petgraph BFS from a primary seed plus additional bounded seeds
    /// such as the top-hit engram membership set.
    pub fn plan_bfs_with_additional_seeds<F>(
        &self,
        seed: EntityId,
        additional_seeds: &[EntityId],
        fetch_neighbors: F,
    ) -> (Neighborhood, GraphExplain)
    where
        F: FnMut(EntityId) -> Vec<(GraphEdge, GraphEntity)>,
    {
        self.plan_with_policy(
            seed,
            additional_seeds,
            fetch_neighbors,
            |_| true,
            ExpansionTraversal::BreadthFirst,
        )
    }

    /// Run a bounded petgraph BFS with an explicit entity policy filter.
    pub fn plan_bfs_with_policy<F, P>(
        &self,
        seed: EntityId,
        additional_seeds: &[EntityId],
        fetch_neighbors: F,
        allow_entity: P,
    ) -> (Neighborhood, GraphExplain)
    where
        F: FnMut(EntityId) -> Vec<(GraphEdge, GraphEntity)>,
        P: FnMut(&GraphEntity) -> bool,
    {
        self.plan_with_policy(
            seed,
            additional_seeds,
            fetch_neighbors,
            allow_entity,
            ExpansionTraversal::BreadthFirst,
        )
    }

    fn plan_with_policy<F, P>(
        &self,
        seed: EntityId,
        additional_seeds: &[EntityId],
        mut fetch_neighbors: F,
        mut allow_entity: P,
        traversal: ExpansionTraversal,
    ) -> (Neighborhood, GraphExplain)
    where
        F: FnMut(EntityId) -> Vec<(GraphEdge, GraphEntity)>,
        P: FnMut(&GraphEntity) -> bool,
    {
        let mut frontier = VecDeque::new();
        frontier.push_back((seed, 0));

        let mut visited = HashSet::new();
        visited.insert(seed);
        for extra_seed in additional_seeds {
            if visited.insert(*extra_seed) {
                frontier.push_back((*extra_seed, 0));
            }
        }

        let mut neighborhood = Neighborhood {
            seed,
            entities: Vec::new(),
            edges: Vec::new(),
            truncated: false,
            depth_reached: 0,
        };

        let explain_seeds: Vec<EntityId> = std::iter::once(seed)
            .chain(additional_seeds.iter().copied())
            .collect();
        let mut explain = GraphExplain {
            seeds: explain_seeds.clone(),
            expanded_nodes: 0,
            edges_followed: 0,
            followed_edges: Vec::new(),
            omitted_neighbors: Vec::new(),
            cutoff_reasons: Vec::new(),
        };

        while let Some((current, depth)) = match traversal {
            ExpansionTraversal::BreadthFirst => frontier.pop_front(),
            ExpansionTraversal::DepthFirst => frontier.pop_back(),
        } {
            if depth > neighborhood.depth_reached {
                neighborhood.depth_reached = depth;
            }

            if depth >= self.constraints.max_depth {
                Self::record_cutoff(&mut explain, CutoffReason::MaxDepthReached(depth));
                neighborhood.truncated = true;
                continue;
            }

            if explain.expanded_nodes >= self.constraints.max_entities {
                let expanded_nodes = explain.expanded_nodes;
                Self::record_cutoff(&mut explain, CutoffReason::MaxNodesReached(expanded_nodes));
                Self::record_cutoff(&mut explain, CutoffReason::BudgetExhausted);
                neighborhood.truncated = true;
                break;
            }

            explain.expanded_nodes += 1;

            let neighbors = fetch_neighbors(current);
            for (edge, entity) in neighbors {
                if edge.strength < self.constraints.min_strength
                    || !self.should_follow_edge(current, &edge)
                {
                    continue;
                }

                if !allow_entity(&entity) {
                    let reason = CutoffReason::PolicyNamespaceBlocked;
                    explain.omitted_neighbors.push(OmittedNeighborSummary {
                        entity_id: entity.id,
                        reason: reason.clone(),
                    });
                    Self::record_cutoff(&mut explain, reason);
                    neighborhood.truncated = true;
                    continue;
                }

                if neighborhood.entities.len() >= self.constraints.max_entities {
                    let reason = CutoffReason::MaxNodesReached(self.constraints.max_entities);
                    explain.omitted_neighbors.push(OmittedNeighborSummary {
                        entity_id: entity.id,
                        reason: reason.clone(),
                    });
                    Self::record_cutoff(&mut explain, reason);
                    Self::record_cutoff(&mut explain, CutoffReason::BudgetExhausted);
                    neighborhood.truncated = true;
                    break;
                }

                if !visited.contains(&entity.id) {
                    let followed_edge = FollowedEdgeSummary {
                        from: edge.from,
                        to: edge.to,
                        relation: edge.relation,
                        via_additional_seed: additional_seeds.contains(&current),
                    };
                    visited.insert(entity.id);
                    neighborhood.edges.push(edge);
                    neighborhood.entities.push(entity.clone());
                    explain.edges_followed += 1;
                    explain.followed_edges.push(followed_edge);

                    frontier.push_back((entity.id, depth + 1));
                }
            }
        }

        (neighborhood, explain)
    }
}

// ── Edge Derivation & Rebuild (mb-23u.8.1) ───────────────────────────────────

/// Source inputs for deriving an edge. Edges are not the source of truth;
/// they are formed dynamically or periodically from immutable Memory items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeDerivationInput {
    pub source_memory: MemoryId,
    pub target_memory: Option<MemoryId>,
    pub extracted_concept: Option<String>,
    pub relation: RelationKind,
    pub confidence: u16,
}

/// Stable graph-repair hook names for rebuild runs and operator traces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphRebuildHook {
    SnapshotDurableTruth,
    RebuildAdjacencyProjection,
    RebuildNeighborhoodCache,
    VerifyConsistencySnapshot,
}

impl GraphRebuildHook {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SnapshotDurableTruth => "snapshot_durable_truth",
            Self::RebuildAdjacencyProjection => "rebuild_adjacency_projection",
            Self::RebuildNeighborhoodCache => "rebuild_neighborhood_cache",
            Self::VerifyConsistencySnapshot => "verify_consistency_snapshot",
        }
    }
}

/// Named failure-injection modes for graph rebuild testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphFailureInjection {
    None,
    DropLastDerivedEdge,
    AbortAfterAdjacencyProjection,
}

impl GraphFailureInjection {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::DropLastDerivedEdge => "drop_last_derived_edge",
            Self::AbortAfterAdjacencyProjection => "abort_after_adjacency_projection",
        }
    }
}

/// Stable graph-repair metrics confirming whether a rebuild succeeded safely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphRebuildMetrics {
    pub durable_inputs_seen: usize,
    pub rebuilt_edges: usize,
    pub dropped_edges: usize,
    pub verification_passed: bool,
}

/// Operator-visible report for one graph rebuild run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphRebuildReport {
    pub hooks_run: Vec<GraphRebuildHook>,
    pub failure_injection: GraphFailureInjection,
    pub metrics: GraphRebuildMetrics,
    pub rebuilt_edges: Vec<GraphEdge>,
    pub operator_log: String,
}

impl GraphRebuildReport {
    pub fn metric_names(&self) -> [&'static str; 4] {
        [
            "graph_rebuild_durable_inputs_seen",
            "graph_rebuild_edges_total",
            "graph_rebuild_dropped_edges_total",
            "graph_rebuild_verification_passed",
        ]
    }

    pub fn hook_names(&self) -> Vec<&'static str> {
        self.hooks_run.iter().map(|hook| hook.as_str()).collect()
    }

    pub const fn succeeded_safely(&self) -> bool {
        self.metrics.verification_passed && self.metrics.dropped_edges == 0
    }
}

/// Defines how the graph recovers from staleness or loss.
pub trait GraphRebuilder {
    /// Rebuilds association edges from a durable payload, ignoring any previously
    /// derived graph state. This ensures the graph remains a derivable index
    /// rather than disjoint state.
    fn rebuild_edges_from_truth(&self, input: EdgeDerivationInput) -> Vec<GraphEdge>;

    /// Runs a named graph rebuild sequence and produces an operator-visible report.
    fn rebuild_with_hooks(
        &self,
        inputs: &[EdgeDerivationInput],
        failure_injection: GraphFailureInjection,
    ) -> GraphRebuildReport;
}

/// Deterministic graph rebuilder used by repair and failure-injection tests.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DerivedGraphRebuilder;

impl DerivedGraphRebuilder {
    fn entity_for_memory(memory_id: MemoryId) -> EntityId {
        EntityId(memory_id.0 + 1)
    }

    fn entity_for_concept(concept: &str) -> EntityId {
        let hash = concept.bytes().fold(0u64, |acc, byte| {
            acc.wrapping_mul(131).wrapping_add(byte as u64)
        });
        EntityId(1_000_000 + hash)
    }

    fn edge_from_input(&self, input: EdgeDerivationInput) -> Option<GraphEdge> {
        match (input.target_memory, input.extracted_concept.as_deref()) {
            (Some(target_memory), _) => Some(GraphEdge {
                from: Self::entity_for_memory(input.source_memory),
                to: Self::entity_for_memory(target_memory),
                relation: input.relation,
                strength: input.confidence,
            }),
            (None, Some(concept)) => Some(GraphEdge {
                from: Self::entity_for_memory(input.source_memory),
                to: Self::entity_for_concept(concept),
                relation: input.relation,
                strength: input.confidence,
            }),
            (None, None) => None,
        }
    }
}

impl GraphRebuilder for DerivedGraphRebuilder {
    fn rebuild_edges_from_truth(&self, input: EdgeDerivationInput) -> Vec<GraphEdge> {
        self.edge_from_input(input).into_iter().collect()
    }

    fn rebuild_with_hooks(
        &self,
        inputs: &[EdgeDerivationInput],
        failure_injection: GraphFailureInjection,
    ) -> GraphRebuildReport {
        let mut hooks_run = vec![GraphRebuildHook::SnapshotDurableTruth];
        let mut rebuilt_edges = inputs
            .iter()
            .cloned()
            .filter_map(|input| self.edge_from_input(input))
            .collect::<Vec<_>>();
        let durable_inputs_seen = inputs.len();
        let mut dropped_edges = 0;
        let mut verification_passed = true;

        hooks_run.push(GraphRebuildHook::RebuildAdjacencyProjection);

        match failure_injection {
            GraphFailureInjection::None => {
                hooks_run.push(GraphRebuildHook::RebuildNeighborhoodCache);
                hooks_run.push(GraphRebuildHook::VerifyConsistencySnapshot);
            }
            GraphFailureInjection::DropLastDerivedEdge => {
                if rebuilt_edges.pop().is_some() {
                    dropped_edges = 1;
                }
                hooks_run.push(GraphRebuildHook::RebuildNeighborhoodCache);
                hooks_run.push(GraphRebuildHook::VerifyConsistencySnapshot);
                verification_passed = false;
            }
            GraphFailureInjection::AbortAfterAdjacencyProjection => {
                verification_passed = false;
            }
        }

        let metrics = GraphRebuildMetrics {
            durable_inputs_seen,
            rebuilt_edges: rebuilt_edges.len(),
            dropped_edges,
            verification_passed,
        };
        let operator_log = format!(
            "hooks={} failure_injection={} durable_inputs_seen={} rebuilt_edges={} dropped_edges={} verification_passed={}",
            hooks_run
                .iter()
                .map(|hook| hook.as_str())
                .collect::<Vec<_>>()
                .join(","),
            failure_injection.as_str(),
            metrics.durable_inputs_seen,
            metrics.rebuilt_edges,
            metrics.dropped_edges,
            metrics.verification_passed,
        );

        GraphRebuildReport {
            hooks_run,
            failure_injection,
            metrics,
            rebuilt_edges,
            operator_log,
        }
    }
}

// ── Graph trait ──────────────────────────────────────────────────────────────

/// Core graph contract for bounded association lookups.
pub trait GraphApi {
    /// Returns the stable component identifier.
    fn api_component_name(&self) -> &'static str;

    /// Expands the neighborhood of an entity within given constraints.
    fn expand_neighborhood(
        &self,
        seed: EntityId,
        constraints: ExpansionConstraints,
    ) -> Neighborhood;
}

// ── Engine ───────────────────────────────────────────────────────────────────

/// Stable graph boundary owned by `membrain-core`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GraphModule;

impl GraphModule {
    /// Returns the stable component identifier for this graph surface.
    pub const fn component_name(&self) -> &'static str {
        "graph"
    }

    /// Builds a bounded invalidation cascade by walking explicit source-backed child links.
    pub fn invalidate_causal_chain(
        &self,
        root_memory_id: MemoryId,
        links: &[CausalLink],
        max_depth: usize,
        max_nodes: usize,
    ) -> CausalInvalidationReport {
        let constraints = ExpansionConstraints {
            max_depth: max_depth.max(1).min(u8::MAX as usize) as u8,
            max_entities: max_nodes.max(1),
            min_strength: 0,
            follow_reverse: false,
        };
        let planner = BoundedExpansionPlanner::new(constraints);
        let seed = Self::entity_for_memory(root_memory_id);
        let links_by_parent = causal_links_by_parent(links);
        let links_by_child = causal_links_by_child(links);
        let (neighborhood, explain) = planner.plan_bfs(seed, |entity_id| {
            let Some(memory_id) = Self::memory_for_entity(entity_id) else {
                return Vec::new();
            };
            links_by_parent
                .get(&memory_id)
                .into_iter()
                .flat_map(|items| items.iter())
                .filter(|link| link.is_source_backed())
                .map(|link| {
                    let child_entity = Self::entity_for_memory(link.dst_memory_id);
                    (
                        GraphEdge {
                            from: entity_id,
                            to: child_entity,
                            relation: RelationKind::Causal,
                            strength: strongest_evidence_confidence(link),
                        },
                        GraphEntity {
                            id: child_entity,
                            kind: EntityKind::Memory,
                            label: format!("memory:{}", link.dst_memory_id.0),
                            namespace: NamespaceId::new("causal-trace")
                                .expect("static causal trace namespace is valid"),
                            memory_id: Some(link.dst_memory_id),
                        },
                    )
                })
                .collect()
        });

        let mut depths = HashMap::new();
        depths.insert(root_memory_id, 0usize);
        let mut queue = VecDeque::from([root_memory_id]);
        while let Some(parent) = queue.pop_front() {
            let Some(parent_depth) = depths.get(&parent).copied() else {
                continue;
            };
            if parent_depth >= max_depth.max(1) {
                continue;
            }
            if let Some(entries) = links_by_parent.get(&parent) {
                for link in entries {
                    if !link.is_source_backed() {
                        continue;
                    }
                    let child = link.dst_memory_id;
                    if depths.len() >= max_nodes.max(1) {
                        break;
                    }
                    if !depths.contains_key(&child) {
                        let next_depth = parent_depth + 1;
                        depths.insert(child, next_depth);
                        queue.push_back(child);
                    }
                }
            }
        }

        let mut seen_ids = neighborhood
            .entities
            .iter()
            .filter_map(|entity| entity.memory_id)
            .collect::<Vec<_>>();
        seen_ids.sort_by_key(|id| id.0);
        seen_ids.dedup_by_key(|id| id.0);

        let mut steps = seen_ids
            .into_iter()
            .filter_map(|memory_id| {
                let incoming_link = links_by_child.get(&memory_id).and_then(|entries| {
                    entries
                        .iter()
                        .find(|link| link.is_source_backed())
                        .copied()
                        .or_else(|| entries.first().copied())
                });
                incoming_link.map(|link| CausalInvalidationStep {
                    memory_id,
                    depth: *depths.get(&memory_id).unwrap_or(&0),
                    confidence_delta: cascade_penalty(*depths.get(&memory_id).unwrap_or(&0)),
                    link_type: link.link_type,
                    source_backed: link.is_source_backed(),
                    evidence_log: Some(link.evidence_log()),
                })
            })
            .collect::<Vec<_>>();
        steps.sort_by_key(|step| (step.depth, step.memory_id.0));

        let chain_length = steps.len();
        let memories_penalized = chain_length;
        let avg_confidence_delta = if memories_penalized == 0 {
            0.0
        } else {
            steps.iter().map(|step| step.confidence_delta).sum::<f32>() / memories_penalized as f32
        };

        CausalInvalidationReport {
            root_memory_id,
            chain_length,
            memories_penalized,
            avg_confidence_delta,
            steps,
            explain,
        }
    }

    /// Builds a bounded causal trace by walking explicit source-backed causal links.
    pub fn trace_causality(
        &self,
        target_memory_id: MemoryId,
        links: &[CausalLink],
        max_depth: usize,
        max_nodes: usize,
    ) -> CausalTrace {
        let constraints = ExpansionConstraints {
            max_depth: max_depth.max(1).min(u8::MAX as usize) as u8,
            max_entities: max_nodes.max(1),
            min_strength: 0,
            follow_reverse: false,
        };
        let planner = BoundedExpansionPlanner::new(constraints);
        let seed = Self::entity_for_memory(target_memory_id);
        let links_by_child = causal_links_by_child(links);
        let (neighborhood, explain) = planner.plan_bfs(seed, |entity_id| {
            let Some(memory_id) = Self::memory_for_entity(entity_id) else {
                return Vec::new();
            };
            links_by_child
                .get(&memory_id)
                .into_iter()
                .flat_map(|items| items.iter())
                .map(|link| {
                    let parent_entity = Self::entity_for_memory(link.src_memory_id);
                    (
                        GraphEdge {
                            from: entity_id,
                            to: parent_entity,
                            relation: RelationKind::Causal,
                            strength: strongest_evidence_confidence(link),
                        },
                        GraphEntity {
                            id: parent_entity,
                            kind: EntityKind::Memory,
                            label: format!("memory:{}", link.src_memory_id.0),
                            namespace: NamespaceId::new("causal-trace")
                                .expect("static causal trace namespace is valid"),
                            memory_id: Some(link.src_memory_id),
                        },
                    )
                })
                .collect()
        });

        let mut depths = HashMap::new();
        depths.insert(target_memory_id, 0usize);
        let mut queue = VecDeque::from([target_memory_id]);
        while let Some(child) = queue.pop_front() {
            let Some(child_depth) = depths.get(&child).copied() else {
                continue;
            };
            if child_depth >= max_depth.max(1) {
                continue;
            }
            if let Some(entries) = links_by_child.get(&child) {
                for link in entries {
                    let parent = link.src_memory_id;
                    if depths.len() >= max_nodes.max(1) {
                        break;
                    }
                    if !depths.contains_key(&parent) {
                        let next_depth = child_depth + 1;
                        depths.insert(parent, next_depth);
                        queue.push_back(parent);
                    }
                }
            }
        }

        let mut seen_ids = vec![target_memory_id];
        seen_ids.extend(
            neighborhood
                .entities
                .iter()
                .filter_map(|entity| entity.memory_id),
        );
        seen_ids.sort_by_key(|id| id.0);
        seen_ids.dedup_by_key(|id| id.0);

        let mut steps = seen_ids
            .into_iter()
            .map(|memory_id| {
                let incoming_link = links_by_child.get(&memory_id).and_then(|entries| {
                    entries
                        .iter()
                        .min_by_key(|link| (link.created_at_ms, link.src_memory_id.0))
                });
                CausalTraceStep {
                    memory_id,
                    depth: *depths.get(&memory_id).unwrap_or(&0),
                    link_type: incoming_link.map(|link| link.link_type),
                    source_backed: incoming_link.is_none_or(|link| link.is_source_backed()),
                    evidence_log: incoming_link.map(|link| link.evidence_log()),
                }
            })
            .collect::<Vec<_>>();
        steps.sort_by_key(|step| (step.depth, step.memory_id.0));

        let mut root_memory_ids = steps
            .iter()
            .filter_map(|step| {
                (!links_by_child.contains_key(&step.memory_id)).then_some(step.memory_id)
            })
            .collect::<Vec<_>>();
        root_memory_ids.sort_by_key(|id| id.0);
        root_memory_ids.dedup_by_key(|id| id.0);

        let traversal_capped = explain.cutoff_reasons.iter().any(|reason| {
            matches!(
                reason,
                CutoffReason::MaxDepthReached(_)
                    | CutoffReason::MaxNodesReached(_)
                    | CutoffReason::BudgetExhausted
            )
        });
        let all_roots_valid = !traversal_capped
            && !root_memory_ids.is_empty()
            && steps
                .iter()
                .filter(|step| root_memory_ids.contains(&step.memory_id))
                .all(|step| step.source_backed);
        let depth = steps.iter().map(|step| step.depth).max().unwrap_or(0);

        CausalTrace {
            target_memory_id,
            steps,
            root_memory_ids,
            depth,
            all_roots_valid,
            explain,
        }
    }

    const fn entity_for_memory(memory_id: MemoryId) -> EntityId {
        EntityId(memory_id.0)
    }

    const fn memory_for_entity(entity_id: EntityId) -> Option<MemoryId> {
        Some(MemoryId(entity_id.0))
    }
}

fn causal_links_by_child<'a>(links: &'a [CausalLink]) -> HashMap<MemoryId, Vec<&'a CausalLink>> {
    let mut by_child = HashMap::<MemoryId, Vec<&'a CausalLink>>::new();
    for link in links {
        by_child.entry(link.dst_memory_id).or_default().push(link);
    }
    for entries in by_child.values_mut() {
        entries.sort_by_key(|link| (link.created_at_ms, link.src_memory_id.0));
    }
    by_child
}

fn causal_links_by_parent<'a>(links: &'a [CausalLink]) -> HashMap<MemoryId, Vec<&'a CausalLink>> {
    let mut by_parent = HashMap::<MemoryId, Vec<&'a CausalLink>>::new();
    for link in links {
        by_parent.entry(link.src_memory_id).or_default().push(link);
    }
    for entries in by_parent.values_mut() {
        entries.sort_by_key(|link| (link.created_at_ms, link.dst_memory_id.0));
    }
    by_parent
}

fn strongest_evidence_confidence(link: &CausalLink) -> u16 {
    link.evidence
        .iter()
        .map(|entry| entry.confidence)
        .max()
        .unwrap_or(0)
}

impl GraphApi for GraphModule {
    fn api_component_name(&self) -> &'static str {
        "graph"
    }

    fn expand_neighborhood(
        &self,
        seed: EntityId,
        _constraints: ExpansionConstraints,
    ) -> Neighborhood {
        // Placeholder: returns empty neighborhood
        Neighborhood {
            seed,
            entities: Vec::new(),
            edges: Vec::new(),
            truncated: false,
            depth_reached: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_neighborhood_expansion() {
        let graph = GraphModule;
        let neighborhood = graph.expand_neighborhood(EntityId(1), ExpansionConstraints::default());
        assert_eq!(neighborhood.seed, EntityId(1));
        assert!(neighborhood.entities.is_empty());
        assert!(!neighborhood.truncated);
        assert_eq!(neighborhood.depth_reached, 0);
    }

    #[test]
    fn relation_kind_properties() {
        assert!(RelationKind::DerivedFrom.is_directed());
        assert!(RelationKind::Causal.is_directed());
        assert!(RelationKind::Supersedes.is_directed());
        assert!(!RelationKind::Mentions.is_directed());
        assert!(!RelationKind::SharedTopic.is_directed());
    }

    #[test]
    fn causal_link_requires_allowed_source_backed_evidence() {
        let link = CausalLink {
            src_memory_id: MemoryId(1),
            dst_memory_id: MemoryId(2),
            link_type: CausalLinkType::Reconsolidated,
            created_at_ms: 42,
            agent_id: Some("agent.writer".to_string()),
            evidence: vec![
                CausalEvidenceAttribution {
                    evidence_kind: CausalEvidenceKind::DurableMemory,
                    source_ref: "memory://tests/1".to_string(),
                    supporting_memory_ids: vec![MemoryId(1)],
                    confidence: 900,
                },
                CausalEvidenceAttribution {
                    evidence_kind: CausalEvidenceKind::ReconsolidationAudit,
                    source_ref: "reconsolidation://tests/1@42".to_string(),
                    supporting_memory_ids: vec![MemoryId(1), MemoryId(2)],
                    confidence: 880,
                },
            ],
        };

        assert!(link.is_source_backed());
        assert!(link
            .evidence_log()
            .contains("durable_memory:memory://tests/1@900"));
        assert!(link
            .evidence_log()
            .contains("reconsolidation_audit:reconsolidation://tests/1@42@880"));
    }

    #[test]
    fn causal_link_rejects_unsupported_evidence_mix() {
        let link = CausalLink {
            src_memory_id: MemoryId(1),
            dst_memory_id: MemoryId(2),
            link_type: CausalLinkType::Derived,
            created_at_ms: 42,
            agent_id: None,
            evidence: vec![CausalEvidenceAttribution {
                evidence_kind: CausalEvidenceKind::ReconsolidationAudit,
                source_ref: "reconsolidation://tests/1@42".to_string(),
                supporting_memory_ids: vec![MemoryId(1)],
                confidence: 700,
            }],
        };

        assert!(!link.is_source_backed());
    }

    #[test]
    fn trace_causality_walks_source_backed_parents_until_roots() {
        let graph = GraphModule;
        let trace = graph.trace_causality(
            MemoryId(30),
            &[
                CausalLink {
                    src_memory_id: MemoryId(20),
                    dst_memory_id: MemoryId(30),
                    link_type: CausalLinkType::Derived,
                    created_at_ms: 120,
                    agent_id: Some("agent.alpha".to_string()),
                    evidence: vec![CausalEvidenceAttribution {
                        evidence_kind: CausalEvidenceKind::DurableMemory,
                        source_ref: "memory://tests/20".to_string(),
                        supporting_memory_ids: vec![MemoryId(20)],
                        confidence: 820,
                    }],
                },
                CausalLink {
                    src_memory_id: MemoryId(10),
                    dst_memory_id: MemoryId(20),
                    link_type: CausalLinkType::Reconsolidated,
                    created_at_ms: 80,
                    agent_id: Some("agent.beta".to_string()),
                    evidence: vec![
                        CausalEvidenceAttribution {
                            evidence_kind: CausalEvidenceKind::DurableMemory,
                            source_ref: "memory://tests/10".to_string(),
                            supporting_memory_ids: vec![MemoryId(10)],
                            confidence: 910,
                        },
                        CausalEvidenceAttribution {
                            evidence_kind: CausalEvidenceKind::ReconsolidationAudit,
                            source_ref: "reconsolidation://tests/10@80".to_string(),
                            supporting_memory_ids: vec![MemoryId(10), MemoryId(20)],
                            confidence: 780,
                        },
                    ],
                },
            ],
            4,
            8,
        );

        assert_eq!(trace.target_memory_id, MemoryId(30));
        assert_eq!(trace.depth, 2);
        assert_eq!(trace.root_memory_ids, vec![MemoryId(10)]);
        assert!(trace.all_roots_valid);
        assert_eq!(trace.steps.len(), 3);
        assert_eq!(trace.steps[0].memory_id, MemoryId(30));
        assert_eq!(trace.steps[0].depth, 0);
        assert_eq!(trace.steps[1].memory_id, MemoryId(20));
        assert_eq!(trace.steps[1].depth, 1);
        assert_eq!(
            trace.steps[1].link_type,
            Some(CausalLinkType::Reconsolidated)
        );
        assert_eq!(trace.steps[2].memory_id, MemoryId(10));
        assert_eq!(trace.steps[2].depth, 2);
        assert_eq!(trace.steps[2].link_type, None);
        assert_eq!(trace.steps[2].evidence_log, None);
        assert_eq!(trace.explain.edges_followed, 2);
        assert_eq!(trace.explain.followed_edges.len(), 2);
        assert_eq!(
            trace.explain.followed_edges[0].relation,
            RelationKind::Causal
        );
    }

    #[test]
    fn trace_causality_marks_roots_invalid_when_traversal_hits_caps() {
        let graph = GraphModule;
        let trace = graph.trace_causality(
            MemoryId(40),
            &[
                CausalLink {
                    src_memory_id: MemoryId(30),
                    dst_memory_id: MemoryId(40),
                    link_type: CausalLinkType::Derived,
                    created_at_ms: 140,
                    agent_id: None,
                    evidence: vec![CausalEvidenceAttribution {
                        evidence_kind: CausalEvidenceKind::DurableMemory,
                        source_ref: "memory://tests/30".to_string(),
                        supporting_memory_ids: vec![MemoryId(30)],
                        confidence: 800,
                    }],
                },
                CausalLink {
                    src_memory_id: MemoryId(20),
                    dst_memory_id: MemoryId(30),
                    link_type: CausalLinkType::Derived,
                    created_at_ms: 120,
                    agent_id: None,
                    evidence: vec![CausalEvidenceAttribution {
                        evidence_kind: CausalEvidenceKind::DurableMemory,
                        source_ref: "memory://tests/20".to_string(),
                        supporting_memory_ids: vec![MemoryId(20)],
                        confidence: 780,
                    }],
                },
                CausalLink {
                    src_memory_id: MemoryId(10),
                    dst_memory_id: MemoryId(20),
                    link_type: CausalLinkType::Derived,
                    created_at_ms: 100,
                    agent_id: None,
                    evidence: vec![CausalEvidenceAttribution {
                        evidence_kind: CausalEvidenceKind::DurableMemory,
                        source_ref: "memory://tests/10".to_string(),
                        supporting_memory_ids: vec![MemoryId(10)],
                        confidence: 760,
                    }],
                },
            ],
            1,
            8,
        );

        assert_eq!(trace.depth, 1);
        assert!(!trace.all_roots_valid);
        assert!(trace.root_memory_ids.is_empty());
        assert!(trace
            .explain
            .cutoff_reasons
            .contains(&CutoffReason::MaxDepthReached(1)));
    }

    #[test]
    fn invalidate_causal_chain_reports_bounded_penalties_by_depth() {
        let graph = GraphModule;
        let report = graph.invalidate_causal_chain(
            MemoryId(10),
            &[
                CausalLink {
                    src_memory_id: MemoryId(10),
                    dst_memory_id: MemoryId(20),
                    link_type: CausalLinkType::Derived,
                    created_at_ms: 80,
                    agent_id: Some("agent.alpha".to_string()),
                    evidence: vec![CausalEvidenceAttribution {
                        evidence_kind: CausalEvidenceKind::DurableMemory,
                        source_ref: "memory://tests/10".to_string(),
                        supporting_memory_ids: vec![MemoryId(10)],
                        confidence: 910,
                    }],
                },
                CausalLink {
                    src_memory_id: MemoryId(20),
                    dst_memory_id: MemoryId(30),
                    link_type: CausalLinkType::Reconsolidated,
                    created_at_ms: 120,
                    agent_id: Some("agent.beta".to_string()),
                    evidence: vec![
                        CausalEvidenceAttribution {
                            evidence_kind: CausalEvidenceKind::DurableMemory,
                            source_ref: "memory://tests/20".to_string(),
                            supporting_memory_ids: vec![MemoryId(20)],
                            confidence: 820,
                        },
                        CausalEvidenceAttribution {
                            evidence_kind: CausalEvidenceKind::ReconsolidationAudit,
                            source_ref: "reconsolidation://tests/20@120".to_string(),
                            supporting_memory_ids: vec![MemoryId(20), MemoryId(30)],
                            confidence: 760,
                        },
                    ],
                },
                CausalLink {
                    src_memory_id: MemoryId(30),
                    dst_memory_id: MemoryId(40),
                    link_type: CausalLinkType::Extracted,
                    created_at_ms: 160,
                    agent_id: Some("agent.gamma".to_string()),
                    evidence: vec![
                        CausalEvidenceAttribution {
                            evidence_kind: CausalEvidenceKind::DurableMemory,
                            source_ref: "memory://tests/30".to_string(),
                            supporting_memory_ids: vec![MemoryId(30)],
                            confidence: 790,
                        },
                        CausalEvidenceAttribution {
                            evidence_kind: CausalEvidenceKind::ConsolidationArtifact,
                            source_ref: "consolidation://tests/30#skill".to_string(),
                            supporting_memory_ids: vec![MemoryId(30), MemoryId(40)],
                            confidence: 740,
                        },
                    ],
                },
            ],
            4,
            8,
        );

        assert_eq!(report.root_memory_id, MemoryId(10));
        assert_eq!(report.chain_length, 3);
        assert_eq!(report.memories_penalized, 3);
        assert!((report.avg_confidence_delta - (0.20 + 0.10 + 0.05) / 3.0).abs() < 1e-6);
        assert_eq!(report.steps[0].memory_id, MemoryId(20));
        assert_eq!(report.steps[0].depth, 1);
        assert!((report.steps[0].confidence_delta - 0.20).abs() < 1e-6);
        assert_eq!(report.steps[1].memory_id, MemoryId(30));
        assert_eq!(report.steps[1].link_type, CausalLinkType::Reconsolidated);
        assert!((report.steps[1].confidence_delta - 0.10).abs() < 1e-6);
        assert_eq!(report.steps[2].memory_id, MemoryId(40));
        assert_eq!(report.steps[2].link_type, CausalLinkType::Extracted);
        assert!((report.steps[2].confidence_delta - 0.05).abs() < 1e-6);
        assert_eq!(cascade_penalty(1), 0.20);
        assert_eq!(cascade_penalty(2), 0.10);
        assert_eq!(cascade_penalty(5), 0.05);
    }

    #[test]
    fn entity_kinds_have_stable_names() {
        assert_eq!(EntityKind::Memory.as_str(), "memory");
        assert_eq!(EntityKind::Concept.as_str(), "concept");
        assert_eq!(EntityKind::Agent.as_str(), "agent");
    }

    #[test]
    fn expansion_constraints_default() {
        let c = ExpansionConstraints::default();
        assert_eq!(c.max_depth, 2);
        assert_eq!(c.max_entities, 50);
        assert_eq!(c.min_strength, 100);
        assert!(!c.follow_reverse);
    }

    #[test]
    fn test_edge_formation_from_memory() {
        let input = EdgeDerivationInput {
            source_memory: MemoryId(0),
            target_memory: Some(MemoryId(1)),
            extracted_concept: None,
            relation: RelationKind::DerivedFrom,
            confidence: 900,
        };
        assert_eq!(input.relation, RelationKind::DerivedFrom);
        assert_eq!(input.confidence, 900);
    }

    #[test]
    fn graph_rebuild_from_truth_uses_named_hooks_and_safe_success_metrics() {
        let rebuilder = DerivedGraphRebuilder;
        let inputs = vec![
            EdgeDerivationInput {
                source_memory: MemoryId(10),
                target_memory: Some(MemoryId(11)),
                extracted_concept: None,
                relation: RelationKind::DerivedFrom,
                confidence: 920,
            },
            EdgeDerivationInput {
                source_memory: MemoryId(10),
                target_memory: None,
                extracted_concept: Some("graph_consistency_snapshot".into()),
                relation: RelationKind::Mentions,
                confidence: 870,
            },
        ];

        let rebuilt = rebuilder.rebuild_edges_from_truth(inputs[0].clone());
        let report = rebuilder.rebuild_with_hooks(&inputs, GraphFailureInjection::None);

        assert_eq!(rebuilt.len(), 1);
        assert_eq!(report.metrics.durable_inputs_seen, 2);
        assert_eq!(report.metrics.rebuilt_edges, 2);
        assert_eq!(report.metrics.dropped_edges, 0);
        assert!(report.metrics.verification_passed);
        assert!(report.succeeded_safely());
        assert_eq!(
            report.hook_names(),
            vec![
                "snapshot_durable_truth",
                "rebuild_adjacency_projection",
                "rebuild_neighborhood_cache",
                "verify_consistency_snapshot",
            ]
        );
        assert!(report.operator_log.contains("hooks=snapshot_durable_truth,rebuild_adjacency_projection,rebuild_neighborhood_cache,verify_consistency_snapshot"));
        assert!(report.operator_log.contains("failure_injection=none"));
        assert!(report.operator_log.contains("verification_passed=true"));
    }

    #[test]
    fn test_bounded_bfs_expansion() {
        let planner = BoundedExpansionPlanner::new(ExpansionConstraints {
            max_depth: 2,
            max_entities: 5,
            min_strength: 50,
            follow_reverse: false,
        });

        let seed = EntityId(1);
        let fetcher = |current: EntityId| -> Vec<(GraphEdge, GraphEntity)> {
            if current == seed {
                vec![(
                    GraphEdge {
                        from: current,
                        to: EntityId(2),
                        relation: RelationKind::Mentions,
                        strength: 100,
                    },
                    GraphEntity {
                        id: EntityId(2),
                        kind: EntityKind::Concept,
                        label: "test".into(),
                        namespace: NamespaceId::new("ns").unwrap(),
                        memory_id: None,
                    },
                )]
            } else {
                vec![]
            }
        };

        let (nb, ex) = planner.plan_bfs(seed, fetcher);
        assert_eq!(nb.depth_reached, 1);
        assert_eq!(ex.expanded_nodes, 2); // Seed + child
        assert_eq!(ex.edges_followed, 1);
    }

    #[test]
    fn engram_assignment_creates_then_joins_using_bounded_similarity_lookup() {
        let mut store = EngramStore::new(0.95).with_lookup_cap(3);

        let created = store.assign_memory(MemoryId(1), vec![1.0, 0.0], 10, "embed.v1");
        let joined = store.assign_memory(MemoryId(2), vec![0.99, 0.01], 11, "embed.v1");

        assert!(created.created_new_cluster);
        assert!(!joined.created_new_cluster);
        assert_eq!(joined.engram_id, created.engram_id);
        assert_eq!(joined.similar_candidates.len(), 1);
        assert_eq!(
            store.lookup_for_memory(MemoryId(2)),
            Some(created.engram_id)
        );
    }

    #[test]
    fn engram_assignment_mints_new_cluster_below_threshold() {
        let mut store = EngramStore::new(0.98).with_lookup_cap(3);

        let first = store.assign_memory(MemoryId(1), vec![1.0, 0.0], 10, "embed.v1");
        let second = store.assign_memory(MemoryId(2), vec![0.0, 1.0], 11, "embed.v1");

        assert!(first.created_new_cluster);
        assert!(second.created_new_cluster);
        assert_ne!(first.engram_id, second.engram_id);
        assert_eq!(second.similar_candidates.len(), 1);
    }

    #[test]
    fn centroid_refresh_and_member_count_stay_deterministic() {
        let mut store = EngramStore::new(0.90);

        let assignment = store.assign_memory(MemoryId(1), vec![1.0, 0.0], 10, "embed.v1");
        store.assign_memory(MemoryId(2), vec![0.95, 0.05], 11, "embed.v1");
        store.assign_memory(MemoryId(3), vec![0.90, 0.10], 12, "embed.v1");
        store.refresh_cluster(assignment.engram_id, 13);

        let cluster = store.cluster(assignment.engram_id).unwrap();
        assert_eq!(cluster.member_count, 3);
        assert_eq!(cluster.last_activated, 13);
        assert_eq!(cluster.centroid, vec![0.95, 0.05]);
        assert_eq!(store.members(assignment.engram_id).len(), 3);
    }

    #[test]
    fn rebuild_restores_centroid_after_partial_divergence_without_losing_membership_truth() {
        let mut store = EngramStore::new(0.90);
        let assignment = store.assign_memory(MemoryId(1), vec![1.0, 0.0], 10, "embed.v1");
        store.assign_memory(MemoryId(2), vec![0.95, 0.05], 11, "embed.v1");

        let cluster = store.clusters.get_mut(&assignment.engram_id).unwrap();
        cluster.centroid = vec![9.0, 9.0];
        cluster.member_count = 99;

        let rebuilt = store.rebuild_from_memberships();
        let rebuilt_cluster = rebuilt.cluster(assignment.engram_id).unwrap();

        assert_eq!(rebuilt_cluster.member_count, 2);
        assert_eq!(rebuilt_cluster.centroid, vec![0.975, 0.025]);
        assert_eq!(
            rebuilt.lookup_for_memory(MemoryId(1)),
            Some(assignment.engram_id)
        );
        assert_eq!(
            rebuilt.lookup_for_memory(MemoryId(2)),
            Some(assignment.engram_id)
        );
    }

    #[test]
    fn lookup_for_memory_avoids_full_store_scan() {
        let mut store = EngramStore::new(0.90);
        let assignment = store.assign_memory(MemoryId(77), vec![0.7, 0.3], 10, "embed.v1");

        assert_eq!(
            store.lookup_for_memory(MemoryId(77)),
            Some(assignment.engram_id)
        );
        assert_eq!(store.lookup_for_memory(MemoryId(999)), None);
    }

    #[test]
    fn rebuild_recovers_missing_cluster_row_from_authoritative_memberships() {
        let mut store = EngramStore::new(0.95).with_lookup_cap(3);
        let assignment = store.assign_memory(MemoryId(20), vec![1.0, 0.0], 300, "embed.v1");
        store.assign_memory(MemoryId(21), vec![0.98, 0.02], 301, "embed.v1");

        let mut divergent = store.clone();
        divergent.clusters.remove(&assignment.engram_id);

        let rebuilt = divergent.rebuild_from_memberships();
        let rebuilt_cluster = rebuilt.cluster(assignment.engram_id).unwrap();

        assert_eq!(rebuilt_cluster.member_count, 2);
        assert_eq!(rebuilt_cluster.centroid, vec![0.99, 0.01]);
        assert_eq!(rebuilt_cluster.last_activated, 301);
        assert_eq!(rebuilt_cluster.formation.formed_at_tick, 300);
        assert_eq!(rebuilt_cluster.formation.seed_memory_id, MemoryId(20));
        assert_eq!(
            rebuilt_cluster.formation.embedding_generation,
            "embed.rebuilt"
        );
        assert_eq!(
            rebuilt.lookup_for_memory(MemoryId(20)),
            Some(assignment.engram_id)
        );
        assert_eq!(
            rebuilt.lookup_for_memory(MemoryId(21)),
            Some(assignment.engram_id)
        );
    }

    #[test]
    fn similar_engram_lookup_stays_capped_to_top_three() {
        let mut store = EngramStore::new(0.999).with_lookup_cap(3);
        store.assign_memory(MemoryId(1), vec![1.0, 0.0], 10, "embed.v1");
        store.assign_memory(MemoryId(2), vec![0.0, 1.0], 11, "embed.v1");
        store.assign_memory(MemoryId(3), vec![-1.0, 0.0], 12, "embed.v1");
        store.assign_memory(MemoryId(4), vec![0.0, -1.0], 13, "embed.v1");

        let candidates = store.similar_engrams(&[0.8, 0.2]);

        assert_eq!(candidates.len(), 3);
        assert!(candidates[0].similarity >= candidates[1].similarity);
        assert!(candidates[1].similarity >= candidates[2].similarity);
    }

    #[test]
    fn bfs_marks_truncation_when_neighbor_fanout_exceeds_entity_budget() {
        let planner = BoundedExpansionPlanner::new(ExpansionConstraints {
            max_depth: 3,
            max_entities: 1,
            min_strength: 50,
            follow_reverse: false,
        });

        let seed = EntityId(1);
        let (nb, ex) = planner.plan_bfs(seed, |current| {
            if current == seed {
                vec![
                    (
                        GraphEdge {
                            from: current,
                            to: EntityId(2),
                            relation: RelationKind::Mentions,
                            strength: 100,
                        },
                        GraphEntity {
                            id: EntityId(2),
                            kind: EntityKind::Concept,
                            label: "first".into(),
                            namespace: NamespaceId::new("ns").unwrap(),
                            memory_id: None,
                        },
                    ),
                    (
                        GraphEdge {
                            from: current,
                            to: EntityId(3),
                            relation: RelationKind::Mentions,
                            strength: 100,
                        },
                        GraphEntity {
                            id: EntityId(3),
                            kind: EntityKind::Concept,
                            label: "second".into(),
                            namespace: NamespaceId::new("ns").unwrap(),
                            memory_id: None,
                        },
                    ),
                ]
            } else {
                Vec::new()
            }
        });

        assert_eq!(nb.entities.len(), 1);
        assert!(nb.truncated);
        assert_eq!(ex.edges_followed, 1);
        assert!(ex
            .cutoff_reasons
            .contains(&CutoffReason::MaxNodesReached(1)));
    }

    #[test]
    fn bfs_supports_multi_seed_expansion_for_top_hit_engram_membership() {
        let planner = BoundedExpansionPlanner::new(ExpansionConstraints {
            max_depth: 2,
            max_entities: 6,
            min_strength: 50,
            follow_reverse: false,
        });

        let seed = EntityId(1);
        let additional_seeds = [EntityId(9)];
        let (nb, ex) =
            planner.plan_bfs_with_additional_seeds(
                seed,
                &additional_seeds,
                |current| match current {
                    EntityId(1) => vec![(
                        GraphEdge {
                            from: current,
                            to: EntityId(2),
                            relation: RelationKind::Mentions,
                            strength: 100,
                        },
                        GraphEntity {
                            id: EntityId(2),
                            kind: EntityKind::Concept,
                            label: "seed-neighbor".into(),
                            namespace: NamespaceId::new("ns").unwrap(),
                            memory_id: None,
                        },
                    )],
                    EntityId(9) => vec![(
                        GraphEdge {
                            from: current,
                            to: EntityId(10),
                            relation: RelationKind::SharedTopic,
                            strength: 100,
                        },
                        GraphEntity {
                            id: EntityId(10),
                            kind: EntityKind::Memory,
                            label: "engram-member-neighbor".into(),
                            namespace: NamespaceId::new("ns").unwrap(),
                            memory_id: Some(MemoryId(10)),
                        },
                    )],
                    _ => Vec::new(),
                },
            );

        assert_eq!(ex.seeds, vec![EntityId(1), EntityId(9)]);
        assert_eq!(ex.expanded_nodes, 4);
        assert_eq!(ex.edges_followed, 2);
        assert_eq!(nb.entities.len(), 2);
        assert!(!nb.truncated);
        assert_eq!(ex.followed_edges.len(), 2);
        assert_eq!(ex.followed_edges[0].from, EntityId(1));
        assert!(!ex.followed_edges[0].via_additional_seed);
        assert_eq!(ex.followed_edges[1].from, EntityId(9));
        assert!(ex.followed_edges[1].via_additional_seed);
    }

    #[test]
    fn bfs_skips_reverse_directed_edges_without_follow_reverse() {
        let planner = BoundedExpansionPlanner::new(ExpansionConstraints {
            max_depth: 2,
            max_entities: 4,
            min_strength: 50,
            follow_reverse: false,
        });

        let seed = EntityId(2);
        let (nb, ex) = planner.plan_bfs(seed, |current| {
            if current == seed {
                vec![(
                    GraphEdge {
                        from: EntityId(1),
                        to: current,
                        relation: RelationKind::DerivedFrom,
                        strength: 100,
                    },
                    GraphEntity {
                        id: EntityId(1),
                        kind: EntityKind::Memory,
                        label: "reverse-only".into(),
                        namespace: NamespaceId::new("ns").unwrap(),
                        memory_id: Some(MemoryId(1)),
                    },
                )]
            } else {
                Vec::new()
            }
        });

        assert!(nb.entities.is_empty());
        assert_eq!(ex.edges_followed, 0);
        assert!(!nb.truncated);
    }

    #[test]
    fn bfs_can_follow_reverse_directed_edges_when_enabled() {
        let planner = BoundedExpansionPlanner::new(ExpansionConstraints {
            max_depth: 2,
            max_entities: 4,
            min_strength: 50,
            follow_reverse: true,
        });

        let seed = EntityId(2);
        let (nb, ex) = planner.plan_bfs(seed, |current| {
            if current == seed {
                vec![(
                    GraphEdge {
                        from: EntityId(1),
                        to: current,
                        relation: RelationKind::DerivedFrom,
                        strength: 100,
                    },
                    GraphEntity {
                        id: EntityId(1),
                        kind: EntityKind::Memory,
                        label: "reverse-allowed".into(),
                        namespace: NamespaceId::new("ns").unwrap(),
                        memory_id: Some(MemoryId(1)),
                    },
                )]
            } else {
                Vec::new()
            }
        });

        assert_eq!(nb.entities.len(), 1);
        assert_eq!(nb.entities[0].id, EntityId(1));
        assert_eq!(ex.edges_followed, 1);
    }

    #[test]
    fn bfs_reports_budget_exhaustion_when_seed_queue_hits_entity_cap() {
        let planner = BoundedExpansionPlanner::new(ExpansionConstraints {
            max_depth: 2,
            max_entities: 1,
            min_strength: 50,
            follow_reverse: false,
        });

        let (nb, ex) =
            planner.plan_bfs_with_additional_seeds(EntityId(1), &[EntityId(9)], |_| Vec::new());

        assert!(nb.entities.is_empty());
        assert_eq!(ex.expanded_nodes, 1);
        assert!(ex
            .cutoff_reasons
            .contains(&CutoffReason::MaxNodesReached(1)));
        assert!(ex.cutoff_reasons.contains(&CutoffReason::BudgetExhausted));
    }

    #[test]
    fn bfs_allows_seed_plus_one_neighbor_when_entity_cap_is_one() {
        let planner = BoundedExpansionPlanner::new(ExpansionConstraints {
            max_depth: 2,
            max_entities: 1,
            min_strength: 50,
            follow_reverse: false,
        });

        let seed = EntityId(1);
        let (nb, ex) = planner.plan_bfs(seed, |current| {
            if current == seed {
                vec![
                    (
                        GraphEdge {
                            from: current,
                            to: EntityId(2),
                            relation: RelationKind::Mentions,
                            strength: 100,
                        },
                        GraphEntity {
                            id: EntityId(2),
                            kind: EntityKind::Concept,
                            label: "first".into(),
                            namespace: NamespaceId::new("ns").unwrap(),
                            memory_id: None,
                        },
                    ),
                    (
                        GraphEdge {
                            from: current,
                            to: EntityId(3),
                            relation: RelationKind::Mentions,
                            strength: 100,
                        },
                        GraphEntity {
                            id: EntityId(3),
                            kind: EntityKind::Concept,
                            label: "second".into(),
                            namespace: NamespaceId::new("ns").unwrap(),
                            memory_id: None,
                        },
                    ),
                ]
            } else {
                Vec::new()
            }
        });

        assert_eq!(nb.entities.len(), 1);
        assert_eq!(nb.entities[0].id, EntityId(2));
        assert_eq!(ex.expanded_nodes, 1);
        assert_eq!(ex.edges_followed, 1);
        assert!(ex
            .cutoff_reasons
            .contains(&CutoffReason::MaxNodesReached(1)));
        assert!(ex.cutoff_reasons.contains(&CutoffReason::BudgetExhausted));
    }

    #[test]
    fn bfs_reports_policy_namespace_cutoff_for_blocked_neighbors() {
        let planner = BoundedExpansionPlanner::new(ExpansionConstraints {
            max_depth: 2,
            max_entities: 4,
            min_strength: 50,
            follow_reverse: false,
        });

        let seed = EntityId(1);
        let allowed_namespace = NamespaceId::new("ns.allowed").unwrap();
        let blocked_namespace = NamespaceId::new("ns.blocked").unwrap();
        let (nb, ex) = planner.plan_bfs_with_policy(
            seed,
            &[],
            |current| {
                if current == seed {
                    vec![
                        (
                            GraphEdge {
                                from: current,
                                to: EntityId(2),
                                relation: RelationKind::Mentions,
                                strength: 100,
                            },
                            GraphEntity {
                                id: EntityId(2),
                                kind: EntityKind::Concept,
                                label: "allowed".into(),
                                namespace: allowed_namespace.clone(),
                                memory_id: None,
                            },
                        ),
                        (
                            GraphEdge {
                                from: current,
                                to: EntityId(3),
                                relation: RelationKind::SharedTopic,
                                strength: 100,
                            },
                            GraphEntity {
                                id: EntityId(3),
                                kind: EntityKind::Memory,
                                label: "blocked".into(),
                                namespace: blocked_namespace.clone(),
                                memory_id: Some(MemoryId(3)),
                            },
                        ),
                    ]
                } else {
                    Vec::new()
                }
            },
            |entity| entity.namespace == allowed_namespace,
        );

        assert_eq!(nb.entities.len(), 1);
        assert_eq!(nb.entities[0].id, EntityId(2));
        assert_eq!(ex.edges_followed, 1);
        assert!(nb.truncated);
        assert!(ex
            .cutoff_reasons
            .contains(&CutoffReason::PolicyNamespaceBlocked));
        assert_eq!(ex.omitted_neighbors.len(), 1);
        assert_eq!(ex.omitted_neighbors[0].entity_id, EntityId(3));
        assert_eq!(
            ex.omitted_neighbors[0].reason,
            CutoffReason::PolicyNamespaceBlocked
        );
    }

    #[test]
    fn bfs_honors_depth_cap_for_engram_seeded_second_hop_expansion() {
        let planner = BoundedExpansionPlanner::new(ExpansionConstraints {
            max_depth: 1,
            max_entities: 6,
            min_strength: 50,
            follow_reverse: false,
        });

        let seed = EntityId(1);
        let additional_seeds = [EntityId(9)];
        let (nb, ex) =
            planner.plan_bfs_with_additional_seeds(
                seed,
                &additional_seeds,
                |current| match current {
                    EntityId(1) => vec![(
                        GraphEdge {
                            from: current,
                            to: EntityId(2),
                            relation: RelationKind::Mentions,
                            strength: 100,
                        },
                        GraphEntity {
                            id: EntityId(2),
                            kind: EntityKind::Concept,
                            label: "primary-hop".into(),
                            namespace: NamespaceId::new("ns").unwrap(),
                            memory_id: None,
                        },
                    )],
                    EntityId(9) => vec![(
                        GraphEdge {
                            from: current,
                            to: EntityId(10),
                            relation: RelationKind::SharedTopic,
                            strength: 100,
                        },
                        GraphEntity {
                            id: EntityId(10),
                            kind: EntityKind::Memory,
                            label: "engram-hop".into(),
                            namespace: NamespaceId::new("ns").unwrap(),
                            memory_id: Some(MemoryId(10)),
                        },
                    )],
                    EntityId(10) => vec![(
                        GraphEdge {
                            from: current,
                            to: EntityId(11),
                            relation: RelationKind::SharedTopic,
                            strength: 100,
                        },
                        GraphEntity {
                            id: EntityId(11),
                            kind: EntityKind::Concept,
                            label: "too-deep".into(),
                            namespace: NamespaceId::new("ns").unwrap(),
                            memory_id: None,
                        },
                    )],
                    _ => Vec::new(),
                },
            );

        assert_eq!(ex.seeds, vec![EntityId(1), EntityId(9)]);
        assert_eq!(nb.entities.len(), 2);
        assert_eq!(nb.depth_reached, 1);
        assert!(nb.truncated);
        assert!(!nb.entities.iter().any(|entity| entity.id == EntityId(11)));
        assert!(ex
            .cutoff_reasons
            .contains(&CutoffReason::MaxDepthReached(1)));
    }

    #[test]
    fn dfs_prefers_last_pushed_branch_within_same_budget() {
        let planner = BoundedExpansionPlanner::new(ExpansionConstraints {
            max_depth: 2,
            max_entities: 2,
            min_strength: 50,
            follow_reverse: false,
        });

        let seed = EntityId(1);
        let (nb, ex) = planner.plan_dfs(seed, |current| match current {
            EntityId(1) => vec![
                (
                    GraphEdge {
                        from: current,
                        to: EntityId(2),
                        relation: RelationKind::Mentions,
                        strength: 100,
                    },
                    GraphEntity {
                        id: EntityId(2),
                        kind: EntityKind::Concept,
                        label: "first-branch".into(),
                        namespace: NamespaceId::new("ns").unwrap(),
                        memory_id: None,
                    },
                ),
                (
                    GraphEdge {
                        from: current,
                        to: EntityId(3),
                        relation: RelationKind::Mentions,
                        strength: 100,
                    },
                    GraphEntity {
                        id: EntityId(3),
                        kind: EntityKind::Concept,
                        label: "second-branch".into(),
                        namespace: NamespaceId::new("ns").unwrap(),
                        memory_id: None,
                    },
                ),
            ],
            EntityId(3) => vec![(
                GraphEdge {
                    from: current,
                    to: EntityId(4),
                    relation: RelationKind::SharedTopic,
                    strength: 100,
                },
                GraphEntity {
                    id: EntityId(4),
                    kind: EntityKind::Memory,
                    label: "deep-second-branch".into(),
                    namespace: NamespaceId::new("ns").unwrap(),
                    memory_id: Some(MemoryId(4)),
                },
            )],
            _ => Vec::new(),
        });

        assert_eq!(nb.entities.len(), 2);
        assert_eq!(nb.entities[0].id, EntityId(2));
        assert_eq!(nb.entities[1].id, EntityId(3));
        assert!(nb.truncated);
        assert_eq!(ex.expanded_nodes, 2);
        assert!(ex
            .cutoff_reasons
            .contains(&CutoffReason::MaxNodesReached(2)));
        assert!(ex.cutoff_reasons.contains(&CutoffReason::BudgetExhausted));
    }

    #[test]
    fn graph_failure_injection_reports_safe_failure_without_losing_truth() {
        let rebuilder = DerivedGraphRebuilder;
        let inputs = vec![
            EdgeDerivationInput {
                source_memory: MemoryId(21),
                target_memory: Some(MemoryId(22)),
                extracted_concept: None,
                relation: RelationKind::DerivedFrom,
                confidence: 910,
            },
            EdgeDerivationInput {
                source_memory: MemoryId(22),
                target_memory: None,
                extracted_concept: Some("repair_run".into()),
                relation: RelationKind::Mentions,
                confidence: 860,
            },
        ];

        let dropped_report =
            rebuilder.rebuild_with_hooks(&inputs, GraphFailureInjection::DropLastDerivedEdge);
        let aborted_report = rebuilder.rebuild_with_hooks(
            &inputs,
            GraphFailureInjection::AbortAfterAdjacencyProjection,
        );

        assert_eq!(dropped_report.metrics.durable_inputs_seen, 2);
        assert_eq!(dropped_report.metrics.rebuilt_edges, 1);
        assert_eq!(dropped_report.metrics.dropped_edges, 1);
        assert!(!dropped_report.metrics.verification_passed);
        assert!(!dropped_report.succeeded_safely());
        assert_eq!(
            dropped_report.hook_names(),
            vec![
                "snapshot_durable_truth",
                "rebuild_adjacency_projection",
                "rebuild_neighborhood_cache",
                "verify_consistency_snapshot",
            ]
        );
        assert!(dropped_report
            .operator_log
            .contains("failure_injection=drop_last_derived_edge"));
        assert!(dropped_report.operator_log.contains("dropped_edges=1"));
        assert!(dropped_report
            .operator_log
            .contains("verification_passed=false"));

        assert_eq!(aborted_report.metrics.durable_inputs_seen, 2);
        assert_eq!(aborted_report.metrics.rebuilt_edges, 2);
        assert_eq!(aborted_report.metrics.dropped_edges, 0);
        assert!(!aborted_report.metrics.verification_passed);
        assert!(!aborted_report.succeeded_safely());
        assert_eq!(
            aborted_report.hook_names(),
            vec!["snapshot_durable_truth", "rebuild_adjacency_projection",]
        );
        assert!(aborted_report
            .operator_log
            .contains("failure_injection=abort_after_adjacency_projection"));
        assert!(aborted_report
            .operator_log
            .contains("verification_passed=false"));

        let rebuilt_from_truth = inputs
            .iter()
            .cloned()
            .flat_map(|input| rebuilder.rebuild_edges_from_truth(input))
            .collect::<Vec<_>>();
        assert_eq!(rebuilt_from_truth.len(), 2);
    }
}
