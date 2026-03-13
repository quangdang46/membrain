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
