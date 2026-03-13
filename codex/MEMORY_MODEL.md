# MEMORY MODEL

## Memory taxonomy
- Event
- Episode
- Fact
- Relation
- Summary
- Goal
- Skill
- Constraint
- Hypothesis
- ConflictRecord
- PolicyArtifact
- Observation
- ToolOutcome
- UserPreference
- SessionMarker

## Core fields
Every memory item should contain:
- id
- memory_type
- namespace
- workspace_id
- agent_id
- session_id
- task_id
- created_at
- updated_at
- source_kind
- source_ref
- authoritativeness
- content_ref
- compact_text
- fingerprint
- tier
- salience
- confidence
- utility_estimate
- recall_count
- last_access_at
- decay_state
- retention_class
- policy_flags
- lineage
- version
- tags
- entity_refs
- relation_refs

## Example schema
```rust
pub enum MemoryTier { Tier1, Tier2, Tier3 }

pub enum MemoryType {
    Event,
    Episode,
    Fact,
    Relation,
    Summary,
    Goal,
    Skill,
    Constraint,
    Hypothesis,
    ConflictRecord,
    PolicyArtifact,
    Observation,
    ToolOutcome,
    UserPreference,
    SessionMarker,
}

pub struct MemoryItem {
    pub id: MemoryId,
    pub memory_type: MemoryType,
    pub namespace: String,
    pub workspace_id: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub tier: MemoryTier,
    pub salience: f32,
    pub confidence: f32,
    pub utility_estimate: f32,
    pub recall_count: u32,
    pub last_access_at_ms: Option<i64>,
    pub retention_class: RetentionClass,
    pub decay_state: DecayState,
    pub fingerprint: u64,
    pub lineage: Vec<MemoryId>,
    pub entity_refs: Vec<EntityRef>,
    pub relation_refs: Vec<RelationRef>,
    pub tags: Vec<String>,
}
```
