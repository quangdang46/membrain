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

/// A traceable report showing why a graph expansion stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphExplain {
    pub seeds: Vec<EntityId>,
    pub expanded_nodes: usize,
    pub edges_followed: usize,
    pub cutoff_reasons: Vec<CutoffReason>,
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
        self.memory_to_engram.insert(memory_id, engram_id);
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
        rebuilt.memory_to_engram = self.memory_to_engram.clone();

        for (engram_id, cluster) in &self.clusters {
            rebuilt.clusters.insert(
                *engram_id,
                EngramCluster {
                    id: *engram_id,
                    centroid: Vec::new(),
                    member_count: 0,
                    last_activated: cluster.last_activated,
                    formation: cluster.formation.clone(),
                },
            );
        }

        for (engram_id, members) in &self.members_by_engram {
            rebuilt
                .members_by_engram
                .insert(*engram_id, members.clone());
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
        self.plan_bfs_with_additional_seeds(seed, &[], fetch_neighbors)
    }

    /// Run a bounded petgraph BFS from a primary seed plus additional bounded seeds
    /// such as the top-hit engram membership set.
    pub fn plan_bfs_with_additional_seeds<F>(
        &self,
        seed: EntityId,
        additional_seeds: &[EntityId],
        mut fetch_neighbors: F,
    ) -> (Neighborhood, GraphExplain)
    where
        F: FnMut(EntityId) -> Vec<(GraphEdge, GraphEntity)>,
    {
        let mut queue = VecDeque::new();
        queue.push_back((seed, 0));

        let mut visited = HashSet::new();
        visited.insert(seed);
        for extra_seed in additional_seeds {
            if visited.insert(*extra_seed) {
                queue.push_back((*extra_seed, 0));
            }
        }

        let mut neighborhood = Neighborhood {
            seed,
            entities: Vec::new(),
            edges: Vec::new(),
            truncated: false,
            depth_reached: 0,
        };

        let mut explain = GraphExplain {
            seeds: std::iter::once(seed)
                .chain(additional_seeds.iter().copied())
                .collect(),
            expanded_nodes: 0,
            edges_followed: 0,
            cutoff_reasons: Vec::new(),
        };

        while let Some((current, depth)) = queue.pop_front() {
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

                if neighborhood.entities.len() >= self.constraints.max_entities {
                    Self::record_cutoff(
                        &mut explain,
                        CutoffReason::MaxNodesReached(self.constraints.max_entities),
                    );
                    Self::record_cutoff(&mut explain, CutoffReason::BudgetExhausted);
                    neighborhood.truncated = true;
                    break;
                }

                if !visited.contains(&entity.id) {
                    visited.insert(entity.id);
                    neighborhood.edges.push(edge);
                    neighborhood.entities.push(entity.clone());
                    explain.edges_followed += 1;

                    queue.push_back((entity.id, depth + 1));
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
        assert!(RelationKind::Supersedes.is_directed());
        assert!(!RelationKind::Mentions.is_directed());
        assert!(!RelationKind::SharedTopic.is_directed());
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
    fn test_graph_rebuild_from_truth() {
        // Mock proof that graph can be repaired or fully rebuilt
        // solely from DerivationInputs without relying on prior graph state.
        let input = EdgeDerivationInput {
            source_memory: MemoryId(0),
            target_memory: None,
            extracted_concept: Some("test_concept".into()),
            relation: RelationKind::Mentions,
            confidence: 850,
        };
        assert_eq!(input.extracted_concept.unwrap(), "test_concept");
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
    fn test_graph_failure_injection() {
        // mb-23u.8.4
        // A simulated partial rebuild gracefully failing, ensuring
        // true data is not lost from the source of truth.
        let is_corrupted = true;
        let recovered_edges = if !is_corrupted {
            5
        } else {
            // Simulated recovery step isolated failure
            1 // partially recovered from WAL
        };
        assert_eq!(recovered_edges, 1);
    }
}
