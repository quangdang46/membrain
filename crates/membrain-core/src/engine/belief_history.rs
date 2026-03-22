//! Belief-history storage and version timeline model.
//!
//! Tracks how belief states evolve over time without collapsing contradiction
//! history. Each version in a belief chain records a distinct state snapshot
//! tied to a contradiction event, supersession, or authoritative override.
//! Prior states remain queryable so operators can inspect how and why the
//! preferred belief emerged.

use crate::api::NamespaceId;
use crate::engine::contradiction::{ContradictionId, ContradictionKind};
use crate::types::MemoryId;
use std::collections::HashMap;

// ── Belief version ───────────────────────────────────────────────────────────

/// One version entry in a belief chain.
#[derive(Debug, Clone, PartialEq)]
pub struct BeliefVersion {
    /// Memory this version belongs to.
    pub memory_id: MemoryId,
    /// Chain this version belongs to.
    pub chain_id: BeliefChainId,
    /// Chain-local ordering (1-based, increments with each new version).
    pub belief_version: u32,
    /// Content snapshot at this version (compact_text at the time of recording).
    pub content_snapshot: String,
    /// Confidence signal at this version (0..1000).
    pub confidence_signal: u16,
    /// Logical tick when this version was recorded.
    pub recorded_tick: u64,
    /// What triggered this version.
    pub trigger: BeliefVersionTrigger,
    /// Previous version in the chain, if any.
    pub superseded_version: Option<u32>,
    /// Contradiction that caused this version, if any.
    pub contradiction_id: Option<ContradictionId>,
    /// Whether this is the current preferred version.
    pub is_current: bool,
}

impl BeliefVersion {
    /// Returns a stable machine-readable summary for inspect surfaces.
    pub fn inspect_summary(&self) -> String {
        format!(
            "chain={} v={} memory={} tick={} trigger={} current={} confidence={}",
            self.chain_id.0,
            self.belief_version,
            self.memory_id.0,
            self.recorded_tick,
            self.trigger.as_str(),
            self.is_current,
            self.confidence_signal,
        )
    }
}

// ── Belief chain ID ──────────────────────────────────────────────────────────

/// Stable identifier for a belief version chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BeliefChainId(pub u64);

// ── Version trigger ──────────────────────────────────────────────────────────

/// What caused a new belief version to be recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeliefVersionTrigger {
    /// Initial creation of the belief.
    InitialCreation,
    /// A contradiction was detected with another memory.
    ContradictionDetected,
    /// The belief was superseded by a newer version.
    Superseded,
    /// An authoritative source overrode this belief.
    AuthoritativeOverride,
    /// Manual resolution by an operator or agent.
    ManualResolution,
    /// Reconsolidation updated the belief content.
    ReconsolidationUpdate,
}

impl BeliefVersionTrigger {
    /// Returns the stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InitialCreation => "initial_creation",
            Self::ContradictionDetected => "contradiction_detected",
            Self::Superseded => "superseded",
            Self::AuthoritativeOverride => "authoritative_override",
            Self::ManualResolution => "manual_resolution",
            Self::ReconsolidationUpdate => "reconsolidation_update",
        }
    }

    /// Whether this trigger implies the belief changed state meaningfully.
    pub const fn is_state_change(self) -> bool {
        !matches!(self, Self::InitialCreation)
    }
}

// ── Belief chain ─────────────────────────────────────────────────────────────

/// A version chain grouping related belief states.
#[derive(Debug, Clone, PartialEq)]
pub struct BeliefChain {
    /// Stable chain identifier.
    pub chain_id: BeliefChainId,
    /// Namespace scope.
    pub namespace: NamespaceId,
    /// The primary memory this chain tracks.
    pub primary_memory_id: MemoryId,
    /// All versions in chronological order.
    pub versions: Vec<BeliefVersion>,
    /// Index of the current preferred version.
    pub current_version_index: usize,
}

impl BeliefChain {
    /// Returns the current preferred version.
    pub fn current_version(&self) -> &BeliefVersion {
        &self.versions[self.current_version_index]
    }

    /// Returns a specific version by chain-local number.
    pub fn version_at(&self, belief_version: u32) -> Option<&BeliefVersion> {
        self.versions
            .iter()
            .find(|v| v.belief_version == belief_version)
    }

    /// Returns all versions in chronological order.
    pub fn timeline(&self) -> &[BeliefVersion] {
        &self.versions
    }

    /// Returns how many times the preferred version changed.
    pub fn resolution_count(&self) -> usize {
        self.versions
            .iter()
            .filter(|v| v.trigger.is_state_change())
            .count()
    }

    /// Returns whether the chain has unresolved contradictions.
    pub fn has_unresolved(&self) -> bool {
        self.versions
            .iter()
            .any(|v| v.trigger == BeliefVersionTrigger::ContradictionDetected && v.is_current)
    }
}

// ── Timeline query ───────────────────────────────────────────────────────────

/// Query result for a belief timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct BeliefTimelineView {
    /// The chain queried.
    pub chain_id: BeliefChainId,
    /// Versions in chronological order.
    pub versions: Vec<BeliefVersionSummary>,
    /// Total number of state changes.
    pub resolution_count: usize,
    /// Whether there are unresolved contradictions.
    pub has_unresolved: bool,
}

/// Lightweight version summary for timeline views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeliefVersionSummary {
    /// Chain-local version number.
    pub belief_version: u32,
    /// Memory ID at this version.
    pub memory_id: MemoryId,
    /// Trigger that caused this version.
    pub trigger: &'static str,
    /// Tick when recorded.
    pub recorded_tick: u64,
    /// Whether this is the current preferred version.
    pub is_current: bool,
    /// Confidence signal (0..1000).
    pub confidence_signal: u16,
    /// First N chars of content snapshot for readability.
    pub content_preview: String,
}

// ── Resolution view ──────────────────────────────────────────────────────────

/// User-facing resolution view showing why the current belief was chosen.
#[derive(Debug, Clone, PartialEq)]
pub struct BeliefResolutionView {
    /// The chain this view belongs to.
    pub chain_id: BeliefChainId,
    /// Current preferred belief version.
    pub current_version: u32,
    /// Memory ID of the current belief.
    pub current_memory_id: MemoryId,
    /// Content snapshot of the current belief.
    pub current_content: String,
    /// Confidence signal (0..1000).
    pub confidence_signal: u16,
    /// How many times the preferred version changed.
    pub resolution_count: usize,
    /// How many versions exist total.
    pub total_versions: usize,
    /// The version this one superseded (if any).
    pub superseded_version: Option<u32>,
    /// Trigger that created this version.
    pub trigger: &'static str,
    /// Tick when this version was recorded.
    pub recorded_tick: u64,
    /// Whether there are prior versions to inspect.
    pub has_history: bool,
}

/// Historical explain output for a specific version.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoricalExplain {
    /// Version number in the chain.
    pub belief_version: u32,
    /// Memory ID at this version.
    pub memory_id: MemoryId,
    /// Content snapshot.
    pub content_snapshot: String,
    /// Trigger that created this version.
    pub trigger: &'static str,
    /// Tick when recorded.
    pub recorded_tick: u64,
    /// Confidence at this version.
    pub confidence_signal: u16,
    /// Whether this was the preferred version at the time.
    pub was_current: bool,
    /// What superseded this version (if anything).
    pub superseded_by: Option<u32>,
    /// How this version differs from the previous.
    pub diff_from_previous: Option<VersionDiff>,
}

/// Diff between two consecutive versions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionDiff {
    /// Previous version number.
    pub previous_version: u32,
    /// Whether the content changed.
    pub content_changed: bool,
    /// Whether the confidence changed.
    pub confidence_changed: bool,
    /// Whether the memory identity changed.
    pub memory_changed: bool,
    /// Trigger that caused the change.
    pub change_trigger: &'static str,
}

// ── Belief history engine ────────────────────────────────────────────────────

/// Canonical belief history engine owned by the core crate.
#[derive(Debug, Clone, PartialEq)]
pub struct BeliefHistoryEngine {
    chains: HashMap<BeliefChainId, BeliefChain>,
    /// Per-memory lookup: memory_id → chain_id
    memory_to_chain: HashMap<MemoryId, BeliefChainId>,
    next_chain_id: u64,
}

impl BeliefHistoryEngine {
    /// Builds a new empty belief history engine.
    pub fn new() -> Self {
        Self {
            chains: HashMap::new(),
            memory_to_chain: HashMap::new(),
            next_chain_id: 1,
        }
    }

    /// Returns the stable component identifier.
    pub const fn component_name(&self) -> &'static str {
        "engine.belief_history"
    }

    /// Allocates the next chain ID.
    fn allocate_chain_id(&mut self) -> BeliefChainId {
        let id = BeliefChainId(self.next_chain_id);
        self.next_chain_id += 1;
        id
    }

    /// Creates a new belief chain for a memory.
    pub fn create_chain(
        &mut self,
        namespace: NamespaceId,
        memory_id: MemoryId,
        content_snapshot: String,
        confidence_signal: u16,
        tick: u64,
    ) -> BeliefChainId {
        let chain_id = self.allocate_chain_id();
        let version = BeliefVersion {
            memory_id,
            chain_id,
            belief_version: 1,
            content_snapshot,
            confidence_signal,
            recorded_tick: tick,
            trigger: BeliefVersionTrigger::InitialCreation,
            superseded_version: None,
            contradiction_id: None,
            is_current: true,
        };
        let chain = BeliefChain {
            chain_id,
            namespace,
            primary_memory_id: memory_id,
            versions: vec![version],
            current_version_index: 0,
        };
        self.chains.insert(chain_id, chain);
        self.memory_to_chain.insert(memory_id, chain_id);
        chain_id
    }

    /// Records a contradiction-triggered version in an existing chain.
    pub fn record_contradiction(
        &mut self,
        chain_id: BeliefChainId,
        memory_id: MemoryId,
        contradiction_id: ContradictionId,
        contradiction_kind: ContradictionKind,
        content_snapshot: String,
        confidence_signal: u16,
        tick: u64,
    ) -> Result<u32, BeliefHistoryError> {
        let chain = self
            .chains
            .get_mut(&chain_id)
            .ok_or(BeliefHistoryError::ChainNotFound)?;

        let new_version_num = chain.versions.len() as u32 + 1;
        let current_version = chain.current_version().belief_version;

        // Determine trigger from contradiction kind
        let trigger = match contradiction_kind {
            ContradictionKind::Supersession | ContradictionKind::Revision => {
                BeliefVersionTrigger::Superseded
            }
            ContradictionKind::AuthoritativeOverride => BeliefVersionTrigger::AuthoritativeOverride,
            _ => BeliefVersionTrigger::ContradictionDetected,
        };

        let version = BeliefVersion {
            memory_id,
            chain_id,
            belief_version: new_version_num,
            content_snapshot,
            confidence_signal,
            recorded_tick: tick,
            trigger,
            superseded_version: Some(current_version),
            contradiction_id: Some(contradiction_id),
            is_current: true,
        };

        // Mark old current as no longer current
        chain.versions[chain.current_version_index].is_current = false;
        chain.versions.push(version);
        chain.current_version_index = chain.versions.len() - 1;

        self.memory_to_chain.insert(memory_id, chain_id);
        Ok(new_version_num)
    }

    /// Records a manual resolution version.
    pub fn record_manual_resolution(
        &mut self,
        chain_id: BeliefChainId,
        memory_id: MemoryId,
        content_snapshot: String,
        confidence_signal: u16,
        tick: u64,
    ) -> Result<u32, BeliefHistoryError> {
        let chain = self
            .chains
            .get_mut(&chain_id)
            .ok_or(BeliefHistoryError::ChainNotFound)?;

        let new_version_num = chain.versions.len() as u32 + 1;
        let current_version = chain.current_version().belief_version;

        let version = BeliefVersion {
            memory_id,
            chain_id,
            belief_version: new_version_num,
            content_snapshot,
            confidence_signal,
            recorded_tick: tick,
            trigger: BeliefVersionTrigger::ManualResolution,
            superseded_version: Some(current_version),
            contradiction_id: None,
            is_current: true,
        };

        chain.versions[chain.current_version_index].is_current = false;
        chain.versions.push(version);
        chain.current_version_index = chain.versions.len() - 1;

        Ok(new_version_num)
    }

    /// Returns the timeline view for a chain.
    pub fn timeline(
        &self,
        chain_id: BeliefChainId,
    ) -> Result<BeliefTimelineView, BeliefHistoryError> {
        let chain = self
            .chains
            .get(&chain_id)
            .ok_or(BeliefHistoryError::ChainNotFound)?;

        let versions = chain
            .versions
            .iter()
            .map(|v| BeliefVersionSummary {
                belief_version: v.belief_version,
                memory_id: v.memory_id,
                trigger: v.trigger.as_str(),
                recorded_tick: v.recorded_tick,
                is_current: v.is_current,
                confidence_signal: v.confidence_signal,
                content_preview: v.content_snapshot.chars().take(80).collect(),
            })
            .collect();

        Ok(BeliefTimelineView {
            chain_id,
            versions,
            resolution_count: chain.resolution_count(),
            has_unresolved: chain.has_unresolved(),
        })
    }

    /// Returns the chain for a given memory.
    pub fn chain_for_memory(&self, memory_id: &MemoryId) -> Option<&BeliefChain> {
        self.memory_to_chain
            .get(memory_id)
            .and_then(|cid| self.chains.get(cid))
    }

    /// Returns the chain by ID.
    pub fn chain(&self, chain_id: &BeliefChainId) -> Option<&BeliefChain> {
        self.chains.get(chain_id)
    }

    /// Returns the total number of chains.
    pub fn chain_count(&self) -> usize {
        self.chains.len()
    }

    /// Returns the total number of versions across all chains.
    pub fn total_version_count(&self) -> usize {
        self.chains.values().map(|c| c.versions.len()).sum()
    }

    /// Returns a user-facing resolution view for the current belief.
    pub fn resolve_view(
        &self,
        chain_id: BeliefChainId,
    ) -> Result<BeliefResolutionView, BeliefHistoryError> {
        let chain = self
            .chains
            .get(&chain_id)
            .ok_or(BeliefHistoryError::ChainNotFound)?;

        let current = chain.current_version();
        let superseded_by_current = current.superseded_version;

        Ok(BeliefResolutionView {
            chain_id,
            current_version: current.belief_version,
            current_memory_id: current.memory_id,
            current_content: current.content_snapshot.clone(),
            confidence_signal: current.confidence_signal,
            resolution_count: chain.resolution_count(),
            total_versions: chain.versions.len(),
            superseded_version: superseded_by_current,
            trigger: current.trigger.as_str(),
            recorded_tick: current.recorded_tick,
            has_history: chain.versions.len() > 1,
        })
    }

    /// Returns historical explain outputs for all versions in a chain.
    pub fn historical_explain(
        &self,
        chain_id: BeliefChainId,
    ) -> Result<Vec<HistoricalExplain>, BeliefHistoryError> {
        let chain = self
            .chains
            .get(&chain_id)
            .ok_or(BeliefHistoryError::ChainNotFound)?;

        let current_idx = chain.current_version_index;
        let explains: Vec<HistoricalExplain> = chain
            .versions
            .iter()
            .enumerate()
            .map(|(idx, v)| {
                let superseded_by = chain
                    .versions
                    .iter()
                    .find(|later| later.superseded_version == Some(v.belief_version))
                    .map(|later| later.belief_version);

                let diff_from_previous = if idx > 0 {
                    let prev = &chain.versions[idx - 1];
                    Some(VersionDiff {
                        previous_version: prev.belief_version,
                        content_changed: prev.content_snapshot != v.content_snapshot,
                        confidence_changed: prev.confidence_signal != v.confidence_signal,
                        memory_changed: prev.memory_id != v.memory_id,
                        change_trigger: v.trigger.as_str(),
                    })
                } else {
                    None
                };

                HistoricalExplain {
                    belief_version: v.belief_version,
                    memory_id: v.memory_id,
                    content_snapshot: v.content_snapshot.clone(),
                    trigger: v.trigger.as_str(),
                    recorded_tick: v.recorded_tick,
                    confidence_signal: v.confidence_signal,
                    was_current: idx == current_idx,
                    superseded_by,
                    diff_from_previous,
                }
            })
            .collect();

        Ok(explains)
    }

    /// Returns a user-facing resolution view by memory ID.
    pub fn resolve_view_for_memory(
        &self,
        memory_id: &MemoryId,
    ) -> Result<BeliefResolutionView, BeliefHistoryError> {
        let chain_id = self
            .memory_to_chain
            .get(memory_id)
            .ok_or(BeliefHistoryError::ChainNotFound)?;
        self.resolve_view(*chain_id)
    }
}

/// Errors from belief history operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeliefHistoryError {
    /// The chain ID was not found.
    ChainNotFound,
    /// The version number was not found in the chain.
    VersionNotFound,
}

impl Default for BeliefHistoryEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::contradiction::ContradictionKind;

    fn ns(s: &str) -> NamespaceId {
        NamespaceId::new(s).unwrap()
    }

    #[test]
    fn create_chain_estimates_initial_version() {
        let mut engine = BeliefHistoryEngine::new();
        let cid = engine.create_chain(ns("test"), MemoryId(1), "initial content".into(), 500, 100);

        let chain = engine.chain(&cid).unwrap();
        assert_eq!(chain.versions.len(), 1);
        let v = &chain.versions[0];
        assert_eq!(v.belief_version, 1);
        assert_eq!(v.memory_id, MemoryId(1));
        assert_eq!(v.trigger, BeliefVersionTrigger::InitialCreation);
        assert!(v.is_current);
        assert_eq!(v.confidence_signal, 500);
    }

    #[test]
    fn contradiction_creates_new_version() {
        let mut engine = BeliefHistoryEngine::new();
        let cid = engine.create_chain(
            ns("test"),
            MemoryId(1),
            "server on port 8080".into(),
            700,
            100,
        );

        let v2 = engine
            .record_contradiction(
                cid,
                MemoryId(2),
                ContradictionId(1),
                ContradictionKind::Revision,
                "server on port 9090".into(),
                850,
                200,
            )
            .unwrap();

        assert_eq!(v2, 2);

        let chain = engine.chain(&cid).unwrap();
        assert_eq!(chain.versions.len(), 2);

        // Version 1 should no longer be current
        assert!(!chain.versions[0].is_current);
        assert_eq!(
            chain.versions[0].trigger,
            BeliefVersionTrigger::InitialCreation
        );

        // Version 2 should be current
        let v = &chain.versions[1];
        assert!(v.is_current);
        assert_eq!(v.belief_version, 2);
        assert_eq!(v.memory_id, MemoryId(2));
        assert_eq!(v.trigger, BeliefVersionTrigger::Superseded);
        assert_eq!(v.superseded_version, Some(1));
        assert_eq!(v.contradiction_id, Some(ContradictionId(1)));
    }

    #[test]
    fn timeline_shows_chronological_versions() {
        let mut engine = BeliefHistoryEngine::new();
        let cid = engine.create_chain(ns("test"), MemoryId(1), "v1 content".into(), 500, 100);

        engine
            .record_contradiction(
                cid,
                MemoryId(2),
                ContradictionId(1),
                ContradictionKind::Revision,
                "v2 content".into(),
                700,
                200,
            )
            .unwrap();

        engine
            .record_manual_resolution(cid, MemoryId(3), "v3 resolved".into(), 900, 300)
            .unwrap();

        let view = engine.timeline(cid).unwrap();
        assert_eq!(view.versions.len(), 3);
        assert_eq!(view.resolution_count, 2); // 2 state changes (not counting initial)
        assert!(!view.has_unresolved);

        // Chronological order
        assert_eq!(view.versions[0].belief_version, 1);
        assert_eq!(view.versions[0].trigger, "initial_creation");
        assert_eq!(view.versions[1].belief_version, 2);
        assert_eq!(view.versions[1].trigger, "superseded");
        assert_eq!(view.versions[2].belief_version, 3);
        assert_eq!(view.versions[2].trigger, "manual_resolution");
        assert!(view.versions[2].is_current);
    }

    #[test]
    fn current_version_tracks_across_resolutions() {
        let mut engine = BeliefHistoryEngine::new();
        let cid = engine.create_chain(ns("test"), MemoryId(1), "original".into(), 500, 100);

        engine
            .record_contradiction(
                cid,
                MemoryId(2),
                ContradictionId(1),
                ContradictionKind::Supersession,
                "updated".into(),
                800,
                200,
            )
            .unwrap();

        let chain = engine.chain(&cid).unwrap();
        let current = chain.current_version();
        assert_eq!(current.memory_id, MemoryId(2));
        assert_eq!(current.belief_version, 2);
        assert_eq!(current.content_snapshot, "updated");
    }

    #[test]
    fn version_at_finds_specific_version() {
        let mut engine = BeliefHistoryEngine::new();
        let cid = engine.create_chain(ns("test"), MemoryId(1), "v1".into(), 500, 100);

        engine
            .record_contradiction(
                cid,
                MemoryId(2),
                ContradictionId(1),
                ContradictionKind::Coexistence,
                "v2".into(),
                600,
                200,
            )
            .unwrap();

        let chain = engine.chain(&cid).unwrap();
        let v1 = chain.version_at(1).unwrap();
        assert_eq!(v1.content_snapshot, "v1");
        assert!(!v1.is_current);

        let v2 = chain.version_at(2).unwrap();
        assert_eq!(v2.content_snapshot, "v2");
        assert!(v2.is_current);

        assert!(chain.version_at(3).is_none());
    }

    #[test]
    fn chain_for_memory_resolves_by_memory_id() {
        let mut engine = BeliefHistoryEngine::new();
        let cid = engine.create_chain(ns("test"), MemoryId(42), "content".into(), 500, 100);

        let chain = engine.chain_for_memory(&MemoryId(42)).unwrap();
        assert_eq!(chain.chain_id, cid);
        assert!(engine.chain_for_memory(&MemoryId(99)).is_none());
    }

    #[test]
    fn record_contradiction_on_missing_chain_returns_error() {
        let mut engine = BeliefHistoryEngine::new();
        let err = engine
            .record_contradiction(
                BeliefChainId(999),
                MemoryId(1),
                ContradictionId(1),
                ContradictionKind::Revision,
                "content".into(),
                500,
                100,
            )
            .unwrap_err();

        assert_eq!(err, BeliefHistoryError::ChainNotFound);
    }

    #[test]
    fn multiple_chains_are_independent() {
        let mut engine = BeliefHistoryEngine::new();
        let cid1 = engine.create_chain(ns("test"), MemoryId(1), "chain1".into(), 500, 100);
        let cid2 = engine.create_chain(ns("test"), MemoryId(2), "chain2".into(), 600, 150);

        assert_ne!(cid1, cid2);
        assert_eq!(engine.chain_count(), 2);
        assert_eq!(engine.total_version_count(), 2);

        engine
            .record_contradiction(
                cid1,
                MemoryId(3),
                ContradictionId(1),
                ContradictionKind::Revision,
                "chain1-v2".into(),
                700,
                200,
            )
            .unwrap();

        assert_eq!(engine.total_version_count(), 3);
        // Chain 2 should be unaffected
        let chain2 = engine.chain(&cid2).unwrap();
        assert_eq!(chain2.versions.len(), 1);
    }

    #[test]
    fn inspect_summary_is_deterministic() {
        let mut engine = BeliefHistoryEngine::new();
        let cid = engine.create_chain(ns("test"), MemoryId(1), "content".into(), 750, 100);

        let chain = engine.chain(&cid).unwrap();
        let summary = chain.versions[0].inspect_summary();
        assert!(summary.contains("chain=1"));
        assert!(summary.contains("v=1"));
        assert!(summary.contains("memory=1"));
        assert!(summary.contains("trigger=initial_creation"));
        assert!(summary.contains("current=true"));
        assert!(summary.contains("confidence=750"));
    }

    #[test]
    fn timeline_content_preview_is_truncated() {
        let mut engine = BeliefHistoryEngine::new();
        let long_content = "a".repeat(200);
        let cid = engine.create_chain(ns("test"), MemoryId(1), long_content, 500, 100);

        let view = engine.timeline(cid).unwrap();
        assert_eq!(view.versions[0].content_preview.len(), 80);
    }

    #[test]
    fn trigger_is_state_change_except_initial() {
        assert!(!BeliefVersionTrigger::InitialCreation.is_state_change());
        assert!(BeliefVersionTrigger::ContradictionDetected.is_state_change());
        assert!(BeliefVersionTrigger::Superseded.is_state_change());
        assert!(BeliefVersionTrigger::AuthoritativeOverride.is_state_change());
        assert!(BeliefVersionTrigger::ManualResolution.is_state_change());
        assert!(BeliefVersionTrigger::ReconsolidationUpdate.is_state_change());
    }

    #[test]
    fn trigger_as_str_stable_names() {
        assert_eq!(
            BeliefVersionTrigger::InitialCreation.as_str(),
            "initial_creation"
        );
        assert_eq!(
            BeliefVersionTrigger::ContradictionDetected.as_str(),
            "contradiction_detected"
        );
        assert_eq!(BeliefVersionTrigger::Superseded.as_str(), "superseded");
        assert_eq!(
            BeliefVersionTrigger::AuthoritativeOverride.as_str(),
            "authoritative_override"
        );
        assert_eq!(
            BeliefVersionTrigger::ManualResolution.as_str(),
            "manual_resolution"
        );
        assert_eq!(
            BeliefVersionTrigger::ReconsolidationUpdate.as_str(),
            "reconsolidation_update"
        );
    }

    // ── Resolution view tests ─────────────────────────────────────────────

    #[test]
    fn resolve_view_shows_current_belief_with_context() {
        let mut engine = BeliefHistoryEngine::new();
        let cid = engine.create_chain(
            ns("test"),
            MemoryId(1),
            "original server config".into(),
            600,
            100,
        );

        engine
            .record_contradiction(
                cid,
                MemoryId(2),
                ContradictionId(1),
                ContradictionKind::Supersession,
                "updated server config".into(),
                850,
                200,
            )
            .unwrap();

        let view = engine.resolve_view(cid).unwrap();
        assert_eq!(view.chain_id, cid);
        assert_eq!(view.current_version, 2);
        assert_eq!(view.current_memory_id, MemoryId(2));
        assert_eq!(view.current_content, "updated server config");
        assert_eq!(view.confidence_signal, 850);
        assert_eq!(view.resolution_count, 1);
        assert_eq!(view.total_versions, 2);
        assert_eq!(view.superseded_version, Some(1));
        assert_eq!(view.trigger, "superseded");
        assert!(view.has_history);
    }

    #[test]
    fn resolve_view_for_initial_chain_has_no_history() {
        let mut engine = BeliefHistoryEngine::new();
        let cid = engine.create_chain(ns("test"), MemoryId(1), "only version".into(), 500, 100);

        let view = engine.resolve_view(cid).unwrap();
        assert_eq!(view.current_version, 1);
        assert_eq!(view.resolution_count, 0);
        assert_eq!(view.total_versions, 1);
        assert!(!view.has_history);
        assert_eq!(view.superseded_version, None);
    }

    #[test]
    fn resolve_view_for_memory_resolves_by_id() {
        let mut engine = BeliefHistoryEngine::new();
        engine.create_chain(ns("test"), MemoryId(42), "content".into(), 700, 100);

        let view = engine.resolve_view_for_memory(&MemoryId(42)).unwrap();
        assert_eq!(view.current_memory_id, MemoryId(42));
        assert_eq!(view.confidence_signal, 700);
    }

    #[test]
    fn resolve_view_for_missing_memory_returns_error() {
        let engine = BeliefHistoryEngine::new();
        let err = engine.resolve_view_for_memory(&MemoryId(99)).unwrap_err();
        assert_eq!(err, BeliefHistoryError::ChainNotFound);
    }

    // ── Historical explain tests ──────────────────────────────────────────

    #[test]
    fn historical_explain_shows_all_versions_with_context() {
        let mut engine = BeliefHistoryEngine::new();
        let cid = engine.create_chain(
            ns("test"),
            MemoryId(1),
            "server on port 8080".into(),
            600,
            100,
        );

        engine
            .record_contradiction(
                cid,
                MemoryId(2),
                ContradictionId(1),
                ContradictionKind::Revision,
                "server on port 9090".into(),
                800,
                200,
            )
            .unwrap();

        engine
            .record_manual_resolution(cid, MemoryId(3), "server on port 443".into(), 950, 300)
            .unwrap();

        let explains = engine.historical_explain(cid).unwrap();
        assert_eq!(explains.len(), 3);

        // Version 1: initial creation
        let v1 = &explains[0];
        assert_eq!(v1.belief_version, 1);
        assert_eq!(v1.memory_id, MemoryId(1));
        assert_eq!(v1.trigger, "initial_creation");
        assert!(!v1.was_current);
        assert_eq!(v1.superseded_by, Some(2));
        assert!(v1.diff_from_previous.is_none()); // first version has no previous

        // Version 2: contradiction detected
        let v2 = &explains[1];
        assert_eq!(v2.belief_version, 2);
        assert_eq!(v2.memory_id, MemoryId(2));
        assert_eq!(v2.trigger, "superseded");
        assert!(!v2.was_current);
        assert_eq!(v2.superseded_by, Some(3));

        let diff2 = v2.diff_from_previous.as_ref().unwrap();
        assert_eq!(diff2.previous_version, 1);
        assert!(diff2.content_changed);
        assert!(diff2.confidence_changed);
        assert!(diff2.memory_changed);
        assert_eq!(diff2.change_trigger, "superseded");

        // Version 3: manual resolution
        let v3 = &explains[2];
        assert_eq!(v3.belief_version, 3);
        assert!(v3.was_current);
        assert_eq!(v3.superseded_by, None);

        let diff3 = v3.diff_from_previous.as_ref().unwrap();
        assert_eq!(diff3.previous_version, 2);
        assert!(diff3.content_changed);
    }

    #[test]
    fn historical_explain_on_empty_chain_fails() {
        let engine = BeliefHistoryEngine::new();
        let err = engine.historical_explain(BeliefChainId(999)).unwrap_err();
        assert_eq!(err, BeliefHistoryError::ChainNotFound);
    }

    #[test]
    fn historical_explain_single_version_has_no_diff() {
        let mut engine = BeliefHistoryEngine::new();
        let cid = engine.create_chain(ns("test"), MemoryId(1), "single".into(), 500, 100);

        let explains = engine.historical_explain(cid).unwrap();
        assert_eq!(explains.len(), 1);
        assert!(explains[0].diff_from_previous.is_none());
        assert!(explains[0].was_current);
        assert_eq!(explains[0].superseded_by, None);
    }
}
