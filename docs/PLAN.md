# PLAN

## Vision
Build a brain-inspired memory engine for AI agents.

Port the core functions of human memory into agent infrastructure:
working memory, episodic memory, semantic memory, associative recall,
consolidation, and intelligent forgetting.

## Product thesis
A useful agent memory system is not just a long-term store.
It must:
- keep hot information extremely fast
- retain important knowledge over long horizons
- reconstruct context from compressed traces
- link memory items by relation, not only similarity
- forget safely to reduce noise and cost
- preserve provenance and explainability

## Performance targets
- Tier1 retrieval: <0.1ms
- Tier2 retrieval: <5ms
- Tier3 retrieval: <50ms
- Fast encode path: <10ms

## Scale target
Effectively unbounded cold storage with bounded hot-path latency.

## Success conditions
1. Agents remember across sessions and tasks.
2. Retrieval improves behavior quality in measurable ways.
3. Latency remains stable as total memory grows.
4. Memory is explainable, governable, and repairable.
5. The system can be integrated through CLI and MCP.
