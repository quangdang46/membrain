# DATA SCHEMAS

## Schema 1: MemoryItem

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned

## Schema 2: Episode

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned

## Schema 3: Fact

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned

## Schema 4: Summary

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned

## Schema 5: Relation

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned

## Schema 6: ConflictRecord

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned

## Schema 7: Goal

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned

## Schema 8: Skill

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned

## Schema 9: Constraint

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned

## Schema 10: DecayState

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned

## Schema 11: RetentionRule

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned

## Schema 12: ShardDescriptor

### Required fields
- id
- created_at_ms
- updated_at_ms
- namespace
- version

### Optional fields
- workspace_id
- agent_id
- session_id
- task_id
- payload_ref
- tags

### Validation rules
- ids must be globally unique within namespace policy
- created_at_ms must not exceed updated_at_ms
- version must increment on mutation
- payload_ref must be stable or tombstoned

## Schema 13: Engram

### Required fields
- id
- namespace
- created_at_ms
- updated_at_ms
- version
- centroid_ref or centroid_embedding
- member_count

### Optional fields
- parent_engram_id
- total_strength
- last_activated_at_ms
- split_at_ms
- lineage
- policy_flags

### Validation rules
- `id` is the stable cluster handle; centroid or ANN-sidecar ids must not replace it
- `parent_engram_id` must resolve to another engram in the same namespace unless an explicit cross-namespace policy says otherwise
- `member_count` must match durable membership rows after repair or emit a staleness record until rebuilt
- centroid storage may be rewritten during rebuild without minting a new engram identity
- split or merge history must remain explainable through lineage or explicit parent-child linkage

## Schema 14: EngramMember

### Required fields
- engram_id
- memory_id
- namespace
- created_at_ms

### Optional fields
- similarity
- membership_source
- membership_confidence
- version

### Validation rules
- `engram_id` and `memory_id` must resolve to stable canonical handles
- membership rows must not be the sole surviving record of memory identity or lineage
- a memory may have no engram, but if it has one active primary membership the mapping must be inspectable without ANN sidecars
- membership updates must preserve auditability across split, merge, repair, or reassignment flows

## Schema 15: GraphEdge

### Required fields
- src_id
- dst_id
- namespace
- edge_type
- created_at_ms

### Optional fields
- weight
- activation_count
- source_relation_id
- source_conflict_id
- source_lineage_id
- version
- tombstone_reason

### Validation rules
- endpoints must resolve to canonical memory ids unless the edge explicitly targets another canonical graph entity class documented by contract
- `edge_type` must be one of `Associative`, `Temporal`, `Causal`, or `Contradictory` for canonical user-visible semantics
- associative weights may be rebuilt from durable evidence; contradiction, lineage, or policy-bearing meaning must not exist only in this edge row
- duplicate or contradictory edges may coexist only when their semantics remain inspectable rather than silently collapsed
- redacted, tombstoned, or superseded endpoints must remain representable with explicit markers instead of disappearing from repair or inspect surfaces
