# STATE MACHINES

This document defines lifecycle states and transitions for major memory objects and subsystem controllers.

## 1. Event

### States
- created
- indexed
- recalled
- reinforced
- decayed
- consolidated
- demoted
- archived
- deleted

### Typical transitions
created -> indexed
indexed -> recalled
recalled -> reinforced
indexed -> decayed
decayed -> demoted
demoted -> archived
archived -> deleted

### Guards
- policy pinning
- namespace access control
- unresolved contradiction state
- lineage preservation requirement
- repair job lock

### Failure handling
If transition for Event fails, write a transition error event, preserve the prior state, and enqueue a repairable task.

## 2. Episode

### States
- created
- indexed
- recalled
- reinforced
- decayed
- consolidated
- demoted
- archived
- deleted

### Typical transitions
created -> indexed
indexed -> recalled
recalled -> reinforced
indexed -> decayed
decayed -> demoted
demoted -> archived
archived -> deleted

### Guards
- policy pinning
- namespace access control
- unresolved contradiction state
- lineage preservation requirement
- repair job lock

### Failure handling
If transition for Episode fails, write a transition error event, preserve the prior state, and enqueue a repairable task.

## 3. Fact

### States
- created
- indexed
- recalled
- reinforced
- decayed
- consolidated
- demoted
- archived
- deleted

### Typical transitions
created -> indexed
indexed -> recalled
recalled -> reinforced
indexed -> decayed
decayed -> demoted
demoted -> archived
archived -> deleted

### Guards
- policy pinning
- namespace access control
- unresolved contradiction state
- lineage preservation requirement
- repair job lock

### Failure handling
If transition for Fact fails, write a transition error event, preserve the prior state, and enqueue a repairable task.

## 4. Summary

### States
- created
- indexed
- recalled
- reinforced
- decayed
- consolidated
- demoted
- archived
- deleted

### Typical transitions
created -> indexed
indexed -> recalled
recalled -> reinforced
indexed -> decayed
decayed -> demoted
demoted -> archived
archived -> deleted

### Guards
- policy pinning
- namespace access control
- unresolved contradiction state
- lineage preservation requirement
- repair job lock

### Failure handling
If transition for Summary fails, write a transition error event, preserve the prior state, and enqueue a repairable task.

## 5. Goal

### States
- created
- indexed
- recalled
- reinforced
- decayed
- consolidated
- demoted
- archived
- deleted

### Typical transitions
created -> indexed
indexed -> recalled
recalled -> reinforced
indexed -> decayed
decayed -> demoted
demoted -> archived
archived -> deleted

### Guards
- policy pinning
- namespace access control
- unresolved contradiction state
- lineage preservation requirement
- repair job lock

### Failure handling
If transition for Goal fails, write a transition error event, preserve the prior state, and enqueue a repairable task.

## 6. Skill

### States
- created
- indexed
- recalled
- reinforced
- decayed
- consolidated
- demoted
- archived
- deleted

### Typical transitions
created -> indexed
indexed -> recalled
recalled -> reinforced
indexed -> decayed
decayed -> demoted
demoted -> archived
archived -> deleted

### Guards
- policy pinning
- namespace access control
- unresolved contradiction state
- lineage preservation requirement
- repair job lock

### Failure handling
If transition for Skill fails, write a transition error event, preserve the prior state, and enqueue a repairable task.

## 7. Constraint

### States
- created
- indexed
- recalled
- reinforced
- decayed
- consolidated
- demoted
- archived
- deleted

### Typical transitions
created -> indexed
indexed -> recalled
recalled -> reinforced
indexed -> decayed
decayed -> demoted
demoted -> archived
archived -> deleted

### Guards
- policy pinning
- namespace access control
- unresolved contradiction state
- lineage preservation requirement
- repair job lock

### Failure handling
If transition for Constraint fails, write a transition error event, preserve the prior state, and enqueue a repairable task.

## 8. ConflictRecord

### States
- created
- indexed
- recalled
- reinforced
- decayed
- consolidated
- demoted
- archived
- deleted

### Typical transitions
created -> indexed
indexed -> recalled
recalled -> reinforced
indexed -> decayed
decayed -> demoted
demoted -> archived
archived -> deleted

### Guards
- policy pinning
- namespace access control
- unresolved contradiction state
- lineage preservation requirement
- repair job lock

### Failure handling
If transition for ConflictRecord fails, write a transition error event, preserve the prior state, and enqueue a repairable task.

## 9. Relation

### States
- created
- indexed
- recalled
- reinforced
- decayed
- consolidated
- demoted
- archived
- deleted

### Typical transitions
created -> indexed
indexed -> recalled
recalled -> reinforced
indexed -> decayed
decayed -> demoted
demoted -> archived
archived -> deleted

### Guards
- policy pinning
- namespace access control
- unresolved contradiction state
- lineage preservation requirement
- repair job lock

### Failure handling
If transition for Relation fails, write a transition error event, preserve the prior state, and enqueue a repairable task.

## 10. RetentionClass

### States
- created
- indexed
- recalled
- reinforced
- decayed
- consolidated
- demoted
- archived
- deleted

### Typical transitions
created -> indexed
indexed -> recalled
recalled -> reinforced
indexed -> decayed
decayed -> demoted
demoted -> archived
archived -> deleted

### Guards
- policy pinning
- namespace access control
- unresolved contradiction state
- lineage preservation requirement
- repair job lock

### Failure handling
If transition for RetentionClass fails, write a transition error event, preserve the prior state, and enqueue a repairable task.

## 11. DecayState

### States
- created
- indexed
- recalled
- reinforced
- decayed
- consolidated
- demoted
- archived
- deleted

### Typical transitions
created -> indexed
indexed -> recalled
recalled -> reinforced
indexed -> decayed
decayed -> demoted
demoted -> archived
archived -> deleted

### Guards
- policy pinning
- namespace access control
- unresolved contradiction state
- lineage preservation requirement
- repair job lock

### Failure handling
If transition for DecayState fails, write a transition error event, preserve the prior state, and enqueue a repairable task.

## 12. ShardState

### States
- created
- indexed
- recalled
- reinforced
- decayed
- consolidated
- demoted
- archived
- deleted

### Typical transitions
created -> indexed
indexed -> recalled
recalled -> reinforced
indexed -> decayed
decayed -> demoted
demoted -> archived
archived -> deleted

### Guards
- policy pinning
- namespace access control
- unresolved contradiction state
- lineage preservation requirement
- repair job lock

### Failure handling
If transition for ShardState fails, write a transition error event, preserve the prior state, and enqueue a repairable task.

