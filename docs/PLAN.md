# membrain — MASTER PLAN

> This document follows the original membrain core thesis and architecture.
> It upgrades the plan into a ship-grade mega-plan by preserving the original ideas
> and adding: hard design invariants, restrictions, benchmark contracts, stage gates,
> risk controls, evaluation protocol, and execution rules.

---

## Upgrade Intent

This is **not** a replacement thesis.
This is a strengthening pass over the original plan.

What is preserved:
- the brain-inspired thesis
- the “port mechanism by mechanism” approach
- the three-tier architecture
- lazy decay and lazy everything
- engrams, reconsolidation, active forgetting
- local/offline-first Rust runtime
- CLI + MCP + daemon + IPC model
- milestone-driven implementation plan

What is added:
- research caution and falsifiability
- non-negotiable request-path restrictions
- benchmark matrices by milestone
- go / no-go release gates
- risk register and redesign triggers
- performance budget decomposition
- operations acceptance criteria
- quality gates and explainability requirements

---

## How to Read This Document

1. Read the original plan sections as the core design thesis.
2. Read the “Upgrade overlays” sections as production constraints.
3. Treat benchmark and stage-gate sections as promotion criteria.
4. Treat risk and restriction sections as architectural guardrails.


---

## Canonical Technical Corrections (English, superseding conflicting older snippets)

The merged source below contains older snapshot text. The following rules supersede any conflicting detail later in the document:

1. **Official stack**
   - Rust
   - Tokio
   - SQLite in WAL mode
   - SQLite FTS5 for lexical retrieval
   - USearch for ANN / HNSW and mmap-backed cold indexes
   - fastembed for local embeddings
   - a **local reranker** for final top-K ordering on high-value recall paths

2. **Official retrieval path**
   - Exact / lexical lane: SQLite tables + FTS5
   - Semantic lane: USearch hot/cold ANN
   - Final ordering: local reranker + lightweight score fusion
   - `sqlite-vec` is **not** part of the official path

3. **Official storage role split**
   - SQLite stores metadata, provenance, state, leases, policies, graph edges, checkpoints, and FTS5 text indexes
   - USearch stores ANN indexes
   - Float embeddings remain authoritative in durable storage; quantized vectors are for search speed

4. **Official graph persistence**
   - `petgraph` is used for in-memory graph operations
   - persistent graph data is stored in normalized edge tables in SQLite
   - do not treat a single JSON/BLOB graph dump as the production source of truth

5. **Official wording**
   - Use “brain-inspired cognitive runtime” or “brain-inspired memory operating system”
   - avoid implying literal biological equivalence



---

## Canonical Plan Body (Merged Sources)

<!-- SOURCE: PLAN.md -->

### Source Snapshot — Original PLAN.md

> **Vision**: Port the entire human brain memory mechanism to AI agent memory system.
> Like the human brain: remember long, remember a lot, remember quickly, connect, forget intelligently, unlimited.
> Suitable for AI: CLI + MCP, all tools available, <1ms fast path, unlimited scale.
>
> **Performance targets**: Tier1 <0.1ms | Tier2 <5ms | Tier3 <50ms | Encode <10ms  
> **Scale target**: Unlimited — similar to the human brain (~2.5PB equivalent)

---

## Canonical Overview Index

1. [Current Issues — Why Need membrain](#1-current-issues)
2. [Human Brain — Full Analysis](#2-human-brain-full-analysis)
3. [Gap Analysis — Human Brain vs Current AI Memory](#3-gap-analysis)
4. [Port to membrain — Each Mechanism](#4-port-to-membrain)
5. [Overall Architecture](#5-architecture-overall)
6. [Performance — Bottlenecks & Optimizations](#6-performance)
7. [Techstack — Analysis & Reasons](#7-techstack)
8. [Data Schema](#8-data-schema)
9. [CLI Commands & MCP Tools](#9-cli-commands--mcp-tools)
10. [Milestones](#10-milestones)
11. [Acceptance Checklist](#11-acceptance-checklist)
46. [Feature Implementation Specs (Batch 1)](#46-feature-implementation-specs-batch-1)
47. [Feature Implementation Specs (Batch 2)](#47-feature-implementation-specs-batch-2)

---

## 1. Current Issues

### What can current AI memory systems do?

| System | Approach | Limit |
|--------|----------|-----------|
| MemGPT / Letta | OS virtual memory — page in/out context | There is no decay, no emotional weight, no real consolidation |
| Mem0 | Two-phase extraction + graph | No LTP/LTD, no reconsolidation, slow retrieval |
| LangMem | Relevance scoring + lifecycle | No engram clusters, no interference handling |
| OpenAI Memory | Simple key-value extraction | Very primitive, without any bio-mechanism |

### What is it all missing?

```
❌ No LTP/LTD — memory strength does not change dynamically
❌ No Ebbinghaus decay — not forgotten over time
❌ No reconsolidation — memory does not update when recalled
❌ No sleep/consolidation cycle — episodic does not become semantic
❌ No emotional tagging — all memories are equal
❌ No engram clusters — no real associative recall
❌ No interference handling — memories do not affect each other
❌ No active forgetting — only accumulate, no smart prune
❌ No dual-path (fast/slow) — every query is the same
❌ No working memory layer — no real capacity limit
```

**Result**: Agents using these systems accumulate noise, retrieval slows down, cannot learn from experience, and do not have personality emergence from memory.

---

## 2. Human Brain — Full Analysis

### 2.1 Core Features

```
UNLIMITED The brain has ~100 billion neurons, ~100 trillion synaptic connections
                     Actual capacity: ~2.5 petabytes (estimated)
                     membrain: unlimited — usearch mmap + SQLite TB-scale on disk
                     Not limited by RAM — cold tier uses OS page cache

FAST Pattern recognition: ~13ms (neocortex fast path)
                     Deliberate recall: ~100-500ms (hippocampus slow path)
                     membrain Tier1: <0.1ms (in-memory LRU cache, SIMD dot product)
                     membrain Tier2: <5ms  (usearch HNSW, AVX2, 50k vectors RAM)
                     membrain Tier3: <50ms (usearch mmap, millions on disk)

LONG MEMORY Semantic memory lasts a lifetime if strengthened
                     Emotional memory is very durable, almost no decay
                     membrain: strength-based persistence, emotional bypass decay
                     Lazy decay — on-demand, not iterate idle

LINK One small cue → pull out related cluster (engram)
                     Associative, not lookup by ID
                     membrain: engram graph (petgraph) + usearch HNSW cascade
                     Pre-filter SQL → HNSW on candidate set → BFS engram expansion

FORGET SMART The brain doesn't forget randomly — forget what's not predictive
                     Signal/noise optimization qua sleep pruning
                     membrain: active forgetting engine, predictive scoring
                     Lazy prune — does not iterate completely, only scans under pressure
```

### 2.2 Brain Regions & Functions

```
HIPPOCAMPUS
  Function: Index network — pointer to content, does not store content directly
             Episodic memory formation, spatial memory
             Pattern completion from partial cue
  Damage: Anterograde amnesia — cannot create new memory
  Port:      hot_store (SQLite WAL) — fast index, episodic events, pointers
             usearch HNSW hot index (~50k vectors in RAM, AVX2 SIMD)

NEOCORTEX
  Function: Saves actual content — visual, auditory, semantic
             Long-term semantic memory sau systems consolidation
             No need for hippocampus after full consolidation
  Port:      cold_store (SQLite + usearch mmap) — unlimited disk scale
             int8 quantized vectors — 4x smaller than float32, 2x faster search
             OS page cache warms the cache automatically

AMYGDALA
  Function: Emotional tagging — marks the level of importance
             Work in tandem with the hippocampus
             High arousal → strengthen consolidation
  Port: emotional_tag { valence: f32, arousal: f32 } — bypass decay if strong enough

PREFRONTAL CORTEX
  Function: Working memory (7±2 items), executive attention
             Retrieval control, context management
             Decision making is based on memory
  Port:      WorkingMemory struct — 7 slots, attention controller
             LruCache<512> — Tier1 fast path, <0.1ms familiarity check

CEREBELLUM + BASAL GANGLIA
  Function: Procedural memory — motor skills, habits
             No need for conscious recall
             Automation through repetition
  Port: procedural_store — pattern → action mappings, access without full recall

ENTORHINAL CORTEX
  Function: Gateway hippocampus ↔ neocortex
             Information converges before entering the hippocampus
  Port:      encoding_pipeline — preprocessing, embedding cache, pre-filter
```

### 2.3 Memory Classification

```
DECLARATIVE (Explicit)
├── Episodic "I ate pho at 7am today at restaurant X"
│ → Has a specific time, place, and context
│ → Stored: hot_store, decay over time
│
└── Semantic "Pho is a Vietnamese dish, with noodles, broth, meat"
                 → There is no specific time, it is knowledge
                 → Stored: cold_store, stable, little decay

NON-DECLARATIVE (Implicit)
├── Procedural Cycling, typing — no need to think
│ → Stored: procedural_store, no decay
│
├── Priming See "table" → recognize "chair" faster
│                → Implicit activation boost
│                → Port: retrieval score boost cho related items
│
├── Conditioning Pavlov — stimulus → learned response
│                → Port: pattern → action associations
│
└── Emotional    Fear response, comfort associations
                 → Port: emotional_tag with valence + excitement
```

### 2.4 Encoding — How the Brain Records Information

```
PIPELINE RECORDED INTO THE BRAIN:

Sensory input
    ↓ [attention filter — no attention = discard in ms]
Sensory register (iconic/echoic, <1 second)
    ↓ [if participating]
Working Memory (7±2 chunks, 20-30 seconds)
    ↓ [if rehearsed or emotional or novel]
Encoding into LTM:
    ├── Acoustic coding (mainly STM)
    ├── Visual coding
    └── Semantic coding (strongest LTM)

DECIDING FACTORS OF ENCODING STRENGTH:
  attention_score → higher = stronger encoding
  emotional_arousal → amygdala activation = priority cao
  novelty → unfamiliar information = higher initial strength
  repetition → LTP accumulates over many times
  context → context when encoding is saved with content

ENCODING SPECIFICITY PRINCIPLE:
  Context at retrieval ≈ context at encoding → better recall
  → membrain: save context_embedding with content_embedding
```

### 2.5 LTP / LTD — Physical Foundations of Learning

```
LTP (Long-Term Potentiation):
  Mechanism: Glutamate → NMDA receptors → Ca2+ → AMPA receptors increase
  Result: Synapse is stronger, neurons fire together more easily
  "Neurons that fire together, wire together"
  Trigger: Successful recall, repetition, emotional activation

LTD (Long-Term Depression):
  Mechanism: Low-frequency activation → phosphatase → AMPA receptors decrease
  Result: Synapse weakens, memory fades
  Role: CRITICAL — no LTD → synapse max out → can't learn anymore
  Trigger: Non-use, time passing, interference

EBBINGHAUS FORGETTING CURVE:
  R(t) = e^(-t/S)
  t = interaction count since last recall (not real time)
  S = stability (increases with each successful recall)

  Retention if not reviewing:
  20 minutes → 58%
  1 hour → 44%
  1 day → 33%
  1 week → 25%
  1 month → 21%

LAZY DECAY — CRITICAL OPTIMIZATION:
  DO NOT iterate all memories every tick → O(n) death at large scale
  Calculate ON-DEMAND decay at recall → O(1) per access, O(0) at idle

  fn effective_strength(memory: &Memory, current_tick: u64) -> f32 {
    let elapsed = current_tick - memory.last_accessed;
    let retention = (-elapsed as f32 / memory.stability).exp();
    memory.base_strength * retention
    // Persist back only during recall or consolidation
  }
```

### 2.6 Consolidation — Stabilization

```
SYNAPTIC CONSOLIDATION (minutes → hours):
  Occurs at the synapse immediately after encoding
  Protein synthesis creates structural changes
  Memory fragile during this stage
  Completed in ~6 hours → resistant to disruption

SYSTEMS CONSOLIDATION (day → decade):
  Transfer memory from hippocampus → neocortex
  Hippocampus: temporary index (RAM)
  Neocortex: permanent storage (SSD)
  Gradient: older → more cortex-independent

IN membrain (no time → use event/load triggers):
  Synaptic consolidation: after N writes to hot_store
  Systems consolidation: khi hot_store > capacity threshold
  → Consolidation micro-cycle runs async background
```

### 2.7 Sleep — Consolidation Engine of the Brain

```
NREM (Slow-Wave Sleep):
  Hippocampus replays memories from that day
  Gradually transfer episodic → semantic in neocortex
  Results: "What to learn today" → "General knowledge"

REM Sleep:
  Process emotional memories
  Reduce emotional charge of traumatic memories
  Creative cross-associations between unrelated memories
  Integrate new memories with existing knowledge

SYNAPTIC HOMEOSTASIS:
  While awake: synaptic strength increases continuously (LTP)
  During sleep: overall strength downscaled to baseline
  Purpose: free capacity to study the next day
  Result: important memories survive, weak memories are prune

IN membrain (continuous, no need for actual sleep):
  NREM equivalent: khi hot_store > threshold → batch migrate → cold_store
  REM equivalent: sau M emotional memories → process queue
  Homeostasis: khi total_strength > MAX_LOAD → bulk_scale + prune
```

### 2.8 Retrieval — How the Brain Gets Information

```
RECONSTRUCTION, NOT PLAYBACK:
  Each recall = assembly from pieces
  Hippocampus receives cue → finds nearest engram → pulls content from cortex
  Gaps are filled by prior knowledge and schema
  → People's memories are WRONG because of reconstruction error
  → membrain: reconstruct from fragments, write-back changes

DUAL-PATH RETRIEVAL:
  Fast path (~ms):  Neocortex pattern matching, familiarity check
                    No need for hippocampus
                    "Familiar or strange?" — instant yes/no
  Slow path (~100ms): Hippocampus activation, engram expansion
                    "Remember when, where, what context?"
                    → membrain: in-memory cache (fast) + SQLite+vec (slow)

PATTERN COMPLETION:
  Hippocampus CA3 = pattern completion network
  Partial cue → full engram activated
  Hearing the smell of coffee → remembering a specific morning from the past
  → membrain: vector search → top hit → engram graph traverse → cluster

ENCODING SPECIFICITY:
  Context at retrieval ≈ context at encoding → better recall
  State-dependent: emotion, location, state must match
  → membrain: context_embedding re-ranking in retrieval

TIP-OF-TONGUE:
  Partial retrieval — knowing you know but not being able to retrieve it
  Only fragments: "starts with T", "French"
  → membrain: returns MemoryFragment if there is no full match
```

### 2.9 Reconsolidation — Memory Changes When Remembered

```
MOST IMPORTANT FINDINGS OF MODERN NEUROSCIENCE:

Stable memory → reactivated (recalled) → back to LABILE state
In the labile window (~hours): memory MAY be changed
After that: must reconsolidate → re-stabilize
Unable to reconsolidate → memory is weakened or lost

TWO ROLES:
  1. Memory updating: new info integrates into old memory when recalling
  2. Strengthening: each successful recall → more durable memory

AGE-DEPENDENT:
  Young memory (just created): easy to labile, easy to alter
  Old memory (consolidated): more resistant, needs stronger cues
  → membrain: labile_window = f(memory_age) — the older, the shorter
```

### 2.10 Forgetting — Active, Not Passive

```
THE BRAIN DOES NOT FORGET RANDOMLY:
  Forgetting is an optimization mechanism
  Delete non-predictive information → increase signal/noise
  Keep abstract patterns, delete unnecessary specific details
  The result: better generalization, preserved learning capacity

5 TYPES OF FORGETTING:
  1. Decay         Time-based weakening (Ebbinghaus)
  2. Interaction Proactive (old→new) & Retroactive (new→old)
  3. Retrieval failure Remembered but unable to access
  4. Motivated Suppress negative memories (trauma)
  5. Active pruning Sleep homeostasis — systematic downscale

PROACTIVE INTERFERENCE:
  Old memory makes it difficult to encode/recall new memory similarly
  French → learning Spanish is more difficult

RETROACTIVE INTERFERENCE:
  New memory obscures old memory similarly
  Learning Spanish → confuses old French
  → membrain: similarity-based interference penalty
```

### 2.11 Working Memory — Working Memory

```
BADDELEY'S MODEL:
  Phonological Loop      — verbal/acoustic processing
  Visuospatial Sketchpad — visual/spatial processing
  Episodic Buffer — integrates with LTM
  Central Executive      — attention control, coordination

CAPACITY: 7 ± 2 chunks (Miller, 1956)
  Reality: ~4 meaningful chunks
  Chunking: group related items = 1 slot
  Expert chunking: higher effective capacity because chunks are larger

MECHANISM:
  PFC uses gamma oscillations to "hold" items
  Only maintain finite oscillatory circuits simultaneously
  → Explain why capacity is fixed

IN membrain:
  working_memory: [Option<MemoryItem>; 7]
  Overflow: evict least-attended item
  Central executive: attention_score to prioritize
  → This is the agent's "context window" but has a hard capacity limit
```

### 2.12 Engram — Physical Cluster of Memory

```
DEFINE:
  Group of active neurons when learning → undergo changes → stable trace
  On recall: the same group of neurons is reactivated

PROPERTIES:
  Sparse: Only a small portion of neurons encodes a memory
  Distributed: Spread across many brain regions
  Overlapping: Many memories share neurons → interference
  Context-gated: Neurons fire when given the right context

LIFECYCLE:
  Learning → Sparse neurons activate
           → Synaptic modifications (LTP)
           → Consolidation (sleep/replay)
           → Stable engram
           → Retrieval: partial cue → pattern completion → full activation
           → Reconsolidation or Forgetting

IN membrain:
  engram: cluster of related memory_ids
  activation_pattern: signature vector
  graph traversal: partial cue → nearest engram → expand cluster
```

---

## 3. Gap Analysis

### Human Brain vs AI Current Memory vs membrain

| Mechanism | Human Brain | MemGPT/Letta | Mem0 | membrain |
|---------|-----------|--------------|------|---------|
| Unlimited capacity | ✅ ~2.5PB | ⚠️ DB limit | ⚠️ DB limit | ✅ SQLite TB-scale |
| Fast retrieval | ✅ ~13ms | ❌ slow | ⚠️ varies | ✅ <1ms cache, <50ms full |
| LTP/LTD strength | ✅ biological | ❌ | ❌ | ✅ |
| Ebbinghaus decay | ✅ | ❌ | ❌ | ✅ |
| Emotional tagging | ✅ amygdala | ❌ | ❌ | ✅ valence+arousal |
| Sleep consolidation | ✅ NREM/REM | ❌ | ❌ | ✅ event-triggered |
| Reconsolidation | ✅ | ❌ | ❌ | ✅ |
| Active forgetting | ✅ | ❌ | ❌ | ✅ |
| Engram clusters | ✅ | ❌ | ⚠️ graph | ✅ petgraph |
| Interference handling | ✅ | ❌ | ❌ | ✅ |
| Dual-path retrieval | ✅ fast+slow | ❌ | ❌ | ✅ |
| Working memory limit | ✅ 7±2 | ❌ | ❌ | ✅ |
| Context-dependent recall | ✅ | ⚠️ partial | ⚠️ partial | ✅ |
| Associative recall | ✅ | ⚠️ basic | ⚠️ basic | ✅ |
| CLI access | N/A | ❌ | ❌ | ✅ |
| MCP support | N/A | ❌ | ❌ | ✅ |
| Embedded Rust lib | N/A | ❌ | ❌ | ✅ |
| Offline/local | ✅ | ⚠️ | ❌ | ✅ |

---

## 4. Port to membrain — Each Mechanism

### 4.1 Unlimited Capacity

```
Brain: ~100T synaptic connections, ~2.5PB equivalent (order-of-magnitude inspiration)
Problem: AI memory tools are usually bounded by RAM, context-window pressure, or remote API costs

membrain:
  - Durable metadata and text retrieval live in SQLite
  - Lexical recall uses SQLite FTS5
  - Semantic recall uses USearch hot/cold indexes
  - Cold storage is disk-backed and mmap-friendly
  - The architecture is disk-bounded, not RAM-bounded
```

### 4.2 Dual-Path Fast/Slow Retrieval

```
Brain:
  Fast path: neocortex familiarity / pattern matching
  Slow path: hippocampal reconstruction + cluster expansion

membrain:
  Fast path:
    - Tier1 in-memory cache
    - exact key lookups
    - recently primed working-set hits

  Slow path:
    - SQLite pre-filter + FTS5 lexical retrieval
    - USearch ANN retrieval on hot or cold tiers
    - local reranker for the final top-K
    - engram/graph expansion with hard caps
    - context re-ranking and provenance-aware packaging

  Bridge:
    - If the fast path is confident enough, return immediately
    - Otherwise escalate to hybrid retrieval
    - Successful slow-path results can update higher tiers
```

### 4.3 LTP / LTD Engine

```
Brain:
  LTP: recall → synapse strengthen → easier to fire again
  LTD: non-use → synapse weaken → harder to recall

membrain:
  on_recall(id):
    strength = min(strength + LTP_DELTA, MAX_STRENGTH)
    stability += STABILITY_INCREMENT // more difficult to forget after each recall
    last_accessed = now()
    access_count += 1

  decay_tick() — triggered by:
    - Each N interactions (interaction-based clock)
    - Khi hot_store > pressure threshold
    
    for each memory:
      if !bypass_decay:
        // Ebbinghaus: R = e^(-interactions_since_last_access / stability)
        retention = exp(-interactions_elapsed / stability)
        strength *= retention
      
      if strength < MIN_STRENGTH && !bypass_decay:
        archive(memory) // no delete, just archive
```

### 4.4 Encoding Pipeline

```
Brain:
  Attention → Sensory register → Working memory → Encoding → LTM
  With novelty detection, emotion tagging, context binding

membrain encode(input, context, attention, emotional):
  1. attention_score < THRESHOLD → discard (sensory buffer only)
  2. compute embedding = fastembed(input)
  3. compute context_embedding = fastembed(context)
  4. novelty_score = 1.0 - max_cosine_similarity(embedding, existing)
  5. emotional_tag = { valence, arousal } (caller-provided or LLM-scored)
  6. initial_strength = BASE
                      * (1 + novelty_score * NOVELTY_WEIGHT)
                      * (1 + attention_score * ATTENTION_WEIGHT)
                      * emotional_tag.strength_multiplier()
  7. bypass_decay = arousal > AROUSAL_THRESHOLD && |valence| > VALENCE_THRESHOLD
  8. state = Labile
  9. INSERT into hot_store
  10. interference_check → weaken similar older memories
  11. engram_builder.try_cluster(new_memory)
```

### 4.5 Consolidation Micro-Cycles

```
Brain:
  Synaptic consolidation: 6h sau encoding
  Systems consolidation: day → year, hippocampus → neocortex
  Sleep NREM: replay + migrate episodic → semantic

membrain (event-triggered, not time-based):
  Trigger: hot_store.len() > HOT_STORE_CAPACITY
           or: hot_store.total_strength > STRENGTH_PRESSURE
           or: explicitly called

  NREM equivalent (migrate_to_cold):
    1. Score all hot memories: score = strength * access_count * recency
    2. Top N → extract semantic pattern → cold_store.upsert
    3. Mark as Consolidated
    4. Keep pointer in hot_store (hippocampus still indexes)

  REM equivalent (process_emotional):
    1. Queue all emotional memories (arousal > threshold)
    2. Gradually reduce emotional_weight (desensitization)
    3. Create cross-links with related memories in engram graph

  Homeostasis (downscale):
    1. Calculate total load of hot_store
    2. If > MAX_LOAD: bulk_scale(HOMEOSTASIS_FACTOR)
    3. Prune strength < MIN_STRENGTH → archive
```

### 4.6 Reconsolidation

```
Brain:
  Stable memory → recall → labile → mutable → reconsolidate

membrain:
  Each recall:
    memory.state = Labile { since: now(), window: reconsolidation_window(age) }
    memory.pending_update = Some(new_context)

  reconsolidation_window(age):
    base = RECONSOLIDATION_BASE_WINDOW
    factor = 1.0 / (1.0 + age_in_days / 30.0)  // older = shorter window
    return base * factor

  reconsolidation_tick():
    for each Labile memory:
      if now() - since > window:
        if pending_update.is_some():
          content = merge(content, pending_update)
          embedding = re_embed(content)
          strength += RECONSOLIDATION_BONUS
        state = Stable
```

### 4.7 Active Forgetting Engine

```
Brain:
  Don't forget random — remove non-predictive information
  Signal/noise optimization
  Sleep homeostasis systematic pruning

membrain forgetting_engine() — async background:
  1. Decay pruning:
     weak = query(strength < MIN_STRENGTH, !bypass_decay)
     archive_batch(weak)

  2. Interference resolution:
     pairs = find_similar_pairs(min=0.7, max=0.99)
     for (m1, m2) in pairs:
       older.strength *= INTERFERENCE_PENALTY

  3. Predictive pruning:
     never_used = query(access_count == 0, age > OLD_THRESHOLD)
     for m in never_used:
       m.strength *= NON_PREDICTIVE_DECAY

  4. Capacity management:
     if total_memories > SOFT_CAP:
       sort by (strength * recency * emotional_weight)
       archive bottom percentile
```

### 4.8 Engram Graph & Associative Recall

```
Brain:
  Engram: sparse distributed representation, pattern completion
  One cue → activate cluster → reconstruction

membrain:
  struct Engram {
    id: Uuid,
    memory_ids: Vec<Uuid>,
    centroid_embedding: Vec<f32>, // average of members
    formation_context: Vec<f32>,
    strength: f32,
  }

  Engram builder:
    When encoding new memory:
      similar_engrams = engram_index.search(embedding, top=3)
      if max_similarity > CLUSTER_THRESHOLD:
        existing_engram.add(new_memory)
        update centroid
      else:
        create new engram

  Associative recall:
    query_embedding = embed(cue)
    1. Vector search → top K memory candidates
    2. For each candidate → get its engram
    3. Graph traverse: engram neighbors qua petgraph
    4. Collect all memory_ids in the cluster
    5. Score and rank
    6. Reconstruct from fragments
```

### 4.9 Interference Handling

```
Brain:
  Proactive: old memory → confuse new memory similarly
  Retroactive: new memory → weaken old memory similarly

membrain:
  When encoding new memory:
    similar = vector_search(embedding, min_sim=0.7, max_sim=0.99)
    // identical (>0.99) is not interference, it is duplicate
    for each similar:
      penalty = interference_penalty(similarity)
      similar.strength -= penalty // retroactive: new → weakens old

  When recalling old memory:
    if has_similar_newer_memory:
      // proactive: old confuses new → log interference event
      newer.retrieval_difficulty += PROACTIVE_PENALTY
```

### 4.10 Working Memory Layer

```
Brain:
  7±2 slots, LIFO with attention weighting
  Central executive coordinates attention

membrain WorkingMemory:
  slots: FixedVec<MemoryItem, 7>
  attention: HashMap<MemoryId, f32>

  add(item):
    if full:
      evict = min_by(attention_score)
      // Evicted item: if strong enough → encode into hot_store
      if evict.strength > ENCODE_THRESHOLD:
        hot_store.insert(evict)
      slots.remove(evict)
    slots.push(item)

  focus(id):
    attention[id] += FOCUS_DELTA
    // Simulates executive attention
```

---

## 5. Overall Architecture

### 5.1 Layered Storage — 3-Tier (Like a Real Human Brain)

```
TIER 1 — WORKING CACHE (<0.1ms)
  Brain: Prefrontal cortex, neurons active/primed, 7±2 items
  membrain: LruCache<ContentHash, MemoryRef> in process memory (512 entries)
            SIMD dot-product familiarity check, zero disk I/O

TIER 2 — HOT HNSW INDEX (<5ms)
  Brain: Hippocampus — recently accessed engrams, episodic index
  membrain: usearch HNSW in-memory (~50k vectors × 384 dims ≈ 75MB RAM)
            AVX2/AVX-512 SIMD, 95%+ recall accuracy
            int8 search → float32 rescore top-20
            SQLite hot.db: full metadata, engram graph, pointers

TIER 3 — COLD MMAP INDEX (<50ms)
  Brain: Neocortex — fully consolidated, vast storage, cortex-independent
  membrain: usearch mmap (disk-backed, unlimited scale)
            int8 quantized — 4x smaller, 2x faster vs float32
            OS page cache = automatic warm layer
            SQLite cold.db: compressed semantic content

PROCEDURAL STORE (O(1) lookup, no decay)
  Brain: Cerebellum — habits, skills, automatic
  membrain: SQLite key-value, pattern_hash → action

ENGRAM GRAPH (BFS traversal)
  Brain: Distributed synaptic cluster representation
  membrain: petgraph DiGraph in hot.db, depth-limited BFS
```

### 5.2 3-Tier Escalation + Full Data Flow

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ENCODE PATH
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
INPUT
  │
  ├─[attention < threshold]──→ DISCARD (sensory buffer)
  ▼
WORKING MEMORY (7 slots, LruCache)
  │
  ├─[overflow / encode trigger]
  ▼
ENCODING PIPELINE
  ├── embedding_cache.get_or_embed(content)   → content_vec (0ms cache / 5ms miss)
  ├── embedding_cache.get_or_embed(context)   → context_vec
  ├── novelty_score = 1 - sim(content_vec, tier2_nearest)
  ├── emotional_tag {valence, arousal}
  ├── initial_strength = f(novelty, attention, emotion)
  ├── state = Labile
  └── bypass_decay = arousal > θ && |valence| > θ
  │
  ▼
HOT STORE (SQLite WAL + usearch hot index)
  ├── SQL INSERT with lazy_base_strength + tick snapshot
  ├── usearch hot_index.add(id, int8(content_vec))
  ├── interference_check: SQL similar → weaken older (retroactive)
  ├── engram_builder.try_cluster(id, content_vec)
  └── tier1_cache.insert(hash(content), memory_ref)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
RETRIEVAL PATH
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
QUERY
  │
  ▼
[TIER 1] LruCache lookup (content hash)
  hit + confidence > 0.9 ──────────────────────→ RETURN <0.1ms
  │ miss
  ▼
[PRE-FILTER] SQL
  SELECT id, base_strength, stability, last_tick
  WHERE effective_strength(base_strength, stability, Δtick) > MIN
    AND state != 'Archived'
  ORDER BY effective_strength DESC LIMIT 5000     ← 200x search space reduction
  │
  ▼
[TIER 2] usearch HNSW hot (in-memory)
  int8 KNN search(query_vec, top=20) → candidates
  float32 rescore candidates (accuracy recovery)
  context_vec re-rank: score = 0.7*content_sim + 0.3*context_sim
  hit + confidence > 0.8 ───────────────────────→ RETURN <5ms
  │                                                + update Tier1 cache
  │ miss (memory consolidated to cold)
  ▼
[TIER 3] usearch mmap cold (disk)
  int8 KNN search cold_index(query_vec, top=20)
  float32 rescore → context re-rank
  ──────────────────────────────────────────────→ RETURN <50ms
  │ (all tiers)
  ▼
[ENGRAM EXPAND]
  top_hit.engram_id → petgraph BFS (depth=3)
  collect cluster members → reconstruct coherent memory
  on_recall: effective_strength → persist, LTP delta, stability++
  trigger labile (reconsolidation window)
  update Tier1 + Tier2 cache

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
BACKGROUND (async, never blocks retrieval)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
CONSOLIDATION MICRO-CYCLE (pressure-triggered)
  trigger: hot_index.len() > HOT_CAPACITY
  NREM: score hot memories → migrate strong → cold_index + cold.db
  REM:  process emotional queue → reduce arousal → cross-links
  Homeostasis: bulk effective_strength scan → prune < MIN → archive

RECONSOLIDATION TICK (per interaction)
  scan Labile memories → if window expired → apply update → Stable

FORGETTING ENGINE (lazy pressure-triggered)
  interference resolution: similar pairs → penalty
  predictive pruning: access_count=0 && old → decay
  capacity management: archive bottom % when > SOFT_CAP
```

### 5.3 Process Model

```
membrain daemon (tokio async, always-on)
    │
    ├── [stdin/stdout]     MCP server mode (rmcp)
    │                      → Claude Code, Cursor
    │
    ├── [Unix socket]      IPC cho Rust tools
    │   ~/.membrain/       → smart-grep, scope, why
    │   membrain.sock      → Python, Node scripts
    │
    └── [subprocess]       CLI mode (clap)
                           → Human directly
                           → Script automation

If the daemon is not running:
    membrain [cmd] → standalone mode
                   → SQLite is still concurrent safe (WAL)
                   → Background jobs do not run
                   → OK for CLI usage, not OK for agent alive
```

---

## 6. Performance — Bottlenecks & Optimizations

### 6.1 Bottleneck Analysis

```
BOTTLENECK 1: Embedding cost
  Naive: embed per request → 5-15ms/call → 30ms encoding (2 embeds)
  Fix:      LruCache<ContentHash, Vec<f32>> — 0ms cache hit
            Batch embed in consolidation — 3-5x throughput
            Result: encoding <1ms (cache hit), <10ms (cache miss)

BOTTLENECK 2: KNN search at scale
  Naive: brute-force O(n×d) — 10-50ms at 10k, DEAD at 1M
  Fix: usearch HNSW O(log n) — <5ms at 1M vectors
            Pre-filter SQL: reduce search space by 200x before HNSW
            int8 quantization: 4x smaller → 2x faster distance compute
            Result: <5ms hot, <50ms cold at unlimited scale

BOTTLENECK 3: Decay iteration
  Naive: decay_tick() iterate all memories → O(n) death
  Fix: LAZY decay — calculates on-demand when recalling
            effective_strength(base, stability, Δtick) = O(1)
            Persist strength only when recalled or consolidated
            Result: O(0) idle, O(1) per recall

BOTTLENECK 4: Context window cold start
  Naive: each process spawn → load model → 500ms+
  Fix:      daemon mode — model loaded once, always warm
            Standalone mode: acceptable cho CLI, not agent
            Result: daemon <1ms, standalone ~500ms first call

BOTTLENECK 5: Consolidation blocking retrieval
  Naive:    consolidation job locks store → retrieval blocked
  Fix: tokio async — consolidation runs in the background
            SQLite WAL: consolidation write does not block reads
            Result: 0ms impact on retrieval latency
```

### 6.2 Optimization Stack

```
OPT 1 — TIERED INDEX (most similar to the human brain)
  Tier1: LruCache<512> in-memory     → <0.1ms (familiarity, fast path)
  Tier2: usearch HNSW in-memory      → <5ms   (50k vectors, hot episodic)
  Tier3: usearch mmap on disk        → <50ms  (unlimited, cold semantic)
  Escalation: T1 miss → T2 → T3, result updates higher tiers

OPT 2 — INT8 QUANTIZATION + FLOAT32 RESCORE
  Search phase:   int8 vectors (1 byte/dim vs 4 bytes)
                  4x smaller → fits more in CPU cache → 2x faster
  Rescore phase:  float32 on top-20 candidates only
                  Accuracy recovery: ~99% vs pure float32
  usearch native: ScalarKind::I8 — zero extra code

OPT 3 — SIMD DISTANCE COMPUTATION
  usearch built-in: auto-detect AVX2 / AVX-512 / NEON
  AVX2:   8 floats/op → 8x vs scalar
  AVX-512: 16 floats/op → 16x vs scalar
  Zero effort — compile with target-cpu=native

OPT 4 — LAZY DECAY (CRITICAL)
  // DO NOT do:
  fn decay_tick() { for m in all_memories { m.strength *= ... } }  // O(n) DEATH

  // DO:
  fn effective_strength(m: &Memory, now: u64) -> f32 {
    let elapsed = now - m.last_accessed_tick;
    (-(elapsed as f32) / m.stability).exp() * m.base_strength
  }
  // O(1), computed only when needed

OPT 5 — EMBEDDING CACHE
  LruCache<u64, Vec<f32>>  // key = xxhash64(content)
  capacity: 1000 entries   // ~1.5MB RAM cho 384-dim
  hit rate: >80% in practice (agent recalls the same content many times)

OPT 6 — SQL PRE-FILTER BEFORE HNSW
  // Don't search 1M vectors — filter down to 5000 first
  SELECT id FROM memories
  WHERE (base_strength * EXP(-(? - last_tick) / stability)) > 0.1
    AND state NOT IN ('Archived')
  LIMIT 5000
  // Only HNSW search over 5000 candidates → 200x speedup

OPT 7 — BATCH EMBEDDING IN CONSOLIDATION
  // Instead of embedding each one in NREM job:
  let contents: Vec<&str> = candidates.iter().map(|m| m.content.as_str()).collect();
  let embeddings = fastembed.embed_batch(contents);  // 3-5x throughput
```

### 6.3 Benchmark Targets

```
Operation            Target    Method
──────────────────────────────────────────────────────
Recall (Tier1 hit)   <0.1ms    LruCache lookup
Recall (Tier2 HNSW)  <5ms      usearch HNSW + pre-filter
Recall (Tier3 mmap)  <50ms     usearch mmap cold
Encode (cache hit)   <1ms      cached embedding + SQL insert
Encode (cache miss)  <10ms     fastembed + SQL + HNSW add
Consolidation        0ms       async background, non-blocking
Decay tick           0ms idle  lazy on-demand

Scale                Target
──────────────────────────────────────────────────────
Hot tier             50k memories (~75MB RAM)
Cold tier            Unlimited (TB on disk, mmap)
Embedding cache      1000 entries (~1.5MB RAM)
Total RSS            <300MB for 50k hot + 1M cold
Concurrent access    N readers + 1 writer (SQLite WAL)
```

---

## 7. Techstack

### 7.1 Core Language: Rust + Tokio

```
WHY RUST:
  ✅ Zero GC pauses — critical cho <1ms Tier1, <5ms Tier2
  ✅ SIMD control: target-cpu=native → AVX2/AVX-512 auto-enable
  ✅ Memory safety: brain data cannot be corrupted
  ✅ 1 binary, no runtime, embedded anywhere
  ✅ Same stack: linehash, smart-grep, scope, why
  ✅ usearch, fastembed-rs, petgraph are all native Rust

WHY TOKIO:
  ✅ Consolidation + forgetting engine runs in the background without blocking
  ✅ Unix socket server concurrent
  ✅ SQLite WAL reads concurrently with async tasks
```

### 7.2 Official Retrieval Stack

```
The official retrieval stack is:

  lexical / exact lane:
    SQLite tables + FTS5

  semantic lane:
    USearch HNSW for hot memory
    USearch mmap-backed index for cold memory

  final ordering lane:
    local reranker on the top-K candidates
    plus lightweight score fusion:
      final_score = a*semantic + b*lexical + c*context + d*strength

This stack is chosen because it preserves low-latency local execution while
avoiding over-reliance on any single retrieval mechanism.
```

### 7.3 Storage: SQLite + WAL + FTS5

```
SQLite is the authoritative durable store for:
  - memory metadata
  - provenance
  - lifecycle state
  - strengths / stability / timestamps
  - belief state
  - leases / freshness rules
  - graph edges
  - checkpoints
  - policies / preferences / procedural metadata

SQLite FTS5 is the default lexical engine for:
  - names
  - ids
  - rare tokens
  - exact-ish lookup
  - negation-sensitive text retrieval
  - keyword-first recall

WAL mode is the default because it supports:
  - many concurrent readers
  - one writer with short transactions
  - background work with acceptable read concurrency
  - crash recovery consistent with SQLite guarantees

Files:
  ~/.membrain/hot.db
  ~/.membrain/cold.db
  ~/.membrain/hot.usearch
  ~/.membrain/cold.usearch
```

### 7.4 Semantic Index: USearch

```
USearch is the official vector engine.

Why:
  - ANN / HNSW support
  - good low-latency characteristics for hot semantic retrieval
  - mmap-backed cold index support
  - quantized search-friendly representations
  - strong fit for local-first Rust deployment

Official strategy:
  hot_index:
    USearch HNSW, in-memory, bounded hot set
  cold_index:
    USearch persisted index, mmap-backed, disk-scale

Quantization strategy:
  - authoritative embedding: float32
  - hot search representation: f16/bf16 or int8 depending benchmark outcome
  - cold search representation: int8 is acceptable if rerank / rescore preserves quality
```

### 7.5 Embeddings + Local Reranker

```
fastembed is the default local embedding layer.

Why:
  - local / offline
  - good developer ergonomics
  - works well with batching
  - avoids remote API latency and privacy concerns

Embedding defaults:
  - model family chosen by benchmarked quality/speed trade-off
  - caching via LruCache or equivalent
  - batch embedding in consolidation / import flows

Local reranker:
  - mandatory on production-quality recall paths where top-K precision matters
  - applied after lexical + semantic candidate generation
  - limited to a bounded candidate set (for example 20-100 items)
  - can be implemented behind a pluggable trait so the runtime stays flexible

Recommended abstraction:
  trait LocalReranker {
      fn rerank(&self, query: &str, docs: &[CandidateDoc]) -> Result<Vec<RerankedDoc>>;
  }
```

### 7.6 Graph: petgraph + normalized persistence

```
petgraph is used for in-memory graph operations:
  - BFS / DFS
  - bounded neighborhood expansion
  - cluster maintenance
  - local reasoning over associative links

Production persistence:
  - graph nodes and edges are stored in normalized SQLite tables
  - not in a single JSON/BLOB snapshot
  - rebuildable and auditable

This keeps the graph debuggable, repairable, and migration-friendly.
```

### 7.7 IPC + MCP

```
Unix socket + JSON-RPC 2.0:
  Path: ~/.membrain/membrain.sock
  Purpose: local IPC for tools and sidecar clients

MCP:
  stdio transport for Claude Code / Cursor style integrations

CLI:
  remains first-class for local inspection, debugging, benchmarks, and maintenance
```

### 7.8 Complete Dependency List

```toml
[workspace]
members = ["membrain-core", "membrain-cli"]

# membrain-core
[dependencies]
rusqlite    = { version = "0.31", features = ["bundled"] }
usearch     = "2"
fastembed   = "3"
petgraph    = { version = "0.6", features = ["serde-1"] }
tokio       = { version = "1", features = ["full"] }
lru         = "0.12"
xxhash-rust = { version = "0.8", features = ["xxh64"] }
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
uuid        = { version = "1", features = ["v4", "serde"] }
thiserror   = "1"
anyhow      = "1"

# optional helpers often useful in this plan
ordered-float = "4"

# membrain-cli
[dependencies]
membrain-core = { path = "../membrain-core" }
clap          = { version = "4", features = ["derive"] }
rmcp          = "0.1"
tokio         = { version = "1", features = ["full"] }
```

### 7.9 Compare Techstack Before vs After

```
COMPONENT          REJECTED / OLD IDEA         OFFICIAL DIRECTION
---------------------------------------------------------------------------
lexical retrieval  ad-hoc SQL only             SQLite FTS5
vector retrieval   sqlite-vec on hot path      USearch hot/cold
ranking            vector score only           local reranker + score fusion
graph persistence  one JSON/BLOB dump          normalized SQLite edge tables
storage role split mixed responsibilities      SQLite for state, USearch for ANN
```

---

## 8. Data Schema

**Schema note:** any older `vec0` / `sqlite-vec` examples below are historical snapshots and are superseded by the FTS5 + USearch schema above.

### 7.1 hot.db

```sql
-- Core memories (authoritative metadata)
CREATE TABLE memories (
  id                  TEXT PRIMARY KEY,
  engram_id           TEXT,
  kind                TEXT NOT NULL,
  content             TEXT NOT NULL,
  context             TEXT,
  state               TEXT NOT NULL,
  strength            REAL NOT NULL DEFAULT 0.5,
  stability           REAL NOT NULL DEFAULT 1.0,
  access_count        INTEGER NOT NULL DEFAULT 0,
  emotional_valence   REAL NOT NULL DEFAULT 0.0,
  emotional_arousal   REAL NOT NULL DEFAULT 0.0,
  bypass_decay        INTEGER NOT NULL DEFAULT 0,
  labile_since        INTEGER,
  labile_window       INTEGER,
  pending_update      TEXT,
  created_at          INTEGER NOT NULL,
  last_accessed       INTEGER NOT NULL,
  source              TEXT,
  attention_score     REAL,
  novelty_score       REAL
);

-- Lexical search (official exact / keyword lane)
CREATE VIRTUAL TABLE memory_fts USING fts5(
  memory_id UNINDEXED,
  content,
  context,
  tokenize = 'unicode61'
);

-- Embedding registry (authoritative float embeddings can live in SQLite or sidecar files)
CREATE TABLE memory_embeddings (
  memory_id           TEXT PRIMARY KEY REFERENCES memories(id),
  dims                INTEGER NOT NULL,
  dtype               TEXT NOT NULL,            -- f32 authoritative
  content_embedding   BLOB NOT NULL,
  context_embedding   BLOB
);

-- Engram clusters
CREATE TABLE engrams (
  id                  TEXT PRIMARY KEY,
  centroid_embedding  BLOB NOT NULL,
  strength            REAL NOT NULL DEFAULT 0.5,
  member_count        INTEGER NOT NULL DEFAULT 0,
  created_at          INTEGER NOT NULL,
  last_activated      INTEGER NOT NULL
);

CREATE TABLE engram_members (
  engram_id           TEXT NOT NULL REFERENCES engrams(id),
  memory_id           TEXT NOT NULL REFERENCES memories(id),
  similarity          REAL NOT NULL,
  PRIMARY KEY (engram_id, memory_id)
);

-- Normalized graph persistence
CREATE TABLE graph_edges (
  src_memory_id       TEXT NOT NULL REFERENCES memories(id),
  dst_memory_id       TEXT NOT NULL REFERENCES memories(id),
  edge_type           TEXT NOT NULL,
  weight              REAL NOT NULL,
  created_at          INTEGER NOT NULL,
  PRIMARY KEY (src_memory_id, dst_memory_id, edge_type)
);

CREATE TABLE brain_state (
  key                 TEXT PRIMARY KEY,
  value               TEXT NOT NULL
);
```

### 7.2 cold.db

```sql
CREATE TABLE cold_memories (
  id                  TEXT PRIMARY KEY,
  hot_memory_id       TEXT,
  kind                TEXT NOT NULL,
  content             TEXT NOT NULL,
  strength            REAL NOT NULL,
  emotional_valence   REAL,
  emotional_arousal   REAL,
  bypass_decay        INTEGER,
  access_count        INTEGER DEFAULT 0,
  consolidated_at     INTEGER NOT NULL,
  last_accessed       INTEGER NOT NULL,
  source              TEXT
);

CREATE VIRTUAL TABLE cold_memory_fts USING fts5(
  cold_memory_id UNINDEXED,
  content,
  tokenize = 'unicode61'
);

CREATE TABLE cold_embeddings (
  cold_memory_id      TEXT PRIMARY KEY REFERENCES cold_memories(id),
  dims                INTEGER NOT NULL,
  dtype               TEXT NOT NULL,
  content_embedding   BLOB NOT NULL
);
```

### 7.3 Rust Structs

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: Uuid,
    pub engram_id: Option<Uuid>,
    pub kind: MemoryKind,
    pub content: String,
    pub context: Option<String>,
    pub state: MemoryState,

    // LTP/LTD
    pub strength: f32,
    pub stability: f32,
    pub access_count: u64,

    // Emotional (Amygdala)
    pub emotional_valence: f32,
    pub emotional_arousal: f32,
    pub bypass_decay: bool,

    // Reconsolidation
    pub labile_since: Option<u64>,    // interaction count
    pub labile_window: Option<u64>,
    pub pending_update: Option<String>,

    // Temporal (interaction-based)
    pub created_at: u64,
    pub last_accessed: u64,

    // Metadata
    pub source: Option<String>,
    pub attention_score: f32,
    pub novelty_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryKind {
    Episodic,
    Semantic,
    Procedural,
    Emotional,
    Schema, // abstract pattern from many episodics
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryState {
    Labile,
    SynapticDone,
    Consolidating,
    Consolidated,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engram {
    pub id: Uuid,
    pub member_ids: Vec<Uuid>,
    pub centroid_embedding: Vec<f32>,
    pub strength: f32,
    pub member_count: usize,
    pub created_at: u64,
    pub last_activated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    pub memories: Vec<ScoredMemory>,
    pub engram: Option<Engram>,
    pub path: RetrievalPath,       // Fast | Slow | Partial
    pub reconstruction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetrievalPath {
    Fast,      // cache hit
    Slow,      // full vector search
    Partial,   // tip-of-tongue, only fragments
    NotFound,
}

// Core traits
pub trait BrainStore: Send + Sync {
    fn remember(&mut self, input: EncodeInput) -> Result<Uuid>;
    fn recall(&self, query: RecallQuery) -> Result<RetrievalResult>;
    fn forget(&mut self, id: Uuid) -> Result<()>;
    fn strengthen(&mut self, id: Uuid) -> Result<()>;
    fn stats(&self) -> Result<BrainStats>;
    fn consolidate(&mut self) -> Result<ConsolidationReport>;
    fn tick(&mut self) -> Result<()>;   // interaction tick
}
```

---

## 9. CLI Commands & MCP Tools

### 8.1 CLI

```bash
# Encoding
membrain remember "Today I learned Rust lifetimes is actually very simple"
membrain remember "Encountered NullPointer bug at 3am" --emotion "-0.7,0.8"
membrain remember "Context is: building membrain" --context "coding session"

# Retrieval
membrain recall "Rust lifetimes"
membrain recall "recently encountered bugs" --top 5
membrain recall "coding session" --context "debugging" --json
membrain recall "negative emotions" --include-emotional

# Management
membrain forget <uuid>
membrain strengthen <uuid>
membrain stats

# Brain operations
membrain consolidate
membrain export > memories.json
membrain import < memories.json

# Daemon & Server
membrain daemon start
membrain daemon stop
membrain daemon status

# MCP server (Claude Code / Cursor used)
membrain mcp

# Debug
membrain inspect <uuid>    # xem full memory details
membrain graph <uuid> # see the engram cluster of memory
membrain health            # brain health report
```

### 8.2 MCP Tools (Claude Code / Cursor)

```
remember(content, context?, emotional_valence?, emotional_arousal?, source?)
  → { id: uuid, strength: f32, engram_id?: uuid }

recall(query, context?, top_k?, min_strength?, include_archived?)
  → { memories: [...], engram?: {...}, path: "fast"|"slow"|"partial" }

forget(id)
  → { success: bool }

strengthen(id)
  → { new_strength: f32 }

consolidate()
  → { migrated: n, pruned: n, engrams_updated: n }

stats()
  → { total: n, hot: n, cold: n, avg_strength: f32, top_engrams: [...] }

inspect(id)
  → full Memory object

graph(id)
  → { engram: {...}, related_memories: [...], cluster_size: n }
```

### 8.3 JSON-RPC Interface (Unix Socket)

```json
// Request
{"jsonrpc":"2.0","id":1,"method":"remember","params":{"content":"...","context":"..."}}

// Response
{"jsonrpc":"2.0","id":1,"result":{"id":"uuid","strength":0.72,"engram_id":"uuid"}}

// Recall request
{"jsonrpc":"2.0","id":2,"method":"recall","params":{"query":"...","top_k":5}}

// Recall response
{"jsonrpc":"2.0","id":2,"result":{
  "memories":[{"id":"...","content":"...","strength":0.9,"score":0.87}],
  "path":"slow",
  "engram":{"id":"...","member_count":12}
}}
```

---

## 10. Milestones

### Milestone 1 — Foundation (Core Schema + Storage)
```
□ Workspace setup: membrain-core, membrain-cli
□ Memory struct with full fields
□ MemoryKind, MemoryState enums
□ hot.db schema: memories + engrams + engram_members
□ cold.db schema: cold_memories
□ rusqlite + WAL mode integration
□ usearch HNSW hot_index (in-memory, int8)
□ usearch mmap cold_index (disk-backed, int8)
□ fastembed-rs integration (all-MiniLM-L6-v2)
□ LruCache Tier1 (512 entries)
□ LruCache embedding cache (1000 entries, xxhash64 key)
□ Interaction counter (replaces real-time clock)
□ BrainStore trait definition
□ effective_strength(base, stability, Δtick) lazy formula

Acceptance: INSERT memory → usearch.add → Tier1 cache → verify 3-tier lookup chain
```

### Milestone 2 — Encoding Pipeline + Optimizations
```
□ embedding_cache.get_or_embed() with LruCache + xxhash64
□ embed(content) → content_vec via fastembed-rs
□ embed(context) → context_vec
□ novelty_score = 1 - max_cosine_sim(content_vec, tier2_nearest)
□ attention_score gating (< threshold → discard)
□ emotional_tag: {valence, arousal} → bypass_decay, strength_multiplier
□ initial_strength formula: f(novelty, attention, emotion)
□ state = Labile on create
□ lazy base_strength + last_accessed_tick stored (not computed strength)
□ SQL pre-filter: effective_strength WHERE clause with Δtick
□ Working memory: LruCache<7> + attention + eviction to hot_store
□ Source tagging (CLI, MCP, Rust embed)

Acceptance: 1000 encodes → verify embedding cache hit rate >80% → verify strength distribution
           → pre-filter returns <5000 candidates from 100k memories
```

### Milestone 3 — LTP/LTD Lazy Decay Engine
```
□ effective_strength(m, now_tick) → O(1) lazy computation
□ on_recall(): base_strength += LTP_DELTA, stability += INCREMENT, update tick
□ Ebbinghaus: R = e^(-Δtick/stability) — computed on-demand only
□ emotional bypass: arousal > θ && |valence| > θ → bypass_decay = true
□ interaction_count global ticker (atomic u64)
□ Periodic prune job (NOT per-tick): scan WHERE effective_strength < MIN
□ Archive table: soft delete, not hard delete
□ Interference check: SQL query similar via usearch → penalty on older
□ Stats: avg_effective_strength, archive_rate

Benchmark: 1M memories idle → decay_tick() = 0ms overhead
           recall 1 memory → O(1) effective_strength, <1ms total

Acceptance: 1000 memories → 100 ticks → verify Ebbinghaus curve matches R=e^(-t/S)
           → verify emotional bypass → verify 0ms idle overhead
```

### Milestone 4 — 3-Tier Retrieval Engine
```
□ Tier1: LruCache<ContentHash, MemoryRef> lookup → <0.1ms
□ Tier2: SQL pre-filter (effective_strength > MIN, LIMIT 5000)
         → usearch HNSW hot_index.search(int8 query, top=20)
         → float32 rescore top-20 from SQLite
         → context_vec re-rank: 0.7*content + 0.3*context
□ Tier3: usearch mmap cold_index search (if Tier2 misses)
□ Escalation logic: Tier1 → Tier2 → Tier3
□ Tier1 cache updated after every Tier2/Tier3 hit
□ Engram expand: top_hit.engram_id → BFS petgraph depth=3
□ Fragment reconstruction
□ Partial retrieval (tip-of-tongue): returns fragments if no full match
□ on_recall: LTP + labile trigger + Tier1 update

Benchmark: Tier1 hit <0.1ms, Tier2 <5ms at 50k, Tier3 <50ms at 1M

Acceptance: recall partial cue → verify 3-tier escalation → verify cluster returned
           → benchmark latency at 50k and 1M memories
```

### Milestone 5 — Reconsolidation
```
□ Labile state tracking after each recall
□ reconsolidation_window(age): age-dependent formula
□ pending_update storage
□ reconsolidation_tick(): apply update → re-embed → strength bonus
□ State machine: Labile → Stable
□ Age-dependent resistance test

Acceptance: recall memory → modify → wait window → verify content updated → verify strength bonus
```

### Milestone 6 — Consolidation Engine
```
□ NREM equivalent: score hot memories → migrate strong → cold_store
□ Cold store insert: compress content → re-embed → store
□ REM equivalent: process emotional queue → reduce arousal → cross-links
□ Homeostasis: bulk_scale(factor) khi total_strength > MAX_LOAD
□ Pressure threshold: hot_store.len() > HOT_CAPACITY → trigger
□ Engram update sau migration
□ ConsolidationReport struct

Acceptance: fill hot_store → trigger consolidation → verify cold migration → verify homeostasis
```

### Milestone 7 — Engram Graph
```
□ petgraph integration (DiGraph<MemoryId, EdgeWeight>)
□ Engram builder: cluster by similarity threshold
□ Centroid update khi add member
□ BFS traversal cho associative recall
□ Engram merge khi overlap > threshold
□ Serialize/deserialize engram graph into hot.db
□ Graph stats: n_engrams, avg_cluster_size, density

Acceptance: encode 500 related memories → verify clusters formed → recall with cue → verify cluster expansion
```

### Milestone 8 — Active Forgetting + Interference
```
□ Forgetting engine: decay pruning, interference resolution, predictive pruning
□ Proactive interference: old memory → boost retrieval_difficulty of similar new
□ Retroactive interference: new memory → weaken similar old
□ Capacity management: archive bottom percentile khi > SOFT_CAP
□ Archive table: memories are not completely deleted
□ Non-predictive decay: access_count == 0 && age > threshold

Acceptance: create interfering pairs → verify penalties → capacity test → verify archive not delete
```

### Milestone 9 — Daemon + IPC + MCP
```
□ Tokio async daemon: tokio::main + background task spawning
□ Unix socket server: JSON-RPC 2.0 handler
□ All BrainStore methods exposed via JSON-RPC
□ MCP server: rmcp integration, stdio transport
□ MCP tools: remember, recall, forget, strengthen, stats, consolidate
□ Daemon lifecycle: start/stop/status
□ membrain daemon subcommand
□ membrain mcp subcommand
□ Python client (1 file)
□ Node client (1 file)

Acceptance: start daemon → call from Python → call from Node → MCP in Claude Code → verify all paths
```

### Milestone 10 — CLI Polish + Production
```
□ All CLI commands: remember, recall, forget, strengthen, stats, inspect, graph, health
□ --json flag cho machine-readable output
□ Config file: ~/.membrain/config.toml (model, thresholds, capacity)
□ export / import JSON
□ Graceful shutdown (save state)
□ Error messages are clear
□ membrain health: brain health report
□ README + usage examples
□ Benchmarks: encoding speed, recall latency, capacity test
□ Cross-platform: Linux, macOS, Windows (WSL)

Acceptance: full integration test suite → all benchmarks pass → README complete
```

---

## 11. Acceptance Checklist

### Biological Accuracy
```
□ LTP: recall → strength increases according to the correct formula
□ LTD: non-use → Ebbinghaus decay correct curve
□ Emotional bypass: high arousal+valence → no decay
□ Reconsolidation: recall → labile → update → re-stable
□ Dual-path: fast familiarity + slow episodic recall
□ Engram clustering: similar memories grouped
□ Associative recall: partial cue → cluster
□ Interference: similar memories affect each other
□ Consolidation: episodic → semantic migration
□ Active forgetting: non-predictive → archived
□ Working memory: 7-slot capacity limit
```

### Performance
```
□ Tier1 retrieval: <0.1ms (LruCache hit)
□ Tier2 retrieval: <5ms (usearch HNSW, 50k memories)
□ Tier3 retrieval: <50ms (usearch mmap, 1M memories)
□ Encode: <1ms (embedding cache hit), <10ms (cache miss)
□ Lazy decay: 0ms idle overhead, O(1) per recall
□ Consolidation: non-blocking (async background, 0ms impact on retrieval)
□ Pre-filter: reduces search space 200x (1M → 5000 candidates)
□ 1M memories: recall Tier3 <50ms
□ 50k hot memories: RSS <150MB (usearch HNSW in-memory)
□ Concurrent reads: N readers + 1 writer (SQLite WAL)
□ Embedding cache hit rate: >80% in steady state
□ int8 quantization: <2% accuracy loss vs float32
```

### Integration
```
□ CLI: all commands work
□ MCP: Claude Code / Cursor tools work
□ Python: 1-file client works
□ Node: 1-file client works
□ Rust embed: membrain-core as library dependency
□ Daemon: start/stop/status active
□ Standalone mode: operates without a daemon
□ Cross-platform: Linux (WSL), macOS
```

### Quality
```
□ Typed error enums (thiserror)
□ No unwrap() in production paths
□ Atomic writes (SQLite transactions)
□ Graceful degradation: daemon down → standalone fallback
□ Data integrity: no memory loss on crash (WAL)
□ Config validation on startup
```

---

## Constants (Tunable)

```toml
# ~/.membrain/config.toml

[brain]
model = "all-MiniLM-L6-v2" # or "nomic-embed-text-v1.5"
hot_capacity      = 50_000               # max memories in hot HNSW index
soft_cap = 1_000_000 # trigger archive bottom % when overtaking
embedding_cache   = 1_000               # LruCache entries cho embedding
tier1_cache       = 512                 # LruCache entries Tier1 fast path

[vector]
dimensions = 384 # must match model
quantization      = "int8"              # int8 | f16 | f32
rescore_top_k     = 20                  # float32 rescore sau int8 search
pre_filter_limit = 5_000 # SQL pre-filter cap before HNSW
hnsw_ef_construct = 200                # HNSW build quality
hnsw_ef_search    = 50                 # HNSW search quality

[ltp_ltd]
ltp_delta              = 0.1
stability_increment    = 0.2
max_strength           = 1.0
min_strength           = 0.05

[emotional]
amygdala_arousal_threshold = 0.6
amygdala_valence_threshold = 0.5

[consolidation]
nrem_importance_threshold  = 0.4
homeostasis_factor         = 0.9
homeostasis_trigger_load   = 0.85

[reconsolidation]
base_window_interactions   = 50
reconsolidation_bonus      = 0.05

[interference]
similarity_min     = 0.7
similarity_max     = 0.99
interference_penalty = 0.05

[retrieval]
tier1_confidence_threshold = 0.9
tier2_confidence_threshold = 0.8
cluster_expansion_depth    = 3
min_edge_weight            = 0.5
```

---

*membrain — port of human brain mechanism to AI agent. Build order: M1 → M10.*
*Stack: Rust + Tokio + SQLite WAL + FTS5 + USearch + fastembed + local reranker + petgraph*  
*Targets: Tier1 <0.1ms | Tier2 <5ms | Tier3 <50ms | Scale: unlimited*


<!-- SOURCE: PLAN_part1.md -->

### Source Snapshot — Part 1
#### Part 1 of 6: Vision · Problem Statement · Human Brain Deep Dive

> **Vision**: A memory system that ports every known mechanism of the human brain into a fast,
> local, offline-first Rust library — giving AI agents the same memory architecture that
> evolution spent 500 million years perfecting.
>
> **Performance targets**: Tier1 <0.1ms | Tier2 <5ms | Tier3 <50ms | Encode <10ms
> **Scale target**: Unlimited — usearch mmap + SQLite, TB-scale on disk
> **Stack**: Rust + Tokio + SQLite WAL + FTS5 + USearch + fastembed + local reranker + petgraph
> **Integration**: CLI + MCP (Claude Code / Cursor) + Unix socket IPC + Python/Node clients

---

### Snapshot TOC (Full Document — 6 Parts)

#### Part 1 (this file)
1. [Problem Statement — Why membrain Exists](#1-problem-statement)
2. [Human Brain — Complete Analysis](#2-human-brain-complete-analysis)
   - 2.1 Core Properties
   - 2.2 Brain Regions & Functions
   - 2.3 Memory Types Taxonomy
   - 2.4 Encoding — How Memories Form
   - 2.5 LTP / LTD — The Physical Basis of Learning
   - 2.6 Forgetting Curve — Ebbinghaus
   - 2.7 Consolidation — Stabilization Process
   - 2.8 Sleep — The Brain's Consolidation Engine
   - 2.9 Reconsolidation — Memory Updates on Recall
   - 2.10 Active Forgetting — Intelligent Pruning
   - 2.11 Engrams — Physical Memory Traces
   - 2.12 Working Memory — The Conscious Workspace
   - 2.13 Emotional Memory — The Amygdala Effect
   - 2.14 Interference — Memory Conflicts
   - 2.15 Pattern Completion — Recall from Partial Cues
   - 2.16 Encoding Specificity — Context Dependency

#### Part 2
3. [Gap Analysis — Human Brain vs Current AI Memory Systems](#3-gap-analysis)
4. [Porting the Brain — Mechanism by Mechanism](#4-porting-the-brain)

#### Part 3
5. [Architecture Overview](#5-architecture-overview)
6. [Performance — Bottlenecks & Optimization Stack](#6-performance)

#### Part 4
7. [Techstack — Analysis & Rationale](#7-techstack)
8. [Data Schema — Full SQL + Rust Structs](#8-data-schema)

#### Part 5
9. [CLI Commands & MCP Tools](#9-cli-and-mcp)
10. [Top 10 Feature Extensions](#10-feature-extensions)
11. [Workspace Structure](#11-workspace-structure)

#### Part 6
12. [Implementation Milestones](#12-milestones)
13. [Acceptance Checklist](#13-acceptance-checklist)
14. [Tunable Constants](#14-constants)
15. [Algorithm Reference](#15-algorithm-reference)

---

## 1. Problem Statement

### 1.1 The Core Problem with AI Agents Today

Every serious AI agent deployment hits the same wall: **memory degrades with scale**.

The more interactions an agent has, the more information accumulates, and the worse retrieval
becomes. You cannot simply add "remember everything" — the signal-to-noise ratio collapses.
Context windows fill up. Retrieval becomes slow. The agent starts confusing old irrelevant
information with current relevant information.

The human brain solved this problem over hundreds of millions of years of evolution. It does not
remember everything equally. It has a sophisticated system for:

- Deciding what is worth encoding in the first place (attention gating)
- Making important things stronger over time (LTP)
- Letting unimportant things decay gracefully (LTD + Ebbinghaus)
- Periodically reorganizing memories from fast-access to deep-storage (consolidation)
- Linking related memories into retrievable clusters (engrams)
- Updating memories when new information conflicts with old (reconsolidation)
- Actively pruning noise to preserve signal (active forgetting)
- Recovering full memories from partial cues (pattern completion)
- Tagging memories with emotional salience (amygdala)
- Keeping a small but powerful working buffer for active reasoning (prefrontal cortex)

No existing AI memory system implements more than 2-3 of these mechanisms. membrain implements
all of them.

---

### 1.2 What Existing Systems Do

| System | Core Approach | Mechanisms Present | Critical Missing |
|--------|--------------|-------------------|-----------------|
| **MemGPT / Letta** | OS virtual memory metaphor: page memories in/out of context window based on relevance scoring | Tiered storage concept, some retrieval prioritization | No LTP/LTD, no decay, no emotional weighting, no consolidation, no engrams, no interference |
| **Mem0** | Two-phase: LLM extracts facts → stores in graph + vector DB | Graph relationships, semantic search | No strength dynamics, no reconsolidation, slow (LLM in critical path), no biological accuracy |
| **LangMem** | Relevance scoring + memory lifecycle states | Lifecycle management, basic decay concept | No engram clusters, no interference handling, no pattern completion, no working memory |
| **OpenAI Memory** | Simple key-value extraction from conversations | Persistent facts | Entirely primitive: no dynamics, no associations, no decay, no structure |
| **Zep** | Timeline-aware memory with entity extraction | Temporal ordering, entity graph | No biological mechanisms, no strength dynamics, no forgetting curve |
| **Cognee** | Knowledge graph construction from documents | Rich graph structure | Static graph, no dynamics, no consolidation, no decay |

### 1.3 What They All Lack

```
MISSING: LTP / LTD
  Memory strength never changes dynamically.
  A memory encoded once stays equally strong forever (or until deleted).
  In the human brain, synaptic strength is constantly modulated by use.
  Consequence: AI memories accumulate noise — old irrelevant memories compete
               equally with recent relevant ones.

MISSING: Ebbinghaus Forgetting Curve
  Nothing decays according to actual forgetting dynamics.
  Either a memory exists or it doesn't.
  In the human brain, retention follows R(t) = e^(-t/S) where S (stability)
  increases with each successful recall.
  Consequence: No natural noise reduction. Store grows unboundedly.

MISSING: Reconsolidation
  When a memory is recalled, it is never updated to incorporate new context.
  In the human brain, every recall destabilizes the memory (labile state),
  allowing it to be updated before re-stabilizing.
  Consequence: Agents cannot update beliefs. Stale information persists.

MISSING: Sleep / Consolidation Cycle
  No episodic → semantic conversion.
  In the human brain, NREM sleep replays episodic memories and gradually
  transfers their essence to stable semantic storage in the neocortex.
  Consequence: Raw episodic events accumulate. No abstraction. No compression.

MISSING: Emotional Tagging
  Every memory is weighted equally.
  In the human brain, the amygdala tags memories with emotional salience,
  causing high-arousal events to be remembered far longer and recalled first.
  Consequence: Agent cannot prioritize high-stakes memories.

MISSING: Engram Clusters
  No associative structure. Each memory is an island.
  In the human brain, related memories form physical clusters (engrams)
  that can be retrieved together from a partial cue.
  Consequence: No associative recall. "What else do I know about X?" fails.

MISSING: Interference Handling
  Memories do not interact with each other.
  In the human brain, similar memories interfere (proactive and retroactive),
  causing natural disambiguation and prioritization of more relevant traces.
  Consequence: Similar contradictory memories coexist without resolution.

MISSING: Active Forgetting
  Systems only accumulate. They never prune intelligently.
  In the human brain, active forgetting removes non-predictive information
  during sleep homeostasis, optimizing the signal-to-noise ratio.
  Consequence: Retrieval quality degrades as noise accumulates.

MISSING: Dual-Path Retrieval (Fast / Slow)
  Every query goes through the same retrieval pipeline.
  In the human brain, pattern recognition is ~13ms (familiarity, neocortex),
  while episodic recall is 100-500ms (hippocampus deliberate search).
  Consequence: No fast familiarity check. Everything is slow and expensive.

MISSING: Working Memory Layer
  No concept of a limited active workspace.
  In the human brain, the prefrontal cortex maintains 7±2 items in active
  working memory for immediate use, with executive attention control.
  Consequence: No capacity limit. No prioritization of "what I'm using right now."
```

### 1.4 The Result

```
Agent using current memory systems:
  tick 1:     1 memory     → recall perfect
  tick 100:   100 memories → recall good
  tick 1000:  1000 memories → recall degrading
  tick 10000: 10000 memories → recall slow, noisy, unreliable
  tick 100000: system essentially broken

Agent using membrain:
  tick 1:     1 memory     → recall perfect
  tick 100:   ~80 memories (20 decayed) → recall perfect
  tick 1000:  ~400 memories (600 decayed/consolidated) → recall perfect
  tick 10000: ~1500 memories (engram-organized, noise pruned) → recall fast + perfect
  tick 100000: scales indefinitely — brain-like performance at brain-like scale
```

---

## 2. Human Brain — Complete Analysis

### 2.1 Core Properties

The human brain is the most sophisticated information processing system known to science.
Understanding its properties is essential for porting them to membrain.

```
PROPERTY 1: EFFECTIVELY UNLIMITED CAPACITY

  Neurons:              ~86 billion
  Synaptic connections: ~100 trillion
  Estimated capacity:   ~2.5 petabytes (Salk Institute, 2016)
  
  How: Capacity is not limited by number of neurons but by synaptic connection weights.
       Each synapse can hold approximately 4.7 bits of information (26 distinguishable
       strengths experimentally measured). 100 trillion synapses × 4.7 bits = 2.5 PB.
  
  membrain port:
    usearch mmap (disk-backed, unlimited scale)
    SQLite (TB-scale on disk)
    No architectural limit — constrained only by available disk space
    At 384 dims × int8 = 384 bytes/vector: 1TB disk = ~2.7 billion memories
```

```
PROPERTY 2: EXTREMELY FAST RETRIEVAL AT SCALE

  Familiarity recognition:  ~13ms  (neocortex pattern matching)
  Simple recall:            ~100ms (hippocampus fast route)
  Deliberate recall:        ~500ms (hippocampus slow route, reconstruction)
  
  Key insight: The brain has MULTIPLE retrieval pathways operating at different speeds.
               Fast path: "have I seen this before?" — very cheap familiarity check.
               Slow path: "what do I know about this?" — full reconstruction.
  
  membrain port:
    Tier 1 (LruCache):        <0.1ms  — familiarity, recently accessed
    Tier 2 (HNSW in-memory):  <5ms    — hot episodic, 50k vectors
    Tier 3 (HNSW mmap):       <50ms   — cold semantic, unlimited
    Three-tier escalation mirrors brain's multi-speed architecture
```

```
PROPERTY 3: GRACEFUL, INTELLIGENT FORGETTING

  The brain does NOT remember everything equally.
  It does NOT forget randomly.
  
  Forgetting is ACTIVE and PURPOSEFUL:
    - Synaptic homeostasis: globally scale down all connections during sleep
      to prevent saturation (Tononi's synaptic homeostasis hypothesis)
    - Active forgetting via RAC1 signaling: specific memories pruned
      based on predictive value
    - Interference-based forgetting: similar memories compete,
      less-accessed one weakens
  
  Result: High signal-to-noise ratio maintained indefinitely.
  
  membrain port:
    Ebbinghaus decay with stability parameter
    Emotional bypass for high-salience memories
    Active forgetting engine (interference + predictive pruning)
    Homeostasis: bulk strength scaling when store overloads
```

```
PROPERTY 4: ASSOCIATIVE RECALL FROM PARTIAL CUES

  Human memory is fundamentally ASSOCIATIVE, not lookup-by-ID.
  You do not need the exact memory address — any partial fragment
  of the original encoding can trigger full reconstruction.
  
  Mechanism: hippocampal CA3 region acts as an autoassociative network.
             Partial input pattern → pattern completion via recurrent connections
             → full pattern activates → spreads to related engrams.
  
  Example: smell of perfume → entire episodic memory from 10 years ago.
           Single word → entire conceptual cluster.
  
  membrain port:
    HNSW vector search (semantic similarity = partial cue matching)
    Engram BFS traversal (pattern completion → cluster expansion)
    Context embedding stored alongside content (encoding specificity)
```

```
PROPERTY 5: DYNAMIC SYNAPTIC STRENGTH

  Synapse strength is CONTINUOUSLY MODULATED:
  
  Long-Term Potentiation (LTP):
    "Neurons that fire together, wire together" (Hebb, 1949)
    Repeated co-activation → stronger synaptic connection
    NMDA receptor activation → Ca2+ influx → AMPA receptor insertion
    → synapse physically stronger
  
  Long-Term Depression (LTD):
    Non-use → gradual weakening
    Low-frequency stimulation → phosphatase activation → AMPA removal
    → synapse physically weaker
  
  Critical: LTD is as important as LTP. Without LTD, all synapses
  eventually saturate at maximum strength → no more learning possible.
  
  membrain port:
    on_recall(): base_strength += LTP_DELTA, stability += INCREMENT
    Lazy Ebbinghaus: effective_strength = base × e^(-Δtick/stability)
    Both mechanisms required — LTP on recall, LTD via decay
```

```
PROPERTY 6: CONTEXT-DEPENDENT RETRIEVAL (ENCODING SPECIFICITY)

  The Encoding Specificity Principle (Tulving & Thomson, 1973):
  "Memory retrieval is most effective when the retrieval cues match
  the encoding context."
  
  Examples:
    - Divers learned word lists underwater recalled better underwater than on land
    - State-dependent memory: learned drunk → recalled better when drunk
    - Mood congruence: happy memories recalled better when happy
  
  Mechanism: context features are co-encoded with content.
             Retrieval = content match AND context match.
  
  membrain port:
    context_embedding stored alongside content_embedding
    Retrieval score = 0.7 × content_sim + 0.3 × context_sim
    Context switches between tasks naturally boost relevant memories
```

---

### 2.2 Brain Regions & Functions

#### 2.2.1 Hippocampus

```
LOCATION: Medial temporal lobe (both hemispheres)
SHAPE:    Seahorse-shaped (hence "hippocampus" = Greek for seahorse)
SIZE:     ~3.5cm long, bilateral

PRIMARY FUNCTIONS:
  ┌─────────────────────────────────────────────────────────────┐
  │ 1. EPISODIC MEMORY FORMATION                                │
  │    Binds together: what + where + when → episodic memory   │
  │    Without hippocampus: cannot form new episodic memories  │
  │    (anterograde amnesia — H.M. case study)                 │
  │                                                             │
  │ 2. SPATIAL MEMORY (COGNITIVE MAPPING)                      │
  │    Place cells: fire when in specific location             │
  │    Grid cells (entorhinal): coordinate system              │
  │    Together: cognitive map of environment                  │
  │                                                             │
  │ 3. PATTERN SEPARATION                                       │
  │    Dentate Gyrus: makes similar inputs very different      │
  │    Prevents interference between similar memories          │
  │    High neurogenesis rate supports this function           │
  │                                                             │
  │ 4. PATTERN COMPLETION (CA3 region)                         │
  │    Autoassociative network with recurrent connections      │
  │    Partial cue → reconstructs full memory pattern         │
  │    Foundation of "tip of tongue" and associative recall    │
  │                                                             │
  │ 5. INDEX FUNCTION                                          │
  │    Does NOT store content directly                         │
  │    Stores POINTERS to content distributed in neocortex    │
  │    Like a database index — fast, small, disposable        │
  └─────────────────────────────────────────────────────────────┘

EVIDENCE FOR INDEX FUNCTION:
  - Older memories (decades) survive hippocampal damage
  - Recent memories (weeks) do not survive
  - After consolidation: memory becomes hippocampus-independent
  - Gradient: older = more neocortex-independent

membrain PORT:
  hot_store (SQLite WAL):
    - Episodic memory storage (recent, labile)
    - Fast index to content vectors
    - usearch HNSW hot_index: 50k vectors, O(log n) search
    - Pattern completion: HNSW search → top hit → engram BFS
    - Pattern separation: novelty score (distance from existing)
    - Temporal ordering: created_tick, last_accessed_tick
  
  The hot_store IS the hippocampus:
    - Temporary (pressure-triggered migration to cold)
    - Index function (pointers, not primary content store)
    - Episodic (timestamped, context-bound)
    - Fast (in-memory HNSW, <5ms)
```

#### 2.2.2 Neocortex

```
LOCATION: Outer surface of cerebral hemispheres
SIZE:     ~2,500 cm² surface area (highly folded)
LAYERS:   6 cortical layers, each with different neuron types
THICKNESS: 2-4mm

PRIMARY FUNCTIONS:
  ┌─────────────────────────────────────────────────────────────┐
  │ 1. LONG-TERM SEMANTIC MEMORY STORAGE                       │
  │    Stores the actual content of consolidated memories      │
  │    Distributed across regions by modality:                 │
  │    - Visual: occipital + temporal cortex                   │
  │    - Auditory: temporal cortex                             │
  │    - Language: Broca + Wernicke areas                     │
  │    - Concepts: prefrontal + temporal association areas    │
  │                                                             │
  │ 2. SLOW INTEGRATION (COMPLEMENTARY LEARNING SYSTEMS)       │
  │    Integrates new information slowly over many exposures   │
  │    Avoids catastrophic forgetting by gradual update        │
  │    Requires hippocampal replay during sleep for updates    │
  │                                                             │
  │ 3. PATTERN RECOGNITION (FAST PATH)                         │
  │    Feedforward processing of familiar patterns             │
  │    ~13ms for recognition of familiar stimuli               │
  │    No hippocampus needed for familiar stimuli              │
  │                                                             │
  │ 4. CORTICAL COLUMNS: UNIT OF PROCESSING                   │
  │    ~2mm × ~2mm columns, ~100 neurons each                 │
  │    Hierarchical feature detection                          │
  │    Similar memories stored in nearby columns              │
  └─────────────────────────────────────────────────────────────┘

KEY PROPERTY: Cortex is SLOW to learn but STABLE once learned.
  - Catastrophic forgetting avoided by slow integration
  - But: new learning takes many repetitions / sleep cycles
  - Trade-off: stability vs plasticity

membrain PORT:
  cold_store (SQLite + usearch mmap):
    - Consolidated semantic memories (stable, compressed)
    - Unlimited scale via mmap (like cortex's vast surface area)
    - int8 quantized vectors (compressed representation)
    - zstd-compressed content (slow to read but rarely needed)
    - OS page cache acts as warm layer (like cortical priming)
  
  The cold_store IS the neocortex:
    - Stable (requires consolidation to write)
    - Content-storing (not just pointers)
    - Slow to write (consolidation overhead)
    - Fast to read known patterns (OS page cache + mmap)
    - Unlimited capacity (disk-bounded)
```

#### 2.2.3 Amygdala

```
LOCATION: Temporal lobe, anterior to hippocampus
SHAPE:    Almond-shaped (Greek: amygdale = almond)
SIZE:     ~1.5cm, bilateral

PRIMARY FUNCTIONS:
  ┌─────────────────────────────────────────────────────────────┐
  │ 1. EMOTIONAL MEMORY TAGGING                                 │
  │    Tags memories with emotional significance               │
  │    High arousal → stronger, more persistent memory        │
  │    Direct input from sensory thalamus (fast path)         │
  │    And from sensory cortex (slow, processed path)         │
  │                                                             │
  │ 2. FEAR CONDITIONING                                        │
  │    Classically conditioned fear responses                  │
  │    Stimulus → threat association → defensive response      │
  │    Lesion: cannot learn fear associations                  │
  │                                                             │
  │ 3. MODULATION OF HIPPOCAMPAL CONSOLIDATION                 │
  │    High emotional arousal → amygdala releases NE          │
  │    Norepinephrine → enhances hippocampal LTP              │
  │    → emotionally significant memories are stronger        │
  │    This is why you remember where you were on 9/11        │
  │                                                             │
  │ 4. EMOTIONAL VALENCE TAGGING                               │
  │    Positive valence: approach, reward, pleasure            │
  │    Negative valence: avoidance, threat, pain              │
  │    Neutral: neither approach nor avoidance                │
  └─────────────────────────────────────────────────────────────┘

TWO-DIMENSIONAL EMOTIONAL SPACE:
  Valence axis:  Negative ←————→ Positive   (-1.0 to +1.0)
  Arousal axis:  Calm     ←————→ Excited    (0.0 to 1.0)

  Examples:
    Terror:   valence=-0.9, arousal=0.9  → extreme negative high arousal
    Joy:      valence=+0.8, arousal=0.7  → positive high arousal
    Disgust:  valence=-0.6, arousal=0.4  → negative moderate arousal
    Serenity: valence=+0.5, arousal=0.1  → positive low arousal
    Boredom:  valence=-0.1, arousal=0.1  → slightly negative low arousal

FLASHBULB MEMORIES:
  Exceptionally high-arousal events create "flashbulb memories":
  - Extremely detailed
  - Long-lasting (potentially lifetime)
  - Resistant to interference
  - Reconsolidated on every recall (slightly updated each time)
  
  Mechanism: amygdala activation → massive NE release → maximal LTP
  In membrain: bypass_decay = true when arousal > 0.8

membrain PORT:
  EmotionalTag { valence: f32, arousal: f32 }:
    - Stored with every memory
    - Determines initial_strength multiplier
    - Controls bypass_decay (high arousal → no decay)
    - Influences consolidation priority (emotional → faster migration)
    - Affects reconsolidation window (emotional → shorter labile window)
  
  Strength multiplier formula:
    emotional_multiplier = 1.0 + (|valence| × arousal × EMOTIONAL_WEIGHT)
    EMOTIONAL_WEIGHT = 0.5 (tunable)
    
    Example: terror (valence=-0.9, arousal=0.9):
    multiplier = 1.0 + (0.9 × 0.9 × 0.5) = 1.405
    Initial strength 40.5% stronger than neutral memory
```

#### 2.2.4 Prefrontal Cortex (PFC)

```
LOCATION: Anterior portion of frontal lobe
SIZE:     ~29% of total cortex in humans (vs 17% in chimpanzees)
NOTABLE:  Largest relative to body size of any brain region in humans

PRIMARY FUNCTIONS:
  ┌─────────────────────────────────────────────────────────────┐
  │ 1. WORKING MEMORY (7±2 items)                              │
  │    Active maintenance of information for immediate use     │
  │    Miller's Law: 7 ± 2 chunks (1956)                       │
  │    Phonological loop (verbal), visuospatial sketchpad      │
  │    Central executive: coordinates, allocates attention     │
  │                                                             │
  │ 2. EXECUTIVE ATTENTION                                      │
  │    Selective attention: what to process, what to ignore    │
  │    Divided attention: multiple tasks simultaneously        │
  │    Sustained attention: maintaining focus over time        │
  │                                                             │
  │ 3. RETRIEVAL CONTROL                                        │
  │    Initiates and monitors memory retrieval                 │
  │    Resolves competition between memory traces              │
  │    Post-retrieval monitoring: "is this right?"             │
  │                                                             │
  │ 4. INHIBITION                                              │
  │    Suppresses irrelevant memories during retrieval         │
  │    Suppresses impulsive responses                          │
  │    Directly relevant: active suppression of distractors   │
  └─────────────────────────────────────────────────────────────┘

WORKING MEMORY CAPACITY:
  The "7±2" is contested — modern research suggests 4±1 "chunks"
  when chunks are truly independent.
  But: chunks can be hierarchically organized ("chunking")
       Expert chess players chunk board positions into ~7 patterns
       Each pattern contains many pieces → much more information
  
  For membrain: 7 slots is architecturally clean and biologically motivated.
  Can be configured (working_memory_capacity: 7)

DORSOLATERAL vs VENTROMEDIAL PFC:
  dlPFC: working memory, executive function, cognitive control
  vmPFC: emotional regulation, risk/reward evaluation, social cognition
  
  For membrain: vmPFC aspects → emotional_tag influences on retrieval
                dlPFC aspects → working memory + attention scoring

membrain PORT:
  WorkingMemory:
    slots: FixedVec<MemoryItem, 7>
    attention: HashMap<MemoryId, f32>
    
  LruCache<ContentHash, MemoryRef> (Tier 1):
    Simulates the fast-access, recency-biased nature of working memory
    512 entries — larger than working memory proper, acting as
    the "primed neocortex" fast path
    
  Attention scoring in encode pipeline:
    attention_score: f32 (0.0 to 1.0, caller-provided)
    Threshold gate: attention_score < 0.2 → discard (not attended to)
    Multiplier: attention_score scales initial_strength
```

#### 2.2.5 Cerebellum

```
LOCATION: Posterior and inferior to cerebral hemispheres
SIZE:     ~10% of brain volume, ~80 billion neurons (close to cerebral cortex)
NOTABLE:  Despite small size, contains more neurons than the rest of brain combined

PRIMARY FUNCTIONS:
  ┌─────────────────────────────────────────────────────────────┐
  │ 1. PROCEDURAL MEMORY (MOTOR LEARNING)                      │
  │    Motor skills: typing, cycling, playing instruments      │
  │    Not recalled consciously — executed automatically       │
  │    Lesion: movement imprecise, cannot learn new skills     │
  │                                                             │
  │ 2. TIMING AND SEQUENCE LEARNING                            │
  │    Precise timing of movements                             │
  │    Sequence of actions learned as a unit                   │
  │    Relevant: procedure = sequence of operations           │
  │                                                             │
  │ 3. CONDITIONING (eyeblink, timing)                         │
  │    Simple associative learning with precise timing         │
  │    CS (tone) → US (airpuff) → CR (eyeblink)               │
  └─────────────────────────────────────────────────────────────┘

KEY PROPERTY: Procedural memories do NOT require hippocampus.
  - Patients with hippocampal damage (amnesia) can still learn motor skills
  - They just don't remember learning them
  - H.M. improved at mirror drawing over days despite no episodic memory of practice

membrain PORT:
  procedural_store (SQLite key-value, no vector search):
    - Pattern → action mappings
    - No decay (skills don't fade)
    - No consolidation needed
    - Direct lookup: O(1) by pattern_hash
    - Not subject to interference or forgetting
    
  Use cases:
    - "When I see error pattern X, I always do Y"
    - "When working in module Z, the standard approach is W"
    - "This API always requires authentication header format F"
```

#### 2.2.6 Entorhinal Cortex

```
LOCATION: Medial temporal lobe, adjacent to hippocampus
ROLE:     Gateway — all information entering/leaving hippocampus passes through here

PRIMARY FUNCTIONS:
  ┌─────────────────────────────────────────────────────────────┐
  │ 1. CONVERGENCE ZONE                                         │
  │    Receives input from: visual, auditory, somatosensory,   │
  │    olfactory, prefrontal cortex                            │
  │    Compresses multimodal input into unified representation  │
  │                                                             │
  │ 2. HIPPOCAMPAL GATEWAY                                     │
  │    Projects to dentate gyrus (via perforant path)          │
  │    Receives output from CA1                                │
  │    Controls what enters long-term memory                   │
  │                                                             │
  │ 3. GRID CELLS                                              │
  │    Hexagonal firing grid — abstract spatial coordinates   │
  │    May generalize to non-spatial "conceptual space"        │
  │    Foundation of cognitive mapping beyond physical space   │
  └─────────────────────────────────────────────────────────────┘

membrain PORT:
  encoding_pipeline:
    - Preprocessing before hot_store insert
    - Embedding computation (fastembed-rs)
    - Embedding cache (LruCache — avoids re-encoding same content)
    - Novelty scoring (is this new information?)
    - Attention gating (is this worth encoding?)
    - Emotional tagging (how significant is this?)
    - Context embedding (what context am I in?)
    - Initial strength calculation
    All of this mirrors the entorhinal cortex's role as
    "quality control" before hippocampal storage.
```

#### 2.2.7 Basal Ganglia

```
LOCATION: Subcortical structures (striatum, globus pallidus, substantia nigra, subthalamic nucleus)
ROLE:     Habit formation, reward-based learning, action selection

PRIMARY FUNCTIONS:
  ┌─────────────────────────────────────────────────────────────┐
  │ 1. HABIT LEARNING                                           │
  │    Converts goal-directed behavior → automatic habits      │
  │    Stimulus → response chains stored as units              │
  │    Dopamine: reward prediction error signal                │
  │                                                             │
  │ 2. ACTION SELECTION                                         │
  │    "What to do next" given current state                   │
  │    Competes between possible actions                       │
  │    Selects winner, suppresses alternatives                 │
  │                                                             │
  │ 3. SEQUENCE LEARNING                                        │
  │    Serial order of actions                                  │
  │    With cerebellum: complete action sequences automated    │
  └─────────────────────────────────────────────────────────────┘

membrain PORT:
  procedural_store (shared with cerebellum port):
    - Habit = pattern → action mapping
    - Access: direct hash lookup, no vector search
    - "This input pattern always leads to this action"
    
  Future (agent-alive): dopamine-like signal
    - Reward feedback → strengthens procedural pathways
    - Not in initial membrain scope
```

---

### 2.3 Memory Types Taxonomy

```
LONG-TERM MEMORY
│
├── DECLARATIVE (Explicit) — consciously recalled
│   │
│   ├── EPISODIC
│   │   Definition: Memories of specific events with temporal + contextual tags
│   │   "What happened to me at time T in place P"
│   │   Brain region: Hippocampus (encoding), Neocortex (storage after consolidation)
│   │   Properties:
│   │     - Highly context-dependent (encoding specificity)
│   │     - Subject to decay and interference
│   │     - Can be updated (reconsolidation)
│   │     - Temporal ordering critical
│   │     - First formed, first forgotten (during hippocampal damage)
│   │   Examples:
│   │     "I fixed that race condition at tick 847"
│   │     "During the auth module refactor, I found the JWT expiry bug"
│   │     "Last time I deployed to prod, the migration failed"
│   │   membrain: hot_store, Labile state, decay-subject, context_embedding
│   │
│   └── SEMANTIC
│       Definition: General knowledge, facts, concepts — no time/place tag
│       "What I know about the world"
│       Brain region: Neocortex (distributed, modality-specific)
│       Properties:
│         - Relatively stable (low decay rate)
│         - Less context-dependent
│         - Updates slowly (requires many repetitions)
│         - Not tied to a specific event
│         - Survives hippocampal damage better than episodic
│       Examples:
│         "Rust's borrow checker prevents data races"
│         "This codebase uses PostgreSQL with SQLx"
│         "The payments module requires PCI compliance"
│       membrain: cold_store, Stable state, high stability value, low decay
│
└── NON-DECLARATIVE (Implicit) — not consciously recalled
    │
    ├── PROCEDURAL
    │   Definition: Motor skills, cognitive skills, habits
    │   Brain region: Basal ganglia + Cerebellum
    │   Properties:
    │     - Does not decay with non-use (unlike declarative)
    │     - Cannot be easily verbalized
    │     - Acquired through repetition (slow)
    │     - Executed automatically without conscious access
    │   Examples (for agent):
    │     "When I see a SQL query without parameterization → flag injection risk"
    │     "When debugging async code → check for missing await first"
    │     "In this repo → always run tests before committing"
    │   membrain: procedural_store, no decay, hash lookup
    │
    ├── PRIMING
    │   Definition: Prior exposure facilitates later processing of related items
    │   Brain region: Neocortex (modality-specific)
    │   Properties:
    │     - Unconscious
    │     - Short-lived (hours to days)
    │     - Speeds up recognition of primed items
    │   membrain port: Tier1 cache + spotlight/priming mode
    │     "Loading context about X → boosts recall of X-related memories"
    │
    ├── CLASSICAL CONDITIONING
    │   Definition: Stimulus-response associations (Pavlov)
    │   Brain region: Cerebellum (timing), Amygdala (fear conditioning)
    │   membrain port: pattern → response in procedural_store
    │
    └── EMOTIONAL MEMORY
        Definition: Emotional responses tied to stimuli/events
        Brain region: Amygdala
        Properties:
          - Extremely durable (some last lifetime)
          - High interference resistance
          - Flashbulb quality for extreme events
        membrain port: emotional_tag + bypass_decay mechanism
```

---

### 2.4 Encoding — How Memories Form

```
THE ENCODING PIPELINE IN THE HUMAN BRAIN:

STAGE 1: Sensory Input
  All information enters through sensory channels:
    Vision:   retina → optic nerve → primary visual cortex (V1) → association areas
    Auditory: cochlea → auditory nerve → primary auditory cortex → Wernicke's
    Touch:    receptors → spinal cord → somatosensory cortex
    Smell:    olfactory bulb → DIRECTLY to amygdala + hippocampus (unique!)
    
  Note: Smell is special — it bypasses the thalamus and reaches memory structures
  directly. This is why smell is the most powerful memory cue.

STAGE 2: Sensory Register (Iconic/Echoic Memory)
  Duration: <500ms
  Capacity: large but decays extremely fast
  Function: brief persistence that allows attention to select relevant features
  
  If attention is NOT directed here: gone forever in <500ms
  If attention IS directed: passes to working memory

STAGE 3: Attention Filter
  THE most critical gating step.
  Resources: limited — only a fraction of sensory input is attended to
  
  What gets attended to:
    - Novel stimuli (novelty detection = automatic attention capture)
    - Emotionally significant stimuli (amygdala fast-path)
    - Goal-relevant stimuli (top-down attention from PFC)
    - High-intensity stimuli (pain, loud noise, bright light)
  
  What gets filtered:
    - Familiar, non-changing background stimuli
    - Stimuli irrelevant to current task
    - Low-intensity stimuli not matching any of above

STAGE 4: Working Memory Encoding
  Duration: 20-30 seconds without rehearsal
  Capacity: 7 ± 2 chunks (Miller, 1956) / 4 ± 1 independent items (Cowan, 2001)
  
  Working memory systems:
    Phonological loop:     verbal/acoustic information (inner speech)
    Visuospatial sketchpad: visual and spatial information
    Episodic buffer:       temporary multimodal binding (Baddeley, 2000)
    Central executive:     allocates resources between the above
  
  If NOT rehearsed AND not emotionally significant AND not novel:
    → decays and lost from working memory
  
  If rehearsed OR emotionally significant OR novel:
    → encoding into long-term memory begins

STAGE 5: Long-Term Memory Encoding
  Multiple encoding levels (Craik & Lockhart, 1972):
    Shallow: phonological → "what does this sound like?" (weak encoding)
    Intermediate: visual → "what does this look like?"
    Deep:    semantic → "what does this MEAN?" (strongest encoding)
  
  Semantic encoding is ~5x more durable than phonological encoding.
  
  Key factors determining encoding strength:
  ┌──────────────────────────────────────────────────────────────┐
  │ FACTOR           │ EFFECT                  │ membrain PARAM  │
  │──────────────────────────────────────────────────────────────│
  │ Attention level  │ Higher → stronger        │ attention_score │
  │ Emotional arousal│ Higher → stronger        │ emotional.arousal│
  │ Novelty          │ Novel → stronger         │ novelty_score   │
  │ Repetition       │ More → stronger (LTP)   │ (via recall LTP)│
  │ Elaboration      │ More meaning → stronger  │ (content depth) │
  │ Context richness │ Rich context → stronger  │ context_embedding│
  │ Self-relevance   │ Personal → stronger      │ source_kind     │
  └──────────────────────────────────────────────────────────────┘

STAGE 6: Initial Consolidation (Synaptic Consolidation)
  Immediately after encoding: memory is FRAGILE (labile state)
  Duration of fragile window: minutes to hours
  
  Molecular process:
    1. NMDA receptors activated by glutamate
    2. Ca2+ influx → activates kinases (CaMKII, PKA)
    3. Kinases → phosphorylate AMPA receptors → insert more AMPA
    4. Also → CREB transcription factor → new protein synthesis
    5. New proteins → structural changes in dendritic spines
    6. After 6+ hours: proteins synthesized, structural changes complete
    7. Memory becomes resistant to disruption (stable)
  
  Disruption during labile window (before stabilization):
    - Electroconvulsive shock
    - Protein synthesis inhibitors
    - Anesthesia
    → Memory lost permanently
  
  membrain port:
    state = MemoryState::Labile on creation
    Labile memories have full reconsolidation window
    state → MemoryState::Stable after initial consolidation tick

membrain ENCODING PIPELINE (complete port):

  encode(content, context, attention_score, emotional_tag, source)
    │
    ├─ [attention_score < ATTENTION_THRESHOLD (0.2)] → DISCARD
    │   Simulates sensory register → attention filter
    │
    ├─ embedding_cache.get_or_compute(content)
    │   → content_vec: Vec<f32>  (384 dims)
    │   Simulates semantic encoding (deep processing)
    │
    ├─ embedding_cache.get_or_compute(context)
    │   → context_vec: Vec<f32>
    │   Simulates encoding specificity (context bound)
    │
    ├─ novelty_score = compute_novelty(content_vec, hot_index)
    │   = 1.0 - max_cosine_similarity(content_vec, top_1_neighbor)
    │   Range: 0.0 (identical to existing) to 1.0 (completely novel)
    │   Simulates novelty detection → automatic attention capture
    │
    ├─ initial_strength = BASE_STRENGTH
    │     × (1.0 + novelty_score × NOVELTY_WEIGHT)         [0.3]
    │     × (1.0 + attention_score × ATTENTION_WEIGHT)     [0.4]
    │     × emotional_tag.strength_multiplier()            [1.0 to 1.5]
    │   Simulates encoding depth and initial synaptic strength
    │
    ├─ bypass_decay = emotional_tag.arousal > AROUSAL_THRESHOLD (0.6)
    │              && |emotional_tag.valence| > VALENCE_THRESHOLD (0.5)
    │   Simulates amygdala → NE → enhanced hippocampal LTP
    │
    ├─ state = MemoryState::Labile
    │   Simulates freshly encoded, fragile memory
    │
    ├─ INSERT INTO hot.db/memories
    │   INSERT INTO hot.db/memory_index (for fast pre-filter)
    │
    ├─ hot_index.add(id, quantize_i16(content_vec))
    │   HNSW index updated for future searches
    │
    ├─ interference_check(content_vec)
    │   Find similar existing memories (0.7 < similarity < 0.99)
    │   Apply retroactive interference penalty to older memories
    │
    └─ engram_builder.try_cluster(id, content_vec)
        Check if this memory belongs to an existing engram
        If similarity > ENGRAM_THRESHOLD: add to cluster
        If isolated: create new engram seed
```

---

### 2.5 LTP / LTD — The Physical Basis of Learning

```
LONG-TERM POTENTIATION (LTP)

Discovery: Terje Lømo, 1966 (published 1973)
  Stimulated perforant path → dentate gyrus in rabbit
  High-frequency stimulation → subsequent single pulses produced LARGER responses
  Effect lasted hours to weeks → "long-term" potentiation

Molecular mechanism (Schaffer collateral LTP, best understood):

  Step 1: PRESYNAPTIC GLUTAMATE RELEASE
    Action potential arrives at presynaptic terminal
    Ca2+ enters through voltage-gated Ca2+ channels
    Synaptic vesicles fuse → glutamate released into synapse

  Step 2: POSTSYNAPTIC RECEPTOR ACTIVATION
    Glutamate binds AMPA receptors → Na+ influx → depolarization (EPSP)
    Glutamate also binds NMDA receptors BUT:
      NMDA receptor has Mg2+ block at resting potential
      Only unblocked when membrane is sufficiently depolarized
      
  Step 3: NMDA RECEPTOR UNBLOCKING (the "coincidence detector")
    If EPSP is large enough (from coincident input):
      Mg2+ block removed → NMDA receptor opens
      Ca2+ flows in through NMDA receptor
      Ca2+ = the critical second messenger for LTP
      
  Step 4: DOWNSTREAM SIGNALING CASCADE
    Ca2+ → activates CaMKII (Ca2+/calmodulin-dependent protein kinase II)
    CaMKII → phosphorylates existing AMPA receptors → increased conductance
    CaMKII → triggers AMPA receptor insertion from nearby endosomes
    → MORE AMPA receptors in synapse → stronger response to same presynaptic input
    
  Step 5: LATE LTP (E-LTP → L-LTP, >3 hours)
    Early LTP (E-LTP): post-translational modifications (minutes)
    Late LTP (L-LTP): requires new protein synthesis (hours to days)
    PKA → CREB (cAMP response element binding protein) → gene transcription
    New structural proteins → dendritic spine grows larger
    More receptor slots available → even stronger synapse
    
  Hebb's rule (theoretical framework):
    "When an axon of cell A is near enough to excite cell B and
    repeatedly and persistently takes part in firing it, some growth
    process or metabolic change takes place in one or both cells
    such that A's efficiency, as one of the cells firing B,
    is increased."
    
    Simplified: "Neurons that fire together, wire together."

membrain LTP port:
  fn on_recall(memory: &mut Memory, now_tick: u64) {
      // Simulate Ca2+ cascade → more AMPA receptors → stronger synapse
      memory.base_strength = (memory.base_strength + LTP_DELTA).min(MAX_STRENGTH);
      // Simulate protein synthesis → structural spine growth → more stable
      memory.stability += STABILITY_INCREMENT;
      // Update "when was this synapse last active"
      memory.last_accessed_tick = now_tick;
      // Update memory's state (Labile after recall — reconsolidation window)
      memory.state = MemoryState::Labile { 
          since_tick: now_tick,
          window: reconsolidation_window(memory.age_ticks(now_tick))
      };
  }

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

LONG-TERM DEPRESSION (LTD)

Equally important as LTP — often overlooked but CRITICAL.
Without LTD: all synapses eventually saturate at maximum strength.
At saturation: no more learning possible (no more differentiation).
LTD is the "reset" mechanism that enables continued learning.

Molecular mechanism (cerebellar LTD, best understood):

  Context: Climbing fiber (error signal) activates simultaneously with
           parallel fiber (stimulus signal)
  
  Step 1: SIMULTANEOUS ACTIVATION
    Parallel fiber: glutamate → AMPA receptors → depolarization
    Climbing fiber: strong depolarization → large Ca2+ influx
    
  Step 2: PROTEIN KINASE C ACTIVATION
    Combined mGluR (metabotropic glutamate receptor) + strong Ca2+
    → activates Protein Kinase C (PKC)
    
  Step 3: AMPA RECEPTOR INTERNALIZATION
    PKC → phosphorylates AMPA receptors at specific site
    Phosphorylated AMPA receptors → internalized (endocytosis)
    FEWER AMPA receptors in synapse → weaker response
    
  In hippocampus (homosynaptic LTD):
    Low-frequency stimulation (1 Hz for 15 min) → LTD
    Moderate Ca2+ influx through NMDA (not enough for LTP)
    → activates phosphatases (PP1, PP2B calcineurin)
    → phosphatases dephosphorylate AMPA receptors
    → AMPA receptor endocytosis → weaker synapse

  HOMEOSTATIC PLASTICITY (Synaptic Scaling):
    If a neuron is chronically under-stimulated:
      → scales UP all synaptic strengths (upward scaling)
    If a neuron is chronically over-stimulated:
      → scales DOWN all synaptic strengths (downward scaling)
    This maintains the neuron in its operational range.
    
    Tononi's Synaptic Homeostasis Hypothesis (SHY):
    Sleep is when synaptic downscaling occurs globally.
    Wake: net increase in synaptic strength (LTP from experiences)
    Sleep: net decrease (NREM → global downscaling → SNR improvement)

membrain LTD port (Ebbinghaus Forgetting Curve):

  EBBINGHAUS FORMULA:
    R(t) = e^(-t/S)
    
    R: retention (0.0 to 1.0)
    t: time since last access (in interaction ticks)
    S: stability (increases with each successful recall)
    
  Original Ebbinghaus data (1885, self-experiment):
    After 1 review:  S ≈ 1 day equivalent
    After 2 reviews: S ≈ 3 days
    After 4 reviews: S ≈ 1 week
    After 8 reviews: S ≈ 1 month
    Stability roughly doubles with each recall → logarithmic growth
  
  membrain implementation:
    
    // LAZY computation — never iterate, only compute on access
    fn effective_strength(memory: &Memory, now_tick: u64) -> f32 {
        if memory.bypass_decay {
            return memory.base_strength;  // emotional memories immune
        }
        
        let elapsed = now_tick.saturating_sub(memory.last_accessed_tick);
        let retention = (-elapsed as f32 / memory.stability).exp();
        
        memory.base_strength * retention
    }
    
    // Called during on_recall() to persist new base strength
    fn persist_decay(memory: &mut Memory, now_tick: u64) {
        if !memory.bypass_decay {
            let new_strength = effective_strength(memory, now_tick);
            memory.base_strength = new_strength;
        }
        // Reset: decay clock restarts from now
        memory.last_accessed_tick = now_tick;
    }
    
  WHY LAZY (critical for performance):
    Eager approach: every N ticks → iterate ALL memories → update strength
    Problem: O(n) operation, blocks retrieval, death at scale
    
    Lazy approach: compute effective_strength(memory, now_tick) ONLY when:
      1. A memory is retrieved (before scoring)
      2. During pre-filter (WHERE clause in SQL)
      3. During consolidation cycle (for migration scoring)
    
    Cost: O(1) per memory access
    Overhead at idle: ZERO
    Result: millions of memories, zero maintenance cost
```

---

### 2.6 Forgetting Curve — Ebbinghaus

```
HISTORICAL CONTEXT:
  Hermann Ebbinghaus (1850-1909)
  Self-experiment: memorized nonsense syllables (DAX, BUP, LOJ...)
  Measured retention at intervals from 20 minutes to 31 days
  Relearning method: measured savings — how much easier was re-learning?
  
  Formula derived from data: R = e^(-t/S)
  This formula has been validated in hundreds of subsequent studies.
  It remains the most empirically robust model of memory decay.

DECAY DATA (Ebbinghaus, 1885):
  ┌────────────────────────────────────┐
  │ Time since learning │  % Retained  │
  │─────────────────────│─────────────│
  │ 20 minutes          │     58%      │
  │ 1 hour              │     44%      │
  │ 8-9 hours           │     36%      │
  │ 1 day               │     33%      │
  │ 2 days              │     28%      │
  │ 6 days              │     25%      │
  │ 31 days             │     21%      │
  └────────────────────────────────────┘

REVIEW EFFECT (spacing):
  The "spacing effect" — one of the most robust findings in cognitive psychology.
  Massed practice: study 10 times in one session → poor long-term retention
  Spaced practice: study 10 times across 10 sessions → excellent long-term retention
  
  Mechanism: each review resets decay AND increases stability:
    After review 1: S ≈ 1 unit
    After review 2: S ≈ 2 units (stability doubled)
    After review 3: S ≈ 4 units
    After review n: S ≈ 2^(n-1) units
    
  membrain stability growth:
    After recall n:  stability = BASE_STABILITY × (1 + n × STABILITY_INCREMENT)
    More precisely: stability += STABILITY_INCREMENT on each recall
    Bounded: stability = stability.min(MAX_STABILITY) [prevents infinite stability]

INTERACTION COUNT vs REAL TIME:
  membrain uses interaction_count instead of real time. Rationale:
  
  1. Agents don't have a clock in the human sense
     An agent running 1000 queries per second and one running 1 per hour
     should have the same decay dynamics — based on usage, not elapsed time
  
  2. Relevance is usage-based, not time-based
     A memory that's been recalled 100 times in the last 50 ticks is more
     relevant than one recalled 0 times in the same period, regardless of age
  
  3. Implementation: global atomic counter
     interaction_count: Arc<AtomicU64>
     Incremented on every encode OR recall
     Stored as last_accessed_tick at recall time
     Elapsed = current_tick - memory.last_accessed_tick

STABILITY INTERPRETATION:
  stability = 100 means: after 100 ticks without recall, retention = e^(-1) ≈ 37%
  stability = 500 means: after 500 ticks, retention = e^(-1) ≈ 37%
  
  A memory with stability=500 takes 5× as long to decay to 37% as stability=100.
  High stability = "well-consolidated, repeatedly recalled" memory.
  Low stability = "freshly encoded or rarely reinforced" memory.
  
  Initial stability (before any recall):
    BASE_STABILITY = 50 interactions (tunable)
    Emotional memories get multiplier: stability × (1 + arousal × 0.5)
  
  Stability growth per recall:
    STABILITY_INCREMENT = 0.2 × current_stability
    (each recall increases stability by 20% → doubling after ~3.8 recalls)
    Matches Ebbinghaus spacing effect data approximately.
```

---

### 2.7 Consolidation — The Stabilization Process

```
TWO PHASES OF CONSOLIDATION:

PHASE 1: SYNAPTIC CONSOLIDATION (minutes to hours)
  ─────────────────────────────────────────────────
  Location:  At the synapse itself
  Duration:  30 minutes → 6 hours
  Process:
    1. Encoding → AMPA insertion (early LTP, minutes)
    2. CaMKII → CREB activation → gene transcription
    3. New mRNA produced, transported to dendrites
    4. Local protein synthesis at synapses (dendritic translation)
    5. New structural proteins → spine enlargement
    6. Spine geometry change → permanent synaptic modification
    
  State: LABILE during this phase
    - Vulnerable to disruption (ECS, protein synthesis inhibitors, trauma)
    - New information can update/overwrite the memory
    - This is the reconsolidation window
    
  After ~6 hours: STABLE
    - Resistant to disruption
    - Protein synthesis complete
    - Structural changes established

PHASE 2: SYSTEMS CONSOLIDATION (days to years)
  ─────────────────────────────────────────────────
  Location:  Brain-wide (hippocampus → neocortex)
  Duration:  Days to years (some memories take decades)
  Process:
    1. Newly encoded memory: hippocampus-dependent
       (hippocampal damage → memory lost)
    2. Hippocampus replays memory during sleep (NREM)
    3. Repeated replay → neocortex gradually learns the pattern
    4. After sufficient replay: neocortex can retrieve independently
    5. Memory becomes hippocampus-independent
    
  Gradient of hippocampal dependence:
    1 week old:  strongly hippocampus-dependent
    1 month old: moderately hippocampus-dependent
    1 year old:  weakly hippocampus-dependent
    10 years old: hippocampus-independent (neocortical memory)
    
  Evidence:
    - Patient H.M.: anterograde amnesia BUT intact memories from before surgery
    - Retrograde amnesia gradient: recent memories lost, old ones intact
    - fMRI: old memories activate neocortex; recent memories activate hippocampus

COMPLEMENTARY LEARNING SYSTEMS (McClelland et al., 1995):
  The fundamental architecture:
    Hippocampus: fast learning, one-shot encoding, temporary
    Neocortex:   slow learning, many exposures needed, permanent
  
  Why two systems?
    Fast learning (hippocampus) is needed for one-shot episodic memories.
    Slow learning (cortex) prevents catastrophic forgetting.
    If cortex learned as fast as hippocampus → new information would
    immediately overwrite all existing knowledge.
    
  The compromise:
    1. Hippocampus encodes everything quickly (fast learning system)
    2. During sleep: hippocampus replays → cortex gradually integrates
    3. Integration is INTERLEAVED with existing cortical representations
       → prevents catastrophic forgetting
    4. Result: both new AND old memories coexist in cortex

membrain CONSOLIDATION PORT:

  TRIGGER CONDITIONS:
    1. hot_index.len() > HOT_CAPACITY (50,000 vectors)
    2. hot_store total effective_strength > STRENGTH_PRESSURE threshold
    3. explicit call: membrain consolidate
    4. periodic: every CONSOLIDATION_INTERVAL interactions
    
  NREM EQUIVALENT (migrate_to_cold):
    ──────────────────────────────────────────────────────
    1. Score all hot memories:
       consolidation_score = effective_strength(m, now)
                           × access_count_decay_weighted
                           × (1.0 + emotional_multiplier)
       
    2. Sort by consolidation_score DESC
    
    3. For top N memories (N = HOT_CAPACITY × MIGRATION_FRACTION):
       a. Re-embed with full float32 (final encoding)
       b. Compress content (zstd level 3)
       c. INSERT INTO cold.db/cold_memories
       d. cold_index.add(id, quantize_i8(embedding))  // int8 for cold
       e. Mark as MemoryState::Consolidated in hot.db
       f. Keep metadata pointer in hot.db (hippocampus keeps index)
    
    4. Update engram centroids for migrated memories
    
  REM EQUIVALENT (process_emotional_queue):
    ──────────────────────────────────────────────────────
    1. Collect all memories with bypass_decay = true
       AND emotional_processed = false
    
    2. For each:
       a. Gradually reduce arousal: arousal *= DESENSITIZATION_FACTOR (0.95)
       b. Create cross-links in engram graph to semantically related memories
       c. Set emotional_processed = true when arousal < PROCESSED_THRESHOLD
    
    3. This simulates REM's role in:
       - Emotional memory processing (reduces acute emotional charge)
       - Integration of emotional events with existing knowledge
       
  HOMEOSTASIS (downscale):
    ──────────────────────────────────────────────────────
    1. Compute total_load = SUM(effective_strength) / COUNT
    2. If total_load > HOMEOSTASIS_TRIGGER (0.85 × MAX_LOAD):
       a. For ALL hot memories:
          base_strength *= HOMEOSTASIS_FACTOR (0.9)
       b. This globally scales down — strong memories stay relatively strong
          weak memories get pushed below MIN_STRENGTH → archive
    3. After scaling: prune all memories below MIN_STRENGTH → archive
    
    Simulates Tononi's Synaptic Homeostasis Hypothesis:
    "Sleep → global synaptic downscaling → improved signal-to-noise ratio"
```

---

### 2.8 Sleep — The Brain's Consolidation Engine

```
SLEEP ARCHITECTURE:

  One sleep cycle = ~90 minutes, repeating 4-6 times per night
  ┌─────────────────────────────────────────────────────────────┐
  │ NREM Stage 1:  Light sleep, 5-10 min, theta waves         │
  │ NREM Stage 2:  Sleep spindles + K-complexes, 20 min       │
  │ NREM Stage 3:  Slow-wave sleep (SWS), 20-40 min           │
  │ REM:           Rapid eye movement, 10-60 min              │
  └─────────────────────────────────────────────────────────────┘
  
  Early night: More SWS (NREM3), less REM
  Late night: Less SWS, more REM
  → Procedural/emotional processing dominated by early vs late night

NREM SLOW-WAVE SLEEP (SWS) — EPISODIC → SEMANTIC TRANSFER:

  Sharp-wave ripples (SWR):
    Hippocampus generates brief (50-100ms) bursts of activity
    During these ripples: recent episodic memories are REPLAYED
    Not replayed verbatim — compressed, pattern-extracted
    
  Sleep spindles:
    Thalamo-cortical oscillations (12-15 Hz, 0.5-3s)
    Synchronize hippocampal-cortical communication
    Each spindle = a "transfer packet" from hippocampus to cortex
    
  Slow oscillations:
    ~0.75 Hz large cortical oscillations
    UP state: cortex receives hippocampal replay
    DOWN state: cortex offline, consolidates received information
    
  Process:
    1. Hippocampus replays compressed episodic memories
    2. Cortex receives replay during UP states
    3. Cortical synapses gradually strengthen for replayed patterns
    4. After many replay sessions across many nights: cortex "knows" the pattern
    5. Hippocampal trace becomes redundant → eventually weakens

  MEMORY SELECTION FOR REPLAY:
    Not all memories replayed equally:
    - Reward-associated memories preferentially replayed
    - Emotional memories preferentially replayed
    - Recently acquired memories preferentially replayed
    - Frequently recalled memories less likely replayed (already consolidated)
    
    membrain consolidation scoring:
      consolidation_score = effective_strength × recency_weight × emotional_weight
      High scoring → migrated to cold_store first

REM SLEEP — EMOTIONAL PROCESSING + INTEGRATION:

  REM characteristics:
    - Brain activity similar to wakefulness
    - Acetylcholine high, norepinephrine/serotonin low
    - NE suppression: allows emotional memories to be processed
      without re-triggering the full emotional response
    - Muscle atonia (paralysis): prevents acting out dreams
    
  Functions:
  
  1. EMOTIONAL MEMORY PROCESSING
     Walker's "Sleep to forget, sleep to remember" hypothesis:
     - Sleep preserves the memory content
     - But dampens the emotional charge
     - NE suppression → "safe" reactivation of emotional memories
     - Results in emotional regulation ("time heals wounds" — sleep-dependent)
     
  2. CREATIVE INTEGRATION / REMOTE ASSOCIATIONS
     Remote associates: connect memories that share no obvious link
     During REM: acetylcholine-driven associative search
     → Distant memories that share abstract features are linked
     → "Sleep on it" effect for creative problem solving
     
  3. SYNAPTIC PRUNING (in some interpretations)
     Hobson & McCarley's Activation-Synthesis:
     Random brainstem activation → cortex synthesizes narrative (dreams)
     But also: weak random activation can cause LTD → synaptic pruning
     → Another mechanism for noise reduction during sleep

  membrain REM port:
    process_emotional_queue():
      - Reduce arousal: simulates NE suppression → emotional dampening
      - Build cross-links: simulates creative integration
      - Result: emotional memories become less "loud" over time
      - Without this: emotional bypass_decay never resolves → memory too heavy

SLEEP SPINDLES + MEMORY REPLAY (technical):

  Timing of replay:
    NOT random — temporally compressed (20-200× faster than original experience)
    Original event: 5 minutes of coding session
    Replay during SWS: 1-3 seconds of hippocampal burst
    → Allows thousands of experiences to be replayed per night
    
  Selective replay and reward:
    Dopamine-tagged memories (reward) → more likely replayed
    Awake replay also occurs (during quiet wakefulness)
    
  membrain async background jobs:
    NREM cycle: pressure-triggered, async tokio task
    REM cycle: after NREM, emotional queue processing
    Homeostasis: after REM, global downscaling if needed
    
    All three run sequentially in one async consolidation cycle:
      async fn consolidation_cycle(store: Arc<BrainStore>) {
          let report = store.nrem_cycle().await?;    // migrate hot → cold
          store.rem_cycle().await?;                  // emotional processing
          store.homeostasis_cycle().await?;          // global scaling
          emit_report(report);
      }
```

---

### 2.9 Reconsolidation — Memory Updates on Recall

```
DISCOVERY:
  Susan Sara (2000) and Karim Nader (2000) independently showed:
  When a consolidated memory is RECALLED, it becomes LABILE AGAIN.
  The memory must be RE-CONSOLIDATED (re-stabilized) after recall.
  
  During the labile window after recall:
    - Memory can be MODIFIED (updated with new information)
    - Memory can be WEAKENED (interfered with)
    - Memory can be STRENGTHENED (LTP during recall)
    - Memory can be EXTINGUISHED (active suppression during labile window)

  This overturned decades of belief that consolidated memories were "fixed."
  Memory is NOT a static recording — it's a dynamic, reconstructive process.

NADER'S EXPERIMENT:
  Protocol:
    Day 1: Train rats on fear conditioning (tone → shock)
    Day 2: Re-expose to tone → fear memory recalled → labile
    Day 2 (immediately after): inject anisomycin (protein synthesis inhibitor)
    Day 3: Test → fear memory GONE
    
  Control: 
    Day 2: No re-exposure to tone (memory not recalled → not labile)
    Day 2: inject anisomycin
    Day 3: fear memory intact
    
  Conclusion: Re-exposure → memory recall → protein synthesis required to restabilize
              Block protein synthesis during labile window → memory lost

CLINICAL IMPLICATIONS:
  PTSD treatment (Memory Reconsolidation Therapy):
    1. Retrieve traumatic memory (make it labile)
    2. During labile window: introduce incompatible information
       (safety cues, context incompatible with threat)
    3. Memory reconsolidates with updated (less threatening) information
    4. Trauma response diminished
    
  This is the basis of EMDR and some CBT protocols.

RECONSOLIDATION WINDOW:
  Duration of labile state after recall:
    Recent memories (< 7 days old): ~6 hours labile window
    Old memories (weeks to months): ~3-4 hours
    Very old memories (years): ~1-2 hours or may not reconsolidate at all
    
  Inverse relationship: older = more stable = shorter labile window
  This makes sense: repeatedly consolidated memories are harder to update
  
  membrain formula:
    fn reconsolidation_window(age_in_ticks: u64) -> u64 {
        let base = RECONSOLIDATION_BASE_WINDOW;  // 50 ticks
        // Older memories have shorter reconsolidation windows
        // Factor approaches 0 as age → ∞
        let age_factor = 1.0 / (1.0 + age_in_ticks as f32 / OLD_MEMORY_THRESHOLD);
        (base as f32 * age_factor) as u64
    }
    
    Result:
      Very new memory (age=0):    window = 50 ticks
      Moderately old (age=500):   window ≈ 25 ticks (half)
      Very old (age=5000):        window ≈ 5 ticks (very short)
      Ancient (age=50000):        window ≈ 0.5 ticks → rounds to 1 (effectively no update)

BOUNDARY CONDITIONS:
  Boundary conditions for reconsolidation (when it DOESN'T occur):
    1. Very weak memories: strength < threshold → no labile state on recall
    2. Very old memories: window too short to exploit in practice
    3. Very strong memories: require more interference to destabilize
    4. Memory not recalled sufficiently: partial cue alone may not trigger labile state
    
  membrain boundary conditions:
    // Only enter labile state if:
    if effective_strength(m, now) > LABILE_STRENGTH_THRESHOLD (0.2)
    && m.access_count > 0  // has been recalled before
    {
        m.state = MemoryState::Labile { ... };
    }

membrain RECONSOLIDATION IMPLEMENTATION:

  On every recall:
    1. Set state = Labile { since_tick: now, window: reconsolidation_window(age) }
    2. Store in labile_memories table: memory_id, labile_since, window_ticks
  
  External update API:
    membrain update <id> --new-content "..."
    or: MCP tool update_memory(id, new_content)
    → stores in pending_updates table: memory_id, new_content, submitted_tick
    
  reconsolidation_tick (runs periodically):
    for each labile memory:
      if current_tick - since_tick < window:
        // Still labile — can be updated
        if has_pending_update:
          apply_update(memory, pending_update)
          re_embed(memory)  // new content → new vector
          memory.base_strength += RECONSOLIDATION_BONUS (0.05)
          hot_index.update(memory.id, memory.embedding)
      else:
        // Window expired — restabilize
        memory.state = MemoryState::Stable
        remove pending_update if any (not applied)
```

---

### 2.10 Active Forgetting — Intelligent Pruning

```
COMMON MISCONCEPTION:
  "We forget because memories fade passively over time."
  
  REALITY: Forgetting is partly ACTIVE and PURPOSEFUL.
  
  Evidence:
    1. Directed forgetting experiments:
       Tell subjects "remember this" or "forget this" after each item.
       Items labeled "forget" are recalled significantly less.
       Cannot be explained by passive decay alone.
       
    2. RAC1-mediated active forgetting (Rac1 GTPase):
       Shuai & Bhatt (2010): small GTPase RAC1 actively degrades memory traces.
       Inhibiting RAC1 → memories that would normally fade are RETAINED.
       RAC1 is bidirectional: also involved in LTP.
       
    3. Motivated forgetting:
       Anderson & Green (2001): think/no-think paradigm.
       Suppressing recall of a memory → subsequent recall reduced.
       Prefrontal-hippocampal pathway: PFC inhibits hippocampal retrieval.
       
    4. Sleep homeostasis (Tononi):
       Active downscaling during NREM sleep.
       Not random — downscaling proportional to usage.
       Weak synapses → scaled below threshold → eliminated.

PURPOSE OF ACTIVE FORGETTING:
  
  SIGNAL-TO-NOISE OPTIMIZATION:
    Without forgetting: every experience equally accessible.
    Problem: common, unimportant events crowd out rare, important ones.
    
    With active forgetting:
      Common events: encoded weakly AND actively pruned → low signal
      Rare, important events: high novelty + emotional → strong + retained
      → Important things are disproportionately remembered.
      
  INTERFERENCE REDUCTION:
    Similar but different memories interfere with each other.
    Active forgetting removes the weaker competitor.
    Result: cleaner, more distinct memory traces.
    
  GENERALIZATION (ABSTRACTION):
    Forgetting specific episodic details allows semantic generalization.
    "I don't remember which Tuesday I learned to ride a bike,
     but I know how to ride a bike."
    Episodic specifics forgotten → procedural generalization remains.

membrain ACTIVE FORGETTING ENGINE:

  COMPONENT 1: Decay-Based Pruning (passive-to-active threshold)
    Periodically (not every tick — every PRUNE_INTERVAL):
    
    // Find memories below threshold
    SELECT id FROM memory_index
    WHERE effective_strength(base_strength, stability, NOW - last_tick) < MIN_STRENGTH
    AND bypass_decay = 0
    AND state != 'Archived'
    LIMIT 10000  // don't scan everything at once
    
    → Archive (soft delete) — never hard delete
    → Archive table retains record for potential recovery/analysis
    
  COMPONENT 2: Proactive Interference
    Old memories interfere with recall of new similar memories.
    Mechanism: when recalling memory B, old similar memory A competes.
    
    During encoding of new memory B:
      Find memories A with 0.7 < sim(A, B) < 0.99
      For each A:
        A.retrieval_difficulty += PROACTIVE_INTERFERENCE_WEIGHT
        // A is now harder to recall (competes with B)
        
  COMPONENT 3: Retroactive Interference  
    New memories interfere with recall of old similar memories.
    New memory B → old memory A weakens.
    
    During encoding of new memory B:
      For each old memory A with 0.7 < sim(A, B) < 0.99:
        A.base_strength *= (1.0 - RETROACTIVE_INTERFERENCE_PENALTY)
        // A is now weaker
    
  COMPONENT 4: Predictive Pruning
    Memories that are NEVER recalled should decay faster.
    Memories recalled repeatedly should be stable.
    
    Predictive value score:
      predictive_value = access_count / age_in_ticks
      (access frequency normalized by age)
      
    Memories with predictive_value < PREDICTIVE_THRESHOLD
    AND age > MINIMUM_AGE_FOR_PREDICTIVE_PRUNE:
      → Apply extra decay: base_strength *= PREDICTIVE_DECAY_FACTOR
      
  COMPONENT 5: Capacity-Based Archive
    When total memory count > SOFT_CAP:
      Sort by effective_strength ASC (weakest first)
      Archive bottom ARCHIVE_FRACTION (10%) of memories
      → This simulates sleep homeostasis global downscaling
```

---

### 2.11 Engrams — Physical Memory Traces

```
DEFINITION:
  "Engram": the physical substrate of a memory trace in the brain.
  Term coined by Richard Semon (1904), popularized by Karl Lashley.
  
  Lashley spent 30 years looking for the "engram" as a single location.
  Failed — concluded "equipotentiality": memory distributed across cortex.
  
  Modern understanding: Engrams ARE distributed, but have STRUCTURE.
  
ENGRAM CELLS (Tonegawa Lab, MIT):
  2012: Xu Liu & Steve Ramirez (Tonegawa lab) identified engram cells.
  
  Method: 
    Activity-dependent labeling (c-fos-tTA system):
    1. Open "labeling window" (doxycycline removal)
    2. Fear condition mouse
    3. Close labeling window
    4. All cells that were active during conditioning: labeled with ChR2 (channelrhodopsin)
    
  Results:
    - Optogenetic activation of labeled cells (blue light) → fear response
    - Even in a neutral context that had NEVER been paired with shock
    - Conclusion: those specific cells ARE the engram for that fear memory
    
  2014: "False memory" implantation:
    1. Label cells active during neutral context A (no shock) → labeled CA1 cells
    2. In context B: shock while optogenetically activating context A cells
    3. Result: mouse freezes in context A (which was NEVER paired with shock)
    4. Conclusion: artificial fear memory created by activating engram cells in new context

ENGRAM STRUCTURE:
  Not a single cell — a SPARSE DISTRIBUTED PATTERN across cells:
  
  Typical engram:
    ~10-30% of neurons in a region involved (sparse coding)
    But those neurons form a SPECIFIC PATTERN
    Partial reactivation of pattern → full memory reconstruction
    
  Engram cells properties:
    - Increased intrinsic excitability (more likely to fire)
    - More synaptic connections to each other (mutual LTP)
    - Same cells → active during encoding AND recall
    - Linked to other engrams via synaptic connections
    
ENGRAM CLUSTERS:
  Related memories share overlapping engram cells.
  
  Example:
    Memory A: "ate sushi in Tokyo last Tuesday"
    Memory B: "visited sushi restaurant that opened in HCM"
    Memory C: "learned sushi-grade fish needs flash-freezing"
    
    These three memories share cells active for the concept "sushi."
    Activating any one → partial activation of the others → associative recall.
    
  Clinically: this is why Alzheimer's damages memory in a structured way.
  When engram cells die → whole cluster of related memories lost together.
  
membrain ENGRAM IMPLEMENTATION:

  Data structure: petgraph DiGraph<Uuid, EdgeWeight>
    Nodes: memory_id (Uuid)
    Edges: EdgeWeight { similarity: f32, edge_type: EdgeType }
    
  EdgeType enum:
    Associative    // semantic similarity (most common)
    Causal         // A led to B (temporal + semantic)
    Contradictory  // A contradicts B (for contrastive recall)
    Temporal       // A happened before B in same session
    
  Engram struct:
    id: Uuid
    centroid: Vec<f32>        // mean of all member embeddings
    member_ids: Vec<Uuid>     // all memories in this cluster
    created_tick: u64
    last_activated_tick: u64
    total_strength: f32       // sum of member strengths (resonance pool)
    
  ENGRAM FORMATION (engram_builder):
    On each new memory encoded:
    1. Query engram_index (centroid HNSW):
       Find top-5 candidate engrams by centroid similarity
    2. If similarity(new_memory_vec, engram_centroid) > ENGRAM_THRESHOLD (0.65):
       Add to that engram:
         engram.member_ids.push(new_id)
         engram.centroid = update_centroid(engram.centroid, new_vec, n_members)
         Add edge: engram_graph.add_edge(new_id, most_similar_member_id, weight)
    3. If no engram matches:
       Create new engram seed (single-member engram)
    4. If new memory matches TWO engrams:
       Add to closer one, add cross-cluster edge
       
  ENGRAM RETRIEVAL (BFS expansion):
    Given recall result top_hit:
    1. Get top_hit.engram_id
    2. BFS from top_hit node in engram_graph:
       - max_depth = 3
       - max_nodes = 50
       - min_edge_weight = 0.5
       - Priority queue: process highest-weight edges first
    3. Collect all BFS-reached nodes
    4. Score each: similarity_to_query × edge_path_weight × effective_strength
    5. Return top-K from expanded cluster
    
  ENGRAM SIZE LIMITS (prevent "god clusters"):
    soft_limit = 200 members:
      Trigger: split engram into 2 sub-engrams using K-means (k=2)
      Parent: becomes abstract node with two children
    hard_limit = 500 members:
      Reject new additions, create sibling engram instead
    
  MEMORY RESONANCE (collective LTP):
    When memory M in engram E is recalled:
      LTP(M): M.base_strength += LTP_DELTA
      Resonance to cluster:
        for neighbor in bfs_depth_1(M, engram_graph):
          neighbor.base_strength += LTP_DELTA × RESONANCE_FACTOR / cluster_size
      RESONANCE_FACTOR = 0.3 (30% of full LTP spread to neighbors)
      Effect: entire cluster slightly strengthened when any member is recalled
              Large clusters with dense recall → collectively very stable
              Simulates CA3 autoassociative network behavior
```

---

### 2.12 Working Memory — The Conscious Workspace

```
BADDELEY'S MODEL OF WORKING MEMORY (1974, revised 2000):

  ┌──────────────────────────────────────────────────────────────┐
  │                    CENTRAL EXECUTIVE                         │
  │           (controls, allocates, coordinates)                │
  │                          │                                   │
  │         ┌────────────────┼────────────────┐                 │
  │         ▼                ▼                ▼                 │
  │  PHONOLOGICAL      VISUOSPATIAL     EPISODIC BUFFER          │
  │     LOOP           SKETCHPAD        (added 2000)            │
  │  (verbal/acoustic) (visual/spatial) (multimodal binding)    │
  └──────────────────────────────────────────────────────────────┘

CENTRAL EXECUTIVE:
  The boss of working memory.
  - Allocates attention between subsystems
  - Switches between tasks
  - Inhibits irrelevant information
  - Coordinates with long-term memory
  Damage (dysexecutive syndrome): difficulty organizing, switching tasks

PHONOLOGICAL LOOP:
  Phonological store: holds ~2 seconds of speech (7 digits in English)
  Articulatory rehearsal process: "inner speech" refreshes store
  Word length effect: shorter words → more words remembered
  Irrelevant speech effect: background speech disrupts phonological loop

VISUOSPATIAL SKETCHPAD:
  Visual buffer: static visual images
  Spatial process: spatial information, movement
  Double dissociation with phonological loop (separate systems)
  Capacity: ~3-4 objects

EPISODIC BUFFER (Baddeley, 2000 addition):
  Temporary multimodal storage
  Binds together visual, spatial, verbal, temporal information
  Interface between working memory and long-term memory
  Capacity: ~4 episodes

CAPACITY DEBATE:
  Miller (1956): 7 ± 2 chunks
  Cowan (2001): 4 ± 1 chunks (independent items)
  
  Resolution: "chunks" can be hierarchically organized.
  Expert chess players chunk board positions → 7 patterns × many pieces.
  
  For membrain: 7 slots (Miller) — clean architecture, well-motivated.
  Can be reduced to 4 (Cowan) in config.

WORKING MEMORY AND LTM INTERACTION:
  Encoding:
    WM → LTM: items in WM that are rehearsed or significant → encoded to LTM
    LTM → WM: retrieval "loads" relevant LTM content into WM
    
  Recency effect: last few items in a list recalled best = WM still holds them
  Primacy effect: first few items recalled well = had time for LTM encoding

membrain WORKING MEMORY PORT:

  struct WorkingMemory {
      slots: VecDeque<MemoryItem>,      // ordered by recency (front = most recent)
      capacity: usize,                   // default 7 (configurable)
      attention: HashMap<Uuid, f32>,    // attention weights per memory
      central_executive: ExecutiveState, // current task context
  }
  
  impl WorkingMemory {
      fn add(&mut self, item: MemoryItem, store: &mut BrainStore) {
          if self.slots.len() >= self.capacity {
              // Evict: remove lowest-attention slot from back
              let evict = self.slots.pop_back().unwrap();
              self.attention.remove(&evict.id);
              
              // If strong enough: persist to hot_store before eviction
              if evict.strength > ENCODE_THRESHOLD {
                  store.encode(evict.into_memory());
              }
          }
          self.slots.push_front(item);
      }
      
      fn focus(&mut self, id: Uuid) {
          // Simulate executive attention directing to specific item
          *self.attention.entry(id).or_insert(0.0) += FOCUS_DELTA;
      }
      
      fn get_active(&self) -> Vec<&MemoryItem> {
          // Return all items, sorted by attention score
          let mut items: Vec<_> = self.slots.iter().collect();
          items.sort_by(|a, b| {
              let att_a = self.attention.get(&a.id).copied().unwrap_or(0.0);
              let att_b = self.attention.get(&b.id).copied().unwrap_or(0.0);
              att_b.partial_cmp(&att_a).unwrap()
          });
          items
      }
  }
```

---

### 2.13 Emotional Memory — The Amygdala Effect

```
WHY EMOTIONAL MEMORIES ARE SPECIAL:
  
  The Brown-Kulik phenomenon (1977): "flashbulb memories"
  Extremely vivid, detailed memories of highly emotional events.
  "Where were you when you heard about 9/11?"
  "Where were you when JFK was shot?" (older generation)
  
  Properties of emotional memories:
    1. Higher initial encoding strength (amygdala → NE → enhanced LTP)
    2. More likely to be rehearsed (thought about repeatedly)
    3. More likely to be talked about (social reinforcement)
    4. Less subject to interference (amygdala shields them)
    5. Updated by reconsolidation over time (emotional charge reduces)

NEUROBIOLOGY:
  
  Fast path (thalamo-amygdala, "low road"):
    Sensory thalamus → amygdala DIRECTLY (bypasses cortex)
    ~12ms latency — faster than conscious perception
    "Rough and ready" — processes basic emotional significance
    Evolutionarily ancient: predator detection
    
  Slow path (thalamo-cortical-amygdala, "high road"):
    Sensory thalamus → sensory cortex → amygdala
    ~30-100ms latency
    More detailed, contextually appropriate
    Can override the fast-path initial response
    
  LeDoux's "two roads to fear":
    Both paths converge on basolateral amygdala (BLA)
    BLA → central amygdala → downstream fear responses
    BLA → hippocampus: modulates memory consolidation
    
  Norepinephrine (NE) mechanism:
    High emotional arousal → locus coeruleus → NE release
    NE → β-adrenergic receptors in hippocampus
    β-AR → cAMP → PKA → enhanced LTP
    Result: emotional events have ~40% stronger memory traces

VALENCE × AROUSAL SPACE:
  
  Russell's Circumplex Model (1980):
  
         High Arousal
              │
    Stressed  │  Excited
    Alarmed   │  Elated
    Tense     │  Happy
              │
  Neg ─────────────────── Pos
  Valence    │           Valence
              │
    Sad       │  Calm
    Depressed │  Relaxed
    Bored     │  Serene
              │
         Low Arousal
         
  HIGH AROUSAL × NEGATIVE: fear, anger, panic → maximum memory enhancement
  HIGH AROUSAL × POSITIVE: joy, excitement → strong memory enhancement
  LOW AROUSAL × EITHER: moderate or no enhancement
  
  membrain formula:
    fn strength_multiplier(tag: &EmotionalTag) -> f32 {
        // Both high arousal AND significant valence needed for max effect
        let emotional_intensity = tag.arousal.abs() * tag.valence.abs();
        1.0 + (emotional_intensity * EMOTIONAL_WEIGHT)
        // EMOTIONAL_WEIGHT = 0.5
        // Max: 1.0 + (1.0 × 1.0 × 0.5) = 1.5
    }
    
    fn should_bypass_decay(tag: &EmotionalTag) -> bool {
        // Flashbulb threshold: high arousal + strong valence
        tag.arousal > AROUSAL_THRESHOLD         // 0.6
        && tag.valence.abs() > VALENCE_THRESHOLD // 0.5
    }

EMOTIONAL MEMORY DECAY OVER TIME:
  
  Emotional memories DO decay but at different rates:
    The emotional CHARGE decays faster than the CONTENT.
    "I remember it happened, but no longer feel the fear."
    
  REM sleep is critical for emotional charge reduction.
  
  membrain:
    bypass_decay = true → content strength maintained (doesn't decay)
    emotional_tag.arousal reduced over time during REM equivalent:
      arousal *= DESENSITIZATION_FACTOR (0.95 per cycle)
    When arousal drops below PROCESSED_THRESHOLD:
      bypass_decay = false (memory now subject to normal decay)
    But by then: stability is HIGH (emotional memory was rehearsed many times)
    → Still strong, just no longer artificially protected from decay
```

---

### 2.14 Interference — Memory Conflicts

```
INTERFERENCE THEORY:
  One of the oldest and most robust theories in memory research.
  Explanation for much of everyday forgetting.

PROACTIVE INTERFERENCE (PI):
  Old learning interferes with new learning.
  "Forward-going" interference.
  
  Classic experiment:
    Group 1: Learn list A, then list B → recall B → poor
    Group 2: No prior learning, then list B → recall B → good
    Difference = proactive interference from list A
    
  Example: You've always dialed 555-1234 for pizza.
           New pizza place: 555-5678.
           When you think "pizza," the old number competes.
           The old number causes proactive interference.
           
  Mechanism: retrieval of B → A also activated → competition → error
  
  membrain port:
    When encoding NEW memory B:
      Find all OLD memories A with high similarity to B (0.7 < sim < 0.99)
      For each A: A.retrieval_difficulty += PROACTIVE_PENALTY
      (A is now harder to recall, competing with B)

RETROACTIVE INTERFERENCE (RI):
  New learning interferes with old learning.
  "Backward-going" interference.
  
  Classic experiment:
    Group 1: Learn list A, then list B → recall A → poor
    Group 2: Learn list A, then sleep/rest → recall A → good
    Difference = retroactive interference from list B
    
  Example: After learning new pizza number, old number is harder to recall.
           New information actively degrades old similar information.
           
  Mechanism: encoding B → updates representations shared with A → A weakened
  
  membrain port:
    When encoding NEW memory B:
      Find all OLD memories A with high similarity to B (0.7 < sim < 0.99)
      For each A (older than B by threshold):
        A.base_strength *= (1.0 - RETROACTIVE_PENALTY)
      This directly weakens similar old memories

RELEASE FROM PROACTIVE INTERFERENCE:
  If category changes between lists → sudden improvement in recall.
  Change: all the PI built up → released because it doesn't apply.
  
  membrain port:
    Context switching (different context_embedding):
      → interference penalties only apply within similar context
      → switching contexts reduces interference effects

INTERFERENCE vs DECAY:
  Debate: is forgetting due to DECAY or INTERFERENCE?
  
  Modern answer: BOTH contribute.
  - Pure decay: passive loss of strength over time
  - Interference: active competition from similar memories
  
  membrain implements both:
    Decay: Ebbinghaus formula (lazy, per-memory)
    Interference: on encode + during forgetting engine cycles
```

---

### 2.15 Pattern Completion — Recall from Partial Cues

```
THE CA3 AUTOASSOCIATIVE NETWORK:

  Hippocampal CA3 region is the brain's pattern completion engine.
  
  Architecture:
    CA3 neurons have recurrent collateral connections (neurons → each other)
    This makes CA3 an autoassociative attractor network.
    
    Mathematical model: Hopfield network
      N neurons, each connected to all others
      Network stores patterns as attractor states
      Noisy/incomplete input → network relaxes to nearest attractor
      = pattern completion
      
  Capacity of Hopfield network:
    Can store ~0.14 × N patterns
    For CA3 with ~300,000 neurons: ~42,000 patterns
    Much less than total hippocampal capacity → CA3 is index, not primary store

  CA3 → CA1 → Subiculum → Entorhinal Cortex → Neocortex:
    CA3 completes pattern → sends to CA1
    CA1 modulates output (novelty detection) → subiculum
    → back to neocortex for full pattern reconstruction
    
PATTERN SEPARATION (Dentate Gyrus):
  The complement of pattern completion.
  DG SEPARATES similar inputs → less interference between similar memories.
  
  How: DG has ~1 million neurons (vs CA3's ~300k)
       Sparse activity (~2% of DG neurons active at any time)
       Different inputs → very different DG activation patterns
       
  Result: Two experiences that share 80% of features
          → DG produces 20% overlapping patterns
          → CA3 stores them as distinct attractors
          → No interference during storage
          
  membrain approximation:
    novelty_score = 1.0 - max_cosine_similarity(new, existing)
    High novelty → low similarity → natural "separation" in embedding space
    Dense/similar content: lower novelty, risk of interference → apply penalty

membrain PATTERN COMPLETION IMPLEMENTATION:

  The HNSW search itself implements a form of pattern completion:
    1. Query vector (partial/noisy) → HNSW nearest neighbor search
    2. Returns most similar vectors → these ARE the pattern-completed memories
    3. Engram BFS expansion: top hit → cluster members
    4. Cluster = "all memories that fire together with this pattern"
    
  Partial cue handling:
    query = "I was fixing that auth bug" (partial)
    → embedding similar to multiple memories
    → HNSW returns: auth_bug_memory_1, auth_bug_memory_2, JWT_memory, ...
    → Engram expansion: auth_bug engram → 15 related memories
    → All returned as "pattern completed" recall
    
  Tip-of-tongue simulation:
    If max similarity to any existing memory < FULL_RECALL_THRESHOLD (0.8)
    but > PARTIAL_RECALL_THRESHOLD (0.4):
    → Return MemoryFragment: partial reconstruction, low confidence
    → Flag as TipOfTongue state
    → Still useful information, just lower confidence
```

---

### 2.16 Encoding Specificity — Context Dependency

```
THE ENCODING SPECIFICITY PRINCIPLE (Tulving & Thomson, 1973):
  "A retrieval cue is effective to the extent that information from
  the cue and information from the encoding situation jointly specify
  the memory."
  
  Simpler: Memories are best recalled when retrieval context matches encoding context.

CLASSIC STUDIES:

  UNDERWATER MEMORY STUDY (Godden & Baddeley, 1975):
    Divers learned word lists either:
      - Underwater (scuba diving) OR
      - On land
    Then recalled either underwater or on land.
    
    Results:
      Learned underwater, recalled underwater: 11.4 words
      Learned underwater, recalled on land:    8.4 words
      Learned on land,    recalled on land:    13.5 words
      Learned on land,    recalled underwater: 8.6 words
      
    Context match → ~35% better recall.
    
  STATE-DEPENDENT MEMORY (Goodwin et al., 1969):
    Participants learned material either sober or drunk.
    Tested sober or drunk.
    
    Results:
      Learned drunk, recalled drunk: best
      Learned sober, recalled sober: best
      Cross-state: significantly worse
      
  MOOD CONGRUENCE:
    Happy mood → happy memories more accessible (congruence effect)
    Depressed mood → negative memories more accessible
    (Clinically important: depression maintains itself via memory bias)

MECHANISM:
  Context features are CO-ENCODED with content.
  They become part of the engram.
  Retrieval cue must overlap with engram to activate it.
  
  Context features include:
    - Physical environment (location)
    - Internal state (mood, physiological arousal)
    - Cognitive state (what you were thinking about)
    - Temporal context (what preceded/followed this event)
    - Social context (who was present)

membrain ENCODING SPECIFICITY PORT:

  Every memory stores TWO embeddings:
    content_embedding: what the memory is about
    context_embedding: the context when it was encoded
    
  Retrieval score formula:
    score = (CONTENT_WEIGHT × cosine_sim(query, content_embed))
          + (CONTEXT_WEIGHT × cosine_sim(current_context, context_embed))
    
    CONTENT_WEIGHT = 0.7  (content is more important)
    CONTEXT_WEIGHT = 0.3  (context boosts relevant memories)
    
  Effect: if agent is currently in "debugging auth" context:
    - All memories encoded during auth debugging get context boost
    - Memories from unrelated contexts are not boosted
    - Natural context-dependent retrieval emerges
    
  Context construction (caller responsibility):
    context = "current_task: debugging | current_module: auth | 
                recent_actions: reading JWT code | goal: fix token expiry"
    → agent passes this context string with every encode/recall
    → membrain embeds it and stores/matches accordingly

  Context switching effect:
    Task A → Task B: context_embedding changes dramatically
    Memories from Task A: lower context similarity → lower retrieval score
    Memories from Task B: higher context similarity → higher score
    Natural focus shift with task switching — exactly like human cognition.

---

### End of Snapshot Part 1

**Next: Part 2 — Gap Analysis & Porting the Brain Mechanism by Mechanism**

Parts list:
- Part 1: Vision, Problem Statement, Human Brain Deep Dive (this file)
- Part 2: Gap Analysis + Full Port (mechanism → Rust code for each)
- Part 3: Architecture + Performance
- Part 4: Techstack + Data Schema
- Part 5: CLI/MCP + Feature Extensions + Workspace Structure  
- Part 6: Milestones + Acceptance Checklist + Constants + Algorithm Reference


<!-- SOURCE: PLAN_part2.md -->

### Source Snapshot — Part 2
#### Part 2 of 6: Gap Analysis · Porting the Brain Mechanism by Mechanism

---

## 3. Gap Analysis — Human Brain vs Current AI Memory Systems

### 3.1 Feature Matrix

```
MECHANISM                    Brain  MemGPT  Mem0  LangMem  OpenAI  Zep  membrain
─────────────────────────────────────────────────────────────────────────────────
LTP / LTD (strength dynamics)  ✅     ❌      ❌    ❌       ❌      ❌    ✅
Ebbinghaus decay (forgetting)  ✅     ❌      ❌    partial  ❌      ❌    ✅
Emotional tagging              ✅     ❌      ❌    ❌       ❌      ❌    ✅
Emotional bypass decay         ✅     ❌      ❌    ❌       ❌      ❌    ✅
Engram clusters                ✅     ❌      partial ❌     ❌      partial ✅
Associative recall             ✅     ❌      partial ❌     ❌      partial ✅
Pattern completion             ✅     ❌      ❌    ❌       ❌      ❌    ✅
Reconsolidation (update on recall)✅  ❌      ❌    ❌       ❌      ❌    ✅
Consolidation (episodic→semantic)✅   conceptual ❌ ❌      ❌      ❌    ✅
NREM equivalent                ✅     ❌      ❌    ❌       ❌      ❌    ✅
REM equivalent                 ✅     ❌      ❌    ❌       ❌      ❌    ✅
Homeostasis                    ✅     ❌      ❌    ❌       ❌      ❌    ✅
Active forgetting              ✅     ❌      ❌    ❌       ❌      ❌    ✅
Interference handling          ✅     ❌      ❌    ❌       ❌      ❌    ✅
Working memory (capacity limit) ✅    ❌      ❌    ❌       ❌      ❌    ✅
Dual-path retrieval (fast/slow) ✅    partial  ❌   partial  ❌      ❌    ✅
Context-dependent retrieval    ✅     ❌      ❌    ❌       ❌      ❌    ✅
Source provenance              brain  ❌      ❌    ❌       ❌      partial ✅
Temporal ordering              ✅     partial  partial ✅   ❌      ✅    ✅
Unlimited scale                ✅     limited  limited limited ❌    limited ✅
Fast path (<1ms familiarity)   ✅     ❌      ❌    ❌       ❌      ❌    ✅
Offline / no API dependency    ✅     ❌      ❌    ❌       ❌      ❌    ✅
─────────────────────────────────────────────────────────────────────────────────
Score / 22                     22/22  3/22   4/22   4/22   2/22   4/22  22/22
```

### 3.2 Detailed Gap Analysis Per System

#### 3.2.1 MemGPT / Letta

```
WHAT IT DOES:
  Models memory like an operating system's virtual memory.
  Main context window = RAM (fast, limited)
  External storage = disk (slow, unlimited)
  
  When context fills up: "page out" least relevant memories.
  When memories needed: "page in" from external storage.
  
  Uses LLM calls to decide what to page in/out.
  
WHAT IT GETS RIGHT:
  - Tiered storage concept (acknowledged, even if not optimized)
  - Persistence beyond single conversation
  - Some form of relevance-based retrieval
  
FUNDAMENTAL PROBLEMS:
  1. No dynamics: a memory is either in context or not. No strength.
     There is no concept of "this memory is getting weaker over time."
     
  2. LLM in critical path: paging decisions made by LLM calls.
     Each context switch costs: LLM call latency (500ms+) + API cost.
     At scale: extremely expensive. Not suitable for real-time agents.
     
  3. No biological mechanisms: no decay, no LTP, no engrams.
     Just a software engineering solution to a context length problem.
     Does not solve the SIGNAL-TO-NOISE problem.
     
  4. No associations: memories are independent chunks.
     "What else do I know related to X?" requires another LLM call.
     
  5. No consolidation: episodic events never become semantic knowledge.
     Raw conversation snippets stored forever without abstraction.
     
  WHY THIS MATTERS:
  Long-running MemGPT agent: each memory is a raw text chunk with equal weight.
  After 10,000 interactions: 10,000 equally-weighted raw chunks.
  Retrieval: cosine similarity → noise dominates signal.
  Result: agent repeats mistakes, contradicts itself, cannot learn patterns.
```

#### 3.2.2 Mem0

```
WHAT IT DOES:
  Two-phase pipeline:
  Phase 1: LLM extracts facts from conversation
  Phase 2: Stores in vector DB + optional graph DB
  
  On retrieval: semantic search → top-K results injected into prompt.
  
WHAT IT GETS RIGHT:
  - Semantic vector search
  - Some graph structure (relationships between entities)
  - Fact extraction (semantic encoding, not raw episodic)
  
FUNDAMENTAL PROBLEMS:
  1. LLM in write path: every memory encode requires LLM call.
     Cost: API call per memory. At 1000 memories: 1000 API calls.
     Latency: 500ms+ per encode. Unusable for real-time.
     
  2. No strength dynamics: extracted facts stored with equal weight.
     "Python is slow" (from 2 years ago) equals "PyPy is fast" (from today).
     
  3. No forgetting: memories accumulate without decay.
     5 years of a developer's interactions → millions of facts.
     Quality degrades as noise builds.
     
  4. No reconsolidation: old facts never updated when new contradicting
     information arrives. Stale facts persist indefinitely.
     
  5. Extraction error propagation: if LLM extracts wrong fact,
     it's stored with same confidence as correct facts.
     No mechanism to detect or correct this.
     
  6. No pattern completion: can only retrieve what was explicitly extracted.
     Cannot reconstruct from partial cues.
     
  BENCHMARK COMPARISON:
  Mem0 retrieval: ~50-200ms (API call + vector search)
  membrain Tier2:  <5ms (local HNSW, no API)
  Speedup: 10-40×
  
  Mem0 encode:    ~500-2000ms (LLM extraction + store)
  membrain encode: <10ms (local embedding + store)
  Speedup: 50-200×
```

#### 3.2.3 LangMem (LangChain)

```
WHAT IT DOES:
  Memory with lifecycle management:
  - Short-term: working context
  - Long-term: persistent store with relevance scoring
  - Lifecycle: memories can be created, updated, expired
  
  Relevance-based retrieval: score = recency × similarity
  
WHAT IT GETS RIGHT:
  - Lifecycle states (partial decay concept)
  - Relevance scoring (recency + similarity)
  - Some structure in storage
  
FUNDAMENTAL PROBLEMS:
  1. "Relevance = recency × similarity" is a crude approximation.
     Does not model stability (reinforcement effect of repeated recall).
     Does not model emotional significance.
     Does not model interference.
     
  2. No engrams: memories are independent.
     No cluster structure for associative recall.
     
  3. No consolidation: no episodic → semantic transformation.
     All memories at same abstraction level.
     
  4. LangChain dependency: heavy framework, slow startup, complex deployment.
     membrain: 1 binary, zero dependencies at runtime.
     
  5. No interference handling: contradicting memories coexist silently.
     
  6. Decay is periodic and eager: iterate all memories on schedule.
     At scale: O(n) operation blocks retrieval.
     membrain: lazy decay, O(0) idle, O(1) per recall.
```

### 3.3 The Root Problem: Naive Vector Stores

```
All existing systems reduce to:
  1. Embed content → vector
  2. Store vector in DB
  3. Query: cosine similarity search → top-K
  4. Return top-K to LLM
  
  This is a simple vector database with extra steps.
  
  Why this fails at scale:
  
  UNIFORM WEIGHT: every vector treated equally.
  
    After 10,000 memories:
      - 9,500 mundane/routine memories (low value)
      - 500 important/significant memories (high value)
      
    Simple cosine search: returns based on semantic similarity only.
    If query is about topic X: returns all 10,000 memories about X equally.
    9,500 mundane memories drown out 500 important ones.
    
  STATIC REPRESENTATIONS: vectors never change.
    
    Memory encoded at t=0: same vector at t=100,000.
    But: the RELEVANCE of that memory changes dramatically over time.
    Solution: strength-weighted cosine similarity (membrain).
    
  NO STRUCTURE: memories are a flat list.
    
    "What else is related to X?" requires:
      a) LLM to reason about relationships (expensive, error-prone)
      b) OR: pre-computed graph structure (membrain engrams)
      
  NO DYNAMICS: no learning from recall patterns.
    
    Which memories are actually useful? Tracked by access_count, strength.
    LRU cache + LTP: frequently useful → stays strong → fast retrieval.
    membrain tracks every recall and uses it to strengthen the memory.
```

---

## 4. Porting the Brain — Mechanism by Mechanism

This section maps every significant brain mechanism to its exact membrain implementation.
For each mechanism: biological description → computational model → Rust implementation.

---

### 4.1 Hippocampal Index → hot_store

```
BIOLOGICAL:
  Hippocampus does not store memory content — it stores POINTERS.
  The actual content is distributed across the neocortex.
  Hippocampus: "memory A is about visual features stored in V4,
                semantic content in temporal cortex, emotional tag in amygdala"
  
  This is why hippocampal damage causes amnesia:
  - Content still exists in neocortex
  - But cannot be ACCESSED without hippocampal index
  - Like deleting a database index: data exists, cannot find it

PORT TO membrain:

  hot_store = hippocampal index:
    - Fast, small, temporary
    - Stores metadata + pointers (not primary content)
    - usearch HNSW: enables O(log n) pointer lookup from partial cue
    - SQLite hot.db: relational metadata with full query capability
    
  cold_store = neocortical content:
    - Slow to write (consolidation required)
    - Stores compressed semantic content
    - Large, stable, permanent-ish
    - No HNSW index required for writing (only for reading)

SCHEMA SEPARATION:
  memory_index table (hot.db) — the hippocampal part:
    id, base_strength, stability, last_tick, bypass_decay, kind, engram_id
    (Everything needed for scoring/filtering — no content)
    
  memory_content table (hot.db) — cached content for fast access:
    id, content, created_tick
    
  cold_memories table (cold.db) — neocortical content:
    id, content_compressed, embedding_f32, created_tick, consolidated_tick
    
  KEY INSIGHT: pre-filter queries ONLY touch memory_index.
    All 50,000 hot memories → single index scan.
    No content fetched until final top-K results returned.
    Exactly like: hippocampus (index only) → neocortex (content fetch).

RUST IMPLEMENTATION:

  pub struct HotStore {
      db: Connection,
      hot_index: Index,           // usearch HNSW, float16, 50k cap
      embed_cache: LruCache<u64, Vec<f32>>,
  }
  
  impl HotStore {
      /// Pre-filter: fast scan of memory_index only
      /// Returns candidate IDs — no content fetch
      pub fn prefilter_candidates(
          &self,
          now_tick: u64,
          min_strength: f32,
          limit: usize,
      ) -> Result<Vec<Uuid>> {
          let sql = "
              SELECT id
              FROM memory_index
              WHERE (base_strength * EXP(-(? - last_tick) / stability)) > ?
              AND bypass_decay = 0
              AND state NOT IN (2, 3)  -- not Archived or Consolidated-out
              UNION
              SELECT id
              FROM memory_index
              WHERE bypass_decay = 1   -- emotional memories always candidates
              AND state NOT IN (2, 3)
              ORDER BY (base_strength * EXP(-(? - last_tick) / stability)) DESC
              LIMIT ?
          ";
          // Execute with (now_tick, min_strength, now_tick, limit)
      }
  }
```

---

### 4.2 Neocortical Content Storage → cold_store

```
BIOLOGICAL:
  Neocortex stores the actual content of consolidated memories.
  Properties:
    - Distributed: no single "location" for a memory
    - Stable: requires many repetitions to update (prevents catastrophic forgetting)
    - Fast for familiar patterns (columnar fast-path recognition)
    - Slow to write (consolidation required)
    - Vast: effectively unlimited by any practical measure

PORT TO membrain:

  cold_store = neocortical content store:
    - usearch mmap: disk-backed, unlimited scale
    - int8 quantized vectors: compressed representation (4× smaller than f32)
    - zstd-compressed content: ~3-4× content compression
    - SQLite cold.db: metadata + compressed content blobs
    - OS page cache: automatic warm layer (frequently accessed → cached in RAM)
    
  WRITE PATH (consolidation only):
    1. NREM cycle triggers migration from hot_store
    2. Re-embed at full float32 precision (final encoding)
    3. Compress content: zstd::encode(content, level=3)
    4. INSERT INTO cold.db/cold_memories
    5. cold_index.add(id, quantize_i8(embedding))
    6. Mark as Consolidated in hot.db/memory_index
    
    This is "slow to write" by design — only consolidation writes cold store.
    Prevents catastrophic forgetting: new information cannot overwrite cold store directly.
    
  READ PATH (retrieval):
    1. SQL pre-filter on cold.db metadata (effective_strength filter)
    2. cold_index HNSW search (int8, mmap, fast even for millions)
    3. Rescore top-20 with float32 embeddings
    4. Fetch content: SELECT content_compressed FROM cold_memories WHERE id = ?
    5. zstd::decode(content_compressed) → content string
    
  MMAP ARCHITECTURE:
    cold.usearch file: memory-mapped vector index
    OS manages: keeps frequently accessed pages in RAM, pages out unused
    Effect: "hot" cold memories (frequently recalled consolidated memories)
            → stay in OS page cache → fast access
            "true cold" memories (rarely accessed) → paged out → slower
    This mirrors the cortex's columnar fast-path for familiar patterns.

RUST IMPLEMENTATION:

  pub struct ColdStore {
      db: Connection,
      cold_index: Index,   // usearch HNSW, int8, mmap-backed
  }
  
  impl ColdStore {
      pub fn consolidate_from_hot(
          &mut self,
          memories: Vec<ConsolidationCandidate>,
      ) -> Result<ConsolidationReport> {
          let mut report = ConsolidationReport::default();
          
          for candidate in memories {
              // Compress content
              let compressed = zstd::encode_all(
                  candidate.content.as_bytes(), 3
              )?;
              
              // Store in cold.db
              self.db.execute(
                  "INSERT INTO cold_memories
                   (id, content_compressed, embedding_f32, consolidated_tick)
                   VALUES (?, ?, ?, ?)",
                  params![
                      candidate.id,
                      compressed,
                      // Store float32 as blob for rescore
                      bytemuck::cast_slice::<f32, u8>(&candidate.embedding),
                      candidate.now_tick,
                  ],
              )?;
              
              // Add to HNSW cold index (int8 quantized)
              let i8_vec = quantize_i8(&candidate.embedding);
              self.cold_index.add(candidate.id.as_u64_pair().0, &i8_vec)?;
              
              report.migrated_count += 1;
          }
          
          Ok(report)
      }
  }
```

---

### 4.3 Amygdala Tagging → EmotionalTag + bypass_decay

```
BIOLOGICAL:
  Amygdala runs in parallel with hippocampal encoding.
  High-arousal events → NE release → amplified LTP in hippocampus.
  Result: emotionally significant memories ~40% stronger initially.
  Also: high-arousal memories resist forgetting (flashbulb effect).

PORT TO membrain:

  EmotionalTag:
    valence: f32,   // -1.0 (very negative) to +1.0 (very positive)
    arousal: f32,   // 0.0 (calm) to 1.0 (highly aroused)
    
  ENCODING EFFECT:
    fn strength_multiplier(tag: &EmotionalTag) -> f32 {
        let intensity = tag.arousal * tag.valence.abs();
        1.0 + (intensity * EMOTIONAL_WEIGHT)
    }
    // EMOTIONAL_WEIGHT = 0.5
    // panic (valence=-0.95, arousal=0.95): multiplier = 1.45 (+45%)
    // mild concern (valence=-0.3, arousal=0.3): multiplier = 1.045 (+4.5%)
    // joy (valence=0.8, arousal=0.7): multiplier = 1.28 (+28%)
    
  BYPASS DECAY:
    fn compute_bypass_decay(tag: &EmotionalTag) -> bool {
        tag.arousal > AROUSAL_THRESHOLD       // 0.6
        && tag.valence.abs() > VALENCE_THRESHOLD // 0.5
    }
    // Only threshold-crossing emotions bypass decay
    // moderate emotions: still decay normally
    // flashbulb-level emotions: permanently strengthened (until REM processing)
    
  REM DESENSITIZATION:
    // During REM equivalent cycle:
    fn desensitize_emotional_memories(store: &mut BrainStore, now_tick: u64) {
        let emotional = store.get_emotional_memories();
        for mut memory in emotional {
            // Reduce arousal gradually (simulates NE suppression during REM)
            memory.emotional_tag.arousal *= DESENSITIZATION_FACTOR; // 0.95
            
            // When arousal drops below threshold:
            if memory.emotional_tag.arousal < PROCESSED_THRESHOLD { // 0.3
                // Emotional bypass removed — memory now subject to normal decay
                // But stability is HIGH by now (was rehearsed many times)
                memory.bypass_decay = false;
                memory.emotional_processed = true;
            }
            
            // Create cross-links to semantically related memories
            // (REM's creative integration function)
            let related = store.find_related_non_emotional(&memory, 3);
            for related_memory in related {
                store.engram_graph.add_edge(
                    memory.id,
                    related_memory.id,
                    EdgeWeight {
                        similarity: 0.5,
                        edge_type: EdgeType::Associative,
                    }
                );
            }
        }
    }

CALLER API (how agents provide emotional tags):

  // Option 1: Explicit (agent knows what happened was significant)
  membrain remember "authentication service crashed in production" \
    --valence -0.8 --arousal 0.9
    
  // Option 2: Implicit scoring (simple heuristic, no LLM needed)
  fn infer_emotional_tag(content: &str) -> EmotionalTag {
      let negative_keywords = ["error", "failed", "crash", "bug", "panic", ...];
      let positive_keywords = ["success", "fixed", "deployed", "works", ...];
      let high_arousal_keywords = ["critical", "production", "urgent", "breaking", ...];
      
      let neg_score = count_matches(content, &negative_keywords);
      let pos_score = count_matches(content, &positive_keywords);
      let arousal_score = count_matches(content, &high_arousal_keywords);
      
      EmotionalTag {
          valence: (pos_score - neg_score) as f32 / total_words as f32,
          arousal: arousal_score as f32 / total_words as f32,
      }
  }
  
  // Option 3: MCP tool with explicit emotional parameter
  // "remember" MCP tool: emotional_valence and emotional_arousal params
```

---

### 4.4 Prefrontal Working Memory → WorkingMemory + Tier1 Cache

```
BIOLOGICAL:
  PFC maintains 7±2 items in active working memory.
  Central executive allocates attention.
  Items that are attended to longer → more likely encoded to LTM.
  When WM full: evict lowest-attention item.

PORT TO membrain (dual structure):

  STRUCTURE 1: WorkingMemory (strict 7-slot simulation)
  ─────────────────────────────────────────────────────
  Used for: explicit working memory management in agent-alive
  
  pub struct WorkingMemory {
      slots: VecDeque<WorkingMemoryItem>,
      capacity: usize,                      // default 7
      attention_weights: HashMap<Uuid, f32>,
      central_executive: TaskContext,
  }
  
  pub struct WorkingMemoryItem {
      memory_id: Uuid,
      content: String,               // raw content for immediate use
      added_tick: u64,
      source: WorkingMemorySource,   // External | FromLTM | JustEncoded
  }
  
  pub struct TaskContext {
      current_task: String,
      task_context_embedding: Vec<f32>,
      focus_history: VecDeque<Uuid>,
  }
  
  impl WorkingMemory {
      pub fn add(&mut self, item: WorkingMemoryItem, store: &mut BrainStore) {
          if self.slots.len() >= self.capacity {
              // Find minimum attention item
              let min_id = self.slots.iter()
                  .min_by(|a, b| {
                      let wa = self.attention_weights.get(&a.memory_id).copied().unwrap_or(0.0);
                      let wb = self.attention_weights.get(&b.memory_id).copied().unwrap_or(0.0);
                      wa.partial_cmp(&wb).unwrap()
                  })
                  .map(|i| i.memory_id);
              
              if let Some(evict_id) = min_id {
                  self.slots.retain(|i| i.memory_id != evict_id);
                  self.attention_weights.remove(&evict_id);
                  // Evicted → encode to hot_store if strong enough
                  // (simulates WM → LTM transfer)
              }
          }
          self.slots.push_front(item);
      }
      
      pub fn focus_on(&mut self, id: Uuid) {
          *self.attention_weights.entry(id).or_insert(0.0) += FOCUS_DELTA;
          self.central_executive.focus_history.push_back(id);
          if self.central_executive.focus_history.len() > 20 {
              self.central_executive.focus_history.pop_front();
          }
      }
      
      pub fn get_focused_content(&self) -> Vec<&WorkingMemoryItem> {
          let mut items: Vec<_> = self.slots.iter().collect();
          items.sort_by(|a, b| {
              self.attention_weights.get(&b.memory_id)
                  .partial_cmp(&self.attention_weights.get(&a.memory_id))
                  .unwrap()
          });
          items
      }
  }
  
  STRUCTURE 2: Tier1 LruCache (performance fast path)
  ─────────────────────────────────────────────────────
  Used for: <0.1ms familiarity check and recent memory caching
  
  // In BrainStore:
  tier1_cache: LruCache<u64, CachedMemory>,   // key: xxhash64(content)
  tier1_id_cache: LruCache<Uuid, CachedMemory>, // key: memory_id
  
  pub struct CachedMemory {
      id: Uuid,
      content: String,
      embedding: Vec<f32>,  // float32 for rescore
      effective_strength: f32,  // at time of caching
      cached_at_tick: u64,
  }
  
  // Tier1 cache is updated on every recall (recent → cached)
  // Tier1 cache evicts LRU on every encode (new → might push out old)
  // This naturally mirrors PFC working memory:
  //   - Recent things stay in cache
  //   - Frequently accessed things stay in cache
  //   - Older, unused things evicted
```

---

### 4.5 LTP → on_recall() Strength Increase

```
BIOLOGICAL:
  Every successful recall fires the memory's neurons again.
  Repeated firing → NMDA → Ca2+ → CaMKII → more AMPA receptors.
  Each recall makes the memory physically stronger.
  "Neurons that fire together, wire together."

PORT TO membrain:

  Full on_recall() implementation:
  
  pub fn on_recall(
      &mut self,
      memory_id: Uuid,
      now_tick: u64,
      store: &mut BrainStore,
  ) -> Result<()> {
      // Fetch current memory state
      let mut memory = store.hot.get_memory(memory_id)?;
      
      // === STEP 1: Persist lazy decay first ===
      // Before applying LTP, compute current effective strength
      // and make it the new base (decay clock resets)
      if !memory.bypass_decay {
          let current_effective = effective_strength(&memory, now_tick);
          memory.base_strength = current_effective;
      }
      
      // === STEP 2: LTP — increase strength ===
      // Simulates NMDA → Ca2+ → CaMKII → AMPA insertion
      memory.base_strength = (memory.base_strength + LTP_DELTA)
          .min(MAX_STRENGTH);
      
      // === STEP 3: Stability increase ===
      // Simulates structural spine growth (L-LTP, protein synthesis)
      // Each recall increases stability → memory decays slower next time
      memory.stability += STABILITY_INCREMENT * memory.stability;
      // Exponential growth (each recall increases by % of current stability)
      // Bounded to prevent infinite stability:
      memory.stability = memory.stability.min(MAX_STABILITY);
      
      // === STEP 4: Update access tracking ===
      memory.last_accessed_tick = now_tick;
      memory.access_count += 1;
      
      // === STEP 5: Reconsolidation window ===
      // Recall makes memory labile again (reconsolidation)
      let window = reconsolidation_window(
          now_tick - memory.created_tick,
          memory.base_strength,
      );
      if window > 0 {
          memory.state = MemoryState::Labile {
              since_tick: now_tick,
              window_ticks: window,
          };
      }
      
      // === STEP 6: Engram resonance ===
      // Partial LTP spread to engram neighbors (CA3 autoassociative effect)
      if let Some(engram_id) = memory.engram_id {
          let neighbors = store.engram_graph
              .neighbors(memory_id)
              .collect::<Vec<_>>();
          let resonance_ltp = LTP_DELTA * RESONANCE_FACTOR / neighbors.len() as f32;
          for neighbor_id in neighbors {
              store.hot.apply_ltp_delta(neighbor_id, resonance_ltp, now_tick)?;
          }
      }
      
      // === STEP 7: Update Tier1 cache ===
      store.tier1_cache.put(
          xxhash64(memory.content.as_bytes()),
          CachedMemory {
              id: memory.id,
              content: memory.content.clone(),
              embedding: memory.embedding.clone(),
              effective_strength: memory.base_strength,
              cached_at_tick: now_tick,
          },
      );
      
      // === STEP 8: Persist to hot.db ===
      store.hot.update_memory(&memory)?;
      
      Ok(())
  }
```

---

### 4.6 LTD + Ebbinghaus → Lazy Decay

```
BIOLOGICAL:
  Non-use → phosphatase activation → AMPA receptor internalization → weaker synapse.
  The less a memory is used, the weaker it becomes.
  This follows Ebbinghaus's forgetting curve: R(t) = e^(-t/S)

PORT TO membrain (LAZY computation — critical architecture decision):

  // WRONG approach — eager O(n) iteration:
  // fn decay_tick(store: &mut BrainStore, now_tick: u64) {
  //     for mut memory in store.hot.all_memories() {  // O(n) — DEATH at scale
  //         if !memory.bypass_decay {
  //             memory.base_strength *= retention_factor(memory.stability, elapsed);
  //         }
  //     }
  // }
  
  // CORRECT approach — lazy O(1) per access:
  
  /// Compute current effective strength WITHOUT writing to DB
  /// Called at every retrieval, not on a schedule
  #[inline]
  pub fn effective_strength(memory: &MemoryIndex, now_tick: u64) -> f32 {
      if memory.bypass_decay {
          return memory.base_strength;
      }
      let elapsed = now_tick.saturating_sub(memory.last_accessed_tick) as f32;
      let retention = (-elapsed / memory.stability).exp();
      memory.base_strength * retention
  }
  
  /// Persist the decay (called only during recall or consolidation)
  /// Resets the decay clock — future decay starts from now
  pub fn persist_decay_and_reset(
      memory: &mut MemoryIndex,
      now_tick: u64,
  ) {
      if !memory.bypass_decay {
          memory.base_strength = effective_strength(memory, now_tick);
      }
      memory.last_accessed_tick = now_tick;
  }
  
  WHY LAZY IS CORRECT:
  
  1. O(0) idle cost: if agent is doing other things, zero decay computation.
     1 million memories idle → zero CPU overhead.
     
  2. O(1) per recall: compute effective strength at recall time.
     Exact same result as if decay was applied every tick.
     R(t) = e^(-t/S) is pure math — commutative, no ordering dependency.
     
  3. SQL pre-filter: SQLite can evaluate the formula in WHERE clause:
     WHERE (base_strength * EXP(-(? - last_tick) / stability)) > 0.05
     → SQLite computes effective_strength during scan
     → Only candidates above threshold returned
     → No false positives, no false negatives
     
  4. No dirty data: base_strength only updated when actually accessed.
     The "true" current strength is ALWAYS computed on demand.
     No risk of decay being applied twice or missed.
     
  PERFORMANCE NUMBERS:
    Eager (O(n)): 50,000 memories × 1 decay tick = 50,000 EXP computations
                  At 100 ticks/second: 5,000,000 EXP/second overhead
                  EXP is ~5ns on modern CPU: 25ms/tick overhead — UNACCEPTABLE
                  
    Lazy (O(1)):  0 EXP during idle ticks
                  5,000 EXP during retrieval (pre-filter of 5,000 candidates)
                  At 100 queries/second: 500,000 EXP/second
                  But amortized: most queries hit Tier1/Tier2 first
                  Practical: <1ms for entire pre-filter operation
  
  SQL PRE-FILTER WITH LAZY DECAY:
  
  -- This SQL computes effective_strength for pre-filter
  -- SQLite evaluates EXP() natively (fast)
  SELECT id, 
         (base_strength * EXP(-(? - last_tick) / CAST(stability AS REAL))) AS eff_str
  FROM memory_index
  WHERE (base_strength * EXP(-(? - last_tick) / CAST(stability AS REAL))) > ?
    AND bypass_decay = 0
    AND state NOT IN (2, 3)  -- Archived, OutOfHot
  UNION ALL
  SELECT id, base_strength as eff_str
  FROM memory_index
  WHERE bypass_decay = 1
    AND state NOT IN (2, 3)
  ORDER BY eff_str DESC
  LIMIT ?;
  
  -- Parameters: (now_tick, now_tick, min_strength, limit)
  -- Index needed: CREATE INDEX idx_memory_index_filter 
  --               ON memory_index(bypass_decay, state, base_strength, stability, last_tick);
```

---

### 4.7 Consolidation → NREM/REM/Homeostasis Cycles

```
BIOLOGICAL:
  NREM: hippocampus replays → cortex learns → episodic → semantic
  REM: emotional processing, creative integration
  Homeostasis: global synaptic downscaling for SNR improvement

PORT TO membrain:

  Trigger conditions (not time-based — interaction-based):
  
  TRIGGER 1: Hot store pressure
    When hot_index.len() > HOT_CAPACITY × 0.9:
    → Consolidation cycle must run (space pressure)
    
  TRIGGER 2: Total strength pressure
    When sum(effective_strength(m, now) for m in hot) > STRENGTH_CEILING:
    → Homeostasis must run (capacity pressure)
    
  TRIGGER 3: Explicit call
    membrain consolidate
    → Immediate manual trigger
    
  TRIGGER 4: Periodic
    Every CONSOLIDATION_INTERVAL interactions (e.g. 1000):
    → Maintenance cycle even without pressure

  ─────────────────────────────────────────────────────────────────
  NREM EQUIVALENT: migrate_to_cold()
  ─────────────────────────────────────────────────────────────────
  
  pub async fn nrem_cycle(
      &self,
      hot: &mut HotStore,
      cold: &mut ColdStore,
      now_tick: u64,
  ) -> Result<NremReport> {
      // Step 1: Score all hot memories for consolidation candidacy
      let candidates = hot.db.query_all_for_consolidation(now_tick)?;
      
      let mut scored: Vec<(f32, MemoryRecord)> = candidates
          .into_iter()
          .map(|m| {
              // Consolidation score: strong + frequently accessed + emotional
              let eff_str = effective_strength(&m.index, now_tick);
              let access_score = (m.index.access_count as f32).ln() + 1.0;
              let recency_score = 1.0 / (1.0 + (now_tick - m.index.last_accessed_tick) as f32 / 1000.0);
              let emotional_score = 1.0 + m.index.emotional_arousal * 0.5;
              
              let score = eff_str * access_score * recency_score * emotional_score;
              (score, m)
          })
          .collect();
      
      // Step 2: Sort by score DESC — most "ripe" for consolidation first
      scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
      
      // Step 3: Migrate top fraction to cold store
      let n_migrate = (hot.len() as f32 * MIGRATION_FRACTION) as usize;
      let to_migrate = scored.into_iter()
          .filter(|(score, m)| *score > CONSOLIDATION_THRESHOLD)
          .take(n_migrate)
          .map(|(_, m)| m)
          .collect::<Vec<_>>();
      
      // Step 4: For each candidate: full encode + cold store write
      let mut report = NremReport::default();
      
      for memory in to_migrate {
          // Re-embed if content was updated since last encoding
          let final_embedding = if memory.index.last_reconsolidated_tick
                                    > memory.index.created_tick {
              self.embed_cache.get_or_embed(&memory.content).await?
          } else {
              memory.embedding_f32.clone()
          };
          
          cold.consolidate_from_hot(ConsolidationCandidate {
              id: memory.id,
              content: memory.content.clone(),
              embedding: final_embedding,
              now_tick,
          })?;
          
          // Mark as consolidated in hot (keep metadata, remove from HNSW hot index)
          hot.mark_consolidated(memory.id, now_tick)?;
          hot.hot_index.remove(memory.id.as_u64_pair().0)?;
          
          report.migrated += 1;
      }
      
      // Step 5: Update engram centroids for migrated memories
      self.update_engram_centroids(&to_migrate, now_tick)?;
      
      Ok(report)
  }
  
  ─────────────────────────────────────────────────────────────────
  REM EQUIVALENT: process_emotional_queue()
  ─────────────────────────────────────────────────────────────────
  
  pub async fn rem_cycle(
      &self,
      hot: &mut HotStore,
      now_tick: u64,
  ) -> Result<RemReport> {
      // Collect all unprocessed emotional memories
      let emotional_memories = hot.db.query_where(
          "bypass_decay = 1 AND emotional_processed = 0"
      )?;
      
      let mut report = RemReport::default();
      
      for mut memory in emotional_memories {
          // Reduce arousal (NE suppression during REM)
          memory.emotional_arousal *= DESENSITIZATION_FACTOR; // 0.95
          
          // When sufficiently processed:
          if memory.emotional_arousal < PROCESSED_THRESHOLD { // 0.3
              memory.bypass_decay = false;
              memory.emotional_processed = true;
              report.fully_processed += 1;
          }
          
          // Cross-link with related non-emotional memories
          // (creative integration during REM)
          let query_vec = self.embed_cache.get_or_embed(&memory.content).await?;
          let related = hot.hot_index.search(&quantize_f16(&query_vec), 5)?;
          
          for (related_id, similarity) in related {
              if similarity > 0.6 && related_id != memory.id.as_u64_pair().0 {
                  let related_uuid = hot.id_for_usearch(related_id)?;
                  self.engram_graph.add_edge(
                      memory.id,
                      related_uuid,
                      EdgeWeight {
                          similarity,
                          edge_type: EdgeType::Associative,
                      }
                  );
              }
          }
          
          hot.db.update_memory_emotional(&memory)?;
          report.processed += 1;
      }
      
      Ok(report)
  }
  
  ─────────────────────────────────────────────────────────────────
  HOMEOSTASIS: scale_and_prune()
  ─────────────────────────────────────────────────────────────────
  
  pub async fn homeostasis_cycle(
      &self,
      hot: &mut HotStore,
      now_tick: u64,
  ) -> Result<HomeostasisReport> {
      // Check if homeostasis needed
      let total_effective_strength = hot.compute_total_effective_strength(now_tick)?;
      
      if total_effective_strength < HOMEOSTASIS_TRIGGER_LOAD { // 0.85 of max
          return Ok(HomeostasisReport::not_triggered());
      }
      
      // Global downscale (Tononi's synaptic homeostasis)
      hot.db.execute_all(
          "UPDATE memory_index SET base_strength = base_strength * ?",
          params![HOMEOSTASIS_FACTOR], // 0.9
      )?;
      
      // Prune memories that fell below MIN_STRENGTH
      let archived = hot.db.query_where(
          "base_strength < ? AND bypass_decay = 0",
          params![MIN_STRENGTH], // 0.05
      )?;
      
      let mut report = HomeostasisReport::new(archived.len());
      
      for memory in archived {
          hot.archive_memory(memory.id, now_tick, ArchiveReason::Homeostasis)?;
          report.archived += 1;
      }
      
      Ok(report)
  }
```

---

### 4.8 Reconsolidation → Labile State + Update Window

```
BIOLOGICAL:
  Recall → memory labile → can be updated → re-consolidation required.
  Window: recent memories ~6h, old memories shorter.
  During labile window: new information can be incorporated.

PORT TO membrain:

  MemoryState enum:
  pub enum MemoryState {
      Labile {
          since_tick: u64,
          window_ticks: u64,  // update window duration
      },
      Stable,
      Consolidated,     // migrated to cold_store
      Archived,         // below MIN_STRENGTH, soft-deleted
  }
  
  Reconsolidation window formula:
  fn reconsolidation_window(
      age_ticks: u64,
      base_strength: f32,
  ) -> u64 {
      if base_strength < LABILE_STRENGTH_MIN { // 0.2
          return 0;  // Too weak to reconsolidate
      }
      
      let base = RECONSOLIDATION_BASE_WINDOW as f32;  // 50 ticks
      
      // Age factor: older → shorter window
      // age=0: factor=1.0
      // age=BASE_WINDOW: factor=0.5
      // age=10×BASE_WINDOW: factor≈0.09
      let age_factor = 1.0 / (1.0 + age_ticks as f32 / (10.0 * base));
      
      // Strength factor: stronger → slightly longer window
      let strength_factor = 0.5 + base_strength * 0.5;
      
      (base * age_factor * strength_factor) as u64
  }
  
  Pending update storage:
  pub struct PendingUpdate {
      memory_id: Uuid,
      new_content: Option<String>,
      new_emotional_tag: Option<EmotionalTag>,
      submitted_tick: u64,
      submitter: UpdateSource,
  }
  
  Reconsolidation tick:
  pub fn reconsolidation_tick(
      &mut self,
      hot: &mut HotStore,
      now_tick: u64,
  ) -> Result<()> {
      let labile_memories = hot.db.query_labile(now_tick)?;
      
      for mut memory in labile_memories {
          let (since, window) = match memory.state {
              MemoryState::Labile { since_tick, window_ticks } => (since_tick, window_ticks),
              _ => continue,
          };
          
          if now_tick - since <= window {
              // Still within reconsolidation window
              // Check for pending updates
              if let Some(update) = hot.db.get_pending_update(memory.id)? {
                  // Apply update (memory being reconsolidated with new info)
                  
                  if let Some(new_content) = update.new_content {
                      memory.content = new_content;
                      // Re-embed with new content
                      let new_embedding = self.embed_cache
                          .get_or_embed(&memory.content)
                          .await?;
                      memory.embedding = new_embedding;
                      // Update HNSW index
                      hot.hot_index.remove(memory.id.as_u64_pair().0)?;
                      hot.hot_index.add(
                          memory.id.as_u64_pair().0,
                          &quantize_f16(&memory.embedding),
                      )?;
                      
                      memory.last_reconsolidated_tick = now_tick;
                  }
                  
                  if let Some(new_tag) = update.new_emotional_tag {
                      memory.emotional_tag = new_tag;
                      memory.bypass_decay = compute_bypass_decay(&new_tag);
                  }
                  
                  // Reconsolidation strengthens the memory
                  memory.base_strength = (memory.base_strength
                      + RECONSOLIDATION_BONUS).min(MAX_STRENGTH);
                  
                  hot.db.delete_pending_update(memory.id)?;
              }
          } else {
              // Window expired — restabilize without update
              memory.state = MemoryState::Stable;
              // Any pending update NOT applied (window closed)
              hot.db.delete_pending_update(memory.id)?;
          }
          
          hot.db.update_memory(&memory)?;
      }
      
      Ok(())
  }
```

---

### 4.9 Active Forgetting → ForgettingEngine

```
BIOLOGICAL:
  RAC1 GTPase actively degrades weak memories.
  Interference between similar memories.
  Sleep homeostasis: global synaptic downscaling.
  Directed forgetting: PFC can suppress hippocampal retrieval.

PORT TO membrain:

  pub struct ForgettingEngine {
      config: ForgettingConfig,
  }
  
  pub struct ForgettingConfig {
      interference_sim_min: f32,       // 0.70 — below this: not similar enough to interfere
      interference_sim_max: f32,       // 0.99 — above this: duplicate, handle differently
      retroactive_penalty: f32,        // 0.05 — strength reduction per interfering memory
      proactive_penalty: f32,          // 0.05 — retrieval_difficulty increase
      predictive_value_threshold: f32, // 0.001 — access_count/age below this → accelerated decay
      predictive_decay_factor: f32,    // 0.85 — additional decay for non-predictive memories
      prune_batch_size: usize,         // 10000 — max memories scanned per prune cycle
      minimum_age_for_predictive: u64, // 500 ticks — don't predictively prune new memories
  }
  
  impl ForgettingEngine {
  
      // ── COMPONENT 1: Retroactive Interference ──────────────────────────
      // New memory B weakens old similar memories A
      // Called during encoding of B
      pub fn apply_retroactive_interference(
          &self,
          new_memory: &Memory,
          hot: &mut HotStore,
          now_tick: u64,
      ) -> Result<usize> {
          // Find memories similar to new memory but not identical
          let similar = hot.hot_index.search_filtered(
              &new_memory.embedding_i16,
              100,
              |id| id != new_memory.id.as_u64_pair().0,
          )?;
          
          let mut affected = 0;
          
          for (old_id, similarity) in similar {
              if similarity < self.config.interference_sim_min
                  || similarity > self.config.interference_sim_max {
                  continue;
              }
              
              let old_uuid = hot.id_for_usearch(old_id)?;
              let mut old_memory = hot.db.get_memory_index(old_uuid)?;
              
              // Only penalize if old memory is older (temporal direction)
              if old_memory.created_tick < new_memory.created_tick {
                  // Retroactive: new weakens old
                  old_memory.base_strength *= 1.0 - self.config.retroactive_penalty;
                  // Also apply lazy decay first
                  old_memory.base_strength = effective_strength(&old_memory, now_tick)
                      * (1.0 - self.config.retroactive_penalty);
                  hot.db.update_strength(old_uuid, old_memory.base_strength)?;
                  affected += 1;
              }
          }
          
          Ok(affected)
      }
      
      // ── COMPONENT 2: Proactive Interference ────────────────────────────
      // Old memory A increases retrieval_difficulty of new similar memory B
      // Called during encoding of B
      pub fn apply_proactive_interference(
          &self,
          new_memory: &mut Memory,
          hot: &HotStore,
      ) -> Result<()> {
          let similar_old = hot.hot_index.search(
              &new_memory.embedding_i16, 50
          )?;
          
          let interference_count = similar_old.iter()
              .filter(|(_, sim)| {
                  *sim > self.config.interference_sim_min
                  && *sim < self.config.interference_sim_max
              })
              .count();
          
          // Retrieval_difficulty increased based on number of interfering memories
          // More competition → harder to recall this specific memory
          new_memory.retrieval_difficulty +=
              interference_count as f32 * self.config.proactive_penalty;
          
          Ok(())
      }
      
      // ── COMPONENT 3: Predictive Pruning ────────────────────────────────
      // Memories never recalled → accelerated decay
      pub fn predictive_pruning_pass(
          &self,
          hot: &mut HotStore,
          now_tick: u64,
      ) -> Result<usize> {
          // Find memories with very low predictive value
          let candidates = hot.db.query_predictive_prune_candidates(
              now_tick,
              self.config.minimum_age_for_predictive,
              self.config.predictive_value_threshold,
              self.config.prune_batch_size,
          )?;
          
          // SQL equivalent:
          // SELECT id, access_count, created_tick FROM memory_index
          // WHERE (CAST(access_count AS REAL) / (? - created_tick)) < ?
          // AND (? - created_tick) > ?
          // AND bypass_decay = 0
          // LIMIT ?
          
          let mut pruned = 0;
          
          for memory in candidates {
              // Apply accelerated decay
              hot.db.execute(
                  "UPDATE memory_index 
                   SET base_strength = base_strength * ?
                   WHERE id = ?",
                  params![self.config.predictive_decay_factor, memory.id],
              )?;
              
              // Check if now below MIN_STRENGTH
              let new_strength = memory.base_strength * self.config.predictive_decay_factor;
              if new_strength < MIN_STRENGTH {
                  hot.archive_memory(memory.id, now_tick, ArchiveReason::PredictivePrune)?;
                  pruned += 1;
              }
          }
          
          Ok(pruned)
      }
      
      // ── COMPONENT 4: Capacity Management ───────────────────────────────
      // When total count > SOFT_CAP: archive weakest memories
      pub fn capacity_management(
          &self,
          hot: &mut HotStore,
          cold: &ColdStore,
          now_tick: u64,
      ) -> Result<usize> {
          let total = hot.count() + cold.count();
          
          if total <= SOFT_CAP {
              return Ok(0);
          }
          
          let excess = total - SOFT_CAP;
          let to_archive = (excess as f32 * 1.1) as usize; // archive 10% more than excess
          
          // Archive weakest memories from hot store
          let weakest = hot.db.query_weakest(now_tick, to_archive)?;
          
          let mut archived = 0;
          for memory in weakest {
              hot.archive_memory(memory.id, now_tick, ArchiveReason::CapacityLimit)?;
              archived += 1;
          }
          
          Ok(archived)
      }
  }
```

---

### 4.10 Engram Formation → EngramBuilder + petgraph

```
BIOLOGICAL:
  Related memories share active cells (engram cells).
  Pattern of co-activation → synaptic connections form between those cells.
  Partial activation of pattern → other engram cells recruited.
  → Full memory reconstruction from partial cue.

PORT TO membrain:

  Engram data structures:
  
  pub struct Engram {
      pub id: Uuid,
      pub centroid: Vec<f32>,              // mean of all member embeddings
      pub member_count: usize,
      pub total_strength: f32,             // sum of effective strengths (resonance pool)
      pub created_tick: u64,
      pub last_activated_tick: u64,
      pub parent_engram_id: Option<Uuid>,  // for hierarchical engrams after split
  }
  
  pub struct EngramGraph {
      graph: DiGraph<Uuid, EdgeWeight>,
      node_index: HashMap<Uuid, NodeIndex>,  // Uuid → graph node index
  }
  
  pub struct EdgeWeight {
      pub similarity: f32,
      pub edge_type: EdgeType,
      pub created_tick: u64,
      pub activation_count: u32,  // how many times this edge was traversed in BFS
  }
  
  pub enum EdgeType {
      Associative,    // semantic similarity
      Causal,         // temporal ordering within session (A happened before B)
      Contradictory,  // high similarity but conflicting content
      Temporal,       // general temporal precedence
  }
  
  pub struct EngramBuilder {
      engram_index: Index,  // usearch HNSW for centroid search
      config: EngramConfig,
  }
  
  pub struct EngramConfig {
      formation_threshold: f32,    // 0.65 — min similarity to join existing engram
      max_soft_size: usize,        // 200 — split trigger
      max_hard_size: usize,        // 500 — hard reject
      centroid_update_alpha: f32,  // 0.1 — exponential moving average for centroid update
  }
  
  impl EngramBuilder {
  
      /// Called on every encode — tries to cluster new memory
      pub fn try_cluster(
          &mut self,
          new_id: Uuid,
          new_vec: &[f32],
          engrams: &mut HashMap<Uuid, Engram>,
          graph: &mut EngramGraph,
          now_tick: u64,
      ) -> Option<Uuid> {
          // Search centroid index for nearest engram
          let i16_vec = quantize_f16(new_vec);
          let candidates = self.engram_index.search(&i16_vec, 5).ok()?;
          
          // Find best matching engram
          let best = candidates.iter()
              .filter_map(|(engram_usearch_id, similarity)| {
                  if *similarity > self.config.formation_threshold {
                      let engram_id = self.id_for_usearch(*engram_usearch_id)?;
                      Some((engram_id, *similarity))
                  } else {
                      None
                  }
              })
              .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
          
          match best {
              Some((engram_id, similarity)) => {
                  let engram = engrams.get_mut(&engram_id)?;
                  
                  // Check size limits
                  if engram.member_count >= self.config.max_hard_size {
                      // Hard limit: create sibling engram instead
                      return self.create_new_engram(new_id, new_vec, engrams, graph, now_tick);
                  }
                  
                  // Add to engram
                  engram.member_count += 1;
                  engram.last_activated_tick = now_tick;
                  
                  // Update centroid (exponential moving average)
                  for (i, &v) in new_vec.iter().enumerate() {
                      engram.centroid[i] = engram.centroid[i]
                          * (1.0 - self.config.centroid_update_alpha)
                          + v * self.config.centroid_update_alpha;
                  }
                  
                  // Update engram centroid index
                  self.engram_index.remove(engram_id.as_u64_pair().0).ok();
                  self.engram_index.add(
                      engram_id.as_u64_pair().0,
                      &quantize_f16(&engram.centroid)
                  ).ok();
                  
                  // Add graph node and edge
                  let new_node = graph.graph.add_node(new_id);
                  graph.node_index.insert(new_id, new_node);
                  
                  // Find most similar existing member to connect to
                  // (this is the "neurons that fire together, wire together" edge)
                  // For simplicity: connect to engram centroid's nearest member
                  let nearest_member = self.find_nearest_member(
                      new_vec, &engram_id, engrams, graph
                  );
                  if let Some(member_id) = nearest_member {
                      let member_node = graph.node_index[&member_id];
                      graph.graph.add_edge(
                          new_node,
                          member_node,
                          EdgeWeight {
                              similarity,
                              edge_type: EdgeType::Associative,
                              created_tick: now_tick,
                              activation_count: 0,
                          }
                      );
                  }
                  
                  // Trigger split if soft limit exceeded
                  if engram.member_count > self.config.max_soft_size {
                      self.split_engram(engram_id, engrams, graph, now_tick);
                  }
                  
                  Some(engram_id)
              }
              None => {
                  // No matching engram — create new one
                  self.create_new_engram(new_id, new_vec, engrams, graph, now_tick)
              }
          }
      }
      
      /// BFS from seed memory → collect cluster
      pub fn bfs_cluster(
          &mut self,
          seed_id: Uuid,
          graph: &EngramGraph,
          engrams: &HashMap<Uuid, Engram>,
          max_depth: usize,
          max_nodes: usize,
          min_edge_weight: f32,
      ) -> Vec<Uuid> {
          let seed_node = match graph.node_index.get(&seed_id) {
              Some(n) => *n,
              None => return vec![seed_id],
          };
          
          let mut visited = HashSet::new();
          let mut result = Vec::new();
          // Priority queue: (weight, depth, NodeIndex)
          // Higher weight edges processed first
          let mut queue: BinaryHeap<(OrderedFloat<f32>, usize, NodeIndex)> = BinaryHeap::new();
          
          queue.push((OrderedFloat(1.0), 0, seed_node));
          visited.insert(seed_node);
          
          while let Some((edge_weight, depth, node)) = queue.pop() {
              if result.len() >= max_nodes {
                  break;
              }
              
              let memory_id = graph.graph[node];
              result.push(memory_id);
              
              if depth >= max_depth {
                  continue;
              }
              
              // Explore neighbors
              for neighbor in graph.graph.neighbors(node) {
                  if visited.contains(&neighbor) {
                      continue;
                  }
                  
                  // Get edge weight
                  let edge = graph.graph.find_edge(node, neighbor).unwrap();
                  let ew = &graph.graph[edge];
                  
                  if ew.similarity >= min_edge_weight {
                      visited.insert(neighbor);
                      queue.push((OrderedFloat(ew.similarity), depth + 1, neighbor));
                  }
              }
          }
          
          result
      }
      
      /// Split an oversized engram into two sub-engrams
      fn split_engram(
          &mut self,
          engram_id: Uuid,
          engrams: &mut HashMap<Uuid, Engram>,
          graph: &mut EngramGraph,
          now_tick: u64,
      ) {
          // Get all member IDs
          let members: Vec<Uuid> = graph.node_index.keys()
              .filter(|id| {
                  // Check if this node belongs to this engram
                  // (stored in memory_index.engram_id)
                  true // simplified — actual impl queries hot.db
              })
              .copied()
              .collect();
          
          if members.len() < 4 {
              return; // Too small to split
          }
          
          // Simple k-means (k=2) to find two clusters
          // For speed: use random seed points, 10 iterations
          let (cluster_a, cluster_b) = self.kmeans_2(members, engrams);
          
          // Create two child engrams
          let child_a_id = Uuid::new_v4();
          let child_b_id = Uuid::new_v4();
          
          let centroid_a = compute_centroid(&cluster_a);
          let centroid_b = compute_centroid(&cluster_b);
          
          engrams.insert(child_a_id, Engram {
              id: child_a_id,
              centroid: centroid_a.clone(),
              member_count: cluster_a.len(),
              parent_engram_id: Some(engram_id),
              created_tick: now_tick,
              last_activated_tick: now_tick,
              total_strength: 0.0,
          });
          
          engrams.insert(child_b_id, Engram {
              id: child_b_id,
              centroid: centroid_b.clone(),
              member_count: cluster_b.len(),
              parent_engram_id: Some(engram_id),
              created_tick: now_tick,
              last_activated_tick: now_tick,
              total_strength: 0.0,
          });
          
          // Add child centroids to engram index
          self.engram_index.add(child_a_id.as_u64_pair().0, &quantize_f16(&centroid_a)).ok();
          self.engram_index.add(child_b_id.as_u64_pair().0, &quantize_f16(&centroid_b)).ok();
          
          // Original engram becomes parent (remove from active search)
          self.engram_index.remove(engram_id.as_u64_pair().0).ok();
          
          // Update memory_index.engram_id for all members
          // cluster_a → child_a_id, cluster_b → child_b_id
      }
  }
```

---

### 4.11 Pattern Completion → 3-Tier Retrieval Engine

```
BIOLOGICAL:
  CA3: partial input → autoassociative completion → full pattern.
  CA1: gates output, checks novelty.
  Entorhinal cortex: distributes to neocortex.
  Result: partial cue → full episodic cluster reconstructed.

PORT TO membrain (complete 3-tier retrieval):

  pub struct RecallQuery {
      pub content: String,
      pub context: Option<String>,
      pub top_k: usize,
      pub confidence_requirement: ConfidenceLevel,
      pub include_kinds: Option<Vec<MemoryKind>>,
      pub min_strength: f32,
      pub include_decaying: bool,   // Feature: surface memories about to be lost
  }
  
  pub enum ConfidenceLevel {
      FastApprox,   // ef=10, ~85% accuracy
      Normal,       // ef=50, ~95% accuracy  
      High,         // ef=100, ~99% accuracy
  }
  
  pub struct RecallResult {
      pub memories: Vec<ScoredMemory>,
      pub tier_used: RetrievalTier,
      pub engram_expanded: bool,
      pub tip_of_tongue: Option<Vec<MemoryFragment>>, // partial if no full match
      pub latency_us: u64,
  }
  
  pub struct ScoredMemory {
      pub id: Uuid,
      pub content: String,
      pub score: f32,
      pub effective_strength: f32,
      pub emotional_tag: EmotionalTag,
      pub kind: MemoryKind,
      pub created_tick: u64,
      pub access_count: u32,
      pub engram_id: Option<Uuid>,
      pub decaying_soon: bool,  // Feature: near MIN_STRENGTH warning
  }
  
  pub async fn recall(
      &mut self,
      query: RecallQuery,
      now_tick: u64,
  ) -> Result<RecallResult> {
      let start = std::time::Instant::now();
      
      // === STEP 1: Embed query ===
      let query_vec = self.embed_cache.get_or_embed(&query.content).await?;
      let context_vec = if let Some(ref ctx) = query.context {
          Some(self.embed_cache.get_or_embed(ctx).await?)
      } else {
          None
      };
      
      // === TIER 1: LruCache familiarity check ===
      let content_hash = xxhash64(query.content.as_bytes());
      if let Some(cached) = self.tier1_cache.get(&content_hash) {
          if cached.effective_strength > TIER1_CONFIDENCE_THRESHOLD { // 0.9
              // Cache hit with high confidence → return immediately
              let scored = self.score_cached(cached, &query_vec, &context_vec, now_tick);
              if scored.score > TIER1_CONFIDENCE_THRESHOLD {
                  return Ok(RecallResult {
                      memories: vec![scored],
                      tier_used: RetrievalTier::Tier1,
                      engram_expanded: false,
                      tip_of_tongue: None,
                      latency_us: start.elapsed().as_micros() as u64,
                  });
              }
          }
      }
      
      // === TIER 2: Hot HNSW search ===
      let ef = self.adaptive_ef(&query, now_tick);
      
      // SQL pre-filter: reduce search space BEFORE HNSW
      let candidate_ids = self.hot.prefilter_candidates(
          now_tick,
          query.min_strength,
          PRE_FILTER_LIMIT, // 5000
      )?;
      
      // HNSW search with int16 query
      let i16_query = quantize_f16(&query_vec);
      let raw_candidates = self.hot.hot_index.search_with_filter(
          &i16_query,
          100,  // top-100 candidates
          ef,
          |id| candidate_ids.contains(&id),  // filter to pre-filtered set
      )?;
      
      // Rescore with float32
      let rescored = self.rescore_candidates(
          &raw_candidates,
          &query_vec,
          &context_vec,
          now_tick,
          &self.primed_contexts,
          self.hot.get_engram_graph(),
      ).await?;
      
      // Check if hot search found confident results
      if let Some(top) = rescored.first() {
          if top.score > TIER2_CONFIDENCE_THRESHOLD { // 0.8
              // Hot hit — update Tier1 cache
              self.tier1_cache.put(content_hash, CachedMemory {
                  id: top.id,
                  content: top.content.clone(),
                  embedding: query_vec.clone(),
                  effective_strength: top.effective_strength,
                  cached_at_tick: now_tick,
              });
              
              // Engram expansion
              let expanded = self.expand_engrams(
                  &rescored,
                  now_tick,
                  query.top_k,
              ).await?;
              
              // on_recall for all returned memories
              for m in &expanded {
                  self.on_recall(m.id, now_tick).await?;
              }
              
              let is_decaying_soon = self.check_decaying_soon(&expanded, now_tick);
              
              return Ok(RecallResult {
                  memories: self.attach_decaying_flag(expanded, is_decaying_soon),
                  tier_used: RetrievalTier::Tier2,
                  engram_expanded: true,
                  tip_of_tongue: None,
                  latency_us: start.elapsed().as_micros() as u64,
              });
          }
      }
      
      // === TIER 3: Cold mmap search ===
      let i8_query = quantize_i8(&query_vec);
      let cold_raw = self.cold.cold_index.search(&i8_query, 100)?;
      
      // Rescore cold candidates
      let cold_rescored = self.rescore_cold_candidates(
          &cold_raw,
          &query_vec,
          &context_vec,
          now_tick,
      ).await?;
      
      // Merge hot + cold results
      let mut all_candidates = rescored;
      all_candidates.extend(cold_rescored);
      all_candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
      all_candidates.truncate(query.top_k * 3); // keep 3× top_k for engram expansion
      
      // Engram expansion from cold results too
      let final_results = self.expand_engrams(&all_candidates, now_tick, query.top_k).await?;
      
      // on_recall for all returned memories
      for m in &final_results {
          self.on_recall(m.id, now_tick).await?;
      }
      
      // Check for tip-of-tongue (no confident match found anywhere)
      let tip_of_tongue = if final_results.is_empty()
          || final_results[0].score < PARTIAL_RECALL_THRESHOLD // 0.4
      {
          Some(self.construct_fragments(&all_candidates))
      } else {
          None
      };
      
      Ok(RecallResult {
          memories: final_results,
          tier_used: RetrievalTier::Tier3,
          engram_expanded: !all_candidates.is_empty(),
          tip_of_tongue,
          latency_us: start.elapsed().as_micros() as u64,
      })
  }
  
  /// Unified scoring function combining all signals
  fn score_candidate(
      &self,
      candidate: &MemoryRecord,
      query_vec: &[f32],
      context_vec: &Option<Vec<f32>>,
      now_tick: u64,
      primed_contexts: &[PrimedContext],
      engram_graph: &EngramGraph,
      resonance_scores: &HashMap<Uuid, f32>,
  ) -> f32 {
      // 1. Semantic similarity (primary signal)
      let semantic_sim = cosine_sim(query_vec, &candidate.embedding_f32);
      
      // 2. Context boost
      let context_boost = if let Some(ctx_vec) = context_vec {
          CONTEXT_WEIGHT * cosine_sim(ctx_vec, &candidate.context_embedding)
      } else {
          0.0
      };
      
      // 3. Memory strength (lazy Ebbinghaus)
      let strength = effective_strength(&candidate.index, now_tick);
      
      // 4. Recency bias (recent memories slightly preferred, log-scaled)
      let age = (now_tick - candidate.index.created_tick) as f32;
      let recency = 1.0 + 0.1 / (1.0 + age.ln().max(0.0));
      
      // 5. Retrieval difficulty penalty (proactive interference)
      let difficulty_penalty = 1.0 - candidate.index.retrieval_difficulty.min(0.5);
      
      // 6. Priming boost (spotlight mode)
      let prime_boost = primed_contexts.iter()
          .filter(|p| now_tick < p.expiry_tick)
          .map(|p| {
              p.boost * cosine_sim(
                  if let Some(cv) = context_vec { cv } else { query_vec },
                  &p.embedding
              )
          })
          .fold(0.0_f32, f32::max);
      
      // 7. Engram resonance (from BFS traversal)
      let resonance = resonance_scores.get(&candidate.id).copied().unwrap_or(0.0);
      
      // Final score
      let base_score = (semantic_sim + context_boost) * strength * recency * difficulty_penalty;
      base_score + prime_boost + resonance
  }
```

---

### 4.12 Encoding Specificity → context_embedding

```
BIOLOGICAL:
  Context at encoding stored alongside content.
  Matching context at retrieval → better recall.
  
PORT TO membrain:

  // Every memory stores two embeddings:
  struct Memory {
      // ... other fields ...
      embedding_f32: Vec<f32>,          // content embedding
      context_embedding: Vec<f32>,       // context embedding (same dims)
  }
  
  // Encoding:
  async fn encode(
      &mut self,
      content: String,
      context: Option<String>,
      // ...
  ) -> Result<Uuid> {
      let content_vec = self.embed_cache.get_or_embed(&content).await?;
      
      // Context: either provided or constructed from current state
      let context_str = context.unwrap_or_else(|| {
          self.working_memory.central_executive.current_task.clone()
      });
      let context_vec = self.embed_cache.get_or_embed(&context_str).await?;
      
      // Both vectors stored — same dimensions (384)
      // Storage overhead: 2× embeddings per memory
      // But: cold tier int8, so 2 × 384 bytes = 768 bytes per memory (manageable)
  }
  
  // Retrieval scoring:
  fn score_with_context(
      content_sim: f32,
      context_sim: f32,
  ) -> f32 {
      CONTENT_WEIGHT * content_sim + CONTEXT_WEIGHT * context_sim
      // = 0.7 × content_sim + 0.3 × context_sim
  }
  
  // EFFECT:
  // Agent working on auth module (context="debugging authentication JWT"):
  //   Memory about JWT from auth session: context_sim ≈ 0.85
  //   Memory about JWT from payments session: context_sim ≈ 0.40
  //   
  //   Score difference for same content:
  //   auth-context JWT memory: 0.7 × 0.9 + 0.3 × 0.85 = 0.63 + 0.255 = 0.885
  //   payment-context JWT memory: 0.7 × 0.9 + 0.3 × 0.40 = 0.63 + 0.12 = 0.750
  //   
  //   Auth-context memory scores 18% higher despite same semantic content.
  //   Natural context-dependent retrieval — no special logic needed.
```

---

### 4.13 Working Memory Capacity → 7-Slot Constraint

```
BIOLOGICAL:
  Miller's Law: 7 ± 2 chunks.
  When full: new items displace old items.
  Displaced items with sufficient strength → encoded to LTM.
  This is the mechanism for converting attended information to long-term memory.

PORT TO membrain:

  The 7-slot constraint creates a NATURAL ENCODING PIPELINE:
  
  Agent receives information:
    → Add to WorkingMemory (slot 1-7)
    → If slot 7+1 needed: evict lowest-attention item
    → Evicted item: if strength > threshold → encode to hot_store
    → If not → lost (simulates forgetting from STM)
    
  Effect:
    - Information that IS focused on (high attention) → persists in WM → eventually encoded
    - Information that ISN'T focused on (low attention) → evicted quickly → not encoded
    - Exactly mirrors human selective encoding
    
  Chunking simulation:
    Agents can "chunk" multiple pieces of information:
    membrain encode --kind Semantic "In this codebase, auth = JWT + OAuth + session tokens"
    This semantic chunk = one WM slot, contains 3 pieces of information.
    Exactly how humans chunk — experts have richer chunks.

  Implementation details:
  
  // Attention scoring for WM items
  fn attention_score_for_item(item: &WorkingMemoryItem, executive: &TaskContext) -> f32 {
      // Items related to current task get attention bonus
      let task_relevance = cosine_sim(
          &executive.task_context_embedding,
          &item.embedding,
      );
      
      // Recent items get recency bonus
      let recency = 1.0 / (1.0 + (executive.current_tick - item.added_tick) as f32 / 10.0);
      
      // Items that have been explicitly focused on score higher
      let explicit_attention = executive.attention_weights
          .get(&item.memory_id)
          .copied()
          .unwrap_or(0.0);
      
      task_relevance * 0.5 + recency * 0.3 + explicit_attention * 0.2
  }
```

---

### 4.14 Pattern Separation → Novelty Detection

```
BIOLOGICAL:
  Dentate Gyrus: separates similar inputs into distinct patterns.
  Prevents interference during storage.
  High neurogenesis rate supports this function.

PORT TO membrain:

  // Novelty score = complement of similarity to nearest neighbor
  // High novelty → memories stored as "separate" from existing
  // Low novelty → interference penalty applied
  
  async fn compute_novelty_score(
      &self,
      new_vec: &[f32],
      hot: &HotStore,
  ) -> f32 {
      // Query HNSW for nearest existing memory
      let i16_vec = quantize_f16(new_vec);
      let nearest = hot.hot_index.search(&i16_vec, 1).await?;
      
      match nearest.first() {
          Some((_, similarity)) => 1.0 - similarity,
          None => 1.0,  // First memory: completely novel
      }
  }
  
  // Novelty → initial strength modifier (novel things remembered better)
  fn novelty_strength_modifier(novelty_score: f32) -> f32 {
      1.0 + novelty_score * NOVELTY_WEIGHT
      // NOVELTY_WEIGHT = 0.3
      // Completely novel (1.0): +30% initial strength
      // Duplicate (0.0): +0% (baseline strength)
  }
  
  // Novelty → duplicate detection
  if novelty_score < DUPLICATE_THRESHOLD { // 0.05
      // Very similar to existing memory → update existing instead of creating new
      let nearest_id = get_nearest_memory_id();
      self.reconsolidate_with_new_content(nearest_id, content, context).await?;
      return Ok(nearest_id);
  }
```

---

### End of Snapshot Part 2

**Next: Part 3 — Architecture Overview & Performance Optimization Stack**

Parts list:
- Part 1: Vision, Problem Statement, Human Brain Deep Dive ✅
- Part 2: Gap Analysis + Full Port (mechanism → Rust code) ✅
- Part 3: Architecture Overview + Performance
- Part 4: Techstack + Data Schema
- Part 5: CLI/MCP + Feature Extensions + Workspace Structure
- Part 6: Milestones + Acceptance Checklist + Constants + Algorithm Reference


<!-- SOURCE: PLAN_part3.md -->

### Source Snapshot — Part 3
#### Part 3 of 6: Architecture Overview · Performance Optimization Stack

---

## 5. Architecture Overview

### 5.1 Design Philosophy

```
THREE LAWS OF membrain ARCHITECTURE:

  LAW 1: MIMIC THE BRAIN'S STRUCTURE
    Every architectural decision maps to a biological counterpart.
    Not metaphorically — mechanistically.
    hippocampus = hot_store, neocortex = cold_store, amygdala = EmotionalTag.
    If brain does X for biological reason Y, membrain does X.
    This is not cargo-culting — evolution optimized these structures for
    exactly the problem we are solving: efficient, scalable, associative memory.
    
  LAW 2: LAZY EVERYTHING
    Compute nothing proactively if it can be computed on demand.
    Decay: computed at recall time (lazy), not on schedule (eager).
    Embedding: cached, computed only on cache miss.
    Index rebuild: only on startup, not during operation.
    Consolidation: pressure-triggered, not scheduled.
    This ensures: O(0) overhead at idle, O(1) overhead per operation.
    
  LAW 3: WRITE TO ONE, READ FROM EVERYWHERE
    Writing (encoding) hits only hot_store.
    Reading (recall) queries tier1 → tier2 → tier3 in order.
    Background processes (consolidation, decay) never block reads.
    SQLite WAL ensures reader/writer don't block each other.
    This ensures: reads are always fast regardless of write activity.
```

### 5.2 Three-Tier Storage Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         MEMBRAIN BRAIN STORE                            │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  TIER 1 — WORKING CACHE (Prefrontal Cortex)                     │   │
│  │  ─────────────────────────────────────────────────────────────  │   │
│  │  Structure:  LruCache<u64, CachedMemory> (512 entries)          │   │
│  │              LruCache<u64, Vec<f32>> (1000 entries, embeddings) │   │
│  │  Latency:    <0.1ms (pure in-process memory)                    │   │
│  │  Capacity:   ~10MB RAM                                          │   │
│  │  Eviction:   LRU (least recently used)                         │   │
│  │  Content:    Recent + frequently accessed memories              │   │
│  │  Writes:     On every recall (cache is populated from recalls)  │   │
│  │  Brain:      7±2 working memory + neocortex familiarity path    │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                              │ miss                                     │
│                              ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  TIER 2 — HOT HNSW INDEX (Hippocampus)                         │   │
│  │  ─────────────────────────────────────────────────────────────  │   │
│  │  Structure:  usearch HNSW, float16, in-memory, 50k limit       │   │
│  │              SQLite hot.db (WAL mode, metadata + content)       │   │
│  │              petgraph DiGraph (engram graph, in-memory)         │   │
│  │  Latency:    <5ms (HNSW O(log n) + float32 rescore)            │   │
│  │  Capacity:   ~75MB RAM (50k × 384 dims × 2 bytes float16)      │   │
│  │  Eviction:   NREM consolidation (pressure-triggered migration)  │   │
│  │  Content:    Recent episodic, partially consolidated memories   │   │
│  │  Brain:      Hippocampus — episodic index, pattern completion   │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                              │ miss                                     │
│                              ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  TIER 3 — COLD MMAP INDEX (Neocortex)                          │   │
│  │  ─────────────────────────────────────────────────────────────  │   │
│  │  Structure:  usearch HNSW, int8, mmap disk-backed, unlimited   │   │
│  │              SQLite cold.db (zstd-compressed content)          │   │
│  │  Latency:    <50ms (mmap HNSW + OS page cache)                 │   │
│  │  Capacity:   Unlimited (disk-bounded, TB-scale)                │   │
│  │  Eviction:   None (archive only — never delete)                │   │
│  │  Content:    Consolidated semantic memories                    │   │
│  │  Brain:      Neocortex — vast permanent storage                │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  PROCEDURAL STORE (Cerebellum)                                  │   │
│  │  SQLite procedural.db — hash → action, no decay, O(1) lookup   │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  ENGRAM GRAPH (Synaptic Network)                                │   │
│  │  petgraph DiGraph<Uuid, EdgeWeight> — in-memory, persisted      │   │
│  │  to hot.db/engrams + edges tables                               │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.3 Complete Encode + Retrieve Data Flows

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ENCODE PATH (Agent stores a new memory)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

INPUT: (content, context?, attention_score, emotional_tag, source)
  │
  ├─ [attention_score < 0.2] ──────────────────────────→ DISCARD
  │   Sensory filter: not attended to → not encoded
  │
  ▼
[EMBEDDING CACHE CHECK]
  content_hash = xxhash64(content)
  if embed_cache.get(content_hash):
      content_vec = cached (0ms)
  else:
      content_vec = fastembed.embed(content) (~5ms)
      embed_cache.put(content_hash, content_vec)
  
  context_hash = xxhash64(context)
  context_vec  = embed_cache.get_or_embed(context) (same pattern)
  │
  ▼
[NOVELTY SCORE]
  i16_query = quantize_f16(content_vec)
  nearest = hot_index.search(i16_query, 1)
  novelty_score = 1.0 - nearest[0].similarity  (or 1.0 if empty)
  
  if novelty_score < DUPLICATE_THRESHOLD (0.05):
      → update existing memory instead of creating new
      → reconsolidate with new content
      → RETURN existing_id
  │
  ▼
[INITIAL STRENGTH CALCULATION]
  emotional_multiplier = 1.0 + (arousal × |valence| × 0.5)
  initial_strength = BASE_STRENGTH (0.5)
                   × (1.0 + novelty_score × 0.3)
                   × (1.0 + attention_score × 0.4)
                   × emotional_multiplier
  
  bypass_decay = arousal > 0.6 && |valence| > 0.5
  state = MemoryState::Labile { since: now_tick, window: reconsolidation_window(0, initial_strength) }
  │
  ▼
[WORKING MEMORY UPDATE]
  working_memory.add(item, &mut hot_store)
  if working_memory was full:
      evicted_item → hot_store.encode(evicted) if strong enough
  │
  ▼
[HOT STORE INSERT]
  BEGIN TRANSACTION
    id = Uuid::new_v4()
    INSERT INTO memory_index (id, base_strength, stability, last_tick, bypass_decay, kind, ...)
    INSERT INTO memory_content (id, content)
    INSERT INTO memory_vectors (id, embedding_f32_blob)
  COMMIT
  │
  ▼
[HNSW HOT INDEX UPDATE]
  hot_index.add(id.as_u64(), quantize_f16(content_vec))
  
  engram_builder.try_cluster(id, content_vec, now_tick)
  → if clusters: add to engram, update centroid, add graph edge
  → if isolated: create new engram seed
  │
  ▼
[INTERFERENCE CHECK]
  forgetting_engine.apply_retroactive(new_memory, hot, now_tick)
  → find memories 0.7 < sim < 0.99
  → weaken old memories: base_strength × (1 - 0.05)
  
  forgetting_engine.apply_proactive(new_memory, hot)
  → new_memory.retrieval_difficulty += count_similar × 0.05
  │
  ▼
[TIER1 CACHE UPDATE]
  tier1_cache.put(content_hash, CachedMemory { id, content, embedding, ... })

RESULT: Uuid of new memory
Total latency: <1ms (cache hit) | <10ms (cache miss, 2 embeds)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
RETRIEVE PATH (Agent recalls memories)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

INPUT: RecallQuery { content, context?, top_k, confidence_level, ... }
  │
  ▼
[EMBED QUERY]
  query_vec    = embed_cache.get_or_embed(query.content)
  context_vec  = embed_cache.get_or_embed(query.context) if present
  content_hash = xxhash64(query.content)
  │
  ▼
[TIER 1 — LruCache lookup] ─────────────────────── Target: <0.1ms
  tier1_cache.get(content_hash)?
  if hit && cached.effective_strength > 0.9:
      score = score_candidate(cached, query_vec, context_vec, ...)
      if score > TIER1_THRESHOLD:
          on_recall(cached.id)   // LTP + labile
          ──────────────────────→ RETURN (tier1 hit)
  │ miss or low confidence
  ▼
[SQL PRE-FILTER] ─────────────────────────────────── Target: <0.5ms
  SELECT id, base_strength, stability, last_tick, ...
  FROM memory_index
  WHERE effective_strength(base_strength, stability, now-last_tick) > MIN_STRENGTH
    AND state NOT IN (Archived, OutOfHot)
  LIMIT 5000
  → candidate_ids: Vec<Uuid>  (search space reduced by 200×)
  │
  ▼
[TIER 2 — HNSW Hot Search] ──────────────────────── Target: <5ms
  ef = adaptive_ef(query.confidence_level, hot_count, tier1_hit_rate)
  i16_query = quantize_f16(query_vec)
  raw_hits = hot_index.search_filtered(i16_query, 100, ef,
      |id| candidate_ids.contains(id))
  
  // Rescore top-100 with float32 (accuracy recovery)
  rescored = []
  for (hit_id, _) in raw_hits[..20]:  // only top-20 need rescore
      embedding_f32 = hot.fetch_embedding(hit_id)?
      score = score_candidate(hit_id, query_vec, context_vec, ...)
      rescored.push((score, hit_id))
  
  rescored.sort_by_score_desc()
  
  if rescored[0].score > TIER2_THRESHOLD (0.8):
      // Engram expansion
      seed_engram = hot.get_engram_id(rescored[0].id)
      cluster = engram_builder.bfs_cluster(
          rescored[0].id,
          max_depth=3, max_nodes=50, min_edge_weight=0.5
      )
      
      // Merge HNSW results + engram cluster
      all_ids = union(rescored.ids, cluster)
      all_scored = rescore_all(all_ids, query_vec, context_vec, ...)
      all_scored.sort_desc().truncate(top_k)
      
      // on_recall for results → LTP + labile + resonance
      for m in all_scored: on_recall(m.id, now_tick)
      
      // Update Tier1 cache
      tier1_cache.put(content_hash, top_result.into_cached())
      
      ──────────────────────────→ RETURN (tier2 hit)
  │ miss
  ▼
[TIER 3 — HNSW Cold Search] ─────────────────────── Target: <50ms
  i8_query = quantize_i8(query_vec)
  cold_hits = cold_index.search(i8_query, 100)
  
  // Rescore: fetch float32 from cold.db
  cold_rescored = rescore_cold(cold_hits, query_vec, context_vec, ...)
  
  // Merge hot + cold
  all = merge_and_deduplicate(rescored_hot, cold_rescored)
  all.sort_desc().truncate(top_k * 3)
  
  // Engram expansion from merged set
  expanded = expand_engrams_from_set(all, top_k)
  for m in expanded: on_recall(m.id, now_tick)
  
  // Tip-of-tongue check
  if expanded.is_empty() || expanded[0].score < 0.4:
      tip_of_tongue = construct_fragments(all)
  
  ────────────────────────────→ RETURN (tier3 hit or tip-of-tongue)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
BACKGROUND PROCESSES (never block encode or retrieve)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

CONSOLIDATION CYCLE (async tokio task):
  Trigger: hot_index.len() > HOT_CAPACITY × 0.9
        OR every CONSOLIDATION_INTERVAL interactions
  
  Step 1 — NREM: score hot → migrate top-N to cold
  Step 2 — REM:  process emotional queue → desensitize + cross-link
  Step 3 — Homeostasis: if total_strength > threshold → scale + prune
  
  SQLite WAL: consolidation writes (to cold.db) do NOT block reads (from hot.db).
              Multiple readers of hot.db work simultaneously.
  
RECONSOLIDATION TICK (async, per interaction):
  For each Labile memory:
    if window not expired && has pending_update:
        apply update, re-embed, strengthens memory
    else if window expired:
        → Stable state
  
FORGETTING ENGINE (async, periodic):
  Trigger: every FORGETTING_ENGINE_INTERVAL interactions
  - Retroactive interference: applied synchronously during encode
  - Proactive interference: applied synchronously during encode
  - Predictive pruning: async batch
  - Capacity management: async if > SOFT_CAP
```

### 5.4 Process Model — Daemon vs Standalone

```
TWO MODES OF OPERATION:

  ─────────────────────────────────────────────────────────────────
  MODE 1: DAEMON (recommended for agents)
  ─────────────────────────────────────────────────────────────────
  
  membrain daemon start
    │
    ▼
  tokio::main runtime spawns:
    ├── embed_model.load() → all-MiniLM-L6-v2 loaded ONCE (~500ms startup)
    ├── hot_index.rebuild_from_db() → HNSW rebuilt from hot.db on start
    ├── engram_graph.load() → petgraph loaded from hot.db
    ├── unix_socket_server (at ~/.membrain/membrain.sock)
    │     → JSON-RPC 2.0 handler (async, concurrent)
    ├── background_task: consolidation_loop (tokio::spawn)
    │     → woken by pressure channels or timer
    ├── background_task: reconsolidation_tick_loop (tokio::spawn)
    │     → woken by interaction counter
    └── background_task: forgetting_engine_loop (tokio::spawn)
          → low-priority, runs when idle
  
  Client connections via Unix socket:
    - membrain CLI: connects via socket
    - Python client: net.connect(~/.membrain/membrain.sock)
    - Node client: net.createConnection(...)
    - Rust: tokio::net::UnixStream
  
  MCP mode (within daemon):
    membrain mcp  →  stdin/stdout JSON-RPC (stdio transport)
    Claude Code / Cursor sees this as MCP server
    Each MCP tool call: client → stdin → daemon → process → stdout → client
  
  ADVANTAGES:
    - Embedding model loaded once (no 500ms cold start per call)
    - HNSW index always warm
    - Background jobs (consolidation) run continuously
    - Multiple clients simultaneously
    - Best performance for long-running agents
  
  ─────────────────────────────────────────────────────────────────
  MODE 2: STANDALONE (CLI, scripts)
  ─────────────────────────────────────────────────────────────────
  
  membrain remember "content"
    │
    ▼
  Binary starts → check for daemon at ~/.membrain/membrain.sock
    → if daemon present: forward request via socket → exit
    → if no daemon: run standalone mode
    
  Standalone mode:
    - Load embedding model (500ms first call, cached thereafter)
    - Open SQLite hot.db (WAL: safe for multiple processes)
    - Execute operation
    - Exit
    
  TRADEOFFS:
    + Simple: no daemon management
    + Works in scripts, CI, one-off calls
    - Cold start latency (~500ms first call per process)
    - No background consolidation (only on explicit call)
    - No warm HNSW cache (rebuilt per process if needed — expensive for large stores)
    
  USE STANDALONE FOR:
    - scripts: echo "content" | membrain remember
    - testing
    - single queries where daemon overhead not worth it
    
  USE DAEMON FOR:
    - Long-running Claude Code sessions
    - Production agent deployments
    - Any scenario with multiple queries per minute

  ─────────────────────────────────────────────────────────────────
  GRACEFUL FALLBACK
  ─────────────────────────────────────────────────────────────────
  
  Daemon down or crashed:
    → CLI automatically falls back to standalone mode
    → No data loss (SQLite WAL ensures durability)
    → Slightly slower (no warm HNSW), but correct
    
  Implementation:
  fn get_connection(config: &Config) -> Connection {
      // Try daemon first
      if let Ok(sock) = UnixStream::connect(&config.socket_path) {
          return Connection::Daemon(sock);
      }
      // Fallback to direct
      Connection::Standalone(open_stores(config))
  }
```

### 5.5 File Layout

```
~/.membrain/
├── config.toml          # User configuration
├── hot.db               # SQLite WAL — hot memories + metadata + engrams
├── cold.db              # SQLite WAL — consolidated semantic memories
├── hot.usearch          # usearch HNSW hot index (float16, rebuilt on daemon start)
├── cold.usearch         # usearch HNSW cold index (int8, mmap persistent)
├── membrain.sock        # Unix socket (daemon mode only)
├── membrain.pid         # PID file (daemon mode only)
├── membrain.log         # Log file (daemon mode)
└── archive/
    └── archive.db       # Archived memories (soft-deleted, recoverable)

NOTE on hot.usearch:
  The hot HNSW index is REBUILT from hot.db on every daemon start.
  It is NOT persisted between restarts (float16, 50k entries, ~75MB RAM).
  Rebuild time: ~2 seconds for 50k entries.
  Why: usearch mmap is for cold only. Hot index fits in RAM and rebuilds fast.
  
NOTE on cold.usearch:
  The cold HNSW index IS persisted (mmap disk-backed).
  It is NOT rebuilt on startup — just remapped.
  This is the unlimited-scale persistent index.
```

---

## 6. Performance — Bottlenecks & Optimization Stack

### 6.1 Bottleneck Analysis

```
The five critical performance bottlenecks in a memory system at scale,
and how membrain addresses each:

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
BOTTLENECK 1: EMBEDDING COST
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Problem:
  Every encode requires 2 embedding calls (content + context).
  fastembed-rs: ~5ms per embedding × 2 = 10ms per encode.
  At 100 encodes/sec: 1000ms in embedding alone — completely blocks.
  
Naive solution: run embeddings synchronously in request path.
Result: 10ms encode latency regardless of other optimizations.

membrain solution: LruCache<u64, Vec<f32>> (embedding cache)
  key   = xxhash64(text_bytes)
  value = Vec<f32> (384 floats)
  capacity = 1000 entries (~1.5MB RAM)
  
  Cache hit rate in practice:
    - Agent recalls same memory content many times → high hit rate
    - Context strings often repeat ("working on auth module")
    - Estimated steady-state hit rate: >80%
    
  Result:
    Cache hit:  0ms (pure HashMap lookup)
    Cache miss: ~5ms (one fastembed call)
    
  Additional optimization: BATCH embedding during consolidation
    Instead of: for m in candidates { embed(m.content) }  // O(n × 5ms)
    Use:        fastembed.embed_batch(all_contents)        // O(1 × 5ms × n/batch_size)
    Speedup: 3-5× for consolidation cycles

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
BOTTLENECK 2: KNN SEARCH COMPLEXITY AT SCALE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Problem:
  With 1M memories and 384 dimensions:
  Brute force: 1M × 384 float multiply-accumulates per query
  = 384M FMAs per query
  At 4 GFLOPS (conservative): ~96ms per query — unacceptable
  
sqlite-vec (the naive choice):
  Uses brute-force KNN.
  Benchmarks: 10k vectors → 10ms, 100k → 100ms, 1M → 1000ms.
  UNUSABLE at scale.
  
membrain solution: usearch HNSW
  Hierarchical Navigable Small World (HNSW) algorithm:
  
  HOW HNSW WORKS:
  ┌──────────────────────────────────────────────────────────────────┐
  │  HNSW builds a multi-layer graph:                               │
  │                                                                   │
  │  Layer 2 (top):  Very sparse graph, long-range connections       │
  │  Layer 1 (mid):  Denser graph, medium-range connections          │
  │  Layer 0 (base): Full graph, all nodes, short-range connections  │
  │                                                                   │
  │  Search:                                                          │
  │  1. Start at entry point in top layer                            │
  │  2. Greedy search: move to nearest neighbor                      │
  │  3. Drop down to lower layer at current position                 │
  │  4. Repeat until Layer 0                                         │
  │  5. At Layer 0: local beam search (ef candidates)               │
  │                                                                   │
  │  Complexity: O(log n) average                                    │
  │  Recall accuracy: ~95% at ef=50 (vs 100% brute force)          │
  │  But: 5% miss compensated by float32 rescore                    │
  └──────────────────────────────────────────────────────────────────┘
  
  Benchmarks (usearch, M=16, ef_construction=200):
    10k vectors:   ~0.5ms
    100k vectors:  ~1ms
    1M vectors:    ~3-5ms (with ef=50)
    100M vectors:  ~10ms (with ef=50)
    
  Improvement over brute force:
    10k:  10ms → 0.5ms  (20×)
    100k: 100ms → 1ms   (100×)
    1M:   1000ms → 5ms  (200×)
    
  usearch specific advantages:
    - AVX2/AVX-512/NEON SIMD: auto-detected at compile time (target-cpu=native)
    - int8/float16/binary quantization: native, zero extra code
    - mmap: disk-backed index, unlimited scale, OS manages paging
    - Multi-threaded search: parallelism for large result sets

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
BOTTLENECK 3: DECAY ITERATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Problem:
  "Decay tick" that iterates all memories is catastrophic at scale.
  50k memories × EXP computation × 100 ticks/sec:
  = 50k × 5ns (EXP op) × 100 = 25ms overhead per second
  This background noise would degrade system performance continuously.
  
Naive solution: decay_tick() iterates all memories periodically.
Result: O(n) operation every K interactions. Blocking. Unacceptable.

membrain solution: LAZY DECAY — computed on demand
  
  core formula: effective_strength(m, now_tick) = m.base_strength × e^(-Δtick/m.stability)
  
  This formula is O(1) — one EXP per memory accessed.
  It is computed:
    1. During recall (score the candidate)
    2. During SQL pre-filter (WHERE clause)
    3. During consolidation (score for migration)
    4. NEVER during idle or background ticks
    
  Mathematical proof of correctness:
    Eager: update every tick T, current strength after N ticks:
      s_N = s_0 × Π(e^(-1/S)) = s_0 × e^(-N/S)
    Lazy: compute once after N ticks:
      s_N = s_0 × e^(-N/S)
    IDENTICAL. Lazy is mathematically equivalent to eager.
    
  Performance impact:
    Idle CPU overhead: ZERO
    Per-recall overhead: 1 EXP + 1 multiply = ~5ns
    Pre-filter overhead: ~5ns × 5000 candidates = 25μs
    Total added latency from decay computation: <1ms

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
BOTTLENECK 4: COLD START EMBEDDING MODEL LOAD
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Problem:
  all-MiniLM-L6-v2 model: 80MB ONNX weights.
  First load: ~500ms (parse model, allocate tensors, warm ONNX runtime).
  
Naive: load model per process invocation.
Result: every CLI call has 500ms latency.

membrain solution: daemon mode — load ONCE
  Daemon startup:
    1. embed_model.load() → 500ms (one time)
    2. All subsequent calls: model warm → ~5ms per embed
    
  Standalone fallback:
    Process-level caching: model loaded once per process lifecycle.
    For scripts calling membrain in loop: 500ms first call, 5ms all subsequent.
    For one-off CLI calls: 500ms is acceptable.
    
  Alternative (future): precompute embeddings for common content patterns.
  The embedding cache already handles this for repeated content.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
BOTTLENECK 5: CONSOLIDATION BLOCKING RETRIEVAL
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Problem:
  Consolidation (NREM/REM/Homeostasis) involves:
    - Re-embedding many memories
    - Writing to cold.db
    - Updating engram centroids
  Could take seconds for large migration batches.
  If synchronous: retrieval blocked during consolidation.
  
membrain solution: async tokio + SQLite WAL
  Consolidation runs as a background tokio task (async).
  SQLite WAL: reads and writes don't block each other.
    - Consolidation writing to cold.db: non-blocking for hot.db reads
    - Engram graph updates: done with tokio::RwLock<EngramGraph>
      → readers can access graph during consolidation
      → writer lock only held during centroid updates (microseconds)
  
  Net impact on retrieval: 0ms
```

### 6.2 Optimization Stack — Seven Layers

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
OPT 1: TIERED INDEX ARCHITECTURE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Three tiers matching brain's three access speeds:

  Tier 1: LruCache (in-process RAM)
    Technology: lru crate, LruCache<u64, CachedMemory>
    Size: 512 entries (configurable)
    Hit rate target: >60%
    Latency: <0.1ms (pure HashMap get + cosine sim)
    Use: familiarity check, recently accessed memories
    
  Tier 2: usearch HNSW float16 (in-memory)
    Technology: usearch, float16 quantization, 50k limit
    Size: ~75MB RAM (50k × 384 × 2 bytes)
    Hit rate target: >90% of non-Tier1 queries
    Latency: <5ms (HNSW O(log n) + SQL pre-filter + float32 rescore)
    Use: episodic memories, recent consolidations
    
  Tier 3: usearch HNSW int8 mmap (disk)
    Technology: usearch mmap, int8 quantization, unlimited
    Size: ~384MB per 1M memories (384 × 1 byte)
    Hit rate: 100% of remaining queries (catch-all)
    Latency: <50ms (mmap HNSW + OS page cache)
    Use: deep semantic memories, historical knowledge

COMBINED PERFORMANCE MODEL:
  Assume: 60% Tier1 hit, 35% Tier2 hit, 5% Tier3
  
  avg_latency = 0.60 × 0.1ms + 0.35 × 5ms + 0.05 × 50ms
              = 0.06ms + 1.75ms + 2.5ms
              = ~4.3ms average
              
  vs naive (all Tier3 brute force): 50-1000ms
  Improvement: 10-230×

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
OPT 2: INT8/FLOAT16 QUANTIZATION + FLOAT32 RESCORE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
The Problem: float32 vectors are large and slow to compare.
384 dims × float32 = 1536 bytes per vector
SIMD processes 8 floats per AVX2 operation = 48 AVX2 ops per distance
At 100 queries × 5000 candidates: 24M AVX2 ops per second — feasible but tight.

Quantization solution:

  float16 (hot tier):
    Range: ±65504 (same numerical range as float32 for typical embeddings)
    Size:  384 × 2 = 768 bytes per vector (50% reduction)
    Speed: 2 floats processed per byte → same AVX2 ops, 2× data bandwidth
    Accuracy: 0.1-0.3% degradation in cosine similarity
    Use: Tier 2 hot HNSW search
    
  int8 (cold tier):
    Range: -128 to 127 (requires normalization of embedding range)
    Size:  384 × 1 = 384 bytes per vector (75% reduction)
    Speed: 4× smaller → 4× more vectors fit in CPU cache → 2-4× faster
    Accuracy: 1-2% degradation in cosine similarity
    Use: Tier 3 cold HNSW search
    
  float32 rescore (final ranking):
    After HNSW returns top-100 candidates (int8/float16 search):
    Fetch float32 embeddings from SQLite for top-20 only
    Compute exact cosine similarity with query (float32)
    Re-rank based on exact similarity
    
    This compensates for quantization error:
    "Coarse search" (quantized) finds the right neighborhood.
    "Fine search" (float32) ranks the neighborhood accurately.
    
    Net accuracy: ~99% vs pure float32 search (within 1% of optimal ranking)
    Net speedup: 4× (int8) vs float32 with <1% quality loss

QUANTIZATION IMPLEMENTATION:

  // Float32 → Float16
  fn quantize_f16(v: &[f32]) -> Vec<f16> {
      v.iter().map(|&x| f16::from_f32(x)).collect()
  }
  
  // Float32 → Int8 (with normalization)
  fn quantize_i8(v: &[f32]) -> Vec<i8> {
      let max_abs = v.iter().map(|x| x.abs()).fold(0.0_f32, f32::max);
      let scale = 127.0 / max_abs.max(1e-8);
      v.iter().map(|&x| (x * scale).round().clamp(-128.0, 127.0) as i8).collect()
  }
  
  // Int8 → approximate Float32 (for rescore, keep scale factor)
  fn dequantize_i8(v: &[i8], scale: f32) -> Vec<f32> {
      v.iter().map(|&x| x as f32 / scale).collect()
  }

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
OPT 3: SIMD DISTANCE COMPUTATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
usearch auto-detects and uses SIMD:
  x86_64:  AVX-512 (16 floats/op) > AVX2 (8 floats/op) > SSE4.2 (4 floats/op)
  ARM64:   NEON (4 floats/op) > SVE (variable)
  
  Enable with: RUSTFLAGS="-C target-cpu=native" cargo build --release
  
  Speedup from AVX2 vs scalar:
    Scalar:  384 multiply-accumulates per distance = 384 ops
    AVX2:    384 / 8 = 48 SIMD ops per distance
    AVX-512: 384 / 16 = 24 SIMD ops per distance
    
    For 5000 pre-filtered candidates:
    Scalar:  5000 × 384 ops = 1.92M ops
    AVX2:    5000 × 48 ops  = 240K ops (8× faster)
    AVX-512: 5000 × 24 ops  = 120K ops (16× faster)
    
  This is why usearch dramatically outperforms naive Rust implementations.
  Zero code changes needed — all happens inside usearch library.
  Just compile with target-cpu=native.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
OPT 4: SQL PRE-FILTER BEFORE HNSW
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Without pre-filter:
  HNSW searches ALL 50k hot memories for every query.
  Many are too weak to be useful. All are searched.
  
With pre-filter:
  SQL WHERE clause computes effective_strength first.
  Only memories above MIN_STRENGTH threshold → candidate list.
  HNSW searches ONLY the candidate list.
  
Typical distribution of effective strengths:
  > 0.5 (strong):     ~10% of memories = 5,000 at 50k store
  0.1-0.5 (moderate): ~30% of memories = 15,000
  0.05-0.1 (weak):    ~20% of memories = 10,000
  < 0.05 (archive):   ~40% of memories = 20,000 (never searched)
  
With MIN_STRENGTH = 0.1 pre-filter:
  HNSW search space: ~20,000 (40% of store)
  With LIMIT 5000: further reduced to 5,000
  
Reduction ratio: 50k → 5k = 10× reduction in search space
HNSW complexity savings: O(log 5000) vs O(log 50000) = small benefit
But: main benefit is HNSW ef reduction (fewer candidates to explore)

SQL INDEX for pre-filter:
  CREATE INDEX idx_memory_strength_filter
  ON memory_index(state, bypass_decay, base_strength DESC, stability, last_tick);
  
  This index allows SQLite to:
  1. Filter state != Archived quickly (index prefix)
  2. Sort by effective_strength in index order (approximately)
  3. LIMIT early-stop without scanning all rows
  
  Performance: 50k rows → <1ms for pre-filter with index.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
OPT 5: ADAPTIVE ef_search
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
HNSW ef_search is the single biggest lever for speed vs accuracy:
  ef=10:  fastest (~85% recall accuracy, 0.5ms at 50k)
  ef=50:  balanced (~95% accuracy, 2ms at 50k)
  ef=100: accurate (~99% accuracy, 4ms at 50k)
  ef=200: most accurate (~99.9%, 8ms at 50k)
  
Adaptive ef strategy:

  fn adaptive_ef(
      query: &RecallQuery,
      hot_count: usize,
      tier1_hit_rate: f32,
  ) -> usize {
      // Base ef from confidence requirement
      let base_ef = match query.confidence_requirement {
          ConfidenceLevel::FastApprox => 10,
          ConfidenceLevel::Normal     => 50,
          ConfidenceLevel::High       => 100,
      };
      
      // Size scaling: small stores don't need high ef
      // (few nodes → easy navigation regardless of ef)
      let size_factor = (hot_count as f32 / 50_000.0).sqrt().min(1.0);
      
      // If Tier1 hit rate is high, most queries are served from cache
      // → lower ef acceptable for Tier2 (it's a fallback for harder queries)
      let cache_factor = if tier1_hit_rate > 0.7 { 0.8 } else { 1.0 };
      
      let ef = (base_ef as f32 * size_factor * cache_factor) as usize;
      ef.max(10).min(200)  // clamp to reasonable range
  }
  
  In practice:
    Small store (5k memories): ef ≈ 10-20 (fast, graph well-connected)
    Medium store (20k): ef ≈ 25-50
    Large store (50k): ef ≈ 40-80
    
  Average reduction from adaptive vs fixed ef=50:
    ~2-3× faster for small/medium stores with no accuracy loss.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
OPT 6: VERTICAL TABLE PARTITION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Problem: Pre-filter SQL scan accesses memory_index.
  If memory_index row is fat (contains content, embeddings, etc.):
    SQLite must load many bytes per row just to check effective_strength.
    50k rows × 2KB per row = 100MB of I/O for a full scan. Terrible.
    
Solution: Split table into hot-path columns vs cold-path columns.

  memory_index table (scan-optimized):
    Row size: ~64 bytes
    Columns: id(16) + base_strength(4) + stability(4) + last_tick(8)
             + bypass_decay(1) + kind(1) + state(1) + engram_id(16)
             + access_count(4) + retrieval_difficulty(4) + emotional_arousal(4)
             + padding(1) = ~64 bytes
    
    At 50k memories: 50k × 64 = ~3.2MB
    SQLite page size 4096 bytes: 64 rows per page
    50k / 64 = 781 pages
    Full scan: 781 page reads = ~3.1MB I/O
    With OS cache: fits entirely in memory → microsecond scans
    
  memory_content table (fetch-on-demand):
    Row size: variable (content text, can be 100B to 10KB)
    Accessed ONLY for final top-K results (not during scan)
    
  memory_vectors table (rescore-on-demand):
    Row size: 384 × 4 = 1536 bytes (float32 blob)
    Accessed ONLY for float32 rescore of top-20 candidates
    
  Effect: pre-filter scan reads 3.2MB instead of potentially 50-100MB.
  Speedup for pre-filter: ~10-30× vs fat table design.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
OPT 7: SQLITE WAL + PRAGMA TUNING
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Default SQLite is optimized for safety over performance.
With WAL mode and tuning, we can 3-5× SQLite performance.

  PRAGMA journal_mode = WAL;
    → Write-Ahead Logging: readers never blocked by writers
    → Multiple concurrent readers (smart-grep, Claude Code, CLI)
    → Single writer doesn't block any readers
    
  PRAGMA synchronous = NORMAL;
    → fsync only on WAL checkpoint, not every write transaction
    → Risk: power failure during WAL write → lose last transaction
    → Acceptable: SQLite WAL is crash-safe (WAL checkpoints are atomic)
    → Speedup: 3× vs synchronous=FULL for write-heavy workloads
    
  PRAGMA cache_size = -131072;  (128MB page cache)
    → hot.db fits mostly in page cache → effectively in-memory
    → Negative value = KB; positive = pages
    → 128MB is generous — tunable based on available RAM
    
  PRAGMA mmap_size = 4294967296;  (4GB mmap)
    → Memory-map the database file
    → OS manages page faulting — frequently accessed pages stay hot
    → For hot.db: might not be needed (already fits in cache)
    → For cold.db: critical (TB-scale file, OS page cache needed)
    
  PRAGMA temp_store = MEMORY;
    → Temporary tables in RAM (not temp file)
    → Used for intermediate query results
    
  PRAGMA optimize;  (run at startup)
    → Updates query planner statistics
    → Ensures optimal query plan selection

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PROGRESSIVE QUANTIZATION LADDER (future optimization, post-M10)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Not implemented in initial milestones, but the architecture supports it.

Memory lifecycle → quantization ladder:
  New (hot):         float32 in SQLite (ground truth), float16 in HNSW
  Active (hot):      float16 in HNSW, float32 in SQLite
  Consolidated:      int8 in cold HNSW, float32 in cold.db (for rescore)
  Aging (cold):      int8 in HNSW, float32 still in cold.db
  Very old (archive): Binary (1-bit) in archive HNSW, float32 optionally discarded

  Product Quantization for archive tier:
    384 dims → 48 subspaces × 8 dims each
    Each subspace: 256 centroids (8-bit codebook)
    Storage: 48 bytes per vector (vs 1536 float32 = 32× compression)
    Accuracy: ~90-95% (acceptable for archive tier)
    
  When to implement: when cold store exceeds 10M memories.
  Until then: int8 (384 bytes/vector) is sufficient and much simpler.
```

### 6.3 Benchmark Targets

```
┌─────────────────────────────────────────────────────────────────────┐
│ OPERATION                  │ TARGET    │ METHOD                      │
│────────────────────────────│───────────│─────────────────────────── │
│ Recall (Tier 1 hit)        │ <0.1ms    │ LruCache get + cosine sim  │
│ Recall (Tier 2, 50k store) │ <5ms      │ HNSW + pre-filter + rescore│
│ Recall (Tier 3, 1M store)  │ <50ms     │ mmap HNSW int8 + rescore   │
│ Recall (Tier 3, 10M store) │ <100ms    │ mmap HNSW int8 + rescore   │
│ Encode (embed cache hit)   │ <1ms      │ LruCache get + SQL insert  │
│ Encode (embed cache miss)  │ <10ms     │ fastembed + SQL + HNSW add │
│ Consolidation cycle        │ 0ms       │ async background, non-block│
│ Decay computation (idle)   │ 0ms       │ lazy — no idle computation │
│ Pre-filter (50k memories)  │ <0.5ms    │ SQL index scan             │
│ Engram BFS (depth=3)       │ <1ms      │ petgraph BFS               │
│ Working memory add         │ <0.01ms   │ VecDeque + HashMap         │
└─────────────────────────────────────────────────────────────────────┘

SCALE TARGETS:
┌────────────────────────────────────────────────────────────────────┐
│ SCALE               │ HOT RSS   │ COLD DISK │ RECALL P99           │
│─────────────────────│───────────│───────────│───────────────────── │
│ 10k hot memories    │ ~15MB     │ N/A       │ <1ms (all Tier2)     │
│ 50k hot memories    │ ~75MB     │ N/A       │ <5ms (Tier2)         │
│ 50k hot + 500k cold │ ~75MB     │ ~200MB    │ <10ms (Tier2+3 mix)  │
│ 50k hot + 5M cold   │ ~75MB     │ ~2GB      │ <50ms (Tier3)        │
│ 50k hot + 50M cold  │ ~75MB     │ ~20GB     │ <100ms (Tier3)       │
└────────────────────────────────────────────────────────────────────┘

RAM BUDGET BREAKDOWN (at 50k hot memories):
  usearch HNSW hot_index (float16):  50k × 768 bytes = ~38MB
  usearch HNSW overhead (graph):     50k × M=16 × 8 bytes = ~6MB
  SQLite hot.db page cache:          ~128MB (configured)
  Tier1 LruCache:                    512 × ~2KB = ~1MB
  Embedding cache:                   1000 × 384 × 4 = ~1.5MB
  Engram graph (petgraph):           ~10k engrams × ~200B = ~2MB
  Working memory:                    7 × ~2KB = ~14KB
  Total active RSS:                  ~180MB
  
  Well within typical agent RAM budget of 4-16GB.
```

### 6.4 Memory Resonance — Collective Strength Algorithm

```
BIOLOGICAL BASIS:
  CA3 autoassociative network: recalling one engram member
  → partial activation of all connected engram cells
  → small LTP boost to all connected cells
  Result: densely connected engrams (expert knowledge) are more stable
          than isolated facts.

ALGORITHM:

  When memory M is recalled:
  1. Compute standard LTP: M.base_strength += LTP_DELTA (0.1)
  2. Get M's engram and neighboring nodes (BFS depth=1)
  3. For each neighbor N:
     resonance_ltp = LTP_DELTA × RESONANCE_FACTOR / neighbor_count
     N.base_strength += resonance_ltp (async, non-blocking)
  
  RESONANCE_FACTOR = 0.3:
    If M has 10 neighbors: each gets 0.1 × 0.3 / 10 = 0.003 LTP boost
    Small individually, but cumulative:
    A 200-member engram where each member is recalled once per 100 ticks:
      = 200 recalls × 0.003 resonance per recall
      = 200 × 200 × 0.003 = 120 units of total resonance LTP distributed
      → Significantly stronger than isolated memories with same recall count

EMERGENT BEHAVIOR:
  Expert knowledge (large, dense engram) → very stable
  Isolated facts (no engram) → decay normally
  
  Agent using membrain for 10,000 ticks:
    Core concepts in agent's domain: recalled frequently → large engrams → very stable
    Peripheral facts: rarely recalled → no engrams → decay → forgotten
    = Exactly the right behavior for a domain expert agent

IMPLEMENTATION:

  fn apply_resonance(
      memory_id: Uuid,
      engram_graph: &EngramGraph,
      hot: &mut HotStore,
      now_tick: u64,
  ) {
      let neighbors: Vec<Uuid> = engram_graph
          .graph
          .neighbors(engram_graph.node_index[&memory_id])
          .map(|ni| engram_graph.graph[ni])
          .collect();
      
      if neighbors.is_empty() {
          return;  // isolated memory, no resonance
      }
      
      let resonance_delta = LTP_DELTA * RESONANCE_FACTOR / neighbors.len() as f32;
      
      // Batch update to avoid N individual SQL writes:
      hot.db.batch_apply_ltp_delta(&neighbors, resonance_delta, now_tick)?;
      
      // SQL: UPDATE memory_index 
      //      SET base_strength = MIN(base_strength + ?, 1.0)
      //      WHERE id IN (?, ?, ...)
  }
```

### 6.5 Prospective Memory — Future-Triggered Recall

```
BIOLOGICAL BASIS:
  "Prospective memory": remembering to do something in the future.
  "When I get to the grocery store, I need to buy milk."
  Encoded as a context-trigger + action pair.
  When context matches → memory fires.

ALGORITHM:

  Data structure:
  pub struct ProspectiveTrigger {
      id: Uuid,
      trigger_embedding: Vec<f32>,  // context to match against
      memory_id: Uuid,              // memory to surface when triggered
      trigger_threshold: f32,       // 0.8 by default
      fire_count: u32,              // how many times it has fired
      max_fires: Option<u32>,       // None = unlimited
      created_tick: u64,
      expires_tick: Option<u64>,
  }
  
  Stored in: hot.db/prospective_triggers table
  
  Trigger check: called on every ENCODE (not just recall):
  
  fn check_prospective_triggers(
      current_context: &[f32],
      triggers: &[ProspectiveTrigger],
      now_tick: u64,
  ) -> Vec<Uuid> {
      triggers.iter()
          .filter(|t| {
              // Not expired
              t.expires_tick.map_or(true, |e| now_tick < e)
              // Not exhausted
              && t.max_fires.map_or(true, |m| t.fire_count < m)
              // Context matches
              && cosine_sim(current_context, &t.trigger_embedding) > t.trigger_threshold
          })
          .map(|t| t.memory_id)
          .collect()
  }
  
  Usage:
  // Agent sets up reminder:
  membrain remind --when "context matches 'deploy to production'" \
                  --memory-id abc123 \
                  --max-fires 3
  
  // Or: create and trigger simultaneously
  membrain remind --when "context matches 'payments module'" \
                  --then "Stripe rate limit: 100 req/sec. Always check x-rate-limit headers."
  
  Effect: automatically surfaces critical procedural knowledge
          when relevant context is detected — without requiring
          explicit recall from the agent.
```

### 6.6 Spotlight / Priming Mode

```
BIOLOGICAL BASIS:
  "Mental preparation": before a difficult task, you mentally review
  relevant knowledge. This pre-activates relevant engrams → faster recall.
  Athletes, surgeons, musicians all use this technique.

ALGORITHM:

  pub struct PrimedContext {
      embedding: Vec<f32>,         // what was primed
      boost_factor: f32,           // 0.0 to 0.5
      created_tick: u64,
      expiry_tick: u64,
      source_description: String,  // "working on auth module"
  }
  
  When priming:
  1. membrain prime --context "debugging JWT authentication"
  2. Compute embedding: primed_vec = embed("debugging JWT authentication")
  3. Query Tier2 HNSW with primed_vec, top=100 (broad, low threshold)
  4. For top-100 results: pre-load into Tier1 LruCache
  5. Store PrimedContext for score boosting during retrieval
  
  Score boost during retrieval:
  fn priming_boost(
      candidate: &Memory,
      primed_contexts: &[PrimedContext],
      context_vec: &[f32],
      now_tick: u64,
  ) -> f32 {
      primed_contexts.iter()
          .filter(|p| now_tick < p.expiry_tick)
          .map(|p| {
              let ctx_match = cosine_sim(context_vec, &p.embedding);
              p.boost_factor * ctx_match
          })
          .fold(0.0_f32, f32::max)
  }
  
  Effect:
    Pre-warms Tier1 cache: subsequent recalls for primed content → <0.1ms
    Score boost: primed memories rank higher in results
    Duration: configurable expiry (default 1000 interactions)
    
  CLI usage:
    membrain prime "fixing the database migration issue"
    membrain recall "database schema" --context "migration"
    → returns migration-related memories at higher priority
    → if any were Tier3 cold, now pre-loaded into Tier1 → 500× faster
```

---

### End of Snapshot Part 3

**Next: Part 4 — Techstack Analysis & Data Schema**

Parts list:
- Part 1: Vision, Problem Statement, Human Brain Deep Dive ✅
- Part 2: Gap Analysis + Full Port (mechanism → Rust code) ✅
- Part 3: Architecture Overview + Performance ✅
- Part 4: Techstack + Data Schema
- Part 5: CLI/MCP + Feature Extensions + Workspace Structure
- Part 6: Milestones + Acceptance Checklist + Constants + Algorithm Reference


<!-- SOURCE: PLAN_part4.md -->

### Source Snapshot — Part 4
#### Part 4 of 6: Techstack Analysis · Data Schema

---

## 7. Techstack — Analysis & Rationale

### 7.1 Core Language: Rust

```
WHY RUST SPECIFICALLY (not Python, not Go, not C++):

  vs PYTHON:
    Python has the best AI/ML ecosystem (PyTorch, transformers, LangChain).
    But: Python has GIL — background consolidation + foreground recall would
    fight each other. asyncio has no true parallelism for CPU-bound work.
    GC pauses: Python's GC can pause 10-100ms unpredictably.
    membrain requires: <0.1ms Tier1 latency. Python cannot guarantee this.

  vs GO:
    Go has excellent concurrency (goroutines, channels).
    But: Go's GC pauses (1-10ms) are unacceptable for <0.1ms Tier1.
    No SIMD in standard library — would require CGo.
    No equivalent of usearch/fastembed-rs/petgraph native bindings.

  vs C++:
    C++ is fastest, has all libraries.
    But: memory safety critical for a memory store.
         (ironic: memory bugs in a memory store)
    Build system: Rust cargo >> CMake complexity.
    Interop: all key crates (usearch, fastembed-rs, petgraph, rusqlite)
             are native Rust — zero FFI overhead.

  RUST ADVANTAGES FOR membrain:

  1. ZERO GC PAUSES
     No garbage collector → no pause times.
     Memory: compile-time borrow checker manages lifetimes.
     Result: <0.1ms Tier1 latency is consistent, not just average.
     P99 latency ≈ P50 latency — no GC spikes.

  2. COMPILE-TIME SIMD SPECIALIZATION
     RUSTFLAGS="-C target-cpu=native" → AVX2/AVX-512 auto-enabled.
     usearch + fastembed-rs use these automatically.
     Zero code changes — just a compile flag.

  3. FEARLESS CONCURRENCY
     tokio async runtime: non-blocking Unix socket server.
     tokio::spawn: background tasks (consolidation, forgetting engine).
     Arc<RwLock<EngramGraph>>: shared graph without data races.
     Compiler enforces: no data races — won't compile if unsafe.

  4. SINGLE BINARY DEPLOYMENT
     cargo build --release → one binary, ~20-40MB.
     Zero runtime dependencies (fastembed model downloaded on first use).
     Works everywhere: Linux, macOS, WSL.

  5. ECOSYSTEM FIT
     usearch:     Rust-first vector index library (Unum Cloud)
     fastembed-rs: Rust port of fastembed, ONNX inference
     petgraph:    Most mature Rust graph library
     rusqlite:    SQLite bindings, feature-rich, well-maintained
     rmcp:        Official Rust MCP SDK (stdio transport)
     lru:         LruCache, zero-cost, well-tested
     tokio:       Industry-standard async runtime
     clap:        Best-in-class CLI argument parsing
     serde/serde_json: Standard serialization
     thiserror:   Idiomatic error types
     anyhow:      Error propagation in application code
     
  6. STACK CONSISTENCY
     Same stack as linehash (hash-anchored editing) and smart-grep
     (semantic code search). Shared embed cache warm-up, shared
     usearch knowledge, shared CI/CD pipeline.
     Developer context switching cost: zero.
```

### 7.2 Async Runtime: Tokio

```
WHY TOKIO:

  membrain daemon needs to do multiple things simultaneously:
    - Serve Unix socket connections (JSON-RPC 2.0)
    - Run consolidation cycle in background
    - Run reconsolidation tick periodically
    - Run forgetting engine periodically
    - Respond to MCP tool calls (stdio)

  tokio provides:
    tokio::net::UnixListener   → async Unix socket server
    tokio::spawn               → spawn background tasks
    tokio::sync::RwLock        → async read-write lock for engram graph
    tokio::sync::mpsc          → channels for consolidation pressure signals
    tokio::time::interval      → periodic background tasks

  TASK ARCHITECTURE:

  #[tokio::main]
  async fn main() {
      let brain = Arc::new(BrainStore::open(config).await?);

      // Background task: consolidation (woken by pressure channel)
      let brain_c = brain.clone();
      tokio::spawn(async move {
          let mut interval = tokio::time::interval(Duration::from_secs(60));
          loop {
              interval.tick().await;
              if brain_c.needs_consolidation().await {
                  brain_c.consolidation_cycle().await.ok();
              }
          }
      });

      // Background task: reconsolidation tick (per-interaction wake)
      let brain_r = brain.clone();
      tokio::spawn(async move {
          let mut rx = brain_r.interaction_rx();
          while let Some(tick) = rx.recv().await {
              brain_r.reconsolidation_tick(tick).await.ok();
          }
      });

      // Background task: forgetting engine (low priority, periodic)
      let brain_f = brain.clone();
      tokio::spawn(async move {
          let mut interval = tokio::time::interval(Duration::from_secs(300));
          loop {
              interval.tick().await;
              brain_f.forgetting_engine_pass().await.ok();
          }
      });

      // Foreground: Unix socket server (concurrent JSON-RPC)
      let listener = UnixListener::bind(&config.socket_path)?;
      loop {
          let (stream, _) = listener.accept().await?;
          let brain_s = brain.clone();
          tokio::spawn(async move {
              handle_jsonrpc_connection(stream, brain_s).await.ok();
          });
      }
  }

  KEY DESIGN: brain: Arc<BrainStore> shared across ALL tasks.
  SQLite WAL: multiple tasks reading/writing simultaneously — safe.
  RwLock on engram graph: readers don't block each other.
  Channels: foreground signals background (pressure notifications).
```

### 7.3 Vector Index: usearch

```
WHY usearch OVER ALTERNATIVES:

  sqlite-vec (naive choice):
    ✅ Embedded in SQLite — simple
    ❌ Brute-force KNN — O(n×d) — unusable at 100k+
    ❌ No HNSW
    ❌ No quantization
    ❌ No mmap
    ❌ Benchmarks: 10k=10ms, 100k=100ms, 1M=1000ms → DEAD

  Qdrant (popular vector DB):
    ✅ Fast HNSW, good ecosystem
    ❌ Requires separate server (Docker/binary)
    ❌ HTTP overhead for every call
    ❌ Not embeddable in process
    ❌ Complex deployment for a CLI tool

  Faiss (Facebook):
    ✅ Extremely fast, battle-tested
    ❌ C++ library — requires Python or CGo bridge
    ❌ No native Rust bindings (unofficial only)
    ❌ Not mmap-native

  Annoy (Spotify):
    ✅ Simple, mmap-based
    ❌ Tree-based, not graph-based (lower accuracy)
    ❌ Build time O(n log n) — slow for large stores
    ❌ No quantization

  usearch:
    ✅ HNSW — O(log n) ANN search
    ✅ int8 / float16 / float32 / binary quantization native
    ✅ SIMD: AVX2, AVX-512, NEON auto-detection
    ✅ mmap: disk-backed unlimited scale
    ✅ Embeddable: library, not server
    ✅ Native Rust API
    ✅ Already used in smart-grep (proven in this ecosystem)
    ✅ MIT license
    ✅ Active development (Unum Cloud)
    ✅ Multi-threaded search

usearch CONFIGURATION:

  Hot index (Tier 2):
    index = Index::new(&IndexOptions {
        dimensions: 384,
        metric: MetricKind::Cos,       // cosine similarity
        quantization: ScalarKind::F16, // float16
        connectivity: 16,              // M parameter (edges per node)
        expansion_add: 200,            // ef_construction (build quality)
        expansion_search: 50,          // ef_search (query quality, adaptive)
        multi: false,
    })?;
    // In-memory: ~75MB at 50k vectors
    // Rebuilt from hot.db on daemon start

  Cold index (Tier 3):
    cold_index = Index::new(&IndexOptions {
        dimensions: 384,
        metric: MetricKind::Cos,
        quantization: ScalarKind::I8,  // int8 — 4× smaller than f32
        connectivity: 8,               // fewer connections — saves RAM for mmap
        expansion_add: 200,
        expansion_search: 50,
        multi: false,
    })?;
    cold_index.save("~/.membrain/cold.usearch")?;  // persist to disk
    // On startup: cold_index.load("~/.membrain/cold.usearch")?;
    // mmap: OS manages page faulting, unlimited scale

usearch API (key operations):

  // Add
  index.add(external_id: u64, vector: &[ScalarType])?;

  // Search (returns Iterator<(external_id, distance)>)
  let results = index.search(query_vector, top_k)?;

  // Remove (marks deleted, does not compact)
  index.remove(external_id)?;

  // Capacity management
  if index.size() > SOFT_LIMIT {
      index.reserve(NEW_CAPACITY)?;
  }

  // Persistence
  index.save(path)?;
  index.load(path)?;  // or load_from_memory for mmap

EXTERNAL ID STRATEGY:
  usearch uses u64 as external ID.
  membrain uses Uuid (128-bit).
  Mapping: store Uuid → u64 in SQLite table (uuid_to_usearch).
  Use: uuid.as_u64_pair().0 as primary key (first 64 bits).
  Collision probability: negligible for <10M memories.
```

### 7.4 Embedding: fastembed-rs

```
WHY fastembed-rs:

  Alternatives considered:
    OpenAI text-embedding-3-small:
      ✅ High quality
      ❌ API call: ~100ms latency + $0.02/1M tokens cost
      ❌ Requires internet + API key
      ❌ Privacy: content sent to external server
      ❌ Rate limits

    sentence-transformers (Python):
      ✅ Excellent quality, many models
      ❌ Python subprocess: IPC overhead
      ❌ Memory: ~200MB Python + model overhead
      ❌ GIL for inference

    candle (Hugging Face Rust):
      ✅ Pure Rust, no Python
      ❌ More complex: requires writing inference code
      ❌ Less mature embedding support

    fastembed-rs:
      ✅ Local, offline — zero API calls
      ✅ ONNX inference — optimized, cross-platform
      ✅ Multiple model support
      ✅ Batch mode: embed_batch() → 3-5× throughput
      ✅ Already used in smart-grep (proven)
      ✅ ~5ms per single embed, <1ms/item in batch
      ✅ 80MB model download (once, cached)
      ✅ MIT license

WHY NOT AN LLM:
  CRITICAL CLARIFICATION — all-MiniLM-L6-v2 is NOT an LLM.

  all-MiniLM-L6-v2:
    Type:   Embedding model (encoder-only transformer)
    Size:   80MB (ONNX weights)
    Output: 384-dimensional float vector
    Speed:  ~5ms per text
    Use:    Convert text → semantic vector (for similarity search)
    Cost:   Zero (runs locally, no API)
    Privacy: Zero data leaves machine

  GPT-4 / Claude:
    Type:   Large Language Model (decoder transformer)
    Size:   Hundreds of GB
    Output: Text tokens
    Speed:  500ms-5s per response
    Use:    Generate text, reason, answer questions
    Cost:   $$$

  membrain uses embedding model ONLY in the memory pipeline.
  Zero LLM calls in encode/recall path.
  This is the key performance advantage over Mem0/LangMem.

MODEL CHOICES:

  DEFAULT: all-MiniLM-L6-v2
    Dimensions: 384
    Model size: 80MB
    Speed: ~5ms single, ~1ms/item batch
    Quality: Good for semantic similarity tasks
    Use: Default for all embeddings
    Config: model = "all-MiniLM-L6-v2"

  HIGH QUALITY: nomic-embed-text-v1.5
    Dimensions: 768
    Model size: 274MB
    Speed: ~15ms single, ~3ms/item batch
    Quality: Significantly better semantic understanding
    Use: When quality matters more than speed
    Config: model = "nomic-embed-text-v1.5"
    Note: requires updating dimensions = 768 in config

  MULTILINGUAL: paraphrase-multilingual-MiniLM-L12-v2
    Dimensions: 384
    Model size: 420MB
    Speed: ~8ms single
    Quality: Good for non-English content
    Use: If agent works in multiple languages
    Config: model = "paraphrase-multilingual-MiniLM-L12-v2"

fastembed-rs API:

  use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};

  // Initialize (once at startup)
  let model = TextEmbedding::try_new(
      InitOptions::new(EmbeddingModel::AllMiniLML6V2)
          .with_show_download_progress(true)
  )?;

  // Single embed (~5ms)
  let embeddings = model.embed(vec!["content here"], None)?;
  let vec: Vec<f32> = embeddings[0].clone();

  // Batch embed (3-5× faster than individual)
  let texts = vec!["content 1", "content 2", ..., "content N"];
  let batch_embeddings = model.embed(texts, Some(256))?; // batch_size=256
  // Returns Vec<Vec<f32>>

EMBEDDING CACHE IMPLEMENTATION:

  pub struct EmbedCache {
      cache: LruCache<u64, Arc<Vec<f32>>>,  // Arc for cheap cloning
      model: TextEmbedding,
      hits: AtomicU64,
      misses: AtomicU64,
  }

  impl EmbedCache {
      pub async fn get_or_embed(&mut self, text: &str) -> Result<Arc<Vec<f32>>> {
          let key = xxhash64(text.as_bytes());

          if let Some(cached) = self.cache.get(&key) {
              self.hits.fetch_add(1, Ordering::Relaxed);
              return Ok(Arc::clone(cached));
          }

          self.misses.fetch_add(1, Ordering::Relaxed);
          let embeddings = self.model.embed(vec![text], None)?;
          let vec = Arc::new(embeddings.into_iter().next().unwrap());
          self.cache.put(key, Arc::clone(&vec));
          Ok(vec)
      }

      pub fn hit_rate(&self) -> f32 {
          let hits = self.hits.load(Ordering::Relaxed) as f32;
          let total = hits + self.misses.load(Ordering::Relaxed) as f32;
          if total == 0.0 { 0.0 } else { hits / total }
      }
  }
```

### 7.5 Graph: petgraph

```
WHY petgraph:

  The engram graph is a directed weighted graph:
    - Nodes: Uuid (memory IDs)
    - Edges: EdgeWeight (similarity, type, activation count)
    - Operations: add_node, add_edge, neighbors, BFS, DFS

  Alternatives:
    Custom adjacency list: would need to reimplement BFS, serialization, etc.
    Neo4j: separate server, overkill, not embeddable.
    DGraph: same issues.

  petgraph:
    ✅ Most mature Rust graph library
    ✅ DiGraph, UnGraph, StableGraph variants
    ✅ BFS, DFS built-in (petgraph::visit::Bfs, Dfs)
    ✅ Serde support (feature = "serde-1")
    ✅ Used in scope tool (this ecosystem)
    ✅ Dijkstra, bellman-ford, tarjan for future use
    ✅ MIT/Apache license

GRAPH MODEL:

  // DiGraph: directed (A→B does not imply B→A)
  // But: we add edges in both directions for undirected behavior
  type MemoryGraph = DiGraph<Uuid, EdgeWeight>;

  pub struct EngramGraph {
      graph: MemoryGraph,
      node_index: HashMap<Uuid, NodeIndex>,    // Uuid → NodeIndex fast lookup
      reverse_index: HashMap<NodeIndex, Uuid>, // NodeIndex → Uuid
  }

  impl EngramGraph {
      pub fn add_memory(&mut self, id: Uuid) -> NodeIndex {
          let ni = self.graph.add_node(id);
          self.node_index.insert(id, ni);
          self.reverse_index.insert(ni, id);
          ni
      }

      pub fn link(&mut self, from: Uuid, to: Uuid, weight: EdgeWeight) {
          let from_ni = self.node_index[&from];
          let to_ni = self.node_index[&to];
          // Bidirectional
          self.graph.add_edge(from_ni, to_ni, weight.clone());
          self.graph.add_edge(to_ni, from_ni, weight);
      }

      pub fn bfs_neighbors(
          &self,
          seed: Uuid,
          max_depth: usize,
          max_nodes: usize,
          min_weight: f32,
      ) -> Vec<Uuid> {
          use petgraph::visit::{Bfs, Walker};

          let seed_ni = match self.node_index.get(&seed) {
              Some(ni) => *ni,
              None => return vec![],
          };

          // Priority-aware BFS using BinaryHeap
          let mut heap: BinaryHeap<(OrderedFloat<f32>, usize, NodeIndex)> = BinaryHeap::new();
          let mut visited = HashSet::new();
          let mut result = Vec::new();

          heap.push((OrderedFloat(1.0f32), 0, seed_ni));
          visited.insert(seed_ni);

          while let Some((weight, depth, node)) = heap.pop() {
              result.push(self.reverse_index[&node]);
              if result.len() >= max_nodes { break; }
              if depth >= max_depth { continue; }

              for neighbor in self.graph.neighbors(node) {
                  if visited.contains(&neighbor) { continue; }
                  let edge = self.graph.find_edge(node, neighbor).unwrap();
                  let ew = &self.graph[edge];
                  if ew.similarity >= min_weight {
                      visited.insert(neighbor);
                      heap.push((OrderedFloat(ew.similarity), depth + 1, neighbor));
                  }
              }
          }

          result
      }
  }

PERSISTENCE:
  petgraph + serde → serialize entire graph to JSON/bincode.
  Stored in hot.db as a BLOB (small overhead, easy backup).

  // Serialize
  let graph_bytes = bincode::serialize(&engram_graph)?;
  db.execute("UPDATE brain_state SET engram_graph = ?", [&graph_bytes])?;

  // Deserialize
  let bytes: Vec<u8> = db.query_row(
      "SELECT engram_graph FROM brain_state", [], |r| r.get(0)
  )?;
  let engram_graph: EngramGraph = bincode::deserialize(&bytes)?;

  At 10k engrams × ~200 bytes each: ~2MB — tiny.
```

### 7.6 IPC: Unix Socket + JSON-RPC 2.0

```
WHY UNIX SOCKET:

  Alternatives:
    HTTP (TCP):
      ✅ Universal, well-understood
      ❌ TCP overhead: SYN/ACK roundtrip even on localhost
      ❌ HTTP framing overhead: headers, encoding
      ❌ ~1ms minimum latency even local
      ❌ Requires port management

    gRPC:
      ✅ Efficient binary protocol, streaming
      ❌ Complex: proto file, codegen, runtime
      ❌ Overkill for local IPC

    Unix Domain Sockets:
      ✅ Zero-copy kernel-level IPC
      ✅ Fastest possible local communication (~0.01ms)
      ✅ File-permission based access control
      ✅ No port conflicts
      ✅ Standard on Linux/macOS/WSL
      ✅ tokio::net::UnixListener native support

  membrain uses: Unix Domain Socket + JSON-RPC 2.0

WHY JSON-RPC 2.0:

  Alternatives:
    Custom binary protocol: fastest but no libraries, hard to debug.
    MessagePack-RPC: binary, compact, but less tooling.
    JSON-RPC 2.0:
      ✅ Standard spec (https://www.jsonrpc.org/specification)
      ✅ Libraries in every language (Python, Node, Rust)
      ✅ Human-readable: easy to debug with cat/nc
      ✅ Request/response + notification support
      ✅ Error codes standardized
      ✅ Batch requests supported

JSON-RPC 2.0 WIRE FORMAT:

  Request:
  {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "remember",
    "params": {
      "content": "JWT tokens expire after 1 hour in this codebase",
      "context": "debugging auth module",
      "attention_score": 0.8,
      "emotional_valence": -0.3,
      "emotional_arousal": 0.5,
      "kind": "Semantic",
      "source": "mcp"
    }
  }

  Response:
  {
    "jsonrpc": "2.0",
    "id": 1,
    "result": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "initial_strength": 0.72,
      "novelty_score": 0.85,
      "engram_id": "7b1c3e42-...",
      "tick": 1042
    }
  }

  Error:
  {
    "jsonrpc": "2.0",
    "id": 1,
    "error": {
      "code": -32603,
      "message": "Internal error",
      "data": "embedding model not loaded"
    }
  }

PYTHON CLIENT (complete, 1 file):

  # membrain_client.py
  import socket, json, os

  SOCKET_PATH = os.path.expanduser("~/.membrain/membrain.sock")

  class MembrainClient:
      def __init__(self, socket_path=SOCKET_PATH):
          self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
          self.sock.connect(socket_path)
          self._id = 0

      def _call(self, method, **params):
          self._id += 1
          req = json.dumps({
              "jsonrpc": "2.0",
              "id": self._id,
              "method": method,
              "params": params
          }) + "\n"
          self.sock.sendall(req.encode())
          resp = b""
          while not resp.endswith(b"\n"):
              resp += self.sock.recv(4096)
          result = json.loads(resp)
          if "error" in result:
              raise RuntimeError(result["error"]["message"])
          return result["result"]

      def remember(self, content, context=None, attention=0.7,
                   valence=0.0, arousal=0.0, kind="Episodic"):
          return self._call("remember", content=content, context=context,
                            attention_score=attention,
                            emotional_valence=valence,
                            emotional_arousal=arousal, kind=kind)

      def recall(self, query, context=None, top_k=5):
          return self._call("recall", content=query,
                            context=context, top_k=top_k)

      def forget(self, memory_id):
          return self._call("forget", id=memory_id)

      def strengthen(self, memory_id):
          return self._call("strengthen", id=memory_id)

      def stats(self):
          return self._call("stats")

      def close(self):
          self.sock.close()

  # Usage:
  # brain = MembrainClient()
  # brain.remember("auth token expires in 1h", context="debugging", valence=-0.2)
  # results = brain.recall("JWT token")

NODE CLIENT (complete, 1 file):

  // membrain_client.js
  const net = require('net');
  const readline = require('readline');

  class MembrainClient {
    constructor(socketPath = `${process.env.HOME}/.membrain/membrain.sock`) {
      this.sock = net.createConnection(socketPath);
      this.pending = new Map();
      this._id = 0;
      const rl = readline.createInterface({ input: this.sock });
      rl.on('line', line => {
        const resp = JSON.parse(line);
        const cb = this.pending.get(resp.id);
        if (cb) {
          this.pending.delete(resp.id);
          if (resp.error) cb[1](new Error(resp.error.message));
          else cb[0](resp.result);
        }
      });
    }

    _call(method, params) {
      return new Promise((resolve, reject) => {
        const id = ++this._id;
        this.pending.set(id, [resolve, reject]);
        const req = JSON.stringify({ jsonrpc: '2.0', id, method, params }) + '\n';
        this.sock.write(req);
      });
    }

    remember(content, context, attention = 0.7, valence = 0.0, arousal = 0.0) {
      return this._call('remember', { content, context, attention_score: attention,
        emotional_valence: valence, emotional_arousal: arousal });
    }

    recall(query, context, top_k = 5) {
      return this._call('recall', { content: query, context, top_k });
    }

    stats() { return this._call('stats', {}); }

    close() { this.sock.destroy(); }
  }

  module.exports = { MembrainClient };
```

### 7.7 MCP: rmcp

```
WHY MCP:
  Model Context Protocol = standard interface for LLM tool use.
  Claude Code, Cursor, and other AI coding tools support MCP.
  membrain as MCP server → every Claude Code session has memory.

WHY rmcp:
  Official Rust MCP SDK from Anthropic.
  stdio transport: Claude Code spawns membrain process, talks via stdin/stdout.
  Zero network overhead: IPC via stdio pipes.

MCP TOOLS DEFINITION:

  Tool: remember
    Description: Store a new memory in the brain
    Input:
      content:            string (required) — what to remember
      context:            string (optional) — current context/task
      attention_score:    float (optional, 0.0-1.0, default 0.7)
      emotional_valence:  float (optional, -1.0 to 1.0, default 0.0)
      emotional_arousal:  float (optional, 0.0-1.0, default 0.0)
      kind:               string (optional: Episodic|Semantic|Procedural, default Episodic)
    Output:
      id:               string (UUID of new memory)
      initial_strength: float
      novelty_score:    float
      engram_id:        string | null

  Tool: recall
    Description: Retrieve memories relevant to a query
    Input:
      query:    string (required) — what to search for
      context:  string (optional) — current context for better matching
      top_k:    integer (optional, default 5)
      kind:     string (optional) — filter by memory kind
    Output:
      memories: array of:
        id:               string
        content:          string
        score:            float
        strength:         float
        kind:             string
        access_count:     integer
        decaying_soon:    boolean
        engram_id:        string | null

  Tool: forget
    Description: Archive (soft-delete) a specific memory
    Input:
      id: string (UUID)
    Output:
      archived: boolean

  Tool: strengthen
    Description: Manually apply LTP to a specific memory (simulate recall)
    Input:
      id: string (UUID)
    Output:
      new_strength: float
      new_stability: float

  Tool: stats
    Description: Get brain health statistics
    Input: none
    Output:
      hot_count:            integer
      cold_count:           integer
      total_count:          integer
      avg_strength:         float
      tier1_hit_rate:       float
      embed_cache_hit_rate: float
      n_engrams:            integer
      interaction_tick:     integer
      last_consolidation:   integer | null
      decaying_count:       integer

  Tool: consolidate
    Description: Manually trigger consolidation cycle (NREM+REM+Homeostasis)
    Input: none
    Output:
      migrated:    integer
      archived:    integer
      duration_ms: integer

  Tool: prime
    Description: Prime working memory with context (spotlight mode)
    Input:
      context: string (required) — task description to prime for
      duration: integer (optional, default 1000 interactions)
    Output:
      primed_count:  integer (memories pre-loaded into Tier1)
      expires_tick:  integer

  Tool: remind
    Description: Set a prospective memory trigger
    Input:
      when:      string (required) — context description to trigger on
      content:   string (required) — memory content to surface when triggered
      max_fires: integer (optional) — how many times to fire
    Output:
      trigger_id: string

MCP SERVER IMPLEMENTATION (rmcp):

  use rmcp::{ServerHandler, tool, McpServer};

  struct MembrainMcpHandler {
      brain: Arc<BrainStore>,
  }

  #[rmcp::tool_handler]
  impl MembrainMcpHandler {
      #[tool(description = "Store a new memory in the brain")]
      async fn remember(
          &self,
          content: String,
          context: Option<String>,
          attention_score: Option<f32>,
          emotional_valence: Option<f32>,
          emotional_arousal: Option<f32>,
          kind: Option<String>,
      ) -> Result<RememberResult> {
          let now_tick = self.brain.tick();
          let result = self.brain.encode(EncodeRequest {
              content,
              context: context.unwrap_or_default(),
              attention_score: attention_score.unwrap_or(0.7),
              emotional_tag: EmotionalTag {
                  valence: emotional_valence.unwrap_or(0.0),
                  arousal: emotional_arousal.unwrap_or(0.0),
              },
              kind: kind.and_then(|k| k.parse().ok()).unwrap_or(MemoryKind::Episodic),
              source: MemorySource::Mcp,
          }, now_tick).await?;
          Ok(result.into())
      }

      #[tool(description = "Retrieve memories relevant to a query")]
      async fn recall(
          &self,
          query: String,
          context: Option<String>,
          top_k: Option<usize>,
      ) -> Result<RecallMcpResult> {
          let now_tick = self.brain.tick();
          let result = self.brain.recall(RecallQuery {
              content: query,
              context,
              top_k: top_k.unwrap_or(5),
              confidence_requirement: ConfidenceLevel::Normal,
              min_strength: MIN_STRENGTH,
              include_decaying: true,
          }, now_tick).await?;
          Ok(result.into())
      }
  }

  // In main: spawn MCP stdio server
  pub async fn run_mcp(config: Config) -> Result<()> {
      let brain = Arc::new(BrainStore::open(&config).await?);
      let handler = MembrainMcpHandler { brain };
      McpServer::new()
          .with_name("membrain")
          .with_version(env!("CARGO_PKG_VERSION"))
          .serve_stdio(handler)
          .await
  }
```

### 7.8 Complete Dependency List

```toml
[workspace]
resolver = "2"
members = [
    "crates/membrain-core",
    "crates/membrain-cli",
]

# ──────────────────────────────────────────────────────────────────
# membrain-core: all brain logic, no CLI concerns
# ──────────────────────────────────────────────────────────────────
[package]
name = "membrain-core"
version = "0.1.0"
edition = "2021"

[dependencies]
# Storage — SQLite with WAL
rusqlite = { version = "0.31", features = ["bundled"] }
# bundled: statically links SQLite — no system SQLite dependency

# Vector index — HNSW, int8/float16, SIMD, mmap
usearch = "2"
# Compile with RUSTFLAGS="-C target-cpu=native" for AVX2/AVX-512

# Embedding — local ONNX inference
fastembed = "3"

# Graph — engram network
petgraph = { version = "0.6", features = ["serde-1"] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# Caching
lru = "0.12"

# Hashing (embedding cache key)
xxhash-rust = { version = "0.8", features = ["xxh64"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
bincode = "1"  # for petgraph serialization

# UUIDs
uuid = { version = "1", features = ["v4", "serde"] }

# Compression (content in cold tier)
zstd = "0.13"

# Error handling
thiserror = "1"
anyhow = "1"

# Float comparison utilities (for priority queues)
ordered-float = "4"

# Atomic reference counting (shared brain state)
# (std::sync::Arc — no extra crate needed)

# float16 support
half = { version = "2", features = ["bytemuck"] }

# Safe casting for vector byte operations
bytemuck = { version = "1", features = ["derive"] }

[dev-dependencies]
tempfile = "3"       # temporary DB files for tests
criterion = "0.5"   # benchmarks
proptest = "1"       # property-based testing

# ──────────────────────────────────────────────────────────────────
# membrain-cli: CLI + daemon + MCP
# ──────────────────────────────────────────────────────────────────
[package]
name = "membrain-cli"
version = "0.1.0"
edition = "2021"

[dependencies]
membrain-core = { path = "../membrain-core" }

# CLI parsing
clap = { version = "4", features = ["derive", "color", "suggestions"] }

# MCP server (stdio transport for Claude Code / Cursor)
rmcp = { version = "0.1", features = ["server", "transport-stdio"] }

# Async runtime (same as core)
tokio = { version = "1", features = ["full"] }

# Unix socket IPC
# (tokio::net::UnixListener — included in tokio)

# Config file
toml = "0.8"
dirs = "5"  # platform-appropriate config directory (~/.membrain)

# Serialization for IPC
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Terminal output
colored = "2"
indicatif = "0.17"   # progress bars for consolidation

# Error handling
anyhow = "1"

# ──────────────────────────────────────────────────────────────────
# Build profile
# ──────────────────────────────────────────────────────────────────
[profile.release]
opt-level     = 3          # maximum optimization
lto           = true       # link-time optimization (smaller binary, faster)
codegen-units = 1          # single codegen unit (enables more optimization)
panic         = "abort"    # no unwinding (smaller, faster)
strip         = "symbols"  # strip debug symbols from release binary

# Build command for maximum performance:
# RUSTFLAGS="-C target-cpu=native" cargo build --release
# This enables: AVX2/AVX-512 (usearch SIMD), native cache line sizes

# ──────────────────────────────────────────────────────────────────
# Techstack comparison table
# ──────────────────────────────────────────────────────────────────
# COMPONENT          BEFORE         AFTER           IMPROVEMENT
# ──────────────────────────────────────────────────────────────────
# Vector index       sqlite-vec     usearch HNSW    100-1000× faster
#                    O(n×d) BF      O(log n) ANN    at 1M memories
# Quantization       float32 only   f16 hot / i8    2-4× faster search
#                                   cold + f32       4× less storage
#                                   rescore
# Decay              eager O(n)     lazy O(1)       ∞ at idle
#                    iteration      on demand        0ms overhead
# Embedding          per-call       LruCache +       0ms cache hit
#                    every time     batch mode       3-5× batch speed
# Search space       full scan      SQL pre-filter   200× reduction
#                                   LIMIT 5000       before HNSW
# Fast path          none           LruCache Tier1   <0.1ms hit
# Cold storage       sqlite-vec     usearch mmap     unlimited scale
#                    RAM-bounded    disk-bounded      OS page cache
# Content compress   none           zstd level 3     3-4× smaller
```

---

## 8. Data Schema — Full SQL + Rust Structs

### 8.1 hot.db Schema

```sql
-- ═══════════════════════════════════════════════════════════════════
-- hot.db — SQLite WAL, primary brain store
-- Equivalent to hippocampal index + recent episodic memories
-- ═══════════════════════════════════════════════════════════════════

PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -131072;   -- 128MB page cache
PRAGMA mmap_size = 4294967296; -- 4GB mmap
PRAGMA temp_store = MEMORY;
PRAGMA foreign_keys = ON;
PRAGMA optimize;

-- ───────────────────────────────────────────────────────────────────
-- memory_index: hot-path columns only
-- Designed for fast pre-filter scans (narrow rows = more rows per page)
-- ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS memory_index (
    -- Identity
    id                  BLOB NOT NULL PRIMARY KEY,  -- UUID (16 bytes)
    usearch_id          INTEGER NOT NULL UNIQUE,     -- u64 for usearch

    -- Lazy decay parameters (ALL needed for effective_strength() in SQL)
    base_strength       REAL NOT NULL DEFAULT 0.5,  -- current base (resets on recall)
    stability           REAL NOT NULL DEFAULT 50.0, -- Ebbinghaus S parameter
    last_accessed_tick  INTEGER NOT NULL,            -- decay clock reference point
    bypass_decay        INTEGER NOT NULL DEFAULT 0, -- 0=normal, 1=emotional bypass

    -- Classification
    kind                INTEGER NOT NULL DEFAULT 0,
    -- 0=Episodic, 1=Semantic, 2=Procedural, 3=Emotional

    -- State machine
    state               INTEGER NOT NULL DEFAULT 0,
    -- 0=Labile, 1=Stable, 2=Consolidated, 3=Archived

    -- Labile window tracking (for reconsolidation)
    labile_since_tick   INTEGER,                    -- NULL when Stable
    labile_window       INTEGER,                    -- NULL when Stable

    -- Access tracking (for predictive pruning + consolidation scoring)
    access_count        INTEGER NOT NULL DEFAULT 0,
    created_tick        INTEGER NOT NULL,

    -- Associative structure
    engram_id           BLOB,                       -- UUID, NULL if no engram yet

    -- Interference tracking
    retrieval_difficulty REAL NOT NULL DEFAULT 0.0, -- increased by proactive interference

    -- Emotional dimensions (for pre-filter and bypass_decay computation)
    emotional_valence   REAL NOT NULL DEFAULT 0.0,  -- -1.0 to 1.0
    emotional_arousal   REAL NOT NULL DEFAULT 0.0,  -- 0.0 to 1.0
    emotional_processed INTEGER NOT NULL DEFAULT 0, -- 0=unprocessed, 1=REM-processed

    -- Source tracking
    source              INTEGER NOT NULL DEFAULT 0,
    -- 0=CLI, 1=MCP, 2=RustEmbed, 3=Working memory eviction

    -- Reconsolidation tracking
    last_reconsolidated_tick INTEGER,               -- NULL if never reconsolidated
    pending_update_tick      INTEGER,               -- tick when update was submitted
    reconsolidation_bonus    REAL NOT NULL DEFAULT 0.0 -- accumulated bonus strength
) STRICT;

-- Indexes for pre-filter performance
CREATE INDEX IF NOT EXISTS idx_memory_prefilter
    ON memory_index(state, bypass_decay, base_strength DESC);

CREATE INDEX IF NOT EXISTS idx_memory_kind
    ON memory_index(kind, state);

CREATE INDEX IF NOT EXISTS idx_memory_engram
    ON memory_index(engram_id) WHERE engram_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_memory_labile
    ON memory_index(state, labile_since_tick)
    WHERE state = 0;  -- Labile only

CREATE INDEX IF NOT EXISTS idx_memory_emotional
    ON memory_index(bypass_decay, emotional_processed)
    WHERE bypass_decay = 1;

CREATE INDEX IF NOT EXISTS idx_memory_predictive_prune
    ON memory_index(access_count, created_tick, state)
    WHERE bypass_decay = 0 AND state NOT IN (2, 3);

-- ───────────────────────────────────────────────────────────────────
-- memory_content: actual text (fetched only for final top-K results)
-- Separate table = pre-filter scans don't touch content bytes
-- ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS memory_content (
    id      BLOB NOT NULL PRIMARY KEY REFERENCES memory_index(id),
    content TEXT NOT NULL,  -- original text content (uncompressed in hot tier)

    -- Context stored as text (embedding stored in memory_vectors)
    context TEXT NOT NULL DEFAULT ''
) STRICT;

-- ───────────────────────────────────────────────────────────────────
-- memory_vectors: float32 embeddings for rescore
-- Fetched only for top-20 candidates during rescore phase
-- ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS memory_vectors (
    id                  BLOB NOT NULL PRIMARY KEY REFERENCES memory_index(id),
    embedding_f32       BLOB NOT NULL,  -- Vec<f32> as raw bytes (384 × 4 = 1536 bytes)
    context_embedding   BLOB NOT NULL,  -- context vector (384 × 4 = 1536 bytes)
    embedding_norm      REAL NOT NULL   -- pre-computed L2 norm (for fast cosine)
) STRICT;

-- ───────────────────────────────────────────────────────────────────
-- engrams: cluster metadata
-- ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS engrams (
    id                  BLOB NOT NULL PRIMARY KEY,  -- UUID
    usearch_id          INTEGER NOT NULL UNIQUE,    -- for centroid HNSW index
    centroid            BLOB NOT NULL,              -- Vec<f32> centroid vector
    member_count        INTEGER NOT NULL DEFAULT 1,
    total_strength      REAL NOT NULL DEFAULT 0.0,  -- sum of effective strengths
    created_tick        INTEGER NOT NULL,
    last_activated_tick INTEGER NOT NULL,
    parent_engram_id    BLOB,                       -- NULL for root engrams
    split_tick          INTEGER                     -- NULL if never split
) STRICT;

CREATE INDEX IF NOT EXISTS idx_engrams_parent
    ON engrams(parent_engram_id) WHERE parent_engram_id IS NOT NULL;

-- ───────────────────────────────────────────────────────────────────
-- engram_edges: persistent graph edges (petgraph in-memory is primary)
-- This table allows graph reconstruction after restart
-- ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS engram_edges (
    from_id          BLOB NOT NULL REFERENCES memory_index(id),
    to_id            BLOB NOT NULL REFERENCES memory_index(id),
    similarity       REAL NOT NULL,
    edge_type        INTEGER NOT NULL DEFAULT 0,
    -- 0=Associative, 1=Causal, 2=Contradictory, 3=Temporal
    created_tick     INTEGER NOT NULL,
    activation_count INTEGER NOT NULL DEFAULT 0,  -- traversal count
    PRIMARY KEY (from_id, to_id)
) STRICT;

CREATE INDEX IF NOT EXISTS idx_engram_edges_from
    ON engram_edges(from_id, similarity DESC);

CREATE INDEX IF NOT EXISTS idx_engram_edges_to
    ON engram_edges(to_id, similarity DESC);

-- ───────────────────────────────────────────────────────────────────
-- prospective_triggers: future-triggered recall
-- ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS prospective_triggers (
    id                   BLOB NOT NULL PRIMARY KEY,
    trigger_context_text TEXT NOT NULL,      -- original trigger description
    trigger_embedding    BLOB NOT NULL,      -- Vec<f32> (384 × 4 bytes)
    memory_id            BLOB NOT NULL REFERENCES memory_index(id),
    trigger_threshold    REAL NOT NULL DEFAULT 0.8,
    fire_count           INTEGER NOT NULL DEFAULT 0,
    max_fires            INTEGER,            -- NULL = unlimited
    created_tick         INTEGER NOT NULL,
    expires_tick         INTEGER             -- NULL = never expires
) STRICT;

-- ───────────────────────────────────────────────────────────────────
-- primed_contexts: active spotlight/priming sessions
-- ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS primed_contexts (
    id                   BLOB NOT NULL PRIMARY KEY,
    context_text         TEXT NOT NULL,
    context_embedding    BLOB NOT NULL,   -- Vec<f32>
    boost_factor         REAL NOT NULL DEFAULT 0.3,
    created_tick         INTEGER NOT NULL,
    expiry_tick          INTEGER NOT NULL
) STRICT;

-- ───────────────────────────────────────────────────────────────────
-- brain_state: singleton row, global brain metadata
-- ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS brain_state (
    id                        INTEGER NOT NULL PRIMARY KEY DEFAULT 1,
    interaction_tick          INTEGER NOT NULL DEFAULT 0,
    last_consolidation_tick   INTEGER,
    last_forgetting_tick      INTEGER,
    total_encoded             INTEGER NOT NULL DEFAULT 0,
    total_recalled            INTEGER NOT NULL DEFAULT 0,
    total_archived            INTEGER NOT NULL DEFAULT 0,
    hot_count                 INTEGER NOT NULL DEFAULT 0,  -- cached count
    config_hash               TEXT,                        -- detect config changes
    schema_version            INTEGER NOT NULL DEFAULT 1,
    created_at                TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (id = 1)  -- only one row
) STRICT;

INSERT OR IGNORE INTO brain_state (id) VALUES (1);

-- ───────────────────────────────────────────────────────────────────
-- uuid_usearch_map: Uuid ↔ u64 mapping for usearch external IDs
-- ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS uuid_usearch_map (
    uuid_bytes  BLOB NOT NULL PRIMARY KEY,  -- 16 bytes UUID
    usearch_id  INTEGER NOT NULL UNIQUE     -- u64 (first 64 bits of UUID)
) STRICT;

-- ───────────────────────────────────────────────────────────────────
-- working_memory: persistent working memory state
-- ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS working_memory (
    slot_index        INTEGER NOT NULL PRIMARY KEY,  -- 0 to 6
    memory_id         BLOB REFERENCES memory_index(id),
    attention_weight  REAL NOT NULL DEFAULT 0.0,
    added_tick        INTEGER NOT NULL DEFAULT 0,
    source            INTEGER NOT NULL DEFAULT 0     -- WorkingMemorySource enum
) STRICT;

-- Initialize 7 slots
INSERT OR IGNORE INTO working_memory (slot_index, added_tick)
    VALUES (0,0),(1,0),(2,0),(3,0),(4,0),(5,0),(6,0);
```

### 8.2 cold.db Schema

```sql
-- ═══════════════════════════════════════════════════════════════════
-- cold.db — SQLite WAL, consolidated semantic memory
-- Equivalent to neocortical long-term storage
-- ═══════════════════════════════════════════════════════════════════

PRAGMA journal_mode = WAL;
PRAGMA synchronous = OFF;     -- cold DB: writes rare, durability less critical
PRAGMA cache_size = -32768;   -- 32MB (less than hot — accessed less often)
PRAGMA mmap_size = 17179869184; -- 16GB mmap (cold DB can be huge)
PRAGMA temp_store = MEMORY;

-- ───────────────────────────────────────────────────────────────────
-- cold_memories: consolidated semantic memories
-- All content is zstd-compressed (3-4× size reduction)
-- ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS cold_memories (
    id                  BLOB NOT NULL PRIMARY KEY,  -- UUID (same as hot_store id)
    usearch_id          INTEGER NOT NULL UNIQUE,    -- for cold HNSW index

    -- Compressed content
    content_compressed  BLOB NOT NULL,  -- zstd::encode(content, level=3)
    context_compressed  BLOB NOT NULL,  -- zstd::encode(context, level=3)

    -- Float32 vectors (ground truth for rescore — not compressed)
    embedding_f32       BLOB NOT NULL,  -- 384 × 4 = 1536 bytes
    context_embedding   BLOB NOT NULL,  -- 384 × 4 = 1536 bytes
    embedding_norm      REAL NOT NULL,  -- pre-computed for fast cosine

    -- Preserved metadata from hot tier
    base_strength       REAL NOT NULL,
    stability           REAL NOT NULL,
    kind                INTEGER NOT NULL,
    emotional_valence   REAL NOT NULL DEFAULT 0.0,
    emotional_arousal   REAL NOT NULL DEFAULT 0.0,
    access_count        INTEGER NOT NULL DEFAULT 0,
    source              INTEGER NOT NULL DEFAULT 0,
    engram_id           BLOB,

    -- Timestamps
    created_tick        INTEGER NOT NULL,   -- when originally encoded
    consolidated_tick   INTEGER NOT NULL,   -- when migrated to cold
    last_accessed_tick  INTEGER NOT NULL    -- updated on cold recall
) STRICT;

CREATE INDEX IF NOT EXISTS idx_cold_engram
    ON cold_memories(engram_id) WHERE engram_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_cold_kind
    ON cold_memories(kind);

CREATE INDEX IF NOT EXISTS idx_cold_strength
    ON cold_memories(base_strength DESC);

-- ───────────────────────────────────────────────────────────────────
-- archive: soft-deleted memories (never hard-deleted)
-- Recovered via: membrain archive restore <id>
-- ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS archive (
    id                  BLOB NOT NULL PRIMARY KEY,
    content_compressed  BLOB NOT NULL,
    base_strength_at_archive REAL NOT NULL,
    archived_tick       INTEGER NOT NULL,
    archive_reason      INTEGER NOT NULL,
    -- 0=Decay, 1=Homeostasis, 2=PredictivePrune, 3=CapacityLimit, 4=Manual
    original_created_tick INTEGER NOT NULL,
    original_kind       INTEGER NOT NULL
) STRICT;
```

### 8.3 procedural.db Schema

```sql
-- ═══════════════════════════════════════════════════════════════════
-- procedural.db — Habit and skill storage
-- No decay, no HNSW, O(1) lookup by pattern hash
-- ═══════════════════════════════════════════════════════════════════

PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;

CREATE TABLE IF NOT EXISTS procedural_memories (
    id              BLOB NOT NULL PRIMARY KEY,  -- UUID
    pattern_hash    BLOB NOT NULL UNIQUE,       -- xxhash128(pattern)
    pattern         TEXT NOT NULL,              -- trigger pattern description
    action          TEXT NOT NULL,              -- what to do when pattern matches
    pattern_embedding BLOB NOT NULL,            -- Vec<f32> for fuzzy matching
    fire_count      INTEGER NOT NULL DEFAULT 0,
    created_tick    INTEGER NOT NULL,
    last_fired_tick INTEGER,
    confidence      REAL NOT NULL DEFAULT 1.0  -- 0.0 to 1.0
) STRICT;

CREATE INDEX IF NOT EXISTS idx_procedural_hash
    ON procedural_memories(pattern_hash);

-- Procedural HNSW index: separate small usearch index for fuzzy pattern matching
-- (~1k entries max — procedural memories are few and highly stable)
```

### 8.4 Rust Structs

```rust
// ═══════════════════════════════════════════════════════════════════
// Core Memory Types
// ═══════════════════════════════════════════════════════════════════

use uuid::Uuid;
use serde::{Serialize, Deserialize};

// ─── Memory Kind ──────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum MemoryKind {
    Episodic    = 0,  // Specific events with temporal/contextual tags
    Semantic    = 1,  // General knowledge, facts, concepts
    Procedural  = 2,  // Habits, patterns → actions (in procedural_store)
    Emotional   = 3,  // High-salience emotional memories
}

impl std::str::FromStr for MemoryKind {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "episodic"   => Ok(MemoryKind::Episodic),
            "semantic"   => Ok(MemoryKind::Semantic),
            "procedural" => Ok(MemoryKind::Procedural),
            "emotional"  => Ok(MemoryKind::Emotional),
            _ => Err(anyhow::anyhow!("Unknown MemoryKind: {}", s)),
        }
    }
}

// ─── Memory State ─────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum MemoryState {
    Labile {
        since_tick:   u64,
        window_ticks: u64,
    },
    Stable       = 1,
    Consolidated = 2,  // migrated to cold_store
    Archived     = 3,  // below MIN_STRENGTH, soft-deleted
}

// ─── Emotional Tag ────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct EmotionalTag {
    pub valence: f32,  // -1.0 (negative) to 1.0 (positive)
    pub arousal: f32,  // 0.0 (calm) to 1.0 (highly excited)
}

impl EmotionalTag {
    pub fn strength_multiplier(&self) -> f32 {
        let intensity = self.arousal * self.valence.abs();
        1.0 + (intensity * EMOTIONAL_WEIGHT)
    }

    pub fn should_bypass_decay(&self) -> bool {
        self.arousal > AROUSAL_THRESHOLD && self.valence.abs() > VALENCE_THRESHOLD
    }

    pub fn is_neutral(&self) -> bool {
        self.arousal < 0.1 && self.valence.abs() < 0.1
    }
}

// ─── Memory Source ────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum MemorySource {
    Cli         = 0,
    Mcp         = 1,
    RustEmbed   = 2,  // Rust code embedding directly via membrain-core API
    WmEviction  = 3,  // Evicted from working memory
    Consolidate = 4,  // Created during consolidation (auto-abstraction)
}

// ─── Archive Reason ───────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ArchiveReason {
    Decay           = 0,
    Homeostasis     = 1,
    PredictivePrune = 2,
    CapacityLimit   = 3,
    Manual          = 4,
}

// ─── Edge Type ────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EdgeType {
    Associative  = 0,  // semantic similarity
    Causal       = 1,  // A preceded B in same session, semantically related
    Contradictory = 2, // high semantic similarity but conflicting content
    Temporal     = 3,  // simple temporal precedence
}

// ─── Edge Weight ─────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeWeight {
    pub similarity:       f32,
    pub edge_type:        EdgeType,
    pub created_tick:     u64,
    pub activation_count: u32,
}

// ─── Memory Index (hot-path struct, matches memory_index table) ────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryIndex {
    pub id:                      Uuid,
    pub usearch_id:              u64,
    pub base_strength:           f32,
    pub stability:               f32,
    pub last_accessed_tick:      u64,
    pub bypass_decay:            bool,
    pub kind:                    MemoryKind,
    pub state:                   MemoryState,
    pub access_count:            u32,
    pub created_tick:            u64,
    pub engram_id:               Option<Uuid>,
    pub retrieval_difficulty:    f32,
    pub emotional_valence:       f32,
    pub emotional_arousal:       f32,
    pub emotional_processed:     bool,
    pub source:                  MemorySource,
    pub last_reconsolidated_tick: Option<u64>,
}

impl MemoryIndex {
    /// Lazy Ebbinghaus — O(1), no DB write
    #[inline]
    pub fn effective_strength(&self, now_tick: u64) -> f32 {
        if self.bypass_decay {
            return self.base_strength;
        }
        let elapsed = now_tick.saturating_sub(self.last_accessed_tick) as f32;
        let retention = (-elapsed / self.stability).exp();
        (self.base_strength * retention).max(0.0)
    }

    pub fn is_decaying_soon(&self, now_tick: u64) -> bool {
        !self.bypass_decay
            && self.effective_strength(now_tick) < (MIN_STRENGTH * 2.0)
    }

    pub fn age_ticks(&self, now_tick: u64) -> u64 {
        now_tick.saturating_sub(self.created_tick)
    }

    pub fn emotional_tag(&self) -> EmotionalTag {
        EmotionalTag {
            valence: self.emotional_valence,
            arousal: self.emotional_arousal,
        }
    }
}

// ─── Full Memory Record (index + content + vectors) ───────────────
#[derive(Debug, Clone)]
pub struct MemoryRecord {
    pub index:            MemoryIndex,
    pub content:          String,
    pub context:          String,
    pub embedding_f32:    Vec<f32>,
    pub context_embedding: Vec<f32>,
}

// ─── Scored Memory (recall result) ────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredMemory {
    pub id:               Uuid,
    pub content:          String,
    pub context:          String,
    pub score:            f32,
    pub effective_strength: f32,
    pub kind:             MemoryKind,
    pub emotional_tag:    EmotionalTag,
    pub access_count:     u32,
    pub created_tick:     u64,
    pub engram_id:        Option<Uuid>,
    pub decaying_soon:    bool,
    pub tier_found:       RetrievalTier,
}

// ─── Memory Fragment (tip-of-tongue partial recall) ───────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFragment {
    pub partial_content: String,   // first N chars or summarized
    pub confidence:      f32,      // how confident we are
    pub kind_hint:       MemoryKind,
    pub engram_id:       Option<Uuid>,
}

// ─── Recall Result ────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallResult {
    pub memories:       Vec<ScoredMemory>,
    pub tier_used:      RetrievalTier,
    pub engram_expanded: bool,
    pub tip_of_tongue:  Option<Vec<MemoryFragment>>,
    pub latency_us:     u64,
    pub total_searched: usize,  // how many candidates were evaluated
}

// ─── Retrieval Tier ───────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetrievalTier {
    Tier1,  // LruCache hit
    Tier2,  // HNSW hot index
    Tier3,  // HNSW cold mmap index
}

// ─── Encode Request ───────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct EncodeRequest {
    pub content:          String,
    pub context:          String,
    pub attention_score:  f32,
    pub emotional_tag:    EmotionalTag,
    pub kind:             MemoryKind,
    pub source:           MemorySource,
}

// ─── Encode Result ────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodeResult {
    pub id:               Uuid,
    pub initial_strength: f32,
    pub novelty_score:    f32,
    pub engram_id:        Option<Uuid>,
    pub tick:             u64,
    pub was_duplicate:    bool,  // true if very similar to existing → updated existing
}

// ─── Engram ───────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engram {
    pub id:                  Uuid,
    pub usearch_id:          u64,
    pub centroid:            Vec<f32>,
    pub member_count:        usize,
    pub total_strength:      f32,
    pub created_tick:        u64,
    pub last_activated_tick: u64,
    pub parent_engram_id:    Option<Uuid>,
}

// ─── Brain Stats ──────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainStats {
    pub hot_count:             usize,
    pub cold_count:            usize,
    pub archive_count:         usize,
    pub total_count:           usize,
    pub avg_hot_strength:      f32,
    pub avg_cold_strength:     f32,
    pub n_engrams:             usize,
    pub avg_engram_size:       f32,
    pub interaction_tick:      u64,
    pub tier1_hit_rate:        f32,
    pub embed_cache_hit_rate:  f32,
    pub last_consolidation:    Option<u64>,
    pub decaying_soon_count:   usize,  // memories near MIN_STRENGTH
    pub emotional_count:       usize,  // bypass_decay = true
    pub labile_count:          usize,  // state = Labile
    pub pending_updates:       usize,
}

// ─── Consolidation Report ─────────────────────────────────────────
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsolidationReport {
    pub nrem_migrated:    usize,
    pub rem_processed:    usize,
    pub homeostasis_archived: usize,
    pub duration_ms:      u64,
    pub trigger:          ConsolidationTrigger,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ConsolidationTrigger {
    #[default]
    Manual,
    Pressure,
    Periodic,
}

// ─── Recall Query ─────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct RecallQuery {
    pub content:               String,
    pub context:               Option<String>,
    pub top_k:                 usize,
    pub confidence_requirement: ConfidenceLevel,
    pub min_strength:          f32,
    pub include_kinds:         Option<Vec<MemoryKind>>,
    pub include_decaying:      bool,
    pub as_of_tick:            Option<u64>,  // time-travel recall
}

#[derive(Debug, Clone, Copy)]
pub enum ConfidenceLevel {
    FastApprox,  // ef=10
    Normal,      // ef=50 (default)
    High,        // ef=100
}

// ─── Working Memory Item ─────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct WorkingMemoryItem {
    pub memory_id:  Uuid,
    pub content:    String,
    pub embedding:  Vec<f32>,
    pub added_tick: u64,
    pub source:     WorkingMemorySource,
}

#[derive(Debug, Clone, Copy)]
pub enum WorkingMemorySource {
    External,      // new information from outside
    FromLtm,       // recalled from long-term memory
    JustEncoded,   // just encoded to hot_store, still in WM
}

// ─── Prospective Trigger ──────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProspectiveTrigger {
    pub id:                 Uuid,
    pub trigger_context:    String,
    pub trigger_embedding:  Vec<f32>,
    pub memory_id:          Uuid,
    pub trigger_threshold:  f32,
    pub fire_count:         u32,
    pub max_fires:          Option<u32>,
    pub created_tick:       u64,
    pub expires_tick:       Option<u64>,
}

// ─── Primed Context ───────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct PrimedContext {
    pub id:               Uuid,
    pub context_text:     String,
    pub embedding:        Vec<f32>,
    pub boost_factor:     f32,
    pub created_tick:     u64,
    pub expiry_tick:      u64,
}

// ─── BrainStore (top-level) ───────────────────────────────────────
pub struct BrainStore {
    pub hot:              HotStore,
    pub cold:             ColdStore,
    pub procedural:       ProceduralStore,
    pub engram_graph:     tokio::sync::RwLock<EngramGraph>,
    pub engram_builder:   EngramBuilder,
    pub working_memory:   WorkingMemory,
    pub embed_cache:      EmbedCache,
    pub tier1_cache:      LruCache<u64, CachedMemory>,
    pub forgetting_engine: ForgettingEngine,
    pub config:           Config,
    pub interaction_tick: Arc<AtomicU64>,
    pub primed_contexts:  Vec<PrimedContext>,
    // Channel to wake consolidation task
    pub consolidation_tx: tokio::sync::mpsc::Sender<ConsolidationTrigger>,
}

impl BrainStore {
    pub fn tick(&self) -> u64 {
        self.interaction_tick.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    pub fn needs_consolidation(&self) -> bool {
        self.hot.len() > (self.config.hot_capacity as f32 * 0.9) as usize
    }
}
```

### 8.5 Config Schema

```toml
# ~/.membrain/config.toml
# All values shown are defaults

[brain]
model             = "all-MiniLM-L6-v2"   # embedding model
hot_capacity      = 50_000               # max memories in hot HNSW index
soft_cap          = 1_000_000            # archive bottom % when total > this
embedding_cache   = 1_000               # LruCache entries for embeddings
tier1_cache       = 512                 # LruCache entries for Tier1

[vector]
dimensions        = 384                  # must match model output
hot_quantization  = "f16"               # f16 | f32
cold_quantization = "i8"                # i8 | f16
rescore_top_k     = 20                  # float32 rescore top-N candidates
pre_filter_limit  = 5_000              # max candidates from SQL pre-filter
hnsw_m            = 16                  # HNSW connectivity (hot)
hnsw_m_cold       = 8                   # HNSW connectivity (cold, saves RAM)
hnsw_ef_construct = 200                 # build quality
hnsw_ef_default   = 50                  # default search quality (adaptive)

[ltp_ltd]
ltp_delta              = 0.1            # LTP boost per recall
stability_increment    = 0.2            # stability growth rate (fraction of current)
max_strength           = 1.0            # strength ceiling
min_strength           = 0.05           # archival threshold
max_stability          = 10_000.0       # stability ceiling

[emotional]
emotional_weight           = 0.5        # how much emotion boosts initial strength
arousal_threshold          = 0.6        # above this: bypass_decay candidate
valence_threshold          = 0.5        # above this (absolute): bypass_decay
desensitization_factor     = 0.95       # arousal reduction per REM cycle
emotional_processed_thresh = 0.3        # arousal below this: mark processed

[consolidation]
nrem_threshold             = 0.4        # min consolidation score to migrate
migration_fraction         = 0.2        # fraction of hot to migrate per cycle
homeostasis_factor         = 0.9        # global scale factor
homeostasis_trigger        = 0.85       # fraction of max load to trigger
consolidation_interval     = 1_000      # interactions between periodic cycles

[reconsolidation]
base_window        = 50                 # ticks for fresh memory reconsolidation window
old_memory_thresh  = 500                # ticks to halve reconsolidation window
labile_min_strength = 0.2              # below this: no reconsolidation window
reconsolidation_bonus = 0.05           # strength bonus on successful update

[interference]
sim_min                  = 0.70         # min similarity to trigger interference
sim_max                  = 0.99         # above this: duplicate (not interference)
retroactive_penalty      = 0.05         # strength reduction for old similar memories
proactive_penalty        = 0.05         # retrieval_difficulty increase for new memory
predictive_value_thresh  = 0.001        # access/age below this: accelerated decay
predictive_decay_factor  = 0.85         # extra decay for non-predictive memories
minimum_prune_age        = 500          # don't predictively prune memories younger than this

[retrieval]
tier1_confidence    = 0.90              # score threshold for Tier1 early return
tier2_confidence    = 0.80              # score threshold for Tier2 early return
partial_threshold   = 0.40              # below this: tip-of-tongue mode
content_weight      = 0.70              # weight for content similarity in score
context_weight      = 0.30              # weight for context similarity in score
cluster_max_depth   = 3                 # BFS max depth in engram graph
cluster_max_nodes   = 50               # BFS max nodes collected
cluster_min_edge    = 0.50              # min edge similarity for BFS traversal
duplicate_threshold = 0.05              # novelty below this: update existing

[engram]
formation_threshold     = 0.65          # min similarity to join existing engram
soft_limit              = 200           # member count to trigger split
hard_limit              = 500           # member count to reject additions
centroid_alpha          = 0.10          # EMA alpha for centroid update
resonance_factor        = 0.30          # fraction of LTP that spreads to neighbors

[forgetting]
prune_interval          = 500           # interactions between pruning passes
prune_batch_size        = 10_000        # max memories scanned per prune pass
archive_fraction        = 0.10          # fraction to archive when > soft_cap

[daemon]
socket_path             = "~/.membrain/membrain.sock"
log_path                = "~/.membrain/membrain.log"
pid_path                = "~/.membrain/membrain.pid"
log_level               = "info"        # error | warn | info | debug | trace
```

---

### End of Snapshot Part 4

**Next: Part 5 — CLI Commands, MCP Tools, Feature Extensions, Workspace Structure**

Parts list:
- Part 1: Vision, Problem Statement, Human Brain Deep Dive ✅
- Part 2: Gap Analysis + Full Port (mechanism → Rust code) ✅
- Part 3: Architecture Overview + Performance ✅
- Part 4: Techstack + Data Schema ✅
- Part 5: CLI/MCP + Feature Extensions + Workspace Structure
- Part 6: Milestones + Acceptance Checklist + Constants + Algorithm Reference


<!-- SOURCE: PLAN_part5.md -->

### Source Snapshot — Part 5
#### Part 5 of 6: CLI Commands · MCP Tools · Feature Extensions · Workspace Structure

---

## 9. CLI Commands & MCP Tools

### 9.1 CLI Overview

```
membrain <COMMAND> [OPTIONS]

COMMANDS:
  remember      Store a new memory
  recall        Retrieve relevant memories
  forget        Archive (soft-delete) a memory
  strengthen    Manually apply LTP to a memory
  update        Submit a pending update during reconsolidation window
  stats         Brain health statistics
  list          List memories (filterable)
  show          Show full details of a specific memory
  diff          Show how memories changed between two ticks
  consolidate   Manually trigger NREM+REM+Homeostasis cycle
  prime         Pre-warm working memory with context (spotlight mode)
  remind        Set a prospective trigger
  watch         Watch for memories approaching decay threshold
  export        Export memories to JSON/NDJSON
  import        Import memories from JSON/NDJSON
  daemon        Daemon management (start|stop|status|restart)
  mcp           Start MCP stdio server
  config        Show/edit configuration
  doctor        Diagnose brain health issues

Global options:
  --json          Output as JSON (all commands support this)
  --quiet, -q     Suppress informational output
  --verbose, -v   Show extra details
  --db-path       Override default database location
  --tick          Show tick numbers in output

Usage style:
  # Pipe-friendly
  echo "JWT tokens expire after 1h" | membrain remember
  membrain recall "auth" | jq '.memories[0].content'
  membrain stats --json | jq '.hot_count'
```

### 9.2 Command: remember

```
USAGE:
  membrain remember [CONTENT] [OPTIONS]
  echo "content" | membrain remember [OPTIONS]

DESCRIPTION:
  Encode a new memory into the brain's hot store.
  Automatically computes embedding, novelty, and initial strength.
  Applies attention gating, emotional tagging, engram clustering.
  Returns the memory ID and encoding metadata.

OPTIONS:
  --context, -c <TEXT>       Current task/context (enhances retrieval later)
  --attention, -a <0.0-1.0>  Attention level (default: 0.7)
                             Below 0.2: memory discarded (not attended to)
  --valence, -V <-1.0-1.0>  Emotional valence (default: 0.0)
                             -1=very negative, 0=neutral, +1=very positive
  --arousal, -A <0.0-1.0>   Emotional arousal (default: 0.0)
                             0=calm, 1=highly excited
  --kind, -k <KIND>          Memory kind: episodic|semantic|procedural
                             (default: episodic)
  --source <SOURCE>          Source tag: cli|mcp|api (default: cli)
  --json                     Output JSON

EXAMPLES:
  # Basic
  membrain remember "Fixed the JWT expiry bug — was using utc() not now()"

  # With context
  membrain remember "Rate limit is 100 req/s for Stripe API" \
    --context "integrating payments" \
    --kind semantic

  # Emotional (production incident)
  membrain remember "Deploy to prod caused 30-min downtime — missing migration" \
    --valence -0.8 --arousal 0.8 \
    --context "deployment"

  # Low attention (background information, weak encoding)
  membrain remember "The office WiFi password changed" --attention 0.2

  # From stdin
  git log --oneline -5 | membrain remember --context "recent commits" --kind semantic

  # JSON output
  membrain remember "Rust lifetimes are the borrow checker's time tracking" \
    --json

OUTPUT (default):
  ✅ Remembered [id: 550e8400...] strength=0.72 novelty=0.85 engram=7b1c3e42

OUTPUT (--json):
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "initial_strength": 0.720,
    "novelty_score": 0.850,
    "engram_id": "7b1c3e42-...",
    "tick": 1042,
    "was_duplicate": false,
    "kind": "Episodic",
    "bypass_decay": false
  }

IMPLEMENTATION NOTES:
  // Check if daemon is running — if yes, forward via socket
  // If no daemon: standalone mode (500ms cold start for embed model)
  
  let request = EncodeRequest {
      content: args.content,
      context: args.context.unwrap_or_default(),
      attention_score: args.attention.unwrap_or(DEFAULT_ATTENTION),
      emotional_tag: EmotionalTag {
          valence: args.valence.unwrap_or(0.0),
          arousal: args.arousal.unwrap_or(0.0),
      },
      kind: args.kind.unwrap_or(MemoryKind::Episodic),
      source: MemorySource::Cli,
  };
  
  let result = brain.encode(request, now_tick).await?;
  
  // Trigger consolidation check (non-blocking)
  if brain.needs_consolidation() {
      brain.consolidation_tx.send(ConsolidationTrigger::Pressure).await.ok();
  }
```

### 9.3 Command: recall

```
USAGE:
  membrain recall <QUERY> [OPTIONS]

DESCRIPTION:
  Retrieve memories relevant to the query using 3-tier search.
  Applies: embedding → Tier1 cache → SQL pre-filter → HNSW search →
           float32 rescore → engram BFS expansion → unified scoring.
  Returns ranked list of memories with scores and metadata.

OPTIONS:
  --context, -c <TEXT>   Current context (boosts contextually relevant memories)
  --top, -n <N>          Number of results to return (default: 5)
  --kind, -k <KIND>      Filter by kind: episodic|semantic|procedural
  --min-strength <F>     Minimum effective strength (default: from config)
  --confidence <LEVEL>   Search confidence: fast|normal|high (default: normal)
  --show-decaying        Include memories close to decay threshold
  --no-engram            Disable engram expansion (pure HNSW results only)
  --as-of <TICK>         Time-travel: reconstruct knowledge state at tick N
  --json                 Output JSON

EXAMPLES:
  # Basic
  membrain recall "JWT authentication"

  # With context (improves relevance for current task)
  membrain recall "database connection" --context "fixing performance issue"

  # More results with high confidence
  membrain recall "Rust async" --top 10 --confidence high

  # Filter to semantic only
  membrain recall "Python" --kind semantic

  # Surface memories about to be forgotten
  membrain recall "anything" --show-decaying

  # Time-travel recall
  membrain recall "what did I know about auth" --as-of 5000

  # Pipe to fzf for interactive selection
  membrain recall "deployment" --json | jq -r '.memories[].content' | fzf

OUTPUT (default):
  Found 5 memories (Tier2, 3ms, engram expanded)
  
  [1] score=0.94 strength=0.81 ◆engram:7b1c3e42
      JWT tokens in this codebase expire after 1 hour — use utc() not now()
      kind=Semantic  accessed=12×  tick=847

  [2] score=0.87 strength=0.63
      Fixed the JWT expiry bug — was calling time::now() in wrong timezone
      kind=Episodic  accessed=3×  tick=923  ⚠️ decaying

  [3] score=0.82 strength=0.71 ◆engram:7b1c3e42
      Auth middleware validates JWT on every request — check expiry first
      kind=Semantic  accessed=8×  tick=756

OUTPUT (--json):
  {
    "memories": [
      {
        "id": "...",
        "content": "JWT tokens in this codebase...",
        "context": "debugging auth",
        "score": 0.94,
        "effective_strength": 0.81,
        "kind": "Semantic",
        "emotional_tag": { "valence": 0.0, "arousal": 0.0 },
        "access_count": 12,
        "created_tick": 847,
        "engram_id": "7b1c3e42-...",
        "decaying_soon": false
      },
      ...
    ],
    "tier_used": "Tier2",
    "latency_us": 3124,
    "engram_expanded": true,
    "total_searched": 847,
    "tip_of_tongue": null
  }

IMPLEMENTATION NOTES:
  // Adaptive ef based on confidence level and store size
  let ef = adaptive_ef(&query, hot_count, tier1_hit_rate);
  
  // Run full 3-tier pipeline
  let result = brain.recall(query, now_tick).await?;
  
  // Apply on_recall to all returned memories (LTP + labile)
  for m in &result.memories {
      brain.on_recall(m.id, now_tick).await?;
  }
  
  // Surface decaying memories if requested
  if args.show_decaying {
      let decaying = brain.find_decaying_soon(now_tick)?;
      // ... merge into results
  }
```

### 9.4 Command: forget

```
USAGE:
  membrain forget <ID> [OPTIONS]

DESCRIPTION:
  Archive (soft-delete) a specific memory.
  Memory is moved to archive table — not hard-deleted.
  Can be recovered with: membrain archive restore <ID>
  Removes from HNSW index and Tier1 cache.

OPTIONS:
  --reason <TEXT>   Optional note for why forgotten
  --json            Output JSON

EXAMPLES:
  membrain forget 550e8400-e29b-41d4-a716-446655440000
  membrain forget 550e8400 --reason "outdated, API changed"

OUTPUT:
  🗑️ Archived memory 550e8400... (was strength=0.42)

IMPLEMENTATION:
  brain.archive_memory(id, now_tick, ArchiveReason::Manual)?;
  // Also removes from HNSW: hot_index.remove(usearch_id)
  // Also removes from Tier1: tier1_cache.pop(content_hash)
```

### 9.5 Command: strengthen

```
USAGE:
  membrain strengthen <ID> [OPTIONS]

DESCRIPTION:
  Manually apply LTP to a specific memory.
  Simulates an explicit recall event.
  Increases base_strength by LTP_DELTA.
  Increases stability (memory will decay slower).
  Sets state to Labile (opens reconsolidation window).

OPTIONS:
  --json   Output JSON

EXAMPLES:
  membrain strengthen 550e8400
  # Use case: you just used this knowledge — reinforce it

OUTPUT:
  ⚡ Strengthened [550e8400...]
     strength: 0.62 → 0.72 (+0.10)
     stability: 150.2 → 180.2 (+30.0)
     reconsolidation window: 48 ticks

IMPLEMENTATION:
  brain.on_recall(id, now_tick).await?;
```

### 9.6 Command: update

```
USAGE:
  membrain update <ID> <NEW_CONTENT> [OPTIONS]

DESCRIPTION:
  Submit a content update for a memory during its reconsolidation window.
  The memory must be in Labile state (recently recalled).
  Update is applied during the next reconsolidation_tick cycle.
  If the reconsolidation window expires before processing: update discarded.

OPTIONS:
  --force    Apply update even if not in Labile state (force Labile)
  --json     Output JSON

EXAMPLES:
  # First recall to open labile window:
  membrain recall "stripe rate limit"
  # Then update with corrected information:
  membrain update <ID> "Stripe rate limit is 200 req/s for paid plans, 100 for free"

OUTPUT:
  📝 Update queued for [550e8400...]
     Window: 47 ticks remaining
     Will apply at next reconsolidation tick

  # If window expired:
  ❌ Reconsolidation window closed — recall first to reopen
```

### 9.7 Command: stats

```
USAGE:
  membrain stats [OPTIONS]

DESCRIPTION:
  Show comprehensive brain health statistics.
  Includes store sizes, performance metrics, memory distribution.

OPTIONS:
  --json      Output JSON
  --watch     Refresh every 5 seconds (live monitoring)

OUTPUT (default):
  ╔═══════════════════════════════════════════════════════╗
  ║              membrain brain stats                     ║
  ╠═══════════════════════════════════════════════════════╣
  ║ Storage                                               ║
  ║   Hot (hippocampus):   12,847 memories               ║
  ║   Cold (neocortex):    48,231 memories               ║
  ║   Archive:             3,241 memories                 ║
  ║   Total:               64,319 memories               ║
  ║                                                       ║
  ║ Strengths                                             ║
  ║   Avg hot strength:    0.612                          ║
  ║   Avg cold strength:   0.841                          ║
  ║   Decaying soon:       234 (< 2× MIN_STRENGTH)        ║
  ║   Emotional bypass:    89 (bypass_decay=true)         ║
  ║   Labile:              42 (reconsolidation open)      ║
  ║                                                       ║
  ║ Engram Graph                                          ║
  ║   Total engrams:       1,847                          ║
  ║   Avg engram size:     6.9 memories                   ║
  ║   Largest engram:      187 members                    ║
  ║                                                       ║
  ║ Performance (last 1000 recalls)                       ║
  ║   Tier1 hit rate:      63.2%   (<0.1ms)              ║
  ║   Tier2 hit rate:      31.4%   (<5ms)                ║
  ║   Tier3 hit rate:      5.4%    (<50ms)               ║
  ║   Avg recall latency:  2.1ms                          ║
  ║   Embed cache hit:     84.7%                          ║
  ║                                                       ║
  ║ System                                                ║
  ║   Interaction tick:    14,823                         ║
  ║   Last consolidation:  tick 14,100 (723 ticks ago)   ║
  ║   Next consolidation:  ~177 ticks                    ║
  ║   Daemon:              running (pid 18342)            ║
  ╚═══════════════════════════════════════════════════════╝
```

### 9.8 Command: list

```
USAGE:
  membrain list [OPTIONS]

DESCRIPTION:
  List memories with optional filters.
  Useful for exploring what's in the brain.

OPTIONS:
  --kind, -k <KIND>      Filter by kind
  --min-strength <F>     Minimum effective strength
  --max-strength <F>     Maximum effective strength
  --since <TICK>         Created after tick N
  --engram <ID>          All members of a specific engram
  --decaying             Only memories near decay threshold
  --emotional            Only memories with bypass_decay=true
  --labile               Only memories in Labile state
  --sort <FIELD>         Sort by: strength|created|accessed|score
  --limit, -n <N>        Max results (default: 20)
  --json                 Output JSON

EXAMPLES:
  membrain list --kind semantic --sort strength --limit 10
  membrain list --decaying
  membrain list --engram 7b1c3e42 --json
  membrain list --since 10000 --sort created
```

### 9.9 Command: show

```
USAGE:
  membrain show <ID> [OPTIONS]

DESCRIPTION:
  Show complete details for a specific memory.
  Includes all metadata, current effective strength, engram membership,
  reconsolidation window status, and related memories.

OPTIONS:
  --related, -r   Show related memories (engram neighbors)
  --json          Output JSON

EXAMPLES:
  membrain show 550e8400
  membrain show 550e8400 --related --json

OUTPUT (default):
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Memory: 550e8400-e29b-41d4-a716-446655440000
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Content:
    JWT tokens in this codebase expire after 1 hour.
    Use utc() not now() when comparing expiry timestamps.

  Context:  debugging authentication module

  Kind:     Semantic
  State:    Labile (32 ticks remaining in reconsolidation window)

  Strength:
    Base:      0.720
    Effective: 0.681 (5.4% decayed since last access)
    Stability: 180.2 (decays slowly)
    Bypass:    No

  Emotional:
    Valence:  -0.30 (slightly negative)
    Arousal:   0.50 (moderate)

  Timing:
    Created:      tick 847
    Last accessed: tick 14,791 (32 ticks ago)
    Access count:  12

  Engram:   7b1c3e42-... (auth cluster, 23 members)

  Source:   MCP
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Related memories:
    [0.91] Auth middleware validates JWT on every request
    [0.88] OAuth2 flow for Google login uses PKCE
    [0.82] Refresh tokens stored in httpOnly cookies
```

### 9.10 Command: diff

```
USAGE:
  membrain diff --from <TICK> --to <TICK> [OPTIONS]

DESCRIPTION:
  Show how the brain's knowledge changed between two interaction ticks.
  Compares: what was added, what decayed significantly, what was forgotten,
  what engrams formed/split.
  
  Feature extension #6 from the brainstormed top-10.
  Low implementation cost: pure SQL queries on timestamps.

OPTIONS:
  --from <TICK>    Start tick (required)
  --to <TICK>      End tick (default: current)
  --kind <KIND>    Filter by kind
  --json           Output JSON

EXAMPLES:
  membrain diff --from 10000
  membrain diff --from 5000 --to 10000 --json

OUTPUT:
  Brain diff: tick 10000 → 14823
  ─────────────────────────────────
  NEW memories (+847):
    + [t=10234] "Migrated payments to Stripe v3 API" [Semantic]
    + [t=10891] "Redis cluster failover during high load" [Episodic]
    ... (845 more)

  FORGOTTEN (archived, -312):
    - "The old AWS Lambda cold start workaround" [Semantic, was 0.07]
    - "Temp debug code in user controller" [Episodic, was 0.06]
    ... (310 more)

  SIGNIFICANTLY DECAYED (strength dropped >30%, 89 memories):
    ↓ "Local dev uses port 3001 for frontend" 0.81→0.51 (-37%)
    ↓ "Prisma migration naming convention" 0.74→0.49 (-34%)
    ... (87 more)

  ENGRAMS changed:
    + New engram: "Stripe integration cluster" (12 members)
    + Split: "deployment" → "prod deployment" + "staging deployment"
    ~ Grew: "auth" cluster 15→23 members
```

### 9.11 Command: consolidate

```
USAGE:
  membrain consolidate [OPTIONS]

DESCRIPTION:
  Manually trigger the consolidation cycle (NREM + REM + Homeostasis).
  Normally runs automatically based on pressure.
  Useful for explicit maintenance or before long agent sessions.

OPTIONS:
  --nrem-only    Only run NREM migration (skip REM and homeostasis)
  --rem-only     Only run REM emotional processing
  --homeostasis  Only run homeostasis scaling
  --dry-run      Show what WOULD be consolidated without doing it
  --json         Output JSON

EXAMPLES:
  membrain consolidate
  membrain consolidate --dry-run
  membrain consolidate --nrem-only

OUTPUT:
  🧠 Running consolidation cycle...
     NREM: migrated 1,247 memories to cold store (3.2s)
     REM: processed 23 emotional memories (0.1s)
     Homeostasis: scaled down 0 memories (total load: 0.62, below threshold)
  ✅ Consolidation complete in 3.3s
```

### 9.12 Command: prime

```
USAGE:
  membrain prime <CONTEXT> [OPTIONS]

DESCRIPTION:
  Pre-warm Tier1 cache with memories relevant to the given context.
  Simulates mental preparation / spotlight attention.
  Boosts score of matching memories during subsequent recalls.
  
  Feature extension #3 from top-10 list.

OPTIONS:
  --boost <0.0-0.5>    Score boost factor (default: 0.3)
  --duration <N>       How many interactions the priming lasts (default: 1000)
  --json               Output JSON

EXAMPLES:
  membrain prime "debugging the authentication service"
  membrain prime "deploying to production" --boost 0.4 --duration 500

OUTPUT:
  🔦 Primed with "debugging authentication service"
     Pre-loaded 47 relevant memories into Tier1 cache
     Score boost: +0.30 for contextually matching memories
     Expires: tick 15,823 (1000 interactions)
```

### 9.13 Command: remind

```
USAGE:
  membrain remind --when <CONTEXT> --then <CONTENT> [OPTIONS]

DESCRIPTION:
  Set a prospective memory trigger.
  When the agent's context matches --when, the --then memory surfaces automatically.
  Simulates human prospective memory ("remember to do X when Y").
  
  Feature extension #5 from top-10 list.

OPTIONS:
  --when <TEXT>      Context description that triggers the reminder (required)
  --then <TEXT>      Memory content to surface when triggered (required)
  --id <UUID>        Surface an existing memory instead of creating new
  --max-fires <N>    How many times to fire (default: unlimited)
  --expires <TICK>   Expiry tick (default: never)
  --threshold <F>    Similarity threshold to trigger (default: 0.8)
  --json             Output JSON

EXAMPLES:
  # New content triggered by context
  membrain remind \
    --when "working on payments or Stripe" \
    --then "Stripe webhook: always verify signature with STRIPE_WEBHOOK_SECRET"

  # Existing memory triggered
  membrain remind \
    --when "deploying to production" \
    --id 550e8400 \
    --max-fires 5

OUTPUT:
  ⏰ Reminder set [trigger_id: abc123...]
     When: "working on payments or Stripe"
     Then: "Stripe webhook: always verify signature..."
     Fires: unlimited
     Threshold: 0.80

AUTOMATIC TRIGGERING:
  Every encode() call checks all active triggers against current context.
  When context embedding similarity > threshold:
  → memory surfaces in next recall results automatically
  → fire_count incremented
  → if fire_count >= max_fires: trigger deactivated
```

### 9.14 Command: watch

```
USAGE:
  membrain watch [OPTIONS]

DESCRIPTION:
  Watch for memories approaching the decay threshold (decaying_soon).
  Surfaces memories at risk of being archived before they're used.
  Feature extension #10: "Forgetting-as-signal".

OPTIONS:
  --threshold <F>   Strength threshold to flag (default: 2× MIN_STRENGTH)
  --kind <KIND>     Filter by kind
  --interval <N>    Check every N interactions (default: 100)
  --json            Output JSON

EXAMPLES:
  membrain watch
  membrain watch --threshold 0.15 --kind semantic

OUTPUT:
  👁️  Watching for decaying memories...
  
  ⚠️  Memory approaching archive threshold (strength=0.11):
      "Local Redis uses port 6380 (not 6379) in Docker Compose setup"
      kind=Semantic  created=tick:4821  last_accessed=tick:10234
      Hint: membrain strengthen <id> to preserve it
  
  ⚠️  Memory approaching archive threshold (strength=0.09):
      "GraphQL schema requires N+1 protection on all list fields"
      kind=Semantic  created=tick:3102  last_accessed=tick:9891
```

### 9.15 Command: export / import

```
USAGE:
  membrain export [OPTIONS] > memories.ndjson
  membrain import < memories.ndjson [OPTIONS]

DESCRIPTION:
  Export: dumps all non-archived memories as NDJSON (one JSON per line).
  Import: reads NDJSON and encodes each memory (re-embeds, re-clusters).
  
  Use cases:
    - Backup before experiments
    - Transfer memories between machines
    - Share domain knowledge between agent instances
    - Restore after database corruption

OPTIONS (export):
  --kind <KIND>      Filter by kind
  --min-strength <F> Minimum strength to include
  --include-cold     Include cold store memories (default: hot only)
  --include-archive  Include archived memories
  --format <FMT>     json|ndjson|csv (default: ndjson)

OPTIONS (import):
  --merge            Merge with existing (default: reject duplicates)
  --kind <KIND>      Override kind for all imported memories
  --source <SRC>     Tag source (default: import)
  --dry-run          Count without importing

EXAMPLES:
  membrain export > backup.ndjson
  membrain export --include-cold --kind semantic > semantic_knowledge.ndjson
  membrain import < backup.ndjson --merge
  membrain import < semantic_knowledge.ndjson --dry-run

NDJSON FORMAT:
  Each line is one complete memory JSON:
  {"content":"JWT tokens expire..","context":"auth","kind":"Semantic","strength":0.72,"tick":847,"emotional":{"valence":0,"arousal":0}}
  {"content":"Stripe rate limit is 200..","context":"payments","kind":"Semantic","strength":0.81,"tick":1023,"emotional":{"valence":0,"arousal":0}}
```

### 9.16 Command: daemon

```
USAGE:
  membrain daemon <SUBCOMMAND>

SUBCOMMANDS:
  start     Start daemon in background
  stop      Stop running daemon
  restart   Restart daemon
  status    Show daemon status

EXAMPLES:
  membrain daemon start
  membrain daemon status
  membrain daemon stop

OUTPUT (status):
  🟢 Daemon running (pid 18342)
     Socket: ~/.membrain/membrain.sock
     Uptime: 3h 42m 17s
     Interactions served: 14,823
     Memory RSS: 187MB
     Last consolidation: 723 ticks ago

IMPLEMENTATION:
  membrain daemon start:
    Fork process (daemonize)
    Write PID to ~/.membrain/membrain.pid
    Redirect stdout/stderr to ~/.membrain/membrain.log
    Start tokio runtime with all tasks

  membrain daemon stop:
    Read PID from membrain.pid
    Send SIGTERM → graceful shutdown:
      1. Stop accepting new connections
      2. Finish in-flight requests (5s timeout)
      3. Flush SQLite WAL checkpoint
      4. Remove membrain.pid and membrain.sock
      5. Exit 0
```

### 9.17 Command: doctor

```
USAGE:
  membrain doctor [OPTIONS]

DESCRIPTION:
  Diagnose brain health and performance issues.
  Checks: database integrity, index consistency, memory distribution,
  engram health, embedding cache performance, config validity.

OPTIONS:
  --fix     Attempt to fix detected issues automatically
  --json    Output JSON

OUTPUT:
  🏥 membrain doctor — brain health check
  ─────────────────────────────────────────
  ✅ hot.db integrity:          OK (SQLite PRAGMA integrity_check)
  ✅ cold.db integrity:         OK
  ✅ hot.usearch index:         OK (50k entries, no corruption)
  ✅ cold.usearch index:        OK (48,231 entries, mmap consistent)
  ✅ engram graph consistency:  OK (all member IDs resolve in hot.db)
  ⚠️  Embedding model:          NOT LOADED (daemon not running)
  ✅ Config file:               OK (all values in valid range)
  ⚠️  High archive rate:        312 memories archived in last 1000 ticks
      Suggestion: consider raising min_strength threshold or
                  increasing stability_increment
  ✅ Tier1 hit rate:            63.2% (target: >60%) ✅
  ✅ Embed cache hit rate:      84.7% (target: >80%) ✅
  ─────────────────────────────────────────
  Overall: 2 warnings, 0 errors
```

---

## 10. Top 10 Feature Extensions

Each section below is written in English and focuses on **how to build the feature pragmatically**.

### 10.1 Dual Memory Output

**What it adds**
- Every important recall returns two parallel products:
  - **Evidence Pack**
  - **Action Pack**

**Why it matters**
- Separates “what the system knows” from “what the system suggests doing”.
- Improves trust, auditability, and UX clarity.

**How to implement**
1. Add `EvidenceItem` and `ActionArtifact` as separate result types.
2. Keep evidence items tied to provenance and timestamps.
3. Let action artifacts be derived summaries, procedures, heuristics, or next-step suggestions.
4. Extend recall API to return:
   - `evidence_pack`
   - `action_pack`
   - `uncertainty`
5. Add a mode flag:
   - `strict`
   - `balanced`
   - `fast`

**Minimum schema impact**
- new `action_artifacts` table
- source linkage from action artifact → supporting evidence ids

---

### 10.2 Belief Ledger

**What it adds**
- A first-class store for what the agent currently believes, not just what it has seen.

**Why it matters**
- Memory is raw material.
- Belief is the current operational stance.

**How to implement**
1. Add `beliefs`, `belief_support`, and `belief_conflicts` tables.
2. Each belief must track:
   - proposition
   - confidence
   - freshness
   - status
3. Status values:
   - `active`
   - `disputed`
   - `stale`
   - `superseded`
4. Never overwrite support/conflict history silently.
5. Add a `resolve_belief()` pipeline after retrieval / verification.

**Minimum API**
- `beliefs.propose`
- `beliefs.verify`
- `beliefs.get`
- `beliefs.list_conflicts`

---

### 10.3 Memory Leases

**What it adds**
- Every memory-like object has a freshness policy or lease.

**Why it matters**
- Some knowledge expires fast.
- Some knowledge should remain trusted much longer.
- This reduces stale-memory failures.

**How to implement**
1. Add `lease_policy` and `freshness_state` fields to key objects.
2. Define policy classes:
   - `volatile`
   - `normal`
   - `durable`
   - `pinned`
3. On recall or action planning:
   - if stale and action-critical → re-check or lower confidence
4. Add a background lease scanner for transitions only; no full expensive recalculation in hot path.

**Minimum schema**
- `lease_policy`
- `lease_expires_at` or interaction-based equivalent
- `freshness_state`

---

### 10.4 Reflection Compiler

**What it adds**
- Converts successful and failed episodes into reusable procedures, anti-patterns, and checklists.

**Why it matters**
- The system should not only accumulate memories.
- It should improve behavior over time.

**How to implement**
1. After a task closes, gather:
   - goal
   - actions
   - tool outcomes
   - outcome quality
2. Run a structured reflection pass:
   - what worked
   - what failed
   - what should be reused
3. Emit:
   - `Procedure`
   - `Checklist`
   - `ReflectionArtifact`
4. Keep reflection artifacts derived from evidence; do not pretend they are raw facts.

**Minimum release rule**
- Reflection is advisory until validated by repeated usefulness or human approval.

---

### 10.5 Cognitive Blackboard

**What it adds**
- A visible working-state object for active cognition.

**Why it matters**
- Agents become easier to understand, steer, debug, and resume.

**How to implement**
1. Add a `blackboard_state` object per active task/session.
2. Store:
   - current goal
   - subgoals
   - active evidence
   - active beliefs
   - unknowns
   - next action
   - blocked reason
3. Let retrieval promote items into the blackboard instead of exposing raw floods of candidates.
4. Snapshot blackboard state at checkpoints.

**Minimum API**
- `blackboard.get`
- `blackboard.pin`
- `blackboard.dismiss`
- `blackboard.snapshot`

---

### 10.6 Resumable Goal Stack + Checkpoints

**What it adds**
- Long-running tasks can survive interruption, crash, or restart.

**Why it matters**
- This is one of the biggest gaps between flashy demos and useful agents.

**How to implement**
1. Represent goals explicitly:
   - goal
   - subgoals
   - plan steps
   - dependencies
2. Create checkpoints at:
   - major decisions
   - tool boundaries
   - user-visible milestones
3. Store:
   - active goal stack
   - blackboard summary
   - selected evidence ids
   - pending dependencies
4. Add restart tests that verify resume quality.

**Minimum acceptance**
- resume must reconstruct task state without guessing from scratch

---

### 10.7 Safe Preflight Sandbox

**What it adds**
- A dry-run / validation layer before risky actions.

**Why it matters**
- Prevents reckless tool usage and incomplete action execution.

**How to implement**
1. Before risky actions, run:
   - policy checks
   - required-input checks
   - freshness checks
   - dependency checks
   - confidence checks
2. Return a preflight report:
   - ready
   - blocked
   - missing data
   - stale knowledge
3. Allow user-facing “why blocked?” diagnostics.

**Minimum API**
- `preflight.run`
- `preflight.explain`
- `preflight.allow`

---

### 10.8 Namespace Lenses / Role Capsules

**What it adds**
- One brain, many explicit contexts.

**Why it matters**
- Reduces cross-contamination between users, workspaces, projects, and operating modes.

**How to implement**
1. Add a `role_capsule` or `namespace_lens` concept.
2. Use it to condition:
   - retrieval priors
   - policies
   - allowed procedures
   - style and preference biases
3. Default every operation to a namespace and optional capsule.
4. Keep shared/global knowledge explicit, not accidental.

**Minimum schema**
- `namespaces`
- `role_capsules`
- mapping tables for visibility and defaults

---

### 10.9 Uncertainty Surface

**What it adds**
- A structured way to say what is known, inferred, uncertain, and missing.

**Why it matters**
- Helps the system avoid presenting guesses as facts.
- Strongly improves trust and debugging.

**How to implement**
1. Extend result objects with:
   - `known`
   - `assumed`
   - `uncertain`
   - `missing`
   - `change_my_mind_conditions`
2. Derive uncertainty from:
   - evidence coverage
   - belief confidence
   - freshness
   - retrieval diversity
3. Surface uncertainty in high-stakes paths by default.

**Minimum rule**
- no high-confidence output without matching evidence or justified belief support

---

### 10.10 Deterministic Journal + Doctor + Time Travel

**What it adds**
- Replayability, repairability, and historical inspection.

**Why it matters**
- Makes the system production-grade rather than merely clever.

**How to implement**
1. Journal all important mutations:
   - encode
   - consolidate
   - archive
   - patch
   - reverify
   - rebuild
2. Keep periodic snapshots for fast recovery.
3. Implement a doctor tool that checks:
   - orphan edges
   - missing embeddings
   - stale indexes
   - broken lineage
   - checkpoint corruption
4. Add time-travel inspection:
   - “what did the system know / believe at tick T?”

**Minimum commands**
- `doctor run`
- `repair dry-run`
- `replay from`
- `diff --from --to`

## 11. Workspace Structure

### 11.1 Design Principles

1. **Domain-based grouping** — modules cluster by cognitive domain, not by file count
2. **Feature-ready from day one** — 20 features have clear placement without restructuring
3. **No god modules** — engine/, CLI, and MCP are all domain-split
4. **CLI calls core, never reimplements** — thin handler → brain_store method
5. **Policy is first-class** — not scattered across unrelated modules
6. **Extension point** — adding Feature 21+ means one new file in the right domain

### 11.2 Full Directory Tree

```
membrain/
│
├── Cargo.toml                  # workspace root
├── Cargo.lock
├── README.md
├── AGENTS.md                   # context for AI coding assistants
├── .github/
│   └── workflows/
│       ├── ci.yml              # test + clippy + fmt on push
│       └── release.yml         # build binaries on tag + publish to crates.io
│
├── crates/
│   │
│   ├── membrain-core/          # all brain logic — no CLI/daemon concerns
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs          # public API surface
│   │   │   ├── brain_store.rs  # BrainStore: top-level coordinator
│   │   │   ├── types.rs        # MemoryIndex, EmotionalTag, MemoryState, all enums
│   │   │   ├── constants.rs    # all tunable constants
│   │   │   ├── config.rs       # Config struct + TOML parsing
│   │   │   │
│   │   │   ├── store/          # === STORAGE LAYER ===
│   │   │   │   ├── mod.rs      # Store trait + shared helpers
│   │   │   │   ├── hot.rs      # HotStore: hot.db + usearch HNSW (float16, in-RAM)
│   │   │   │   ├── cold.rs     # ColdStore: cold.db + usearch HNSW (int8, mmap)
│   │   │   │   ├── archive.rs  # ArchiveStore: archived/soft-deleted access
│   │   │   │   └── migrate.rs  # SQL migration runner (schema versioning)
│   │   │   │
│   │   │   ├── embed/          # === VECTOR OPERATIONS ===
│   │   │   │   ├── mod.rs
│   │   │   │   ├── cache.rs    # EmbedCache: LruCache<u64, Vec<f32>> with xxhash
│   │   │   │   ├── model.rs    # TextEmbedding wrapper (fastembed-rs)
│   │   │   │   └── quantize.rs # f32↔f16↔i8 conversion
│   │   │   │
│   │   │   ├── graph/          # === ENGRAM SYSTEM ===
│   │   │   │   ├── mod.rs
│   │   │   │   ├── engram_graph.rs   # petgraph DiGraph + priority BFS
│   │   │   │   └── engram_builder.rs # formation + split + centroid EMA
│   │   │   │
│   │   │   ├── engine/         # === CORE ENCODE/RECALL PIPELINE ===
│   │   │   │   ├── mod.rs
│   │   │   │   ├── encode.rs         # full encode pipeline (attention → embed → insert → cluster)
│   │   │   │   ├── recall.rs         # 3-tier retrieval (Tier1 → Tier2 → Tier3)
│   │   │   │   ├── on_recall.rs      # LTP + stability + labile + engram resonance
│   │   │   │   ├── scoring.rs        # unified score function + context re-rank
│   │   │   │   └── working_memory.rs # 7-slot buffer with attention eviction
│   │   │   │
│   │   │   ├── lifecycle/      # === CONSOLIDATION + FORGETTING + SYNTHESIS ===
│   │   │   │   ├── mod.rs
│   │   │   │   ├── consolidation.rs   # NREM + REM + homeostasis cycles
│   │   │   │   ├── reconsolidation.rs # labile state management + pending updates
│   │   │   │   ├── forgetting.rs      # interference + predictive + capacity pruning
│   │   │   │   ├── dream.rs           # F1: offline synthesis (idle cross-link)
│   │   │   │   └── compression.rs     # F17: schema compression (cross-engram patterns)
│   │   │   │
│   │   │   ├── knowledge/      # === BELIEF + CAUSALITY + SKILLS ===
│   │   │   │   ├── mod.rs
│   │   │   │   ├── conflict.rs    # F2: contradiction detection + belief versioning
│   │   │   │   ├── confidence.rs  # F7: confidence intervals + corroboration
│   │   │   │   ├── causal.rs      # F11: causal chain tracking + invalidation cascade
│   │   │   │   └── skill.rs       # F8: TF-IDF skill extraction from episodic clusters
│   │   │   │
│   │   │   ├── temporal/       # === TIME + EMOTION AWARENESS ===
│   │   │   │   ├── mod.rs
│   │   │   │   ├── landmark.rs    # F5: temporal landmarks + era management
│   │   │   │   ├── snapshot.rs    # F12: named snapshots + time-travel recall
│   │   │   │   └── emotional.rs   # F18: mood tracking + congruent retrieval boost
│   │   │   │
│   │   │   ├── intake/         # === PASSIVE INGESTION + QUERY ROUTING ===
│   │   │   │   ├── mod.rs
│   │   │   │   ├── observe.rs     # F6: stdin/file/directory observation + topic segmentation
│   │   │   │   └── intent.rs      # F20: query intent classification + auto-routing
│   │   │   │
│   │   │   ├── sharing/        # === MULTI-AGENT + NAMESPACES ===
│   │   │   │   ├── mod.rs
│   │   │   │   ├── namespace.rs   # F9: namespace_id + agent_id + visibility filters
│   │   │   │   └── fork.rs        # F15: fork + merge brain states
│   │   │   │
│   │   │   ├── observability/  # === DIAGNOSTICS + AUDIT ===
│   │   │   │   ├── mod.rs
│   │   │   │   ├── audit.rs       # F19: write-ahead mutation log (append-only, capped)
│   │   │   │   ├── heatmap.rs     # F13: recall_log + hot_path_cache + prewarm
│   │   │   │   ├── predictive.rs  # F16: recall sequence learning + speculative pre-load
│   │   │   │   ├── health.rs      # F10: BrainHealthReport + ASCII dashboard
│   │   │   │   └── diff.rs        # F14: semantic diff between ticks/snapshots
│   │   │   │
│   │   │   └── policy/         # === GOVERNANCE + RETENTION ===
│   │   │       ├── mod.rs
│   │   │       ├── retention.rs   # retention classes (volatile/normal/durable/pinned)
│   │   │       └── governance.rs  # namespace ACL, policy checks, audit events
│   │   │
│   │   ├── benches/
│   │   │   ├── encode_bench.rs   # criterion: encode throughput
│   │   │   ├── recall_bench.rs   # criterion: recall latency by tier
│   │   │   └── hnsw_bench.rs     # criterion: HNSW vs brute-force at scale
│   │   │
│   │   └── tests/
│   │       ├── integration/
│   │       │   ├── test_encode_recall.rs    # M1: basic round-trip
│   │       │   ├── test_decay.rs            # M1: lazy Ebbinghaus
│   │       │   ├── test_ltp_ltd.rs          # M3: on_recall + stability
│   │       │   ├── test_consolidation.rs    # M6: NREM + REM + homeostasis
│   │       │   ├── test_reconsolidation.rs  # M5: labile window + update
│   │       │   ├── test_interference.rs     # M2: proactive + retroactive
│   │       │   ├── test_engrams.rs          # M7: formation + BFS + split
│   │       │   ├── test_working_memory.rs   # M2: 7-slot eviction
│   │       │   ├── test_tier3_cold.rs       # M4: cold store + mmap
│   │       │   ├── test_beliefs.rs          # F2+F7: contradiction + confidence
│   │       │   ├── test_dream.rs            # F1: idle synthesis
│   │       │   ├── test_observation.rs      # F6: topic segmentation
│   │       │   └── test_sharing.rs          # F9: namespace + visibility
│   │       └── unit/
│   │           ├── test_lazy_decay.rs
│   │           ├── test_scoring.rs
│   │           ├── test_quantize.rs
│   │           ├── test_confidence.rs       # F7: confidence update rules
│   │           ├── test_intent.rs           # F20: pattern matching accuracy
│   │           └── test_causal.rs           # F11: cascade penalty formula
│   │
│   └── membrain-cli/            # CLI binary + daemon + MCP server
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs           # clap CLI entry point + subcommand dispatch
│           │
│           ├── cmd/              # CLI commands — grouped by domain
│           │   ├── mod.rs        # re-exports all submodules
│           │   ├── memory.rs     # remember, recall, forget, strengthen, update, inspect, show
│           │   ├── query.rs      # ask (F20), budget (F4), recall-patterns
│           │   ├── knowledge.rs  # beliefs (F2), why (F11), invalidate, skills (F8), schemas (F17)
│           │   ├── observe.rs    # observe (F6) — pipe + watch
│           │   ├── lifecycle.rs  # consolidate, compress (F17), dream (F1)
│           │   ├── temporal.rs   # timeline (F5), landmark, snapshot (F12), diff (F14), mood (F18)
│           │   ├── sharing.rs    # namespace (F9), share, unshare, fork (F15), merge
│           │   ├── diagnostics.rs # health (F10), stats, doctor, audit (F19),
│           │   │                  # hot-paths (F13), dead-zones, uncertain (F7)
│           │   ├── data.rs       # export, import, list
│           │   └── system.rs     # daemon, mcp, config, benchmark
│           │
│           ├── mcp/              # MCP tools — grouped by domain (mirrors cmd/)
│           │   ├── mod.rs
│           │   ├── server.rs      # rmcp stdio server + dispatch router
│           │   ├── memory.rs      # memory_put, memory_get, memory_search, memory_recall,
│           │   │                   # memory_link, memory_pin, memory_forget
│           │   ├── knowledge.rs   # belief_history, why, invalidate, skills, extract_skills
│           │   ├── intake.rs      # ask (F20), observe (F6), context_budget (F4)
│           │   ├── lifecycle.rs   # consolidate, dream (F1), compress (F17), schemas
│           │   ├── temporal.rs    # timeline (F5), snapshot (F12), list_snapshots, diff (F14),
│           │   │                   # mood_history (F18)
│           │   ├── sharing.rs     # share (F9), fork (F15), merge_fork
│           │   └── diagnostics.rs # health (F10), inspect, explain, repair, audit (F19),
│           │                       # hot_paths (F13), dead_zones, uncertain (F7)
│           │
│           ├── daemon/
│           │   ├── mod.rs
│           │   ├── server.rs     # tokio Unix socket server + JSON-RPC 2.0
│           │   ├── handler.rs    # JSON-RPC method dispatch → brain_store
│           │   └── lifecycle.rs  # start/stop/status daemon management
│           │
│           ├── ipc/
│           │   ├── mod.rs
│           │   ├── client.rs     # Unix socket client (CLI → daemon)
│           │   └── protocol.rs   # JSON-RPC message types
│           │
│           └── output/
│               ├── mod.rs
│               ├── text.rs       # ANSI colored text + dashboard rendering
│               └── json.rs       # JSON serialization for --json flag
│
├── clients/
│   ├── python/
│   │   ├── membrain.py           # Python client (stdlib only, zero deps)
│   │   ├── __init__.py
│   │   └── README.md
│   └── node/
│       ├── membrain.js           # Node.js client (stdlib only, zero deps)
│       ├── package.json
│       └── README.md
│
├── docs/                         # all documentation
│   ├── PLAN.md                   # canonical mega-plan
│   ├── INDEX.md                  # doc pointer
│   ├── CLI.md                    # CLI command reference
│   ├── MCP_API.md                # MCP tool contract
│   ├── MEMORY_MODEL.md           # memory types, fields, lifecycle
│   ├── NEURO_MAPPING.md          # brain → code mapping
│   ├── OPERATIONS.md             # production runbooks
│   └── CONTRIBUTING.md           # contributor workflow
│
├── install.sh                   # production-grade curl-pipe installer (root level)
│
└── scripts/
    ├── bench.sh                 # run all benchmarks
    └── test-all.sh              # run full test suite with coverage
```

### 11.3 Core Module Responsibilities

```
brain_store.rs — THE ORCHESTRATOR
  Holds all sub-components. Provides the primary API:
  encode(), recall(), on_recall(), consolidation_cycle(), dream_cycle(), etc.
  Manages shared state (interaction_tick, primed_contexts).
  Routes between standalone and daemon modes.
  All other modules are called through BrainStore — no cross-module imports.

store/ — STORAGE LAYER
  hot.rs:    SQLite WAL + HNSW hot_index (float16, in-RAM). 3-table vertical partition.
  cold.rs:   SQLite + HNSW cold_index (int8, mmap). Unlimited disk scale.
  archive.rs: Read-only access to archived/soft-deleted memories.
  migrate.rs: Schema version tracking + migration runner.

embed/ — VECTOR OPERATIONS
  cache.rs:    LruCache<u64, Vec<f32>> with xxhash64 keys. Second embed of same content is free.
  model.rs:    TextEmbedding wrapper (fastembed-rs). Loads model once, shared across threads.
  quantize.rs: f32↔f16↔i8 conversion. Hot uses f16, cold uses i8.

graph/ — ENGRAM SYSTEM
  engram_graph.rs:   petgraph DiGraph, O(1) UUID→NodeIndex lookup, priority BFS with caps.
  engram_builder.rs: try_cluster(), split_engram(k=2), centroid EMA update.

engine/ — CORE ENCODE/RECALL (hot path — must stay bounded)
  encode.rs:         attention_gate → embed → novelty → initial_strength → insert → cluster → interference.
  recall.rs:         tier1 → prefilter → tier2_hnsw → rescore → engram_expand → tier3_if_needed → on_recall.
  on_recall.rs:      LTP + stability_increment + labile_state + engram_resonance + cache_update.
  scoring.rs:        score_candidate() — all ranking signals in one place for tuning.
  working_memory.rs: 7-slot VecDeque, attention-weighted eviction → encode to hot_store.

lifecycle/ — BACKGROUND PROCESSING (runs in daemon, never on hot path)
  consolidation.rs:   NREM (migrate hot→cold) + REM (desensitize + cross-link) + homeostasis.
  reconsolidation.rs: Labile window tracking, pending_update merge, re-embed.
  forgetting.rs:      Retroactive/proactive interference, predictive pruning, capacity management.
  dream.rs:           F1 — idle scan for high-sim but unlinked memories → create dream_links.
  compression.rs:     F17 — TF-IDF across episodic clusters → synthesize Schema memories.

knowledge/ — BELIEF + CAUSALITY + SKILLS (reasoning about what the brain knows)
  conflict.rs:   F2 — detect contradiction on encode, create belief_conflicts, supersede old.
  confidence.rs: F7 — confidence update on reconsolidate/corroborate/conflict. Filter by min_confidence.
  causal.rs:     F11 — link_causal(), trace_causality() BFS, invalidate_causal_chain() cascade.
  skill.rs:      F8 — evaluate mature engrams, TF-IDF keyword extraction → Procedural memory.

temporal/ — TIME + EMOTION (when/how memories relate to timeline and mood)
  landmark.rs:  F5 — auto-detect high-arousal+novelty → landmark, open/close eras, era filtering.
  snapshot.rs:  F12 — create_snapshot() (zero-copy metadata), recall_at_snapshot(), time-travel.
  emotional.rs: F18 — mood timeline, MoodSnapshot, mood-congruent retrieval boost.

intake/ — PASSIVE INGESTION + QUERY ROUTING (getting data in and queries out)
  observe.rs: F6 — stdin/file/directory observation, topic shift detection, auto-encode chunks.
  intent.rs:  F20 — classify query intent from keywords → auto-route to optimal RecallQuery config.

sharing/ — MULTI-AGENT (collaboration between agents)
  namespace.rs: F9 — namespace_id + agent_id + visibility (private/shared/public) filters on all queries.
  fork.rs:      F15 — fork brain state (inherit by reference), merge with conflict strategy.

observability/ — DIAGNOSTICS + AUDIT (understanding what the brain is doing)
  audit.rs:      F19 — append-only mutation log, capped at 200k rows. Every encode/recall/archive logged.
  heatmap.rs:    F13 — recall_log + hot_path_cache + dead_zones + Tier1 prewarm on daemon start.
  predictive.rs: F16 — recall sequence A→B learning, speculative Tier1 pre-load. Branch prediction for memory.
  health.rs:     F10 — BrainHealthReport struct, ASCII dashboard, --watch mode.
  diff.rs:       F14 — tick-range diff across all categories (new/strengthened/archived/conflicts/engrams).

policy/ — GOVERNANCE + RETENTION (rules the brain must follow)
  retention.rs:  Retention classes (volatile/normal/durable/pinned), lease policies, freshness checks.
  governance.rs: Namespace ACL, agent ACL, session visibility, policy precedence, audit events.
```

### 11.4 CLI + MCP Domain Grouping

```
cmd/ and mcp/ mirror each other by domain — same 8 groups, same naming.
CLI handler is always: parse args → call brain_store method → format output.
MCP handler is always: validate params → call brain_store method → serialize JSON.

Domain          │ CLI (cmd/)         │ MCP (mcp/)          │ Core module(s)
────────────────┼────────────────────┼─────────────────────┼──────────────────
Memory ops      │ memory.rs          │ memory.rs           │ engine/
Queries         │ query.rs           │ intake.rs           │ intake/, engine/
Knowledge       │ knowledge.rs       │ knowledge.rs        │ knowledge/
Observation     │ observe.rs         │ (in intake.rs)      │ intake/
Lifecycle       │ lifecycle.rs       │ lifecycle.rs        │ lifecycle/
Temporal        │ temporal.rs        │ temporal.rs         │ temporal/
Sharing         │ sharing.rs         │ sharing.rs          │ sharing/
Diagnostics     │ diagnostics.rs     │ diagnostics.rs      │ observability/
Data I/O        │ data.rs            │ —                   │ store/
System          │ system.rs          │ —                   │ daemon/

Adding a new feature:
  1. Create core logic in the right domain module (e.g. knowledge/new_feature.rs)
  2. Add BrainStore method in brain_store.rs
  3. Add CLI handler in the matching cmd/ domain file
  4. Add MCP handler in the matching mcp/ domain file
  5. Add integration test in tests/integration/
```

### 11.5 Boundary Rules

```
1. engine/ is HOT PATH — no unbounded work, no background jobs, no I/O beyond store
2. lifecycle/ is BACKGROUND — runs in daemon tokio tasks, never called from recall/encode
3. policy/ is CHECKED FIRST — namespace/ACL validated before store or engine is touched
4. knowledge/ is CALLED FROM engine/ — conflict detection runs inside encode, not standalone
5. observability/ is NON-BLOCKING — audit.log() is sync single-INSERT; heatmap/predictive are async
6. CLI calls brain_store only — never imports engine/ or store/ directly
7. store/ never decides product semantics — it persists what brain_store tells it to
8. graph/ is optional in retrieval — if budget exhausted, skip BFS expansion
9. Each domain module exposes a trait or struct — brain_store composes them, no cross-domain imports
```

### 11.6 AGENTS.md Template

```markdown
# membrain — AGENTS.md
# Context for AI coding assistants (Claude Code, Cursor, etc.)

## Project
membrain: Rust memory system porting human brain mechanisms to AI agents.
Stack: Rust + Tokio + SQLite WAL + FTS5 + USearch + fastembed + local reranker + petgraph + rmcp

## Workspace Layout
- crates/membrain-core/  — all brain logic (no CLI/daemon concerns)
- crates/membrain-cli/   — CLI + daemon + MCP server
- clients/python/        — Python Unix socket client
- clients/node/          — Node.js Unix socket client

## Key Architecture Decisions
- 3-tier storage: Tier1 LruCache (<0.1ms) → Tier2 HNSW hot (<5ms) → Tier3 mmap cold (<50ms)
- Lazy decay: effective_strength computed on access (never iterated eagerly)
- Vertical SQL partition: memory_index (scan) / memory_content / memory_vectors (fetch)
- Engrams: petgraph DiGraph with centroid HNSW for O(log E) cluster routing
- No daemons in tests: BrainStore::open_temp() creates isolated tempdir DB

## Critical Constants (in crates/membrain-core/src/constants.rs)
- HOT_CAPACITY: 50_000
- LTP_DELTA: 0.1
- STABILITY_INCREMENT: 0.2
- MIN_STRENGTH: 0.05
- EMOTIONAL_WEIGHT: 0.5
- AROUSAL_THRESHOLD: 0.6
- PRE_FILTER_LIMIT: 5_000
- ENGRAM_FORMATION_THRESHOLD: 0.65
- RESONANCE_FACTOR: 0.3

## Test Strategy
- Unit tests: deterministic, no fastembed (use fixed vectors)
  - test_lazy_decay.rs: verify effective_strength formula correctness
  - test_scoring.rs: verify score function with fixed inputs
- Integration tests: use tempdir DBs + real fastembed
  - test_encode_recall.rs: encode → recall round trip
  - test_decay.rs: encode → tick forward → verify decay
- Benchmarks: criterion, requires --release build
  cargo bench --bench recall_bench

## Running
  cargo build --release  (or with RUSTFLAGS="-C target-cpu=native" for SIMD)
  ./target/release/membrain daemon start
  ./target/release/membrain remember "test content"
  ./target/release/membrain recall "test"

## Common Gotchas
- usearch ID: uses u64 (first 64 bits of Uuid), not full Uuid
- float16 in hot HNSW, int8 in cold HNSW — always quantize before add/search
- effective_strength() in SQL: must use (? - last_tick) not (NOW() - last_tick)
  because SQLite doesn't know about interaction_tick
- WAL: hot.db and cold.db are separate files — don't open same Connection across threads
  (use rusqlite connection pool or per-task connections)
- petgraph NodeIndex is not stable across serialization — always use Uuid as primary key
```

### 11.7 GitHub Actions CI/CD + curl-pipe Installer

Three drop-in files implement the full distribution pipeline:

```
membrain/
├── .github/workflows/
│   ├── ci.yml        ← fmt + clippy + test, 3-OS matrix
│   └── release.yml   ← cross-compile 5 targets on vX.Y.Z tag
└── install.sh        ← production-grade curl-pipe installer
```

#### 11.7.1 ci.yml — Format + Clippy + Test (3-OS matrix)

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, dev]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    name: Check (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2

      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets --all-features -- -D warnings
      - run: cargo test --workspace --all-features
```

#### 11.7.2 release.yml — Cross-compile 5 targets + GitHub Release

Trigger: push tag matching `vX.Y.Z`.

Build matrix (5 targets):

| Target | Runner | Method | Asset Suffix |
|---|---|---|---|
| `x86_64-unknown-linux-musl` | ubuntu-latest | `cross` | `linux-x86_64` |
| `aarch64-unknown-linux-musl` | ubuntu-latest | `cross` | `linux-aarch64` |
| `x86_64-apple-darwin` | macos-latest | native | `macos-x86_64` |
| `aarch64-apple-darwin` | macos-latest | native | `macos-aarch64` |
| `x86_64-pc-windows-msvc` | windows-latest | native | `windows-x86_64` |

Two jobs:
1. **build** — parallel matrix → `.tar.gz` (Unix) or `.zip` (Windows) per target + `.sha256` sidecar
2. **release** — download all artifacts → attach to GitHub Release via `softprops/action-gh-release@v2`

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ["v[0-9]+.[0-9]+.[0-9]+*"]

permissions:
  contents: write

env:
  CARGO_TERM_COLOR: always
  BIN_NAME: membrain

jobs:
  build:
    name: Build ${{ matrix.suffix }}
    runs-on: ${{ matrix.runner }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-unknown-linux-musl
            runner: ubuntu-latest
            suffix: linux-x86_64
            use_cross: true
          - target: aarch64-unknown-linux-musl
            runner: ubuntu-latest
            suffix: linux-aarch64
            use_cross: true
          - target: x86_64-apple-darwin
            runner: macos-latest
            suffix: macos-x86_64
            use_cross: false
          - target: aarch64-apple-darwin
            runner: macos-latest
            suffix: macos-aarch64
            use_cross: false
          - target: x86_64-pc-windows-msvc
            runner: windows-latest
            suffix: windows-x86_64
            use_cross: false
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cross
        if: matrix.use_cross
        run: cargo install cross --git https://github.com/cross-rs/cross

      - uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.target }}

      - name: Build release binary
        run: |
          ${{ matrix.use_cross && 'cross' || 'cargo' }} build \
            --release --locked --target ${{ matrix.target }} \
            -p membrain-cli
        shell: bash

      - name: Package (Unix)
        if: runner.os != 'Windows'
        run: |
          TAG="${GITHUB_REF_NAME}"
          ARCHIVE="${BIN_NAME}-${TAG}-${{ matrix.suffix }}.tar.gz"
          cd target/${{ matrix.target }}/release
          tar -czf "../../../${ARCHIVE}" "${BIN_NAME}"
          cd ../../..
          sha256sum "${ARCHIVE}" > "${ARCHIVE}.sha256"
        shell: bash

      - name: Package (Windows)
        if: runner.os == 'Windows'
        run: |
          $TAG = $env:GITHUB_REF_NAME
          $ARCHIVE = "${env:BIN_NAME}-${TAG}-${{ matrix.suffix }}.zip"
          Compress-Archive -Path "target/${{ matrix.target }}/release/${env:BIN_NAME}.exe" -DestinationPath $ARCHIVE
          $hash = (Get-FileHash -Algorithm SHA256 $ARCHIVE).Hash.ToLower()
          "$hash  $ARCHIVE" | Out-File -Encoding ascii "${ARCHIVE}.sha256"
        shell: pwsh

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: dist-${{ matrix.suffix }}
          path: |
            ${{ env.BIN_NAME }}-*.tar.gz
            ${{ env.BIN_NAME }}-*.zip
            ${{ env.BIN_NAME }}-*.sha256

  release:
    name: GitHub Release
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: dist
          pattern: dist-*
          merge-multiple: true

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          generate_release_notes: true
          files: dist/*
```

Key design decisions:
- **musl for Linux**: produces fully static binaries — no glibc dependency on user machines
- **cross for Linux ARM64**: builds aarch64 from ubuntu-latest (x86_64 runner) via QEMU
- **.sha256 sidecar per archive**: `install.sh` verifies checksum after download
- **`softprops/action-gh-release@v2`**: not v1 — v2 supports `generate_release_notes`
- **`permissions: contents: write`**: required to create releases from GitHub Actions

#### 11.7.3 install.sh — Production-Grade curl-pipe Installer

Modeled after `beads_rust` by Dicklesworthstone. Root-level file (`install.sh`, not `scripts/install.sh`).

Install command for README:
```bash
curl -fsSL "https://raw.githubusercontent.com/quangdang46/membrain/main/install.sh?$(date +%s)" | bash
```

Features:
- `set -euo pipefail` + `umask 022` safety header
- Platform detection: `uname -s` / `uname -m` → asset suffix mapping
- Version resolution: GitHub API → redirect fallback → die
- Download with retry + resume (`--continue-at -`) + proxy support
- `.sha256` checksum verification (sidecar from release.yml)
- Atomic binary installation (`install -m 0755` + `mv`)
- Concurrent install locking (`mkdir` lock dir)
- `--easy-mode` auto-PATH update in `~/.bashrc` / `~/.zshrc`
- `--from-source` fallback via `cargo build --release`
- `--uninstall` cleanup (binary + PATH lines)
- `--verify` post-install self-test (`membrain --version`)
- curl|bash buffering safety wrapper at script end

Configuration block:
```bash
BINARY_NAME="membrain"
OWNER="quangdang46"
REPO="membrain"
DEST="${DEST:-$HOME/.local/bin}"
VERSION="${VERSION:-}"
QUIET=0; EASY=0; VERIFY=0; FROM_SOURCE=0; UNINSTALL=0
MAX_RETRIES=3; DOWNLOAD_TIMEOUT=120
```

Flags:
| Flag | Effect |
|---|---|
| `--dest <path>` | Install to custom directory |
| `--version <tag>` | Pin to specific release (e.g. `v0.3.0`) |
| `--system` | Install to `/usr/local/bin` (requires sudo) |
| `--easy-mode` | Auto-append `export PATH` to shell rc files |
| `--verify` | Run `membrain --version` after install |
| `--from-source` | Skip binary download, build from source via cargo |
| `--quiet` / `-q` | Suppress info logs |
| `--uninstall` | Remove binary + PATH lines from rc files |

Full script skeleton (all production patterns):

```bash
#!/usr/bin/env bash
set -euo pipefail
umask 022

# === Config ===
BINARY_NAME="membrain"
OWNER="quangdang46"
REPO="membrain"
DEST="${DEST:-$HOME/.local/bin}"
VERSION="${VERSION:-}"
QUIET=0; EASY=0; VERIFY=0; FROM_SOURCE=0; UNINSTALL=0
MAX_RETRIES=3; DOWNLOAD_TIMEOUT=120
LOCK_DIR="/tmp/${BINARY_NAME}-install.lock.d"
TMP=""

# === Logging ===
log_info()    { [ "$QUIET" -eq 1 ] && return; echo "[${BINARY_NAME}] $*" >&2; }
log_warn()    { echo "[${BINARY_NAME}] WARN: $*" >&2; }
log_success() { [ "$QUIET" -eq 1 ] && return; echo "✓ $*" >&2; }
die()         { echo "ERROR: $*" >&2; exit 1; }

# === Cleanup & lock ===
cleanup() { rm -rf "$TMP" "$LOCK_DIR" 2>/dev/null || true; }
trap cleanup EXIT
acquire_lock() {
    mkdir "$LOCK_DIR" 2>/dev/null || die "Another install running. rm -rf $LOCK_DIR"
    echo $$ > "$LOCK_DIR/pid"
}

# === Args ===
while [ $# -gt 0 ]; do
    case "$1" in
        --dest)       DEST="$2";   shift 2;;
        --dest=*)     DEST="${1#*=}"; shift;;
        --version)    VERSION="$2"; shift 2;;
        --version=*)  VERSION="${1#*=}"; shift;;
        --system)     DEST="/usr/local/bin"; shift;;
        --easy-mode)  EASY=1;      shift;;
        --verify)     VERIFY=1;    shift;;
        --from-source) FROM_SOURCE=1; shift;;
        --quiet|-q)   QUIET=1;     shift;;
        --uninstall)  UNINSTALL=1; shift;;
        *) shift;;
    esac
done

# === Uninstall ===
if [ "$UNINSTALL" -eq 1 ]; then
    rm -f "$DEST/$BINARY_NAME"
    for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
        [ -f "$rc" ] && sed -i "/${BINARY_NAME} installer/d" "$rc" 2>/dev/null || true
    done
    echo "✓ ${BINARY_NAME} uninstalled"; exit 0
fi

# === Platform ===
detect_platform() {
    local os arch
    case "$(uname -s)" in
        Linux*)  os="linux";;   Darwin*) os="darwin";;
        MINGW*|MSYS*|CYGWIN*) os="windows";;
        *) die "Unsupported OS";;
    esac
    case "$(uname -m)" in
        x86_64|amd64)  arch="x86_64";;
        aarch64|arm64) arch="aarch64";;
        *) die "Unsupported arch";;
    esac
    echo "${os}_${arch}"
}

# === Version ===
resolve_version() {
    [ -n "$VERSION" ] && return 0
    VERSION=$(curl -fsSL --connect-timeout 10 --max-time 30 \
        "https://api.github.com/repos/${OWNER}/${REPO}/releases/latest" 2>/dev/null \
        | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/') || true
    if ! [[ "$VERSION" =~ ^v[0-9] ]]; then
        VERSION=$(curl -fsSL -o /dev/null -w '%{url_effective}' \
            "https://github.com/${OWNER}/${REPO}/releases/latest" 2>/dev/null \
            | sed -E 's|.*/tag/||') || true
    fi
    [[ "$VERSION" =~ ^v[0-9] ]] || die "Could not resolve version"
    log_info "Latest: $VERSION"
}

# === Download ===
download_file() {
    local url="$1" dest="$2" partial="${2}.part" attempt=0
    while [ $attempt -lt $MAX_RETRIES ]; do
        attempt=$((attempt + 1))
        curl -fL --connect-timeout 30 --max-time "$DOWNLOAD_TIMEOUT" \
             -sS --retry 2 \
             $( [ -s "$partial" ] && echo "--continue-at -") \
             -o "$partial" "$url" \
          && mv -f "$partial" "$dest" && return 0
        [ $attempt -lt $MAX_RETRIES ] && { log_warn "Retry $attempt..."; sleep 3; }
    done
    return 1
}

# === Atomic install ===
install_binary_atomic() {
    local tmp="${2}.tmp.$$"
    install -m 0755 "$1" "$tmp" && mv -f "$tmp" "$2" || { rm -f "$tmp"; die "Install failed"; }
}

# === PATH ===
maybe_add_path() {
    case ":$PATH:" in *":$DEST:"*) return 0;; esac
    if [ "$EASY" -eq 1 ]; then
        for rc in "$HOME/.zshrc" "$HOME/.bashrc"; do
            [ -f "$rc" ] && [ -w "$rc" ] || continue
            grep -qF "$DEST" "$rc" && continue
            printf '\nexport PATH="%s:$PATH"  # %s installer\n' "$DEST" "$BINARY_NAME" >> "$rc"
        done
    fi
    log_warn "Restart shell or: export PATH=\"$DEST:\$PATH\""
}

# === Source build ===
build_from_source() {
    command -v cargo >/dev/null || die "cargo not found — install Rust: https://rustup.rs"
    git clone --depth 1 "https://github.com/${OWNER}/${REPO}.git" "$TMP/src"
    (cd "$TMP/src" && CARGO_TARGET_DIR="$TMP/target" cargo build --release)
    install_binary_atomic "$TMP/target/release/$BINARY_NAME" "$DEST/$BINARY_NAME"
}

# === Main ===
main() {
    acquire_lock
    TMP=$(mktemp -d)
    mkdir -p "$DEST"

    local platform; platform=$(detect_platform)
    log_info "Platform: $platform | Dest: $DEST"

    if [ "$FROM_SOURCE" -eq 0 ]; then
        resolve_version
        local ext="tar.gz"; [[ "$platform" == windows* ]] && ext="zip"
        # Map platform to asset suffix used by release.yml
        local suffix
        case "$platform" in
            linux_x86_64)   suffix="linux-x86_64";;
            linux_aarch64)  suffix="linux-aarch64";;
            darwin_x86_64)  suffix="macos-x86_64";;
            darwin_aarch64) suffix="macos-aarch64";;
            windows_x86_64) suffix="windows-x86_64";;
            *) die "No prebuilt binary for $platform";;
        esac
        local archive="${BINARY_NAME}-${VERSION}-${suffix}.${ext}"
        local url="https://github.com/${OWNER}/${REPO}/releases/download/${VERSION}/${archive}"

        if download_file "$url" "$TMP/$archive"; then
            # Verify checksum if sidecar exists
            if download_file "${url}.sha256" "$TMP/checksum.sha256" 2>/dev/null; then
                local expected actual
                expected=$(awk '{print $1}' "$TMP/checksum.sha256")
                actual=$(sha256sum "$TMP/$archive" 2>/dev/null | awk '{print $1}' \
                      || shasum -a 256 "$TMP/$archive" | awk '{print $1}')
                [ "$expected" = "$actual" ] || die "Checksum mismatch"
                log_info "Checksum verified"
            fi
            # Extract
            case "$archive" in
                *.tar.gz) tar -xzf "$TMP/$archive" -C "$TMP";;
                *.zip)    unzip -q "$TMP/$archive" -d "$TMP";;
            esac
            local bin; bin=$(find "$TMP" -name "$BINARY_NAME" -type f -perm -111 \
                          2>/dev/null | head -1)
            [ -n "$bin" ] || die "Binary not found after extract"
            install_binary_atomic "$bin" "$DEST/$BINARY_NAME"
        else
            log_warn "Binary download failed — building from source..."
            build_from_source
        fi
    else
        build_from_source
    fi

    maybe_add_path

    [ "$VERIFY" -eq 1 ] && "$DEST/$BINARY_NAME" --version

    echo ""
    echo "✓ $BINARY_NAME installed → $DEST/$BINARY_NAME"
    echo "  $("$DEST/$BINARY_NAME" --version 2>/dev/null || true)"
    echo ""
    echo "  Usage: $BINARY_NAME --help"
}

# curl|bash safety: buffer entire script before executing
if [[ "${BASH_SOURCE[0]:-}" == "${0:-}" ]] || [[ -z "${BASH_SOURCE[0]:-}" ]]; then
    { main "$@"; }
fi
```

#### 11.7.4 First Release

After all three files are committed:
```bash
git tag v0.1.0 && git push origin main --tags
```

This triggers `release.yml` → builds 5 targets → creates GitHub Release with archives + checksums → `install.sh` can resolve `v0.1.0` automatically.

---

### End of Snapshot Part 5

**Next: Part 6 — Implementation Milestones, Acceptance Checklist, Constants, Algorithm Reference**

Parts list:
- Part 1: Vision, Problem Statement, Human Brain Deep Dive ✅
- Part 2: Gap Analysis + Full Port (mechanism → Rust code) ✅
- Part 3: Architecture Overview + Performance ✅
- Part 4: Techstack + Data Schema ✅
- Part 5: CLI/MCP + Feature Extensions + Workspace Structure ✅
- Part 6: Milestones + Acceptance Checklist + Constants + Algorithm Reference


<!-- SOURCE: PLAN_part6.md -->

### Source Snapshot — Part 6
#### Part 6 of 6: Implementation Milestones · Acceptance Checklist · Constants · Algorithm Reference

---

## 12. Implementation Milestones

Each milestone is independently shippable. Later milestones build on earlier ones.
Each milestone includes: goal, deliverables, key files, test coverage, and acceptance criteria.

---

### Milestone 1 — Foundation + Lazy Decay

```
GOAL:
  Working binary that can encode and recall memories.
  Lazy Ebbinghaus decay implemented and verified.
  3-table vertical partition schema in place.
  usearch HNSW hot index functional.
  fastembed-rs embedding with LruCache.
  
DELIVERABLES:
  - Workspace setup (Cargo.toml, crates/membrain-core, crates/membrain-cli)
  - hot.db schema: memory_index + memory_content + memory_vectors
  - brain_state singleton row
  - interaction_tick: Arc<AtomicU64>
  - effective_strength() lazy formula: base × e^(-Δtick/stability)
  - EmbedCache: LruCache<u64, Vec<f32>> with xxhash64 keys
  - HotStore: rusqlite connection + WAL PRAGMA
  - usearch HNSW hot_index (float16, in-memory)
  - Basic encode(): attention_gate → embed → insert → hnsw_add
  - Basic recall(): embed → sql_prefilter → hnsw_search → sort → return
  - Tier1 LruCache (512 entries, basic)
  - `membrain remember` and `membrain recall` CLI commands (no daemon)
  - `membrain stats` (basic counts)
  - Config file parsing (config.toml)

KEY FILES:
  crates/membrain-core/src/
    constants.rs       — all tunable constants (HOT_CAPACITY, LTP_DELTA, etc.)
    types.rs           — MemoryIndex, MemoryKind, EmotionalTag, MemoryState
    config.rs          — Config struct + toml loading
    brain_store.rs     — BrainStore struct + open() + tick()
    store/hot.rs       — HotStore: DB + HNSW + PRAGMA setup
    embed/cache.rs     — EmbedCache
    embed/model.rs     — TextEmbedding wrapper
    engine/encode.rs   — encode() stub (attention → embed → insert → hnsw)
    engine/recall.rs   — recall() stub (embed → prefilter → hnsw → sort)
    scoring.rs         — effective_strength() + basic score_candidate()
    quantize.rs        — f32↔f16 conversion
  crates/membrain-cli/src/
    main.rs            — clap CLI entry point
    cli/remember.rs
    cli/recall.rs
    cli/stats.rs

TESTS:
  tests/unit/test_lazy_decay.rs:
    - verify effective_strength(base=1.0, stability=100, elapsed=0) == 1.0
    - verify effective_strength(base=1.0, stability=100, elapsed=100) ≈ 0.368
    - verify bypass_decay=true: effective_strength always returns base_strength
    - verify elapsed=∞: returns ≈ 0.0
    - verify elapsed < 0 (saturating_sub): returns base_strength unchanged
    
  tests/unit/test_scoring.rs:
    - verify score with fixed float32 vectors
    - verify context_weight = 0.3 applied correctly
    
  tests/integration/test_encode_recall.rs:
    - encode one memory → recall it → assert top result matches

ACCEPTANCE CRITERIA:
  ✅ `membrain remember "test content"` returns valid UUID
  ✅ `membrain recall "test"` returns the stored memory
  ✅ effective_strength(memory, tick=0) == initial_strength
  ✅ effective_strength(memory, tick=1000) < initial_strength (decay happened)
  ✅ bypass_decay memories return full base_strength regardless of tick
  ✅ embedding cache: second embed of same content is instant (xxhash hit)
  ✅ WAL mode confirmed (PRAGMA journal_mode returns 'wal')
  ✅ usearch index: add 100 vectors → search top-5 → results in <5ms
  ✅ SQLite vertical partition: prefilter query touches only memory_index
```

---

### Milestone 2 — Full Encode Pipeline

```
GOAL:
  Complete biological encoding pipeline.
  Novelty scoring, emotional tagging, initial strength calculation.
  Working memory (7-slot buffer).
  Source tagging.
  Interference (proactive + retroactive) on encode.

DELIVERABLES:
  - novelty_score = 1.0 - max_cosine_sim(new_vec, top_1_neighbor)
  - duplicate detection (novelty < DUPLICATE_THRESHOLD → update existing)
  - initial_strength formula: BASE × novelty_mod × attention_mod × emotional_mod
  - EmotionalTag: strength_multiplier() + should_bypass_decay()
  - Working memory: WorkingMemory struct, 7 slots, VecDeque, attention scoring
  - Working memory eviction → hot_store encode if strong enough
  - ForgettingEngine: apply_retroactive() + apply_proactive() (on encode)
  - MemorySource enum + CLI --source flag
  - Engram formation stub: engram_builder.try_cluster() (without split)
  - hot.db: engrams table + engram_edges table schema
  - `membrain remember --valence --arousal --attention --context` flags

KEY FILES:
  engine/encode.rs        — full pipeline
  engine/working_memory.rs — WorkingMemory impl
  engine/forgetting.rs    — retroactive + proactive interference
  graph/engram_builder.rs — try_cluster() (formation only, no split yet)

TESTS:
  tests/integration/test_encode_pipeline.rs:
    - high attention → higher initial strength than low attention
    - high emotional arousal → bypass_decay = true
    - high similarity to existing → duplicate detection → update existing
    - two similar memories: second causes retroactive penalty to first
    - working memory: add 8 memories → first is evicted → encoded to hot_store
    
ACCEPTANCE CRITERIA:
  ✅ attention_score=0.0 → memory discarded (below ATTENTION_THRESHOLD)
  ✅ attention_score=1.0 → maximum initial strength
  ✅ emotional_arousal=0.9, emotional_valence=-0.9 → bypass_decay=true
  ✅ emotional_arousal=0.3, emotional_valence=0.3 → bypass_decay=false
  ✅ novelty < 0.05 → memory updates existing (not creates new)
  ✅ working memory full (7 items) → evicts lowest attention → encodes to hot
  ✅ encode memory B → similar memory A has lower base_strength (retroactive)
  ✅ encode memory B → B has higher retrieval_difficulty (proactive)
  ✅ new engram created for first memory in a cluster
  ✅ second similar memory joins existing engram (centroid updated)
```

---

### Milestone 3 — LTP/LTD + on_recall()

```
GOAL:
  on_recall() pipeline: LTP + stability increase + labile state + engram resonance.
  Decay persisted on recall (clock reset).
  Full recall pipeline using on_recall for every returned memory.

DELIVERABLES:
  - on_recall(): persist_decay() + LTP_DELTA + stability_increment + labile_state
  - reconsolidation_window() formula
  - engram resonance: spread partial LTP to depth-1 engram neighbors
  - access_count increment on recall
  - Tier1 cache update on every recall (cache recent results)
  - labile_memories table tracking
  - MemoryState::Labile { since_tick, window_ticks } persisted
  - SQL: effective_strength in WHERE clause (lazy pre-filter)
  - `membrain strengthen <ID>` CLI command

KEY FILES:
  engine/on_recall.rs    — complete on_recall() implementation
  engine/recall.rs       — on_recall() called for all returned memories
  graph/engram_graph.rs  — resonance spread to neighbors

TESTS:
  tests/integration/test_ltp_ltd.rs:
    - encode memory → tick forward 100 → recall → effective_strength increased vs pre-recall
    - recall increases stability: subsequent decay is slower
    - recalled memory → state = Labile
    - engram resonance: recall memory → engram neighbors gain small LTP boost
    - memory with bypass_decay: strength not reduced by decay formula
    
  tests/unit/test_lazy_decay.rs (extended):
    - encode memory → don't recall → effective_strength at t=100 < at t=0
    - encode memory → recall at t=50 → effective_strength at t=100 > un-recalled memory
    - stability doubles after ~3.8 recalls (exponential growth)

ACCEPTANCE CRITERIA:
  ✅ on_recall() increases base_strength by LTP_DELTA (bounded at MAX_STRENGTH)
  ✅ on_recall() increases stability by STABILITY_INCREMENT × stability
  ✅ on_recall() resets decay clock (last_accessed_tick = now_tick)
  ✅ on_recall() sets state = Labile with correct window
  ✅ after recall, memory decays slower (stability increased)
  ✅ engram neighbors receive resonance_ltp = LTP_DELTA × RESONANCE_FACTOR / n
  ✅ Tier1 cache updated with recalled memory
  ✅ access_count incremented on every recall
  ✅ `membrain strengthen <id>` applies identical effects to on_recall()
```

---

### Milestone 4 — 3-Tier Retrieval Engine

```
GOAL:
  Complete 3-tier retrieval: Tier1 → Tier2 → Tier3.
  Unified scoring function with context re-ranking.
  Engram BFS expansion.
  Tip-of-tongue mode.
  Adaptive ef_search.
  cold_store (cold.db + cold.usearch mmap) — read path.

DELIVERABLES:
  - ColdStore: cold.db schema + cold.usearch mmap index read path
  - Complete 3-tier pipeline (Tier1 fast return → Tier2 → Tier3)
  - adaptive_ef() function
  - Unified score_candidate() with all signals:
    semantic_sim + context_boost + strength + recency + difficulty_penalty + prime_boost + resonance
  - EngramGraph BFS expansion from top recall hits
  - MemoryFragment + tip_of_tongue mode (when max_score < PARTIAL_THRESHOLD)
  - RecallResult struct with tier_used, latency_us, engram_expanded fields
  - `membrain recall --confidence fast|normal|high`
  - `membrain recall --no-engram` flag

KEY FILES:
  store/cold.rs             — ColdStore read path
  engine/recall.rs          — complete 3-tier pipeline
  graph/engram_graph.rs     — bfs_neighbors() with priority queue
  scoring.rs                — unified score_candidate() (complete)
  crates/membrain-cli/src/cli/recall.rs — full CLI output

TESTS:
  tests/integration/test_recall_pipeline.rs:
    - encode 10 memories → recall → Tier1 hit for recently encoded
    - encode 600 memories (> Tier1 capacity) → recall → Tier2 hit
    - encode + consolidate to cold → recall → Tier3 hit
    - engram expansion: recall seed → also returns engram cluster members
    - tip-of-tongue: no close match → returns fragments not full memories
    - context boost: memory encoded with context A scores higher when recalled with context A
    - adaptive ef: high confidence → more ef → higher recall accuracy
    
ACCEPTANCE CRITERIA:
  ✅ Tier1 hit: latency < 0.1ms for 1000 consecutive identical queries
  ✅ Tier2 search: 50k vectors → top-5 result in < 5ms (release build, native CPU)
  ✅ Tier3 search: cold mmap index → top-5 result in < 50ms
  ✅ Engram expansion: top hit in engram → recall returns related cluster members
  ✅ Context re-ranking: memory encoded in context A scores 20%+ higher when recalled in context A vs context B
  ✅ Tip-of-tongue: when no memory above PARTIAL_THRESHOLD, returns fragments
  ✅ RecallResult.tier_used correctly reports which tier was used
  ✅ RecallResult.latency_us measured accurately
```

---

### Milestone 5 — Reconsolidation Engine

```
GOAL:
  Full reconsolidation lifecycle: Labile state management, pending updates,
  window expiry, re-embedding, strength bonus.

DELIVERABLES:
  - pending_updates table in hot.db
  - reconsolidation_tick() async task: check all Labile memories
  - apply_update(): re-embed + update hot.db + update HNSW + Stable/Labile
  - reconsolidation_window() formula (age-dependent, strength-dependent)
  - `membrain update <ID> <NEW_CONTENT>` CLI command
  - `membrain show <ID>` shows Labile state + window remaining
  - RECONSOLIDATION_BONUS applied on successful update

KEY FILES:
  engine/reconsolidation.rs — complete reconsolidation cycle
  hot.rs — pending_updates table + labile_memories queries
  cli/update.rs
  cli/show.rs (extended with Labile state display)

TESTS:
  tests/integration/test_reconsolidation.rs:
    - encode memory → recall → state = Labile
    - submit update during window → reconsolidation_tick → content updated
    - submit update after window expired → NOT applied (window closed)
    - very old memory → shorter reconsolidation window
    - successful update → base_strength += RECONSOLIDATION_BONUS
    - updated memory → HNSW index reflects new embedding
    - updated memory → Tier1 cache invalidated

ACCEPTANCE CRITERIA:
  ✅ Fresh memory (age=0): reconsolidation window = BASE_WINDOW ticks
  ✅ Old memory (age >> BASE_WINDOW): reconsolidation window ≈ 0-5 ticks
  ✅ Update submitted during window → applied at next reconsolidation_tick
  ✅ Update submitted after window expired → discarded (not applied)
  ✅ Applied update → memory content changed in DB
  ✅ Applied update → HNSW search now returns updated embedding
  ✅ Applied update → base_strength += RECONSOLIDATION_BONUS
  ✅ `membrain update` with no prior recall → rejected (not Labile)
  ✅ `membrain update --force` → forces Labile state before update
```

---

### Milestone 6 — Consolidation Engine (NREM + REM + Homeostasis)

```
GOAL:
  Complete consolidation cycle running as async background task.
  ColdStore write path.
  Pressure-triggered consolidation.
  REM emotional processing.
  Homeostasis scaling.

DELIVERABLES:
  - ColdStore: cold.db write path + cold.usearch mmap write
  - nrem_cycle(): score hot → migrate top-N to cold → update engrams
  - rem_cycle(): desensitize emotional memories + cross-link
  - homeostasis_cycle(): global scale + prune below MIN_STRENGTH
  - Tokio background task: consolidation_loop (woken by channel)
  - Pressure signal: when hot_index.len() > 90% HOT_CAPACITY → send to channel
  - ConsolidationReport struct
  - `membrain consolidate` CLI command (manual trigger)
  - `membrain consolidate --dry-run`

KEY FILES:
  store/cold.rs          — write path: consolidate_from_hot()
  engine/consolidation.rs — nrem_cycle + rem_cycle + homeostasis_cycle
  daemon/server.rs       — background task spawn

TESTS:
  tests/integration/test_consolidation.rs:
    - encode HOT_CAPACITY + 100 memories → nrem_cycle → cold_count > 0
    - migrated memories: cold.db has record, hot.db memory_index state=Consolidated
    - migrated memories: hot_index.len() reduced
    - migrated memories: cold_index can find them
    - rem_cycle: emotional memory arousal reduced by DESENSITIZATION_FACTOR
    - rem_cycle: after enough cycles, emotional_processed = true, bypass_decay = false
    - homeostasis: if total_load > trigger → all base_strengths × 0.9
    - homeostasis: memories below MIN_STRENGTH after scaling → archived
    - dry-run: no changes to DB

ACCEPTANCE CRITERIA:
  ✅ nrem_cycle migrates memories from hot to cold without data loss
  ✅ migrated memory: content retrievable from cold.db (decompressed correctly)
  ✅ migrated memory: cold HNSW returns it for semantic query
  ✅ hot_index.len() reduced after nrem_cycle (HNSW entries removed)
  ✅ rem_cycle reduces arousal by DESENSITIZATION_FACTOR each cycle
  ✅ rem_cycle sets emotional_processed=true when arousal < PROCESSED_THRESHOLD
  ✅ rem_cycle sets bypass_decay=false when processed
  ✅ homeostasis scales ALL hot memories by HOMEOSTASIS_FACTOR
  ✅ background consolidation task: runs without blocking encode/recall
  ✅ WAL: consolidation writes to cold.db don't block reads from hot.db
```

---

### Milestone 7 — Engram Graph (Formation + BFS + Split)

```
GOAL:
  Full engram system: formation, BFS traversal, split on overflow,
  centroid HNSW index, resonance, serialization/deserialization.

DELIVERABLES:
  - EngramGraph: petgraph DiGraph + node_index HashMap
  - EngramBuilder: centroid HNSW (usearch, float16, unlimited)
  - try_cluster(): formation threshold → join or create
  - update_centroid(): EMA α=0.1 on new member
  - split_engram(): K-means (k=2) when member_count > SOFT_LIMIT
  - bfs_neighbors(): priority-weighted BFS with depth/node limits
  - engram_graph serialization: bincode → brain_state blob in hot.db
  - graph load on daemon startup from hot.db
  - edge activation_count++ on BFS traversal
  - `membrain show <ID> --related` shows BFS results
  - Engram stats in `membrain stats` output

KEY FILES:
  graph/engram_graph.rs    — complete with bfs_neighbors
  graph/engram_builder.rs  — complete with split_engram

TESTS:
  tests/integration/test_engrams.rs:
    - encode 5 similar memories → all in same engram (centroid formed)
    - encode 1 dissimilar memory → new engram created
    - encode SOFT_LIMIT + 1 similar memories → engram splits into 2
    - bfs: seed → returns up to max_nodes depth-first expansion
    - centroid: mean of all member embeddings (verified numerically)
    - serialization: serialize → deserialize → same graph structure
    - edge activation_count: BFS traversal increments count
    - engram centroid HNSW: find nearest engram by centroid similarity

ACCEPTANCE CRITERIA:
  ✅ 5 similar memories → same engram_id in memory_index
  ✅ 5 dissimilar memories → 5 different engrams
  ✅ Engram centroid ≈ mean of member embeddings (< 0.01 cosine distance)
  ✅ SOFT_LIMIT + 1 members → split → 2 child engrams with parent_engram_id set
  ✅ BFS max_nodes=50 enforced (never returns more than 50 nodes)
  ✅ BFS max_depth=3 enforced
  ✅ BFS priority: high-similarity edges traversed before low-similarity edges
  ✅ engram graph survives daemon restart (serialized to hot.db)
  ✅ recall returns engram members alongside HNSW top hits
```

---

### Milestone 8 — Active Forgetting + Interference Engine

```
GOAL:
  Complete ForgettingEngine: all four components operational.
  Predictive pruning batch pass.
  Capacity management.
  `membrain watch` command.
  `membrain list --decaying`.

DELIVERABLES:
  - ForgettingEngine: apply_retroactive + apply_proactive (M2 already done)
  - predictive_pruning_pass(): batch accelerated decay for non-predictive memories
  - capacity_management(): archive weakest when > SOFT_CAP
  - Background forgetting task (low priority tokio task, periodic)
  - is_decaying_soon() fn exposed in MemoryIndex
  - decaying_soon: bool field in ScoredMemory
  - `membrain watch` command (continuous decaying-soon monitor)
  - `membrain list --decaying` filter
  - ArchiveReason enum + archive.db write path
  - `membrain archive restore <ID>` (reads archive.db → re-encodes to hot)

KEY FILES:
  engine/forgetting.rs     — complete ForgettingEngine
  store/hot.rs             — archive_memory() implementation
  store/archive.rs         — ArchiveStore (archive.db)
  cli/watch.rs
  cli/list.rs (extended with --decaying filter)

TESTS:
  tests/integration/test_forgetting.rs:
    - predictive prune: encode old never-recalled memory → prune pass → archived
    - capacity management: encode > SOFT_CAP → weakest memories archived
    - retroactive interference: encode B → A (similar) has lower strength
    - proactive interference: encode B → B has higher retrieval_difficulty
    - archive restore: archived memory → archive restore → back in hot_store
    - watch: decaying memories surface before reaching MIN_STRENGTH
    - is_decaying_soon: returns true when strength < 2 × MIN_STRENGTH

ACCEPTANCE CRITERIA:
  ✅ Predictive prune: memory with access_count=0 and age=MINIMUM_PRUNE_AGE → archived
  ✅ Capacity management: encode 200k memories → weakest archived to stay ≤ SOFT_CAP
  ✅ Retroactive interference: old memory base_strength reduced after encoding similar new
  ✅ Proactive: new memory retrieval_difficulty > 0 when similar old memories exist
  ✅ archive.db: archived memories stored with reason and tick
  ✅ archive restore: memory re-encoded into hot_store with original content
  ✅ `membrain watch` prints memories when effective_strength < 2 × MIN_STRENGTH
  ✅ `membrain list --decaying` returns only decaying memories
```

---

### Milestone 9 — Daemon + IPC + MCP Server

```
GOAL:
  Full daemon mode with Unix socket JSON-RPC 2.0 server.
  MCP stdio server via rmcp.
  Python + Node clients functional.
  CLI transparently forwards to daemon if running.

DELIVERABLES:
  - tokio Unix socket server: accept → spawn per-connection task → handle JSON-RPC
  - JSON-RPC 2.0 dispatcher: method → brain_store call → response
  - All methods: remember, recall, forget, strengthen, update, stats,
                 consolidate, prime, remind, watch, export, import
  - Daemon lifecycle: start (daemonize) + stop (SIGTERM graceful) + status
  - PID file + socket file management
  - CLI: detect daemon socket → forward vs standalone fallback
  - rmcp MCP server: stdio transport with all tools defined
  - Python client: membrain.py (zero deps, socket + json)
  - Node client: membrain.js (zero deps, net + readline)
  - `membrain daemon start|stop|status` CLI subcommand
  - `membrain mcp` CLI subcommand (starts stdio MCP server)

KEY FILES:
  crates/membrain-cli/src/
    daemon/server.rs    — tokio Unix socket server
    daemon/handler.rs   — JSON-RPC dispatch
    daemon/lifecycle.rs — start/stop/status
    mcp/server.rs       — rmcp stdio server
    ipc/client.rs       — CLI → daemon forwarding
  clients/python/membrain.py
  clients/node/membrain.js

TESTS:
  tests/integration/test_daemon.rs:
    - start daemon → socket exists at expected path
    - Python client: remember → recall round trip via socket
    - Node client: remember → recall round trip via socket
    - Concurrent clients: 10 simultaneous connections → all served correctly
    - Daemon stop: SIGTERM → graceful shutdown → socket removed
    - CLI fallback: no daemon → standalone mode → same results
    - MCP: spawn `membrain mcp` → send tool call JSON → receive result
    
ACCEPTANCE CRITERIA:
  ✅ `membrain daemon start` → daemon running, PID file written
  ✅ `membrain daemon status` → shows PID, uptime, memory RSS
  ✅ `membrain daemon stop` → clean shutdown, PID + socket removed
  ✅ Python client: MembrainClient().remember()/recall() → correct results
  ✅ Node client: MembrainClient().remember/recall → correct results
  ✅ 10 concurrent Python client connections → all succeed without errors
  ✅ MCP tool `remember` → JSON response with valid UUID
  ✅ MCP tool `recall` → JSON array of memories
  ✅ CLI without daemon → falls back to standalone mode automatically
  ✅ Daemon warm: embedding model loaded once at startup
  ✅ recall latency with daemon: Tier1 < 0.5ms, Tier2 < 10ms (IPC overhead included)
```

---

### Milestone 10 — CLI Polish + Production Readiness

```
GOAL:
  All CLI commands complete and polished.
  Benchmark suite meeting all targets.
  Export/import working.
  Doctor command.
  Diff command.
  README complete.
  Release CI/CD pipeline.

DELIVERABLES:
  - All CLI commands: diff, export, import, doctor, config, archive, context-for
  - `--json` flag for every command
  - `membrain prime` + `membrain remind` (Features 3 and 5)
  - `membrain watch` (Feature 10)
  - `membrain diff --from --to` (Feature 6)
  - `membrain context-for "task"` (Feature 9)
  - Benchmark suite (criterion): encode, recall (Tier1/2/3), consolidation
  - All benchmark targets verified:
    Tier1 <0.1ms, Tier2 <5ms, Tier3 <50ms, encode <10ms
  - README.md: installation (curl-pipe command), usage, configuration, MCP setup
  - AGENTS.md: context for AI coding assistants
  - `.github/workflows/ci.yml`: fmt + clippy + test, 3-OS matrix (ubuntu, macos, windows)
  - `.github/workflows/release.yml`: cross-compile 5 targets on vX.Y.Z tag,
    .tar.gz/.zip archives with .sha256 sidecars, GitHub Release via softprops/action-gh-release@v2
  - `install.sh` (root level): production-grade curl-pipe installer with retry, checksum
    verification, atomic install, --easy-mode PATH, --from-source fallback, --uninstall

KEY FILES:
  crates/membrain-cli/src/cli/ — all remaining command handlers
  benches/ — complete benchmark suite
  README.md
  AGENTS.md
  .github/workflows/ci.yml
  .github/workflows/release.yml
  install.sh

ACCEPTANCE CRITERIA (all must pass):
  ✅ All 17 CLI commands functional and return correct JSON with --json flag
  ✅ Benchmark: encode with cache hit < 1ms (p99)
  ✅ Benchmark: encode with cache miss < 10ms (p99)
  ✅ Benchmark: recall Tier1 < 0.1ms (p99)
  ✅ Benchmark: recall Tier2 at 50k memories < 5ms (p99)
  ✅ Benchmark: recall Tier3 at 500k cold memories < 50ms (p99)
  ✅ Export: dumps all memories as valid NDJSON
  ✅ Import: reads NDJSON → re-encodes → memories retrievable
  ✅ Doctor: detects DB corruption, index inconsistency, config errors
  ✅ Diff: shows correct added/forgotten/decayed counts
  ✅ context-for: returns token-budget-aware prompt prefix
  ✅ Release binary: single statically-linked binary < 50MB (musl, fully static on Linux)
  ✅ CI: green on push across 3 OS (ubuntu, macos, windows) — fmt + clippy + test
  ✅ Release: 5-target matrix builds on vX.Y.Z tag push:
      linux-x86_64 (cross/musl), linux-aarch64 (cross/musl),
      macos-x86_64, macos-aarch64, windows-x86_64
  ✅ Release: each archive has .sha256 sidecar, GitHub Release auto-created
  ✅ install.sh: `curl -fsSL ... | bash` works on Linux x86_64, Linux aarch64,
      macOS x86_64, macOS aarch64; verifies sha256; falls back to source build
  ✅ install.sh: --uninstall removes binary + PATH lines from rc files
```

---

## 13. Acceptance Checklist

Complete project acceptance: all items must pass before v1.0.0 tag.

### 13.1 Biological Mechanism Coverage

```
MECHANISM                   IMPLEMENTED   TESTED   VERIFIED
─────────────────────────────────────────────────────────
LTP (on_recall strength++)      ☐           ☐        ☐
LTD (Ebbinghaus lazy decay)     ☐           ☐        ☐
Emotional bypass_decay          ☐           ☐        ☐
REM desensitization             ☐           ☐        ☐
NREM migration (hot→cold)       ☐           ☐        ☐
Synaptic homeostasis            ☐           ☐        ☐
Engram formation                ☐           ☐        ☐
Engram BFS expansion            ☐           ☐        ☐
Engram split (k-means)          ☐           ☐        ☐
Engram resonance                ☐           ☐        ☐
Reconsolidation (labile)        ☐           ☐        ☐
Reconsolidation (update)        ☐           ☐        ☐
Active forgetting (predictive)  ☐           ☐        ☐
Retroactive interference        ☐           ☐        ☐
Proactive interference          ☐           ☐        ☐
Pattern completion (BFS)        ☐           ☐        ☐
Working memory 7-slot           ☐           ☐        ☐
Attention gating                ☐           ☐        ☐
Encoding specificity (context)  ☐           ☐        ☐
Novelty detection               ☐           ☐        ☐
Duplicate detection             ☐           ☐        ☐
Tip-of-tongue mode              ☐           ☐        ☐
Prospective triggers            ☐           ☐        ☐
Spotlight priming               ☐           ☐        ☐
```

### 13.2 Performance Benchmarks

```
BENCHMARK                           TARGET    MEASURED   PASS
──────────────────────────────────────────────────────────
Tier1 recall (cache hit)            <0.1ms      ___ms     ☐
Tier2 recall (50k hot)              <5ms        ___ms     ☐
Tier3 recall (500k cold)            <50ms       ___ms     ☐
Encode (embed cache hit)            <1ms        ___ms     ☐
Encode (embed cache miss)           <10ms       ___ms     ☐
SQL pre-filter (50k memories)       <0.5ms      ___ms     ☐
Engram BFS (depth=3, max=50)        <1ms        ___ms     ☐
Consolidation (1k migration)        non-block   pass/fail  ☐
Decay idle overhead                 0ms         ___ms     ☐
Embed cache hit rate (steady state) >80%        ___%      ☐
Tier1 hit rate (steady state)       >60%        ___%      ☐
```

### 13.3 Correctness

```
CORRECTNESS ITEM                               PASS
──────────────────────────────────────────────────
effective_strength formula matches Ebbinghaus   ☐
bypass_decay ignores elapsed tick entirely      ☐
LTP bounded at MAX_STRENGTH                     ☐
stability bounded at MAX_STABILITY              ☐
reconsolidation window correct for age=0        ☐
reconsolidation window shorter for old memories ☐
update discarded after window expiry            ☐
engram centroid = mean of member embeddings     ☐
engram split produces 2 children                ☐
BFS never returns > max_nodes results           ☐
SQL effective_strength matches Rust formula     ☐
float32 rescore improves on int8/f16 ranking    ☐
context weight = 0.3 applied consistently       ☐
```

### 13.4 API Completeness

```
CLI COMMAND             WORKS   --JSON   DAEMON   STANDALONE
──────────────────────────────────────────────────────────
membrain remember         ☐       ☐        ☐         ☐
membrain recall           ☐       ☐        ☐         ☐
membrain forget           ☐       ☐        ☐         ☐
membrain strengthen       ☐       ☐        ☐         ☐
membrain update           ☐       ☐        ☐         ☐
membrain stats            ☐       ☐        ☐         ☐
membrain list             ☐       ☐        ☐         ☐
membrain show             ☐       ☐        ☐         ☐
membrain diff             ☐       ☐        ☐         ☐
membrain consolidate      ☐       ☐        ☐         ☐
membrain prime            ☐       ☐        ☐         ☐
membrain remind           ☐       ☐        ☐         ☐
membrain watch            ☐       ☐        ☐         ☐
membrain export           ☐       ☐        ☐         ☐
membrain import           ☐       ☐        ☐         ☐
membrain daemon start     ☐       ☐        n/a       n/a
membrain daemon stop      ☐       ☐        n/a       n/a
membrain daemon status    ☐       ☐        n/a       n/a
membrain mcp              ☐       n/a      ☐         ☐
membrain doctor           ☐       ☐        ☐         ☐
membrain context-for      ☐       ☐        ☐         ☐

MCP TOOL                WORKS   CORRECT OUTPUT
──────────────────────────────────────────────
remember                  ☐       ☐
recall                    ☐       ☐
forget                    ☐       ☐
strengthen                ☐       ☐
stats                     ☐       ☐
consolidate               ☐       ☐
prime                     ☐       ☐
remind                    ☐       ☐
```

---

## 14. Tunable Constants

All constants live in `crates/membrain-core/src/constants.rs`.
All are overridable via `~/.membrain/config.toml`.

```rust
// crates/membrain-core/src/constants.rs

// ── Storage ──────────────────────────────────────────────────────────
/// Maximum memories in hot HNSW index (usearch float16, in-memory)
pub const HOT_CAPACITY: usize = 50_000;

/// Tier1 LruCache entries
pub const TIER1_CACHE_CAPACITY: usize = 512;

/// Embedding LruCache entries (~1.5MB at 384 dims × f32)
pub const EMBED_CACHE_CAPACITY: usize = 1_000;

/// Archive when total memory count exceeds this
pub const SOFT_CAP: usize = 1_000_000;

// ── LTP / LTD ─────────────────────────────────────────────────────────
/// Strength boost per recall (simulates AMPA receptor insertion)
pub const LTP_DELTA: f32 = 0.1;

/// Stability growth per recall (fraction of current stability)
pub const STABILITY_INCREMENT: f32 = 0.2;

/// Initial stability for new memories (Ebbinghaus S parameter)
pub const BASE_STABILITY: f32 = 50.0;

/// Maximum allowed stability (prevents infinite stability)
pub const MAX_STABILITY: f32 = 10_000.0;

/// Maximum allowed strength
pub const MAX_STRENGTH: f32 = 1.0;

/// Below this: memory archived (soft-deleted)
pub const MIN_STRENGTH: f32 = 0.05;

/// Initial base_strength for new memories
pub const BASE_STRENGTH: f32 = 0.5;

// ── Emotional Memory ──────────────────────────────────────────────────
/// How much emotion multiplies initial strength
/// Formula: 1.0 + (arousal × |valence| × EMOTIONAL_WEIGHT)
pub const EMOTIONAL_WEIGHT: f32 = 0.5;

/// Arousal above this: bypass_decay candidate
pub const AROUSAL_THRESHOLD: f32 = 0.6;

/// Absolute valence above this: bypass_decay candidate
pub const VALENCE_THRESHOLD: f32 = 0.5;

/// Arousal reduction per REM cycle (simulates NE suppression)
pub const DESENSITIZATION_FACTOR: f32 = 0.95;

/// Arousal below this: mark emotional_processed = true
pub const EMOTIONAL_PROCESSED_THRESHOLD: f32 = 0.3;

// ── Attention / Encoding ──────────────────────────────────────────────
/// Below this: memory discarded (not attended to)
pub const ATTENTION_THRESHOLD: f32 = 0.2;

/// Default attention if not specified
pub const DEFAULT_ATTENTION: f32 = 0.7;

/// How much attention modifies initial strength
pub const ATTENTION_WEIGHT: f32 = 0.4;

/// How much novelty modifies initial strength
pub const NOVELTY_WEIGHT: f32 = 0.3;

/// Below this novelty: update existing instead of creating new
pub const DUPLICATE_THRESHOLD: f32 = 0.05;

// ── Retrieval / Scoring ───────────────────────────────────────────────
/// Score weight for content similarity
pub const CONTENT_WEIGHT: f32 = 0.7;

/// Score weight for context similarity  
pub const CONTEXT_WEIGHT: f32 = 0.3;

/// Above this: Tier1 early return (confident cache hit)
pub const TIER1_CONFIDENCE_THRESHOLD: f32 = 0.90;

/// Above this: Tier2 early return (confident HNSW hit)
pub const TIER2_CONFIDENCE_THRESHOLD: f32 = 0.80;

/// Below this: tip-of-tongue mode (partial recall only)
pub const PARTIAL_RECALL_THRESHOLD: f32 = 0.40;

/// Decay warning at 2× MIN_STRENGTH
pub const DECAY_WARNING_FACTOR: f32 = 2.0;

/// SQL pre-filter: max candidates to fetch before HNSW search
pub const PRE_FILTER_LIMIT: usize = 5_000;

/// float32 rescore: how many HNSW candidates to rescore
pub const RESCORE_TOP_K: usize = 20;

// ── Engrams ───────────────────────────────────────────────────────────
/// Min cosine similarity to join existing engram
pub const ENGRAM_FORMATION_THRESHOLD: f32 = 0.65;

/// Member count: trigger k-means split
pub const ENGRAM_SOFT_LIMIT: usize = 200;

/// Member count: hard reject new members (create sibling instead)
pub const ENGRAM_HARD_LIMIT: usize = 500;

/// Exponential moving average alpha for centroid update
pub const ENGRAM_CENTROID_ALPHA: f32 = 0.10;

/// Fraction of LTP that spreads to engram neighbors
pub const RESONANCE_FACTOR: f32 = 0.30;

/// BFS maximum depth
pub const ENGRAM_BFS_MAX_DEPTH: usize = 3;

/// BFS maximum nodes collected
pub const ENGRAM_BFS_MAX_NODES: usize = 50;

/// Minimum edge similarity to traverse in BFS
pub const ENGRAM_BFS_MIN_EDGE: f32 = 0.50;

// ── Consolidation ─────────────────────────────────────────────────────
/// Minimum consolidation score to migrate to cold
pub const CONSOLIDATION_THRESHOLD: f32 = 0.4;

/// Fraction of hot memories to migrate per nrem_cycle
pub const MIGRATION_FRACTION: f32 = 0.2;

/// Global scaling factor during homeostasis
pub const HOMEOSTASIS_FACTOR: f32 = 0.9;

/// Total load fraction to trigger homeostasis
pub const HOMEOSTASIS_TRIGGER: f32 = 0.85;

/// Interactions between periodic consolidation cycles
pub const CONSOLIDATION_INTERVAL: u64 = 1_000;

// ── Reconsolidation ───────────────────────────────────────────────────
/// Reconsolidation window for freshly encoded memory (age=0)
pub const RECONSOLIDATION_BASE_WINDOW: u64 = 50;

/// Age at which window halves (inverse-linear formula)
pub const OLD_MEMORY_THRESHOLD: f32 = 500.0;

/// Below this strength: no reconsolidation window
pub const LABILE_STRENGTH_MIN: f32 = 0.2;

/// Strength bonus on successful reconsolidation update
pub const RECONSOLIDATION_BONUS: f32 = 0.05;

// ── Interference ──────────────────────────────────────────────────────
/// Min cosine similarity to trigger interference
pub const INTERFERENCE_SIM_MIN: f32 = 0.70;

/// Above this: duplicate (not interference candidate)
pub const INTERFERENCE_SIM_MAX: f32 = 0.99;

/// Strength reduction for old memories (retroactive)
pub const RETROACTIVE_PENALTY: f32 = 0.05;

/// Retrieval_difficulty increase for new memories (proactive)
pub const PROACTIVE_PENALTY: f32 = 0.05;

// ── Active Forgetting ─────────────────────────────────────────────────
/// access_count/age below this: accelerated decay
pub const PREDICTIVE_VALUE_THRESHOLD: f32 = 0.001;

/// Extra decay multiplier for non-predictive memories
pub const PREDICTIVE_DECAY_FACTOR: f32 = 0.85;

/// Don't predictively prune memories younger than this
pub const MINIMUM_PRUNE_AGE: u64 = 500;

/// Max memories scanned per forgetting engine pass
pub const PRUNE_BATCH_SIZE: usize = 10_000;

/// Interactions between forgetting engine passes
pub const FORGETTING_ENGINE_INTERVAL: u64 = 500;

/// Archive bottom fraction when total > SOFT_CAP
pub const ARCHIVE_FRACTION: f32 = 0.10;

// ── Prospective Triggers ─────────────────────────────────────────────
/// Default context match threshold to fire a trigger
pub const PROSPECTIVE_TRIGGER_THRESHOLD: f32 = 0.80;

/// Default priming boost factor
pub const PRIMING_BOOST_DEFAULT: f32 = 0.30;

/// Default priming expiry duration (interactions)
pub const PRIMING_DEFAULT_DURATION: u64 = 1_000;

// ── HNSW Configuration ────────────────────────────────────────────────
/// HNSW connectivity parameter M (hot index)
pub const HNSW_M_HOT: usize = 16;

/// HNSW connectivity parameter M (cold index — lower to save disk)
pub const HNSW_M_COLD: usize = 8;

/// HNSW build quality parameter
pub const HNSW_EF_CONSTRUCTION: usize = 200;

/// HNSW default search quality (adaptive, this is the base)
pub const HNSW_EF_DEFAULT: usize = 50;

/// Embedding dimension (must match model)
pub const EMBEDDING_DIMS: usize = 384;

// ── zstd Compression ─────────────────────────────────────────────────
/// Compression level for cold content (3=fast, good ratio)
pub const ZSTD_LEVEL_COLD: i32 = 3;

/// Compression level for archive content (9=slow, best ratio)
pub const ZSTD_LEVEL_ARCHIVE: i32 = 9;
```

---

## 15. Algorithm Reference

### 15.1 Lazy Ebbinghaus Decay

```
FORMULA:
  effective_strength(m, t) = m.base_strength × exp(-Δt / m.stability)
  
  Where:
    Δt = now_tick - m.last_accessed_tick
    m.stability = initialized at BASE_STABILITY, grows with each recall
    m.base_strength = reset to effective_strength on each recall (clock resets)
  
PROPERTIES:
  - O(1) computation (no iteration)
  - Mathematically equivalent to eager per-tick update
  - Decay rate = -1/stability (larger stability → slower decay)
  - At Δt = stability: retention = e^(-1) ≈ 36.8% (Ebbinghaus landmark)
  
STABILITY GROWTH:
  on_recall(): stability += STABILITY_INCREMENT × stability
  Effect: stability grows exponentially with recall count
  After n recalls: stability ≈ BASE_STABILITY × (1 + STABILITY_INCREMENT)^n
  
FULL RECALL CYCLE:
  1. Read: eff = base × exp(-(now-last)/stability)
  2. LTP:  base = eff + LTP_DELTA (reset + boost)
           stability += STABILITY_INCREMENT × stability
           last_accessed_tick = now_tick
  3. Write: persist new base, stability, last_tick to DB
  
SQL EXPRESSION (for pre-filter):
  (base_strength * EXP(-(? - last_accessed_tick) / CAST(stability AS REAL)))
  Parameter: ? = now_tick (f64, avoid integer division)
```

### 15.2 Unified Scoring Function

```
FORMULA:
  score = (semantic_sim + context_boost)
          × effective_strength
          × recency_bias
          × difficulty_penalty
          + prime_boost
          + resonance

WHERE:
  semantic_sim = cosine_sim(query_vec, memory_content_vec)
  
  context_boost = CONTEXT_WEIGHT × cosine_sim(current_context_vec, memory_context_vec)
                = 0.3 × cosine_sim(...)
  
  effective_strength = base × exp(-Δt/stability) [see 15.1]
  
  recency_bias = 1.0 + 0.1 / (1.0 + ln(age).max(0))
    age = now_tick - memory.created_tick
    Effect: recent memories score 10% higher for age=0, approaches 1.0 for old
  
  difficulty_penalty = 1.0 - memory.retrieval_difficulty.min(0.5)
    retrieval_difficulty increased by proactive interference
    Max penalty: 50% score reduction
  
  prime_boost = max(active_primed_contexts
    .filter(|p| now_tick < p.expiry_tick)
    .map(|p| p.boost_factor × cosine_sim(context_vec, p.embedding)))
    Range: 0.0 to PRIMING_BOOST_DEFAULT (0.3)
  
  resonance = resonance_scores.get(memory_id).unwrap_or(0.0)
    Set by engram BFS: neighbors of strongly recalled memories
    Range: 0.0 to ~0.1 (small boost, emergent from engram structure)

RANGE: 0.0 to ~2.0 (in practice 0.0-1.5)
RANKING: sort DESC, top_k returned
```

### 15.3 Reconsolidation Window Formula

```
FORMULA:
  window(age, strength) = BASE_WINDOW × age_factor × strength_factor
  
  age_factor = 1.0 / (1.0 + age / (10 × BASE_WINDOW))
    age=0:               factor = 1.0       window = 50 ticks
    age=BASE_WINDOW:     factor = 0.5       window = 25 ticks
    age=10×BASE_WINDOW:  factor ≈ 0.09      window ≈ 4.5 ticks
    age=100×BASE_WINDOW: factor ≈ 0.0099    window ≈ 0.5 ticks → rounds to 0
  
  strength_factor = 0.5 + strength × 0.5
    strength=0.0:   factor = 0.5  (very weak → shorter window)
    strength=0.5:   factor = 0.75
    strength=1.0:   factor = 1.0  (very strong → full window)
  
  If effective_strength < LABILE_STRENGTH_MIN (0.2): window = 0 (no reconsolidation)
  
INTERPRETATION:
  Fresh, strong memory: full window = 50 ticks
  Old, weak memory: tiny window → effectively not reconsolidatable
  This matches: well-consolidated memories are hard to update (biological LTM)
```

### 15.4 Engram Centroid Update

```
FORMULA: Exponential Moving Average (EMA)
  new_centroid[i] = old_centroid[i] × (1 - α) + new_member_vec[i] × α
  α = ENGRAM_CENTROID_ALPHA (0.1)
  
PROPERTIES:
  - O(D) per update (D = dimensions = 384)
  - Recent members have more influence than old members
  - Does not require storing all member vectors
  - Centroid naturally drifts toward newer semantic content
  - After n updates: old_vec weight ≈ (1-α)^n (decays exponentially)
  
APPROXIMATE CENTROID vs TRUE CENTROID:
  EMA centroid ≠ exact mean of all members
  But: close enough for engram routing (O(log E) centroid HNSW search)
  True mean would require storing all member vectors → expensive
  EMA is O(1) storage, O(D) update → practical choice
  
CENTROID HNSW:
  Separate usearch index: one vector per engram (centroid)
  On new memory encode: search centroid index → find nearest engram
  If sim > ENGRAM_FORMATION_THRESHOLD: join that engram
  Else: create new engram, add its centroid to centroid index
```

### 15.5 Engram K-Means Split (k=2)

```
TRIGGER: engram.member_count > ENGRAM_SOFT_LIMIT (200)
ALGORITHM: Mini-batch k-means (k=2, 10 iterations, 50-sample batches)

STEPS:
  1. Collect all member embeddings (from memory_vectors table)
  2. Random seed selection: pick 2 random members as initial centroids
  3. Iterate 10 times:
     a. Assign each member to nearest centroid
     b. Recompute centroids as mean of assigned members
  4. Create child_a engram: members assigned to centroid_a
     Create child_b engram: members assigned to centroid_b
  5. Add child centroids to centroid HNSW
  6. Remove parent centroid from HNSW
  7. Update memory_index.engram_id for all affected memories

PERFORMANCE:
  200 members × 384 dims × 10 iterations × 2 centroids:
  = 200 × 384 × 10 × 2 = 1.5M float ops
  At 4 GFLOPS: ~0.4ms
  Acceptable: split is rare (only when engram exceeds 200 members)
  
POST-SPLIT:
  Original engram entry preserved in DB with split_tick set
  Two new child engrams created with parent_engram_id = original
  Engram hierarchy queryable for analysis/visualization
```

### 15.6 Adaptive ef_search Algorithm

```
INPUTS:
  query.confidence_requirement: ConfidenceLevel (FastApprox | Normal | High)
  hot_count: usize (current hot store size)
  tier1_hit_rate: f32 (rolling average of last 1000 queries)

ALGORITHM:
  // Base ef from confidence level
  base_ef = match confidence_level {
      FastApprox => 10,
      Normal     => 50,
      High       => 100,
  };
  
  // Size scaling: small stores need less ef (graph well-navigated)
  // At 1k memories: sqrt(0.02) ≈ 0.14 → tiny ef
  // At 50k memories: sqrt(1.0) = 1.0 → full ef
  size_factor = (hot_count as f32 / HOT_CAPACITY as f32).sqrt().min(1.0);
  
  // Cache factor: if Tier1 is serving most queries, Tier2 is the hard cases
  // Hard cases need more ef; if Tier1 is low, queries are routine
  cache_factor = if tier1_hit_rate > 0.7 { 0.8 } else { 1.0 };
  
  ef = (base_ef as f32 × size_factor × cache_factor) as usize;
  ef = ef.max(10).min(200);

INTUITION:
  - Small store: fewer nodes → graph easily navigated → low ef sufficient
  - Large store: more nodes → higher ef needed for accuracy
  - High Tier1 rate: Tier2 only handles "hard" non-cached queries → boost ef for accuracy
  - FastApprox: if you just want "something close, quickly" → ef=10
  
EXAMPLE VALUES:
  5k store, normal, 80% Tier1:   ef = 50 × 0.32 × 0.8 ≈ 12
  50k store, normal, 60% Tier1:  ef = 50 × 1.0 × 1.0 = 50
  50k store, high, 60% Tier1:    ef = 100 × 1.0 × 1.0 = 100
  50k store, fast, 90% Tier1:    ef = 10 × 1.0 × 0.8 = 8 → clamped to 10
```

### 15.7 Interference Detection Algorithm

```
RETROACTIVE INTERFERENCE (applied during encode of new memory B):

  1. Get embedding of new memory B: vec_B
  2. HNSW search: top-100 nearest to vec_B (excluding B itself)
  3. For each candidate A with similarity ∈ (SIM_MIN, SIM_MAX) = (0.70, 0.99):
     if A.created_tick < B.created_tick:  // A is older than B
         A.base_strength = effective_strength(A, now) × (1 - RETROACTIVE_PENALTY)
         // persist to DB
  
  RATIONALE:
    - 0.70 threshold: only sufficiently similar memories interfere
    - 0.99 threshold: near-duplicates handled differently (update, not interfere)
    - Temporal direction: new information interferes backward with old
    - Only old memories weakened (retroactive = backward in time)
    
PROACTIVE INTERFERENCE (applied during encode of new memory B):

  1. Same HNSW search as above
  2. Count memories A with similarity ∈ (SIM_MIN, SIM_MAX) = (0.70, 0.99)
  3. new_memory_B.retrieval_difficulty += count × PROACTIVE_PENALTY
  
  RATIONALE:
    - Many similar old memories → new memory harder to recall specifically
    - Retrieval_difficulty acts as divisor in scoring: harder to retrieve
    - Forward in time: old memories make new ones harder to recall (proactive = forward)

COMBINED EFFECT:
  When B is encoded in a crowded semantic neighborhood:
    A (old, similar): base_strength reduced (retroactive)
    B (new): retrieval_difficulty increased (proactive)
    
  Over time:
    A decays faster (already weakened + continued Ebbinghaus)
    B harder to recall but has novelty boost (balances out)
    Gradually: A fades, B becomes the dominant memory in that neighborhood
    = Natural disambiguation: newer information wins over time
```

### 15.8 Consolidation Scoring Algorithm

```
PURPOSE: Rank hot memories for NREM migration to cold store.
HIGH SCORE → migrated first; LOW SCORE → stays in hot.

FORMULA:
  consolidation_score = effective_strength(m, now)
                      × access_frequency_factor
                      × recency_weight
                      × emotional_bonus

WHERE:
  effective_strength(m, now):
    current strength (accounts for decay)
    strong memories → more stable → should be preserved in cold
  
  access_frequency_factor = ln(access_count + 1) + 1.0
    access_count=0:  factor = 1.0
    access_count=1:  factor = 1.69
    access_count=10: factor = 3.40
    access_count=100: factor = 5.61
    Logarithmic: prevents runaway for heavily accessed memories
    
  recency_weight = 1.0 / (1.0 + (now - last_accessed_tick) / 1000.0)
    last_tick=now:    weight = 1.0
    last_tick=now-500: weight = 0.67
    last_tick=now-1000: weight = 0.50
    Recently accessed → higher weight (brain prioritizes recent)
    
  emotional_bonus = 1.0 + emotional_arousal × 0.5
    neutral:  bonus = 1.0
    arousal=0.5: bonus = 1.25
    arousal=1.0: bonus = 1.5
    Emotional memories → migrate faster (brain consolidates emotional faster)

RESULT:
  Memories are ranked by this score.
  Top MIGRATION_FRACTION (20%) of hot_count are migrated.
  Those below CONSOLIDATION_THRESHOLD (0.4) are never migrated (too weak).
  
INTUITION:
  A memory that's STRONG, FREQUENTLY ACCESSED, RECENTLY RECALLED, and EMOTIONAL
  scores highest → it's "ripe" for consolidation → becomes long-term semantic knowledge.
  
  A memory that's WEAK, NEVER RECALLED, OLD, and NEUTRAL
  scores lowest → stays in hot tier until it decays to MIN_STRENGTH → archived.
  This is Tononi's insight: sleep consolidates what matters, discards noise.
```

---

### End of Snapshot Part 6 (Final Part)

---

### Snapshot Document Summary

This PLAN.md spans 6 parts covering:

```
Part 1: Vision, Problem Statement, Human Brain Deep Dive
  - Full analysis of all 16 brain mechanisms ported to membrain
  - All 7 major brain regions documented with membrain port details
  - Memory types taxonomy (episodic, semantic, procedural, emotional)
  - LTP/LTD molecular mechanisms → Rust implementation
  - Ebbinghaus forgetting curve → lazy decay formula
  - Consolidation phases → NREM/REM/Homeostasis
  - Sleep architecture → async background tasks
  - Reconsolidation discovery → Labile state management
  - Active forgetting → ForgettingEngine
  - Engrams → petgraph DiGraph
  - Working memory 7±2 → WorkingMemory + Tier1 LruCache
  - Emotional memory → EmotionalTag + bypass_decay
  - Interference → retroactive + proactive engines
  - Pattern completion → 3-tier HNSW + BFS
  - Encoding specificity → context_embedding

Part 2: Gap Analysis + Full Port
  - Feature matrix comparing 6 AI memory systems
  - Detailed gap analysis per system (MemGPT, Mem0, LangMem, etc.)
  - Complete mechanism-by-mechanism port with Rust code
  - 14 mechanisms fully implemented with function signatures

Part 3: Architecture + Performance
  - 3-tier storage diagram (Tier1/2/3 with latency targets)
  - Complete encode + retrieve data flow pseudocode
  - Daemon vs standalone process model
  - File layout (~/.membrain/)
  - 7 performance optimization layers
  - Benchmark targets + scale targets
  - Memory resonance algorithm
  - Prospective memory algorithm
  - Spotlight/priming algorithm

Part 4: Techstack + Data Schema
  - Full rationale for Rust, Tokio, usearch, fastembed-rs, petgraph
  - usearch HNSW configuration (hot/cold)
  - fastembed-rs model options + EmbedCache implementation
  - Complete Cargo.toml with all dependencies
  - hot.db schema (6 tables, all indexes)
  - cold.db schema (3 tables)
  - procedural.db schema
  - All Rust structs (20+ types)
  - config.toml with all defaults

Part 5: CLI + MCP + Features + Workspace
  - 17 CLI commands with full usage docs
  - 8 MCP tools with complete input/output schemas
  - JSON-RPC 2.0 wire format + Python/Node client code
  - Top 10 feature extensions with Rust pseudocode
  - Full workspace directory tree
  - AGENTS.md template
  - GitHub Actions CI/CD (ci.yml 3-OS matrix + release.yml 5-target cross-compile)
  - Production-grade curl-pipe installer (install.sh)

Part 6: Milestones + Checklist + Constants + Algorithms
  - 10 implementation milestones with acceptance criteria
  - Complete acceptance checklist (mechanism/perf/correctness/API)
  - All constants with formulas and rationale
  - 8 algorithm references with formulas and pseudocode
```

**Total: ~18,000+ lines across 6 parts.**
**Assemble: cat PLAN_part1.md PLAN_part2.md PLAN_part3.md PLAN_part4.md PLAN_part5.md PLAN_part6.md > PLAN.md**


---

# UPGRADE OVERLAYS

## 1. Research framing

The original plan is strongest when interpreted as a **functional translation** of neuroscience into systems design.
Keep this framing explicit:

- hippocampus ↔ hot episodic index
- neocortex ↔ deep semantic cold store
- amygdala ↔ emotional salience and retention bias
- prefrontal cortex ↔ working-memory and executive control
- engrams ↔ clustered associative recall units
- reconsolidation ↔ update-on-recall mutation window
- sleep / homeostasis ↔ background consolidation and pruning jobs

### Hard rule
Do not claim biological identity.
Claim **engineering correspondence** and **behavioral analogy** unless the mechanism is directly justified by measurement.

### Research claim classes
Every neuroscience-inspired claim should be tagged as one of:
- `analogy`
- `evidence-backed inspiration`
- `engineering hypothesis`
- `production assumption`
- `benchmark target`

### Recommended upgrade to wording
Replace “port the entire human brain memory mechanism” with:
“map the key functional mechanisms of human memory into a production-grade memory operating system for AI agents.”

That keeps your ambition while reducing overclaim risk.

## 2. Core design invariants

1. Foreground recall must remain bounded even if total memory grows by 100x.
2. Encode fast path must never depend on remote APIs.
3. No background job may block foreground retrieval beyond latency budget.
4. Every memory mutation must preserve provenance or emit an explicit loss event.
5. Contradictions must be represented, not silently overwritten.
6. Tier transitions must be auditable.
7. Cold payload fetch must occur only after candidate trimming.
8. Graph expansion must obey hard caps.
9. Standalone mode and daemon mode must preserve semantic equivalence for core APIs.
10. Benchmarks must be reproducible in release mode on declared hardware.

## 3. Non-negotiable restrictions

### Foreground path restrictions
- No LLM calls in encode, recall, on_recall, reconsolidation apply path, or forgetting eligibility path.
- No full-store O(n) scan in any request path.
- No decompression of cold payload before final candidate cut.
- No graph BFS without hard depth and node caps.
- No policy bypass in CLI, daemon, MCP, or IPC wrappers.

### Storage restrictions
- Tier1 must not own giant payloads.
- Tier2 must separate metadata from large content.
- Tier3 must remain recoverable after rebuild from durable records.
- Archive must be reversible by default.

### Research restrictions
- No benchmark claim without dataset cardinality, machine profile, and warm/cold declaration.
- No p95 claim from microbench-size sample counts unless labeled as exploratory.

## 4. Performance budget decomposition

### Encode fast path
Budget targets:
- cache lookup and hash: microseconds
- embedding cache hit: near zero
- cache miss embedding: bounded under target hardware profile
- novelty search: bounded by top-1 or top-k small search
- DB insert + HNSW add: bounded
- total p95 fast path: <10ms

### Tier1
Budget targets:
- exact lookup + score + return
- p95 <0.1ms
- p99 must remain close enough that tail does not invalidate fast-path narrative

### Tier2
Budget targets:
- metadata prefilter
- HNSW search
- float32 rescore
- optional engram expansion within hard budget
- p95 <5ms at declared hot cardinality

### Tier3
Budget targets:
- mmap probe
- sparse metadata fetch
- float32 rescore
- cold payload only for final selection
- p95 <50ms at declared cold cardinality

## 5. Benchmark contracts by stage

### Stage 1 — Foundation + Lazy Decay
Must pass:
- encode→recall roundtrip
- WAL verified
- effective_strength formula stable
- embedding cache measurable benefit
- hot prefilter touches metadata table only

### Stage 2 — Full Encode Pipeline
Must pass:
- attention gating
- novelty and duplicate routing
- emotional bypass rules
- working-memory deterministic eviction
- bounded interference updates

### Stage 3 — on_recall / LTP-LTD
Must pass:
- idempotent request-bounded on_recall
- stability growth monotonic
- recall overhead bounded
- labile transition durable through restart

### Stage 4 — 3-tier retrieval
Must pass:
- tier escalation determinism
- adaptive ef bounded
- context rerank measurable
- engram expansion within budget
- partial / tip-of-tongue path does not leak full payload incorrectly

### Stage 5 — Reconsolidation
Must pass:
- valid labile-window enforcement
- re-embed and reindex coherence
- cache invalidation correctness
- crash-safe update application

### Stage 6 — Consolidation
Must pass:
- NREM migration retrievable after move
- REM-like cross-linking auditable
- homeostasis never prunes pinned or authoritative evidence
- background work does not break foreground SLOs

### Stage 7 — Engram maturity
Must pass:
- centroid stability
- split and sibling creation rules
- BFS caps
- restart serialization integrity

### Stage 8 — Forgetting engine
Must pass:
- prune eligibility is policy-safe
- archive restore roundtrip
- overload convergence
- recall quality improves or remains stable after pruning

### Stage 9 — Daemon + IPC + MCP
Must pass:
- semantic parity with standalone mode
- socket lifecycle robustness
- concurrency safety
- IPC overhead bounded

### Stage 10 — Production readiness
Must pass:
- export/import roundtrip
- corruption detection via doctor
- reproducible benchmark suite
- rollback notes and repair playbooks

## 6. Go / no-go redesign triggers

Redesign instead of patching if any remain unresolved after one bounded redesign cycle:
- Tier2 p95 cannot remain under target at declared hot set size
- engram expansion tail latency cannot be capped
- reconsolidation leaves cache/index divergence
- forgetting still removes high-utility memories under realistic load
- daemon mode introduces correctness divergence not present in standalone mode

## 7. Suggested additions to the original milestone structure

For every milestone, add five explicit sections:
- Restrictions
- Benchmarks
- Regression budget
- Kill criteria
- Exit artifacts

### Exit artifacts example
For each stage completion:
- benchmark report
- failure matrix
- design note
- migration note if schema changed
- rollback note if behavior changed
- ops note if background jobs changed

## 8. Suggested benchmark tables to add directly into the plan

### Retrieval benchmark template
| Scenario | Corpus size | Warm/Cold | Concurrency | p50 | p95 | p99 | Pass? |
|---|---:|---|---:|---:|---:|---:|---|

### Encode benchmark template
| Scenario | Cache hit rate | Avg payload size | p50 | p95 | p99 | Pass? |
|---|---:|---:|---:|---:|---:|---|

### Consolidation benchmark template
| Job | Items moved | Foreground load | p95 foreground delta | Duration | Pass? |
|---|---:|---:|---:|---:|---|

### Forgetting benchmark template
| Prune class | Eligible set | False prune rate | Restore success | Recall quality delta | Pass? |
|---|---:|---:|---:|---:|---|

## 9. Quality gates

### Correctness
- no silent contradiction overwrite
- no lost committed memories
- no orphaned engram edges after mutation
- no stale cache after accepted update

### Utility
- retrieval precision remains acceptable on canonical corpora
- context reranking improves same-context retrieval
- forgetting reduces noise without destroying key facts
- consolidation improves utility, not just storage efficiency

### Operability
- doctor can detect seeded failure cases
- export/import works
- repair paths documented
- dashboard metrics exist for tier hit rates, cache hit rates, p95/p99

## 10. Recommended structure for the true mega-plan

Keep your current core sections intact, then append these new parts:

12. Stage Gates
13. Stage Restrictions
14. Benchmark Protocol
15. Performance Budgets
16. Go / No-Go Decision Rules
17. Quality Gates
18. Risk Register
19. Repair & Operations Acceptance
20. Research Notes / Falsifiable Claims

That gives you a mega-plan that still feels like your own plan, not a replacement.

---

## 12. Canonical Architecture Invariants

These invariants are elevated from the supporting design docs into non-negotiable system rules.

### 12.1 Request-path invariants

1. **Hot path must stay bounded**
   - No unbounded graph walks.
   - No full scans on request path.
   - No compaction, repair, or large migrations in foreground.
   - Every retrieval mode must have a hard candidate budget.

2. **Every memory item must have provenance**
   - Each item must retain source kind, source reference, timestamps, and lineage.
   - Any summary, consolidation artifact, or extracted fact must point back to source evidence.

3. **No silent overwrite of contradiction**
   - If new information conflicts with existing information, the system must represent the contradiction explicitly.
   - Ranking may prefer one side, but storage must preserve the disagreement.

4. **Tier routing decisions must be traceable**
   - Why an item entered Tier1, Tier2, or Tier3 must be inspectable after the fact.
   - Promotion and demotion must be auditable.

5. **Retrieval must be explainable**
   - Returned context must be explainable in terms of score components, source evidence, graph hops, and policy filters.
   - A result that cannot be explained is not production-grade.

6. **No hard delete without policy permission**
   - Hard deletion is for explicit policy / compliance / retention expiry paths only.
   - Utility-based forgetting must not masquerade as compliance deletion.

### 12.2 Background-job invariants

1. Background jobs must not violate latency budgets for online recall.
2. Repair jobs must preserve authoritative evidence.
3. Compaction jobs must preserve lineage or emit a precise irreversible-loss record.
4. Indexes and graph structures must be rebuildable from durable evidence.
5. Every destructive or semi-destructive maintenance task must emit before/after telemetry.

### 12.3 Governance invariants

1. Namespace isolation is checked before expensive retrieval work.
2. Workspace ACL, agent ACL, and session visibility apply equally to write, read, and background execution.
3. Policy precedence is deterministic and auditable.
4. Any policy violation must emit an incident-grade auditable event.

---

## 13. Canonical Memory Model Extension

The supporting docs define a compact but strong object model. This section makes that model normative for implementation.

### 13.1 Canonical memory taxonomy

The system must support at least these memory categories:

- **Event**: raw observed occurrence, tool call, message, action, or state change.
- **Episode**: grouped sequence of related events with temporal continuity.
- **Fact**: distilled proposition intended for repeated recall.
- **Relation**: link between entities, memories, goals, or concepts.
- **Summary**: compressed representation of lower-level evidence.
- **Goal**: active or historical objective shaping retrieval priority.
- **Skill**: reusable procedural knowledge extracted from repeated success.
- **Constraint**: rules, limits, or obligations that must remain visible.
- **Hypothesis**: tentative belief awaiting confirmation.
- **ConflictRecord**: explicit contradiction artifact.
- **PolicyArtifact**: retention/governance/compliance-relevant item.
- **Observation**: state observation or environmental signal.
- **ToolOutcome**: result of tool execution with operational value.
- **UserPreference**: stable user-specific preference or convention.
- **SessionMarker**: boundary and context marker for session-level grouping.

### 13.2 Required fields for every memory item

Every stored item should carry, directly or derivably, the following attributes:

- `id`
- `memory_type`
- `namespace`
- `workspace_id`
- `agent_id`
- `session_id`
- `task_id`
- `created_at`
- `updated_at`
- `source_kind`
- `source_ref`
- `authoritativeness`
- `content_ref`
- `compact_text`
- `fingerprint`
- `tier`
- `salience`
- `confidence`
- `utility_estimate`
- `recall_count`
- `last_access_at`
- `decay_state`
- `retention_class`
- `policy_flags`
- `lineage`
- `version`
- `tags`
- `entity_refs`
- `relation_refs`

### 13.3 Schema rules

1. **Version increments on accepted mutation**
2. **Lineage is preserved across summarize / merge / extract / repair operations**
3. **Payload and summary should be separable**
4. **Fingerprints are stable enough for duplicate-family handling**
5. **Policy flags travel with the memory, not with only one index layer**
6. **Tier location is state, not an inference**

### 13.4 Contradiction handling contract

When a new fact conflicts with an existing fact:
- do not overwrite the older fact silently;
- create or update a `ConflictRecord`;
- attach evidence references for both sides;
- let retrieval/ranking choose presentation order later;
- preserve enough metadata for audit and repair.

---

## 14. Lifecycle and State Transition Rules

### 14.1 Canonical lifecycle

A memory item may move through the following conceptual states:

`created -> indexed -> recalled -> reinforced -> decayed -> demoted -> archived -> deleted`

This is a logical model; some implementations may encode the state implicitly across fields.

### 14.2 Transition guards

Before any transition is committed, the system must validate:
- namespace access control
- policy pinning
- retention constraints
- legal hold state
- lineage preservation
- unresolved contradiction semantics
- job lock / repair lock safety

### 14.3 Failure behavior

If a transition fails mid-flight:
- preserve the last known valid state
- emit a transition error event
- enqueue a repairable job if possible
- never leave the item in a silent half-mutated state

### 14.4 Reinforcement and decay rules

1. Recall can strengthen memory importance, but not bypass policy.
2. Decay should be lazy where possible, using derived effective strength rather than heavy eager writes.
3. Reinforcement must not create runaway immortal noise.
4. Important but rarely accessed memories may surface as `decaying soon` rather than silently disappearing.

---

## 15. Retrieval Architecture Contract

The retrieval docs define the retrieval objective as: **return the smallest evidence set that maximizes downstream task success**.

### 15.1 Supported retrieval modes

At minimum, the architecture should support:
- exact retrieval
- recent retrieval
- semantic retrieval
- associative retrieval
- constraint retrieval
- reconstruction retrieval

### 15.2 Canonical candidate generation pipeline

The canonical order is:

1. direct key or id hints
2. Tier1 active-window scan
3. Tier2 exact index search
4. Tier2 graph neighborhood expansion
5. Tier2 semantic candidate generation
6. Tier3 fallback
7. dedup and diversify
8. ranking
9. packaging

This order may be short-circuited by planner logic, but not violated semantically.

### 15.3 Candidate explosion controls

Every request path implementation must include:
- hard caps by query type
- per-edge traversal budgets
- early-stop thresholds
- stale candidate penalties
- namespace pruning
- low-confidence suppression
- duplicate family collapse
- diversity constraints

### 15.4 Packaging contract

Returned memory bundles should contain:
- enough evidence to be useful,
- enough provenance to be explainable,
- enough compactness to stay within token budget,
- enough structure for downstream prompt builders to separate facts, reminders, episodes, and warnings.

### 15.5 Retrieval explainability

For a given response, the system should be able to explain:
- why each candidate entered the pool,
- which policy filters removed alternatives,
- how many graph hops were used,
- why final items ranked above other candidates,
- which items were omitted because of budget or policy.

---

## 16. Ranking and Scoring Rules

The ranking docs and formulas should be treated as a tunable scoring framework, not as fixed dogma.

### 16.1 Ranking principles

1. Policy masks apply before soft ranking.
2. Ranking should be simple enough to reason about and debug.
3. A small number of strong signals beats a large number of fragile heuristics.
4. The ranker must penalize stale, low-confidence, or noisy candidates.
5. Contradictions should not be erased; they should be represented and scored responsibly.

### 16.2 Expected score inputs

The ranker may combine:
- recency
- salience
- confidence
- utility estimate
- query alignment
- goal alignment
- memory type priors
- contradiction penalty or contradiction surfacing bonus
- duplicate-family collapse penalties
- noise penalty

### 16.3 Ranking output requirements

Each scored result should ideally expose:
- total score
- component breakdown
- decaying-soon signal if applicable
- contradiction/conflict marker if applicable
- source tier
- source lineage handle

---

## 17. Storage and Tiering Design Addendum

### 17.1 Tier responsibilities

**Tier1**
- in-process hot memory
- bounded by strict size limits
- optimized for ultra-fast exact and recent recall
- must not own large payloads

**Tier2**
- warm indexed store
- primary home for exact search, filtered retrieval, and bounded hybrid recall
- must be write-friendly and repairable

**Tier3**
- cold durable archive
- optimized for cheap storage and reconstructable recall
- may tolerate higher latency but not correctness loss

### 17.2 Tier routing rules

Routing should consider at least:
- salience
- recency
- utility estimate
- access frequency
- retention class
- policy pinning
- payload size
- summary availability

### 17.3 Tier transition principles

1. Transitions are explicit state changes.
2. Demotion is preferred over deletion.
3. Compression is preferred over deletion when possible.
4. Tier1 should store handles, compact forms, or small summaries rather than heavy blobs.
5. Tier3 must retain enough evidence for later rebuild and repair.

---

## 18. Indexing Plan

The indexing docs recommend a multi-index strategy. This section makes those choices concrete.

### 18.1 Required logical indexes

The design should support at least these index families:
- primary id index
- entity inverted index
- tag index
- session index
- goal index
- time-bucket index
- graph adjacency index
- ANN sidecar index
- bloom filters
- prefix indexes

### 18.2 Index design rules

1. Tier2 indexes should be write-friendly.
2. Tier3 indexes should be sparse and cheap.
3. Every index must be rebuildable from durable records.
4. Rebuild commands and repair paths must exist.
5. Index health must be observable.

### 18.3 Index telemetry

Each major index family should expose:
- hit rate
- miss rate
- stale index ratio
- repair backlog
- rebuild duration
- item count divergence from durable truth

### 18.4 Rebuild rule

If an index disagrees with durable evidence, durable evidence wins.

---

## 19. Association Graph Design

The graph-related docs imply a distinct graph subsystem, even if final implementation details vary.

### 19.1 Graph purpose

The graph exists to support:
- associative recall
- entity neighborhood expansion
- relation traversal
- contradiction surfacing
- episodic reconstruction
- skill extraction and clustering support

### 19.2 Graph constraints

1. Graph traversal on request path must be budgeted.
2. Graph edges must have provenance or reproducible derivation.
3. Graph repair must be possible from lineage and durable indexes.
4. Cross-namespace leakage through graph edges is forbidden.
5. Fanout explosions must be detectable and containable.

### 19.3 Graph operational requirements

The system should track:
- average node degree
- high-fanout nodes
- traversal depth distribution
- graph repair queue
- graph/index disagreement rate

---

## 20. Cache and Prefetch Plan

Cache and prefetch are only valid if they reduce tail latency without poisoning correctness.

### 20.1 Cache families

The design may include:
- Tier1 item cache
- negative cache
- result cache
- entity neighborhood cache
- summary cache
- ANN probe cache
- prefetch hints
- session warmup
- goal-conditioned cache
- cold-start mitigation cache

### 20.2 Cache guardrails

All caches should obey:
- version-aware invalidation
- namespace-aware keys
- bounded memory usage
- stale-result observability
- cache hit and miss metrics

### 20.3 Prefetch restrictions

1. Prefetch must be hint-driven, never mandatory for correctness.
2. Prefetch must not starve real foreground work.
3. Prefetch should be cancelable when user intent changes.
4. Prefetch should not cross namespace boundaries.

---

## 21. Consolidation Plan

Consolidation is not optional polish. It is the mechanism that turns raw accumulation into durable utility.

### 21.1 Consolidation goals

- compress repeated evidence
- extract stable facts
- derive reusable skills
- collapse duplicates
- strengthen useful relations
- reduce noise before it becomes archival debt

### 21.2 Canonical consolidation operations

- episode summarization
- fact extraction
- skill extraction
- duplicate family collapse
- contradiction detection
- relation reinforcement
- archive compaction support

### 21.3 Episode formation heuristics

Events are good candidates for episode formation when they share:
- task continuity
- session continuity
- goal continuity
- temporal proximity
- entity overlap
- tool-chain continuity
- fail/retry continuity

### 21.4 Consolidation safety rules

1. Do not destroy authoritative evidence when generating summaries.
2. A summary without back-links is insufficient.
3. Consolidation quality matters more than raw compression ratio.
4. Consolidation jobs must be benchmarked for utility, not just bytes saved.

---

## 22. Forgetting Plan

Forgetting is an active design feature, not an admission of failure.

### 22.1 Forgetting operations

The architecture should support:
- suppress
- decay
- demote
- compact
- summarize
- archive
- redact
- soft delete
- hard delete

### 22.2 Forgetting principles

1. Prefer compression over deletion.
2. Prefer demotion over deletion.
3. Never silently remove the last authoritative evidence unless policy explicitly permits it.
4. Separate utility forgetting from privacy/compliance deletion.
5. Preserve enough lineage to explain why something became less visible.

### 22.3 Forgetting-as-signal

Near-decay items may be surfaced as a signal:
- to warn that useful knowledge is fading,
- to prompt self-rehearsal or reinforcement,
- to protect critical but rarely used knowledge from accidental archival disappearance.

---

## 23. Compaction and Repair Acceptance

Compaction reduces cost. Repair restores correctness after crashes, drift, or partial failure.

### 23.1 Core maintenance operations

The design must account for:
- segment compaction
- duplicate family collapse
- lineage pruning
- index rebuild
- graph repair
- tombstone sweep
- payload detachment
- summary regeneration
- shard repair
- backfill re-encoding

### 23.2 Safety invariants

Each operation must respect:
- do not lose authoritative evidence
- preserve lineage or record irreversible loss
- keep before/after metrics
- rate-limit large jobs
- bound foreground interference

### 23.3 Telemetry expectations

Every operation should emit:
- bytes before and after
- affected item count
- error count
- rebuild duration
- rollback or fallback status if applicable

### 23.4 Acceptance rule

A maintenance subsystem is not complete until it is:
- documented,
- benchmarked,
- repairable,
- observable,
- safe under partial failure.

---

## 24. Governance and Security Enforcement Matrix

The supporting security document identifies twelve governance domains that must be enforced everywhere.

### 24.1 Domains

- namespace isolation
- workspace ACL
- agent ACL
- session visibility
- redaction
- retention compliance
- legal hold
- deletion guarantees
- audit logs
- secrets handling
- cross-tenant protection
- policy precedence

### 24.2 Enforcement rule

Each domain must be enforced consistently across:
- write path
- read path
- background jobs
- maintenance operations
- export/import or migration paths

### 24.3 Security implementation requirements

1. Policy checks happen before expensive retrieval work where possible.
2. Cache keys and indexes must respect namespace boundaries.
3. Explain and inspect APIs must never bypass governance.
4. Background repair must not surface redacted payloads to unauthorized actors.
5. Policy decisions must be reproducible under audit.

---

## 25. Operations Acceptance Criteria

The operations doc describes a repeated runbook shape that should be standardized.

### 25.1 Runbooks that must exist

At minimum, production documentation should include runbooks for:
- capacity planning
- daily health review
- backpressure
- compaction windows
- shard balancing
- index rebuild operations
- retention enforcement
- incident response
- migration
- version rollout

### 25.2 Standard runbook shape

Each runbook should define:
- preconditions
- command sequence
- metrics to watch
- rollback conditions
- post-run validation

### 25.3 Operational success criteria

A workflow is accepted only if it completes without violating:
- latency budgets
- data integrity guarantees
- lineage guarantees
- policy guarantees

---

## 26. Failure Mode Matrix

The failure playbook identifies recurring failure classes that the design must anticipate.

### 26.1 Canonical failure modes

- Tier1 overflow
- Tier2 index drift
- Tier3 segment corruption
- contradiction masking
- false association
- duplicate storms
- planner budget blow-up
- graph fanout explosion
- repair backlog growth
- latency regression
- cross-namespace leakage
- retention-policy bug

### 26.2 Immediate-response pattern

For all major incident classes, the system should support the following immediate-response shape:
- isolate affected namespace or shard if needed
- stop destructive jobs
- preserve forensic logs
- enable degraded mode if available

### 26.3 Root-cause investigation pattern

The investigation checklist should cover:
- lineage validation
- index count comparison against durable records
- recent deploy inspection
- repair queue growth inspection
- compaction history inspection

### 26.4 Design implication

A production architecture is incomplete if it cannot enter a safe degraded mode under these failures.

---

## 27. Benchmark Protocol

### 27.1 Benchmark dimensions

The benchmark suite must measure:
- latency
- throughput
- quality
- stability
- rebuild performance
- compaction overhead
- shard movement cost

### 27.2 Minimum latency benchmark set

The plan should explicitly benchmark:
- Tier1 exact handle get
- Tier1 recent-window search
- Tier2 session search
- Tier2 entity search
- Tier2 hybrid retrieval
- Tier3 archive reconstruction
- fast encode path
- full encode path

### 27.3 Benchmark philosophy

1. A feature is not real until benchmarked on representative workloads.
2. Average latency is insufficient; p95 and p99 matter.
3. Quality metrics must be tracked alongside latency.
4. Maintenance overhead belongs in the benchmark story, not in a footnote.

---

## 28. Test Strategy Addendum

The test strategy must cover correctness, performance, durability, and explainability.

### 28.1 Test suites that must exist

- unit tests
- property tests
- integration tests
- latency tests
- load tests
- chaos tests
- rebuild tests
- migration tests
- policy tests
- cross-namespace isolation tests
- recall quality tests
- compression utility tests

### 28.2 Coverage targets for every suite

Each test family should intentionally probe:
- normal flow
- edge cases
- adversarial inputs
- crash-recovery behavior
- observability signal verification

### 28.3 Output contract

Every suite should emit structured artifacts that support regression analysis over time.

### 28.4 Acceptance rule

A subsystem is not accepted when it merely works once; it is accepted when its failure classes are testable and observable.

---

## 29. Sharding and Distribution Plan

Scale-out should only be introduced when workload patterns justify it.

### 29.1 Candidate strategies

The design space includes:
- namespace sharding
- workspace sharding
- time-range sharding
- hot/cold split
- rebalancing
- tenant isolation
- cross-shard recall
- shard-local caching
- replication
- disaster recovery

### 29.2 Shared trade-offs

Across these strategies, the recurring advantages are:
- better locality for some workloads
- bounded shard sizes
- easier maintenance windows

The recurring costs are:
- cross-shard recall complexity
- rebalancing cost
- metadata coordination overhead

### 29.3 Rule for adoption

No distribution strategy should be made default until:
- the dominant workload is understood,
- rebalancing cost is benchmarked,
- failure and repair paths are defined,
- governance boundaries remain enforceable across shards.

---

## 30. Open Research and Falsifiable Claims

The neuro-mapping and research docs are valuable as design inspiration, but every biological analogy must survive empirical validation.

### 30.1 Claims that must remain falsifiable

- that salience-driven routing improves downstream task success
- that reconsolidation-like updates improve memory utility without destabilizing truth
- that active forgetting reduces noise while preserving critical knowledge
- that skill extraction improves repeated-task performance
- that graph-assisted recall beats simpler retrieval under bounded budgets
- that decaying-soon signals improve preservation of rare but important knowledge

### 30.2 Questions that should not be closed too early

- how to estimate utility robustly
- how strong contradiction-aware ranking should be
- how aggressive memory compression can be before utility drops
- how dense association graphs should become
- how to calibrate hybrid retrieval and hybrid ranking
- how to avoid false-memory style associations at scale
- how to measure reconstruction fidelity in realistic agent tasks

### 30.3 Research discipline rule

If a brain-inspired mechanism cannot demonstrate measurable benefit under benchmark and ablation, it should remain optional rather than canonical.

---

## 31. Implementation Priority Overlay

To keep the mega-plan executable, the supporting docs imply this priority order:

### 31.1 Priority order

1. freeze object model and invariants
2. build benchmark and test harnesses
3. implement Tier1 fast path
4. implement Tier2 indexed retrieval baseline
5. add ranking explainability and contradiction handling
6. add graph-assisted recall under strict budgets
7. add consolidation and forgetting
8. add compaction and repair
9. add sharding/distribution only when justified

### 31.2 Reason for this order

This order minimizes the risk of building a large but unmeasurable system. It front-loads observability, correctness, and benchmarkability before scale complexity.

---

## 32. Canonical Summary of What Supporting Docs Add to This Plan

The additional docs do not replace the original thesis. They sharpen it.

- `ARCHITECTURE.md` adds explicit invariants and system decomposition.
- `MEMORY_MODEL.md` defines the canonical taxonomy and required fields.
- `RETRIEVAL.md` defines the retrieval objective and candidate pipeline.
- `STORAGE.md` defines the tier responsibilities and storage principles.
- `INDEXING_STRATEGIES.md` defines the index families and observability expectations.
- `CACHE_AND_PREFETCH.md` defines the cache families and correctness guardrails.
- `COMPACTION_AND_REPAIR.md` defines safe maintenance operations.
- `SECURITY_GOVERNANCE.md` defines universal policy enforcement domains.
- `OPERATIONS.md` defines the standard production runbook shape.
- `FAILURE_PLAYBOOK.md` defines the failure matrix and degraded-mode assumptions.
- `BENCHMARKS.md` defines what must be measured.
- `TEST_STRATEGY.md` defines what must be verified.
- `SHARDING_AND_DISTRIBUTION.md` defines the scale-out decision space.

Together, these docs transform the project from a strong idea into a design that can be audited, benchmarked, repaired, and shipped.

---

## 33. Detailed Data Schema

This section appends a more implementation-oriented schema layer to the plan. It should be treated as the canonical data-contract baseline for storage, indexing, repair, and policy enforcement.

### 33.1 Schema goals

The schema layer must satisfy all of the following at once:
- preserve provenance
- preserve lineage
- support fast retrieval and filtering
- support compaction and repair
- support contradiction representation
- support retention and deletion policy enforcement
- support sharding and migration
- support explainability after retrieval

A schema that is fast but not repairable is insufficient.
A schema that is expressive but not benchmarkable is insufficient.
A schema that stores memories but cannot explain them is insufficient.

### 33.2 Canonical base object: `MemoryItem`

All major memory-like objects should either be stored directly as `MemoryItem` records with type-specific extensions, or be translatable to that shape without loss of policy- or lineage-critical information.

#### Required base fields

- `id`
- `memory_type`
- `namespace`
- `created_at_ms`
- `updated_at_ms`
- `version`

#### Strongly expected core fields

- `workspace_id`
- `agent_id`
- `session_id`
- `task_id`
- `source_kind`
- `source_ref`
- `authoritativeness`
- `content_ref`
- `payload_ref`
- `compact_text`
- `fingerprint`
- `tier`
- `salience`
- `confidence`
- `utility_estimate`
- `recall_count`
- `last_access_at_ms`
- `retention_class`
- `decay_state`
- `policy_flags`
- `lineage`
- `tags`
- `entity_refs`
- `relation_refs`

#### Base validation rules

1. `id` must be globally unique within the applicable namespace policy boundary.
2. `created_at_ms <= updated_at_ms`.
3. `version` must increment on accepted mutation.
4. `payload_ref` / `content_ref` must be stable, resolvable, or explicitly tombstoned.
5. `namespace` must always be present and valid before persistence.
6. `tier`, `retention_class`, and `decay_state` must be representable even if encoded compactly.
7. `lineage` must never be dropped silently during merge, summarization, compaction, or repair.

### 33.3 Canonical Rust-style structural sketch

```rust
pub enum MemoryTier {
    Tier1,
    Tier2,
    Tier3,
}

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
    pub source_kind: Option<String>,
    pub source_ref: Option<String>,
    pub authoritativeness: Option<f32>,
    pub content_ref: Option<String>,
    pub payload_ref: Option<String>,
    pub compact_text: String,
    pub fingerprint: u64,
    pub tier: MemoryTier,
    pub salience: f32,
    pub confidence: f32,
    pub utility_estimate: f32,
    pub recall_count: u32,
    pub last_access_at_ms: Option<i64>,
    pub retention_class: RetentionClass,
    pub decay_state: DecayState,
    pub policy_flags: Vec<String>,
    pub lineage: Vec<MemoryId>,
    pub version: u64,
    pub tags: Vec<String>,
    pub entity_refs: Vec<EntityRef>,
    pub relation_refs: Vec<RelationRef>,
}
```

This sketch is illustrative, but the semantic content is mandatory even if the final Rust layout differs.

### 33.4 Type-specific schema families

The `DATA_SCHEMAS.md` document identifies a base shape repeated across major memory families. The plan should formalize these as type-specific overlays on top of `MemoryItem`.

#### 33.4.1 Event

**Purpose**
- capture raw observed occurrences
- preserve tool outputs, actions, and state transitions before consolidation

**Required fields**
- `id`
- `created_at_ms`
- `updated_at_ms`
- `namespace`
- `version`

**Typical optional fields**
- `workspace_id`
- `agent_id`
- `session_id`
- `task_id`
- `payload_ref`
- `tags`

**Additional expected fields in practice**
- `source_kind`
- `source_ref`
- `compact_text`
- `tier`
- `salience`
- `confidence`
- `lineage`

**Validation rules**
- must preserve source traceability
- may be consolidated later, but never without lineage links
- should be safe to demote or archive when summarized elsewhere

#### 33.4.2 Episode

**Purpose**
- group related events into a higher-value temporal unit

**Required fields**
- `id`
- `created_at_ms`
- `updated_at_ms`
- `namespace`
- `version`

**Typical optional fields**
- `workspace_id`
- `agent_id`
- `session_id`
- `task_id`
- `payload_ref`
- `tags`

**Additional expected fields**
- `lineage` back to source events
- `entity_refs`
- `compact_text` summary of the episode
- optional `goal` linkage
- optional `failure/success` outcome markers

**Validation rules**
- must point to constituent evidence
- must not erase the original event family
- must be reconstructable enough for explainability

#### 33.4.3 Fact

**Purpose**
- represent distilled semantic knowledge for repeated reuse

**Required fields**
- `id`
- `created_at_ms`
- `updated_at_ms`
- `namespace`
- `version`

**Typical optional fields**
- `workspace_id`
- `agent_id`
- `session_id`
- `task_id`
- `payload_ref`
- `tags`

**Additional expected fields**
- `authoritativeness`
- `confidence`
- `source_ref`
- `lineage`
- contradiction linkage where applicable

**Validation rules**
- facts must never silently replace conflicting facts
- confidence must be mutable without breaking provenance
- fact extraction must preserve a path back to evidence

#### 33.4.4 Summary

**Purpose**
- compress lower-level evidence into a bounded, explainable representation

**Required fields**
- `id`
- `created_at_ms`
- `updated_at_ms`
- `namespace`
- `version`

**Typical optional fields**
- `workspace_id`
- `agent_id`
- `session_id`
- `task_id`
- `payload_ref`
- `tags`

**Additional expected fields**
- `lineage`
- `compact_text`
- optional source counts / scope metadata
- optional utility metrics

**Validation rules**
- a summary without lineage is invalid for canonical storage
- summary regeneration must be possible if the summary is stale or corrupted

#### 33.4.5 Relation

**Purpose**
- link entities, memories, goals, or concepts in a form usable by graph-assisted retrieval

**Required fields**
- `id`
- `created_at_ms`
- `updated_at_ms`
- `namespace`
- `version`

**Typical optional fields**
- `workspace_id`
- `agent_id`
- `session_id`
- `task_id`
- `payload_ref`
- `tags`

**Additional expected fields**
- relation endpoints
- relation kind / edge label
- confidence
- provenance or derivation handle
- graph repair metadata

**Validation rules**
- relation endpoints must resolve or be tombstoned explicitly
- relation edges must never bypass namespace policy
- relation derivation should be reproducible or auditable

#### 33.4.6 ConflictRecord

**Purpose**
- represent contradiction explicitly rather than hiding it in overwrite behavior

**Required fields**
- `id`
- `created_at_ms`
- `updated_at_ms`
- `namespace`
- `version`

**Typical optional fields**
- `workspace_id`
- `agent_id`
- `session_id`
- `task_id`
- `payload_ref`
- `tags`

**Additional expected fields**
- references to conflicting memories
- evidence handles for both sides
- optional conflict status (open/resolved/superseded)
- optional resolution explanation

**Validation rules**
- must be created or updated when contradiction is detected
- must preserve both sides of the disagreement
- may influence ranking, but not storage erasure

#### 33.4.7 Goal

**Purpose**
- represent active or historical objectives that shape relevance and behavior

**Required fields**
- `id`
- `created_at_ms`
- `updated_at_ms`
- `namespace`
- `version`

**Typical optional fields**
- `workspace_id`
- `agent_id`
- `session_id`
- `task_id`
- `payload_ref`
- `tags`

**Additional expected fields**
- goal status
- priority
- utility linkage
- parent/child goal references

**Validation rules**
- goal state changes must remain auditable
- inactive goals may still remain relevant for episodic reconstruction

#### 33.4.8 Skill

**Purpose**
- represent procedural knowledge extracted from repeated successful behavior

**Required fields**
- `id`
- `created_at_ms`
- `updated_at_ms`
- `namespace`
- `version`

**Typical optional fields**
- `workspace_id`
- `agent_id`
- `session_id`
- `task_id`
- `payload_ref`
- `tags`

**Additional expected fields**
- lineage to repeated episodes or outcomes
- success signals
- context applicability metadata
- confidence / maturity indicator

**Validation rules**
- a skill extracted from thin evidence should remain tentative
- skill extraction should not delete underlying procedural evidence

#### 33.4.9 Constraint

**Purpose**
- preserve rules, obligations, limits, and non-negotiable instructions

**Required fields**
- `id`
- `created_at_ms`
- `updated_at_ms`
- `namespace`
- `version`

**Typical optional fields**
- `workspace_id`
- `agent_id`
- `session_id`
- `task_id`
- `payload_ref`
- `tags`

**Additional expected fields**
- constraint scope
- priority or hardness
- policy interaction markers
- source authority metadata

**Validation rules**
- constraints must remain highly retrievable
- constraints should resist accidental forgetting more strongly than normal events

#### 33.4.10 DecayState

**Purpose**
- track current forgetting posture or effective decay status

**Required fields**
- `id`
- `created_at_ms`
- `updated_at_ms`
- `namespace`
- `version`

**Typical optional fields**
- `workspace_id`
- `agent_id`
- `session_id`
- `task_id`
- `payload_ref`
- `tags`

**Additional expected fields**
- decay bucket / effective strength
- decay warning status
- last reinforcement marker
- bypass-decay or pinned-decay flags

**Validation rules**
- decay metadata must not violate retention policy
- decay state should be derivable or repairable when possible

#### 33.4.11 RetentionRule

**Purpose**
- capture explicit retention and deletion policy constraints

**Required fields**
- `id`
- `created_at_ms`
- `updated_at_ms`
- `namespace`
- `version`

**Typical optional fields**
- `workspace_id`
- `agent_id`
- `session_id`
- `task_id`
- `payload_ref`
- `tags`

**Additional expected fields**
- retention class
- effective period
- legal hold marker
- deletion mode and guarantees

**Validation rules**
- must be authoritative for deletion behavior
- must remain auditable after migration and rebuild

#### 33.4.12 ShardDescriptor

**Purpose**
- represent shard allocation and distribution metadata

**Required fields**
- `id`
- `created_at_ms`
- `updated_at_ms`
- `namespace`
- `version`

**Typical optional fields**
- `workspace_id`
- `agent_id`
- `session_id`
- `task_id`
- `payload_ref`
- `tags`

**Additional expected fields**
- shard key or shard range
- placement metadata
- balancing status
- migration / repair markers

**Validation rules**
- shard descriptors must remain compatible with disaster recovery and rebalancing workflows
- shard metadata must not be the only source of truth for memory existence

### 33.5 Shared validation rules across all schema families

The repeated schema docs imply these common validation rules:

1. ids must be globally unique within namespace policy
2. `created_at_ms` must not exceed `updated_at_ms`
3. `version` must increment on mutation
4. `payload_ref` must be stable or tombstoned

This plan extends that into a stronger shared contract:

5. no object may exist without namespace binding
6. no object may bypass policy metadata when persisted
7. no object may lose lineage silently during consolidation or repair
8. no object may cross namespace boundaries through relation fields without explicit policy support
9. any tombstoned payload must still preserve audit-safe metadata unless compliance requires stronger deletion

### 33.6 Type families not fully expanded in DATA_SCHEMAS but required by the memory model

The `MEMORY_MODEL.md` taxonomy includes several types that should also receive schema overlays even if not spelled out in `DATA_SCHEMAS.md` yet.

#### 33.6.1 Hypothesis
- tentative statement
- requires confidence and validation status
- should link to supporting and disconfirming evidence

#### 33.6.2 PolicyArtifact
- policy-relevant memory object
- should carry stronger audit fields and retention controls

#### 33.6.3 Observation
- raw or near-raw observation distinct from action outcome
- useful for episodic grouping and factual extraction

#### 33.6.4 ToolOutcome
- normalized representation of tool execution result
- should preserve tool identity, execution context, outcome category, and provenance

#### 33.6.5 UserPreference
- durable user-specific preference
- should be highly retrievable and policy-scoped
- should support supersession without silent erasure

#### 33.6.6 SessionMarker
- session boundary / checkpoint marker
- should support reconstruction, grouping, and debugging of temporal context

### 33.7 Suggested relational storage shape

The docs do not hard-code SQL DDL, but they strongly imply a practical decomposition.

#### Core durable tables
- `memory_items`
- `memory_payloads`
- `memory_lineage_edges`
- `memory_entity_refs`
- `memory_relation_refs`
- `memory_tags`
- `conflict_records`
- `goals`
- `skills`
- `retention_rules`
- `shard_descriptors`
- `transition_events`
- `repair_jobs`

#### Why split these tables
- keep hot metadata compact
- keep payloads detachable
- make lineage explicit and repairable
- allow typed overlays without bloating every hot-row access path
- preserve indexability for session/entity/goal filters

### 33.8 Suggested index coverage for schema fields

At the schema layer, the following indexability assumptions should be built in:

- `id` -> primary lookup
- `namespace` -> universal filter
- `workspace_id` -> workspace isolation and recall
- `agent_id` -> actor-scoped retrieval
- `session_id` -> episodic grouping and session replay
- `task_id` -> task-context lookup
- `created_at_ms` / `updated_at_ms` -> temporal windows
- `tags` -> thematic lookup
- `entity_refs` -> entity-centric retrieval
- `goal` or goal linkage -> goal-aware ranking and retrieval
- `fingerprint` -> duplicate family handling
- `tier` -> maintenance and routing analysis
- `retention_class` / `policy_flags` -> governance checks

### 33.9 State-machine alignment with schema

The `STATE_MACHINES.md` document shows a repeated lifecycle pattern for major object types.

For Event, Episode, Fact, Summary, Goal, Skill, Constraint, ConflictRecord, Relation, RetentionClass-like objects, DecayState-like objects, and ShardState-like objects, the common lifecycle is:

- `created`
- `indexed`
- `recalled`
- `reinforced`
- `decayed`
- `consolidated`
- `demoted`
- `archived`
- `deleted`

#### Required transition guards
- policy pinning
- namespace access control
- unresolved contradiction state
- lineage preservation requirement
- repair job lock

#### Failure handling contract
If a transition fails:
- write a transition error event
- preserve the prior state
- enqueue a repairable task

This means the schema must have room for:
- current lifecycle state or equivalent derived status
- transition audit history
- repair queue references
- policy and contradiction guards

### 33.10 Data durability and repair rules

The schema must support all of the following repairs without inventing missing truth:
- index rebuild from durable records
- graph repair from relation and lineage data
- summary regeneration from source evidence
- payload detachment and reattachment where allowed
- duplicate-family collapse with preserved ancestry
- shard repair after balancing or migration

If a repair cannot reconstruct full original fidelity, the system must record the loss explicitly rather than hiding it.

### 33.11 Schema evolution rules

When the schema evolves:

1. old data must remain interpretable or migratable
2. migration must preserve lineage, timestamps, namespace, and version semantics
3. policy-critical metadata must survive migrations exactly
4. index rebuild must be possible after migration
5. schema versioning must be explicit and testable

### 33.12 Canonical schema acceptance checklist

A data schema design is only acceptable if it can answer yes to all of the following:

- Can every stored memory be traced back to source or lineage?
- Can contradictions be represented without overwrite?
- Can policy be enforced from stored metadata alone?
- Can payloads be detached without losing core memory identity?
- Can indexes be rebuilt from durable truth?
- Can background compaction run without losing authoritative evidence?
- Can retrieval explain why a result was returned?
- Can migrations preserve policy and lineage semantics?
- Can sharding metadata evolve without becoming the sole source of truth?
- Can failure handling preserve the prior valid state?

If not, the schema is not yet ready to be called canonical.

---

## 34. Detailed MCP API Contract

This section expands the MCP surface into a more explicit contract. The names come from `MCP_API.md`; the semantics here make them implementation-ready.

### 34.1 Global MCP design rules

Every MCP tool must:
- preserve namespace and policy context
- never bypass governance checks
- return enough metadata for explainability
- distinguish user error, policy denial, and internal failure
- preserve idempotency where practical
- expose stable machine-readable outputs for automation

### 34.2 Common request envelope

Every MCP request should conceptually carry:
- `namespace`
- `workspace_id` if applicable
- `agent_id` if applicable
- `session_id` if applicable
- `task_id` if applicable
- `request_id`
- `policy_context`
- `time_budget_ms` or retrieval budget where relevant

### 34.3 Common response envelope

Every MCP response should be able to carry:
- `ok`
- `request_id`
- `namespace`
- `result`
- `warnings`
- `policy_filters_applied`
- `explain_handle` or embedded explanation
- `metrics` for latency / candidate counts where relevant

### 34.4 Tool: `memory_put`

**Purpose**
- ingest a new memory item or structured memory payload

**Expected inputs**
- namespace context
- memory type
- content or payload reference
- source metadata
- optional salience / tags / entity refs / relation refs
- optional explicit retention or pinning hints

**Expected outputs**
- memory id
- chosen tier
- validation outcome
- routing reason summary
- deferred enrichment job handle if created

**Rules**
- writes must validate policy first
- contradictory writes must not silently overwrite existing evidence
- the response should indicate whether the write created conflict metadata

### 34.5 Tool: `memory_get`

**Purpose**
- retrieve a memory item by id or canonical handle

**Expected outputs**
- typed memory view
- provenance fields
- current tier
- policy-redacted fields where applicable

**Rules**
- exact lookup does not bypass redaction or namespace checks
- missing and unauthorized must be distinguishable at the internal API boundary, even if collapsed externally for security reasons

### 34.6 Tool: `memory_search`

**Purpose**
- run bounded search over indexes, tags, entities, time ranges, or filtered text/semantic hints

**Expected inputs**
- query string or structured filters
- namespace and scope filters
- optional memory types
- optional session/task/goal filters
- result budget

**Expected outputs**
- candidate list
- filter summary
- index families used
- omitted-result note if capped

### 34.7 Tool: `memory_recall`

**Purpose**
- perform task-oriented bounded retrieval for context construction

**Expected inputs**
- task or goal description
- retrieval mode hints
- token budget or result budget
- namespace / actor context

**Expected outputs**
- ranked evidence set
- score summaries
- contradiction markers
- decaying-soon markers if enabled
- packaging metadata suitable for prompt construction

### 34.8 Tool: `memory_link`

**Purpose**
- create or update explicit relations between memories, entities, or goals

**Rules**
- links require namespace compatibility and policy approval
- link provenance must be stored
- graph repair must be possible after link creation

### 34.9 Tool: `memory_inspect`

**Purpose**
- retrieve diagnostic and structural details about a memory item or memory family

**Should expose**
- current tier
- lineage
- policy flags
- lifecycle state
- index presence
- graph neighborhood summary
- decay / retention information

### 34.10 Tool: `memory_explain`

**Purpose**
- explain why a memory was stored, routed, recalled, ranked, filtered, demoted, or forgotten

**Should explain**
- routing signals
- ranking components
- policy filters
- lineage ancestry
- consolidation ancestry
- forgetting / demotion reasons

### 34.11 Tool: `memory_consolidate`

**Purpose**
- trigger or schedule consolidation workloads

**Should support**
- session-scoped consolidation
- task-scoped consolidation
- duplicate collapse
- fact extraction
- summary generation
- skill extraction

**Rules**
- must preserve evidence
- must emit artifact ids for generated summaries/facts/relations
- must be safe to run in bounded background windows

### 34.12 Tool: `memory_pin`

**Purpose**
- raise retention protection or bypass normal forgetting/demotion behavior

**Rules**
- pinning is policy-relevant and auditable
- pinning should not bypass redaction or governance
- pinning reason should be recorded

### 34.13 Tool: `memory_forget`

**Purpose**
- perform controlled forgetting operations

**Operations may include**
- suppress
- decay
- demote
- compact
- summarize
- archive
- redact
- soft delete
- hard delete where policy permits

**Rules**
- must distinguish utility-driven forgetting from compliance deletion
- must preserve lineage when required
- must never remove last authoritative evidence unless policy explicitly allows it

### 34.14 Tool: `memory_repair`

**Purpose**
- run or schedule repair actions for indexes, graph, lineage, summaries, or shards

**Rules**
- durable evidence wins over derived state
- repair output should include what was fixed, rebuilt, or left unresolved
- partial-fidelity repair must record explicit loss

---

## 35. Detailed CLI Contract

The CLI is the operator and developer surface for the same core system. It must expose power without bypassing policy.

### 35.1 CLI design principles

- CLI commands map cleanly onto core memory actions
- CLI should be scriptable and machine-readable
- human-readable output should be layered on top of a structured result model
- CLI must not create hidden behavior different from MCP behavior

### 35.2 Core commands from `CLI.md`

```bash
membrain put event --namespace ws/app --type user_message --content "..."
membrain get --id mem_123
membrain search --query "rust linker failure"
membrain recall --goal "fix build pipeline"
membrain consolidate --session sess_42
membrain inspect --id mem_456
membrain benchmark tier1
membrain repair index --namespace ws/app
```

### 35.3 Required command families

The plan should treat the following CLI families as canonical:
- `put`
- `get`
- `search`
- `recall`
- `consolidate`
- `inspect`
- `benchmark`
- `repair`
- `pin`
- `forget`
- `stats`
- `doctor`
- `export`
- `import`

### 35.4 `put` contract

Should support:
- event ingestion
- typed content
- namespace selection
- structured metadata
- tags/entity refs/relation refs
- optional retention hints

### 35.5 `get` contract

Should support:
- lookup by id
- raw JSON output
- human-readable pretty output
- optional lineage expansion
- optional policy-debug info for authorized operators

### 35.6 `search` contract

Should support:
- query text
- structured filters
- namespace restriction
- type restriction
- time range filters
- result limits
- JSON output

### 35.7 `recall` contract

Should support:
- goal-based recall
- task-text recall
- bounded result count
- bounded token budget output
- explain mode
- include-conflicts mode
- include-decaying mode

### 35.8 `consolidate` contract

Should support:
- per-session consolidation
- per-task consolidation
- duplicate collapse
- summary regeneration
- dry-run mode
- metrics output

### 35.9 `inspect` contract

Should support:
- memory item inspection
- lineage view
- graph neighborhood preview
- retention and decay state
- tier routing explanation

### 35.10 `benchmark` contract

Should support:
- Tier1 benchmark
- Tier2 benchmark
- Tier3 benchmark
- encode benchmark
- retrieval benchmark
- maintenance benchmark
- JSON artifact emission

### 35.11 `repair` contract

Should support:
- index repair
- graph repair
- summary regeneration
- duplicate-family cleanup
- shard repair
- dry-run mode
- bounded execution mode

### 35.12 CLI output modes

Every major command should support:
- human-readable text
- structured JSON
- exit codes that separate validation failure, policy denial, and internal error

---

## 36. Algorithm and Pseudocode Canonicalization

The pseudocode and algorithm docs should be interpreted as canonical patterns, not as line-by-line required implementations.

### 36.1 Canonical encode pattern

```text
fn fast_encode(event):
    norm = normalize(event)
    fp = fingerprint(norm)
    class = shallow_classify(norm)
    sal = provisional_salience(norm, class)
    tier = route_fast(class, sal, norm.payload_size)
    item = make_memory_item(norm, fp, class, sal, tier)
    persist(item)
    schedule_deferred_enrichment(item.id)
    return item.id
```

### 36.2 Canonical retrieval planner pattern

```text
fn retrieval_plan(query_ctx):
    if query_ctx.id_hint:
        return ExactById
    if query_ctx.active_session and query_ctx.is_small_lookup:
        return Tier1RecentThenTier2Exact
    if query_ctx.entity_heavy:
        return Tier2EntityThenGraph
    if query_ctx.semantic_need_high:
        return Tier2HybridWithBudget
    return Tier2ExactThenTier3Fallback
```

### 36.3 Canonical Tier1 access pattern

```text
fn tier1_get(key):
    slot = hot_index.lookup(key)
    if slot is None:
        return None
    item = arena.read(slot)
    if item.expired():
        return None
    return item
```

### 36.4 Canonical hybrid recall pattern

```text
fn hybrid_recall(query):
    cands = []
    cands += exact_indexes(query)
    cands += entity_indexes(query)
    if budget_left():
        cands += ann_candidates(query)
    cands = dedup(cands)
    cands = bounded_graph_expand(cands, query)
    return rank(cands, query)
```

### 36.5 Canonical decay update pattern

```text
fn decay_update(item, now):
    age = now - item.last_access_or_create()
    disuse = sigmoid(age / tau_age)
    penalty = disuse * (1 - item.utility_estimate)
    if item.retention_class.is_pinned():
        penalty *= 0.1
    item.decay_score = clamp(penalty, 0, 1)
    return item
```

### 36.6 Canonical consolidation pattern

```text
fn consolidate_episode(events):
    cluster = sort_by_time(events)
    summary = summarize(cluster)
    facts = extract_facts(cluster)
    relations = derive_relations(cluster)
    write(summary)
    for fact in facts:
        write(fact)
    for rel in relations:
        write(rel)
    mark_cluster_consolidated(cluster.ids)
```

### 36.7 Algorithm families from `ALGORITHM_CATALOG.md`

The algorithm catalog is broad, but it clearly identifies the major subsystems that deserve explicit implementations:
- Tier1 algorithms
- Tier2 algorithms
- Tier3 algorithms
- encode algorithms
- ranking algorithms
- graph algorithms
- compaction algorithms
- rebuild algorithms
- sharding algorithms
- caching algorithms

### 36.8 Implementation rule

For each algorithm family, the production code should prefer:
- a small number of benchmarked implementations,
- explicit invariants,
- explainable failure modes,
- replaceable strategy boundaries,
- no hidden unbounded work on the request path.

---

## 37. Detailed Ranking and Formula Calibration

`RANKING_FORMULAS.md` intentionally keeps formulas simple. This is a strength, not a weakness.

### 37.1 Shared formula shape

Most ranking-like scores can start from this common template:

```text
score = bias
      + w_recency * recency_feature
      + w_salience * salience_feature
      + w_confidence * confidence_feature
      + w_utility * utility_feature
      + w_goal * goal_alignment_feature
      - w_noise * noise_feature
```

### 37.2 Scores that should share this calibration philosophy

- salience score
- confidence score
- utility estimate
- decay score
- promotion score
- demotion score
- conflict severity
- retrieval relevance
- novelty score
- compression value

### 37.3 Calibration notes

- normalize features into bounded ranges
- prefer monotonic transforms
- calibrate by workload, not globally across all tasks
- apply hard policy masks before soft ranking
- keep score decomposition inspectable

### 37.4 Practical consequence

The architecture should expose score components as first-class observability data so tuning can happen without guesswork.

---

## 38. Performance Budget and Fast Path Restrictions

The performance docs all repeat the same core message: low latency only becomes believable when the hot path is structurally constrained.

### 38.1 Universal fast path tactics

- choose stable and compact representations
- avoid dynamic allocations where possible
- split hot metadata from cold payloads
- cap work by query class
- prefer precomputed handles over expensive reconstruction

### 38.2 Engineering patterns repeatedly emphasized

The hot path should preferentially use or evaluate:
- CPU-cache-friendly layouts
- arena allocation where beneficial
- branch-predictable control flow
- stable hashing strategy
- small-object layout discipline
- Tier1 ring buffer or equivalent bounded hot structure
- metadata splitting
- hot/cold field separation
- syscall avoidance on request path
- bounded candidate generation
- graph traversal budgeting
- SIMD-friendly scans where justified
- lock avoidance or read-optimized concurrency patterns
- batched writes and write coalescing off the request path
- pinned-object fast path for high-priority memories

### 38.3 Performance budget decomposition

A request budget should be decomposed into at least:
- planner budget
- index lookup budget
- graph expansion budget
- ranking budget
- packaging budget
- policy-check budget

### 38.4 Hard restrictions

1. no full archive scan on request path
2. no unbounded graph traversal on request path
3. no maintenance job on request path
4. no payload-heavy reconstruction on request path unless the query class explicitly allows it
5. no namespace check after expensive work that could have been pruned earlier

---

## 39. Speed Checklist Canonicalization

Although `SPEED_CHECKLIST.md` is expansive, the intent can be collapsed into a deployment-quality checklist.

### 39.1 Pre-merge performance checklist

Before accepting a hot-path-sensitive change, verify:
- bounded work remains bounded
- no large new allocations were introduced on request path
- no payload bloat entered Tier1
- score explanation still works
- namespace checks still happen early
- graph traversal remains budgeted
- p95 and p99 were measured
- stale cache behavior remains observable

### 39.2 Pre-release performance checklist

Before promoting a release, verify:
- Tier1 exact latency target still holds on representative workloads
- Tier2 retrieval latency still holds under mixed load
- Tier3 fallback remains bounded enough for its declared class
- compaction and repair jobs do not break foreground SLOs
- shard balancing and rebuild paths were tested if enabled

---

## 40. Milestone Gates and Go / No-Go Rules

The roadmap gives phases. This section adds promotion criteria.

### 40.1 Phase 0 gate

Phase 0 is complete only when:
- object model is frozen enough for benchmarkable work
- core invariants are written down and testable
- benchmark harness exists
- Tier1 MVP exists with measurable latency

### 40.2 Phase 1 gate

Phase 1 is complete only when:
- Tier2 indexed retrieval exists
- session and entity queries work
- ranking baseline is measurable
- retrieval explanations exist at least in debug/operator form

### 40.3 Phase 2 gate

Phase 2 is complete only when:
- graph support is budgeted and repairable
- contradiction records exist
- explainable packaging exists for recall output

### 40.4 Phase 3 gate

Phase 3 is complete only when:
- consolidation improves utility on benchmark corpora
- forgetting reduces noise without unacceptable fact loss
- compaction and repair are safe under failure injection

### 40.5 Phase 4 gate

Phase 4 is complete only when:
- sharding strategy is justified by actual workload pressure
- operations runbooks exist
- shard movement, repair, and recovery are benchmarked
- governance remains enforceable across shards

### 40.6 Global no-go rules

Do not promote a phase if any of the following are true:
- retrieval quality regresses without explanation
- contradiction handling is still silent overwrite
- policy enforcement is incomplete
- repairs are not observable
- p95/p99 latency is unknown
- maintenance work can corrupt durable truth

---

## 41. Rust Module and Workspace Skeleton

The current plan already sketches a workspace tree. This section turns it into a more explicit module contract.

### 41.1 Suggested workspace modules

At minimum, the project should maintain a shape like:
- `membrain-core`
- `membrain-cli`
- optional daemon/service crate
- benchmark crates or bench targets
- integration test support modules

### 41.2 Suggested `membrain-core` boundaries

Core library modules should be explicitly separated around these responsibilities:
- `types`
- `constants`
- `config`
- `brain_store`
- `store::hot`
- `store::warm` or `store::tier2`
- `store::cold` / `archive`
- `engine::encode`
- `engine::recall`
- `engine::ranking`
- `engine::consolidation`
- `engine::forgetting`
- `engine::repair`
- `graph`
- `embed`
- `index`
- `migrate`
- `observability`
- `policy`

### 41.3 Boundary rules

- policy logic should not be hidden inside unrelated modules
- store modules should not decide product semantics silently
- repair logic should be testable independently
- graph logic should be optional in retrieval plans when budget requires it
- CLI should call core APIs, not reimplement memory semantics

---

## 42. Contributor Workflow and PR Acceptance

`CONTRIBUTING.md` adds important execution discipline that belongs in the mega-plan.

### 42.1 Contributor principles

- keep hot path measurable
- preserve provenance
- write repairable code
- prefer explicit invariants over hidden behavior
- benchmark before and after performance-sensitive changes

### 42.2 Required for major PRs

Every major change should include:
- a design note
- a benchmark result
- a migration note if schema changes
- a rollback note if behavior changes

### 42.3 PR rejection rules

A major PR should be rejected or sent back if:
- it changes hot path behavior without benchmark evidence
- it alters schema without migration notes
- it changes forgetting/deletion semantics without governance analysis
- it adds performance-sensitive complexity without observability
- it weakens repairability or lineage preservation

---

## 43. README and Index Role Clarification

The docs set suggests a simple hierarchy:
- `README.md` is the project entry point
- `INDEX.md` is a light doc pointer
- `PLAN.md` is the canonical mega-plan
- topic-specific docs exist to deepen one subsystem at a time

### 43.1 Documentation rule

`PLAN.md` should remain the canonical design contract.
Subsystem docs should elaborate, not contradict.
If a subsystem doc and the plan diverge, the conflict should be resolved explicitly rather than left implicit.

---

## 44. Final Execution Order for Building membrain

To make the plan directly actionable, the full docs imply this concrete build order.

### 44.1 Step-by-step execution order

1. define canonical types, schema semantics, and invariants
2. define policy model and namespace enforcement
3. implement Tier1 fast encode and exact/recent retrieval
4. implement Tier2 durable indexed storage and search
5. implement ranking explanation and inspect/explain surfaces
6. implement contradiction representation and conflict-aware storage
7. implement graph-assisted retrieval under hard budgets
8. implement consolidation pipelines
9. implement forgetting and demotion pipelines
10. implement repair and rebuild paths
11. implement benchmark harnesses and regression artifacts
12. implement operational tooling and doctor commands
13. introduce sharding only if empirical workload demands it

### 44.2 Why this order matters

This order ensures the system becomes:
- measurable before it becomes complex,
- correct before it becomes distributed,
- explainable before it becomes highly optimized,
- repairable before it becomes operationally large.

---

## 45. Final Canonical Thesis After Merging All Supporting Docs

After incorporating every supporting document, the true thesis of membrain becomes:

1. build an agent memory system that is inspired by human memory functions but constrained by engineering budgets;
2. separate hot, warm, and cold memory responsibilities clearly;
3. treat provenance, lineage, policy, and repairability as first-class system properties;
4. use bounded retrieval and explainable ranking to maximize downstream task success;
5. compress and forget intelligently rather than accumulate noise forever;
6. refuse biological metaphor unless it survives benchmarking and operational scrutiny;
7. ship only what can be benchmarked, inspected, repaired, and governed.

That is the complete, append-only expansion implied by the rest of `docs/*.md`.

---

## 46. Feature Implementation Specs (Batch 1)

> 10 implementation-ready feature specs to complement the high-level extensions in Section 10.
> Each spec covers: concept, schema changes, core logic, API/CLI surface, and milestone placement.
> Where a feature overlaps with a Section 10 extension, the cross-reference is noted.

---

### 46.1 Dream Mode (Offline Synthesis Engine)

**Concept**

When the daemon is idle (no agent activity for N ticks), membrain runs a background
"dream" job that scans for memories with high embedding similarity but no existing
graph edge between them. It creates new synthetic engram links autonomously.
This mirrors REM sleep — cross-domain association formation without conscious input.
The system becomes smarter while idle.

**Schema Changes**

```sql
-- Add to hot.db
CREATE TABLE dream_links (
  src_memory_id  TEXT NOT NULL REFERENCES memories(id),
  dst_memory_id  TEXT NOT NULL REFERENCES memories(id),
  similarity     REAL NOT NULL,
  created_at_tick INTEGER NOT NULL,
  confidence     REAL NOT NULL DEFAULT 0.5,
  PRIMARY KEY (src_memory_id, dst_memory_id)
);

-- Add to brain_state
-- key: 'last_dream_tick', value: INTEGER
-- key: 'dream_links_created', value: INTEGER (cumulative)
```

**Core Logic**

```rust
pub struct DreamEngine {
    idle_threshold_ticks: u64,   // default: 100
    similarity_floor: f32,       // default: 0.65
    similarity_ceiling: f32,     // default: 0.92 (avoid near-duplicates)
    max_links_per_dream: usize,  // default: 200
    batch_size: usize,           // default: 500
}

impl DreamEngine {
    pub async fn run_dream_cycle(&self, store: &mut BrainStore) -> DreamReport {
        // 1. Sample N random hot memories (avoid iterating all)
        // 2. For each sample: usearch ANN search, floor < sim < ceiling
        // 3. Filter: no existing graph_edge AND no existing dream_link
        // 4. Insert dream_link with confidence proportional to similarity
        // 5. If two memories share >= 3 dream_links to same engram: trigger engram merge
        // 6. Return DreamReport { links_created, engrams_merged, duration_ms }
    }
}

pub struct DreamReport {
    pub links_created: usize,
    pub engrams_merged: usize,
    pub duration_ms: u64,
    pub tick: u64,
}
```

**Config**

```toml
[dream]
enabled                = true
idle_threshold_ticks   = 100
similarity_floor       = 0.65
similarity_ceiling     = 0.92
max_links_per_dream    = 200
```

**CLI / MCP**

```bash
membrain dream              # trigger manually
membrain dream --status     # last run, links created
membrain dream --disable    # pause background dreaming
```

```
MCP tool: dream()
  → { links_created: n, engrams_merged: n, last_run_tick: n }
```

**Milestone Placement**

Implement after **Milestone 7 (Engram Graph)**. Requires: engram BFS, graph_edges table,
usearch ANN. Add as optional sub-step in Milestone 7 or early Milestone 8.

---

### 46.2 Contradiction Detection + Belief Versioning

> Cross-ref: Extends Section 10.2 (Belief Ledger) with concrete schema, detection logic, and state machine.

**Concept**

When encoding a new memory that semantically conflicts with an existing one
(high similarity + divergent content), instead of silent overwrite or duplicate,
membrain creates a belief version chain. Old memory is marked `Superseded` and
linked to the new version. The agent can query belief evolution over time.

**Schema Changes**

```sql
ALTER TABLE memories ADD COLUMN superseded_by TEXT REFERENCES memories(id);
ALTER TABLE memories ADD COLUMN belief_version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE memories ADD COLUMN belief_chain_id TEXT;

CREATE TABLE belief_conflicts (
  id              TEXT PRIMARY KEY,
  chain_id        TEXT NOT NULL,
  old_memory_id   TEXT NOT NULL REFERENCES memories(id),
  new_memory_id   TEXT NOT NULL REFERENCES memories(id),
  similarity      REAL NOT NULL,
  detected_at     INTEGER NOT NULL,
  resolution      TEXT NOT NULL  -- 'superseded' | 'coexist' | 'merged'
);
```

**Core Logic**

```rust
pub struct ConflictDetector {
    conflict_sim_min: f32,  // default: 0.82
    conflict_sim_max: f32,  // default: 0.97
}

impl ConflictDetector {
    pub fn detect_on_encode(
        &self,
        new_memory: &Memory,
        new_vec: &[f32],
        store: &BrainStore,
    ) -> Option<ConflictResolution> {
        // 1. ANN search in similarity band [0.82, 0.97]
        // 2. For each candidate: content divergence check
        //    (if content_sim > 0.97 → duplicate, skip)
        //    (if content_sim in band → likely contradiction)
        // 3. Return ConflictResolution describing what to do
    }
}

pub enum ConflictResolution {
    Supersede { old_id: Uuid },
    Coexist,
    UpdateExisting { target_id: Uuid },
}

pub enum MemoryState {
    Labile,
    SynapticDone,
    Consolidating,
    Consolidated,
    Superseded,   // replaced by newer belief
    Archived,
}
```

**CLI / MCP**

```bash
membrain beliefs "user preferences"   # show belief chain for topic
membrain beliefs --conflicts          # list all detected contradictions
membrain beliefs --resolve <id>       # manually resolve a pending conflict
membrain inspect <uuid> --history     # show full version chain for a memory
```

```
MCP tool: belief_history(query)
  → { chain_id, versions: [{id, content, tick, superseded_by}], conflicts: n }
```

**Milestone Placement**

Implement in **Milestone 3 (LTP/LTD)** — the interference check already does
a similar ANN search. Contradiction detection is a natural extension of that pass.
Add `superseded_by` and `belief_version` to schema in **Milestone 1**.

---

### 46.3 Query-by-Example (`--like` / `--unlike`)

**Concept**

Allow using an existing memory's stored vector as the query instead of
re-embedding a text string. `--like <uuid>` finds semantically similar memories.
`--unlike <uuid>` finds the most distant memories — useful for diversity,
counterargument retrieval, and avoiding echo chambers in agent reasoning.

**Schema Changes**

None. Uses existing `memory_embeddings` table.

**Core Logic**

```rust
pub enum RecallQuerySource {
    Text(String),
    LikeMemory(Uuid),
    UnlikeMemory(Uuid),
}

pub struct RecallQuery {
    pub source: RecallQuerySource,
    pub context: Option<String>,
    pub top_k: usize,
    pub min_strength: f32,
    pub filters: RecallFilters,
}

fn resolve_query_vector(source: &RecallQuerySource, store: &BrainStore) -> Vec<f32> {
    match source {
        Text(s) => embed(s),
        LikeMemory(id) => store.get_embedding(id).content_embedding,
        UnlikeMemory(id) => store.get_embedding(id).content_embedding, // sort inverted
    }
}
```

**CLI / MCP**

```bash
membrain recall --like <uuid>             # find similar memories
membrain recall --like <uuid> --top 10
membrain recall --unlike <uuid>           # find most different memories
membrain recall --unlike <uuid> --top 5  # counterexamples
```

```
MCP tool: recall(like_id?: uuid, unlike_id?: uuid, ...)
  → same RetrievalResult shape
```

**Milestone Placement**

Add during **Milestone 4 (3-Tier Retrieval Engine)**. Zero new storage.
One enum variant change to RecallQuery. Very low risk.

---

### 46.4 Context Budget API

**Concept**

Agent calls `context_budget(n_tokens)` with its remaining context window budget.
membrain returns a ranked, deduplicated, pre-formatted list of memories to inject —
scored not just by relevance but by `utility = relevance × strength × (1 − overlap_with_working_memory)`.
Memories already in working memory are penalized. Output is ready-to-inject text.

**Schema Changes**

None. Operates on existing working_memory state + recall pipeline.

**Core Logic**

```rust
pub struct ContextBudgetRequest {
    pub token_budget: usize,
    pub current_context: Option<String>,
    pub working_memory_ids: Vec<Uuid>,
    pub format: InjectionFormat,
}

pub struct ContextBudgetResponse {
    pub injections: Vec<InjectionItem>,
    pub tokens_used: usize,
    pub tokens_remaining: usize,
}

pub struct InjectionItem {
    pub memory_id: Uuid,
    pub content: String,
    pub utility_score: f32,
    pub token_count: usize,
    pub reason: String,
}

impl BrainStore {
    pub fn context_budget(&self, req: ContextBudgetRequest) -> ContextBudgetResponse {
        // 1. Recall top-50 by relevance to current_context
        // 2. Score each: utility = relevance * strength * (1 - wm_overlap_penalty)
        // 3. Sort by utility desc
        // 4. Greedy pack: add items until token_budget exhausted
        // 5. Format output as ready-to-inject string
    }
}

fn wm_overlap_penalty(candidate: &Memory, wm_ids: &[Uuid]) -> f32 {
    if wm_ids.contains(&candidate.id) { 1.0 } else { 0.0 }
}
```

**Token Counting**

Use a simple approximation: `tokens ≈ content.len() / 4`. No tokenizer dependency.
Configurable via `token_chars_ratio` constant.

**CLI / MCP**

```bash
membrain budget --tokens 2000                          # what to inject given 2k token budget
membrain budget --tokens 2000 --context "debugging"   # context-aware
membrain budget --tokens 2000 --format markdown
```

```
MCP tool: context_budget(token_budget, current_context?, working_memory_ids?, format?)
  → { injections: [{memory_id, content, utility_score, token_count, reason}], tokens_used }
```

**Milestone Placement**

Add in **Milestone 9 (Daemon + MCP)** — needs working recall pipeline.
This is a high-value MCP tool that makes membrain directly useful to Claude Code.

---

### 46.5 Temporal Landmark System

**Concept**

Certain memories act as temporal anchors — "project started", "switched stack",
"user mentioned deadline". Landmarks are auto-detected when a memory has high
emotional arousal + high novelty + no similar memory in a recent time window.
They define "eras" that other memories are anchored to, enabling timeline queries.

**Schema Changes**

```sql
ALTER TABLE memories ADD COLUMN is_landmark INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memories ADD COLUMN landmark_label TEXT;
ALTER TABLE memories ADD COLUMN era_id TEXT;

CREATE TABLE landmarks (
  id           TEXT PRIMARY KEY REFERENCES memories(id),
  label        TEXT NOT NULL,
  era_start    INTEGER NOT NULL,
  era_end      INTEGER,
  memory_count INTEGER DEFAULT 0
);

CREATE INDEX idx_memories_era ON memories(era_id);
```

**Core Logic**

```rust
pub struct LandmarkDetector {
    arousal_threshold: f32,   // default: 0.7
    novelty_threshold: f32,   // default: 0.75
    min_era_gap_ticks: u64,   // default: 50
    similarity_floor: f32,    // default: 0.85
}

impl LandmarkDetector {
    pub fn evaluate_on_encode(&self, memory: &Memory, store: &BrainStore) -> bool {
        // 1. Check arousal > threshold AND novelty > threshold
        // 2. Check no existing landmark with similarity > floor in last min_era_gap_ticks
        // 3. If landmark: close current era, open new era, assign label
    }

    fn auto_label(memory: &Memory) -> String {
        memory.content.chars().take(50).collect()
    }
}
```

**CLI / MCP**

```bash
membrain timeline                         # list all landmarks in order
membrain timeline --detail                # landmarks + memory count per era
membrain recall "debugging" --era <id>   # recall within a specific era
membrain landmark <uuid>                  # manually promote memory to landmark
membrain landmark --label "v2 launch" <uuid>
```

```
MCP tool: timeline()
  → { landmarks: [{id, label, era_start, era_end, memory_count}] }

MCP tool: recall(query, era_id?)
  → existing RetrievalResult, filtered to era
```

**Milestone Placement**

Schema columns in **Milestone 1**. Detection logic in **Milestone 2 (Encoding Pipeline)**.
CLI commands in **Milestone 10**. Low risk — is_landmark is just a boolean flag.

---

### 46.6 Passive Observation Mode

**Concept**

membrain reads from stdin pipe or watches a file/directory, automatically segments
the stream into discrete memories using embedding-based topic boundary detection,
and encodes them without explicit `remember()` calls. This eliminates the biggest
adoption friction: agents don't need to be instrumented to call `remember`.

**Schema Changes**

```sql
ALTER TABLE memories ADD COLUMN observation_source TEXT;
ALTER TABLE memories ADD COLUMN observation_chunk_id TEXT;
```

**Core Logic**

```rust
pub struct ObserveConfig {
    chunk_size_chars: usize,      // default: 500
    topic_shift_threshold: f32,   // default: 0.35
    min_chunk_chars: usize,       // default: 50
    overlap_chars: usize,         // default: 50
    default_attention: f32,       // default: 0.6
    context: Option<String>,
}

pub struct ObserveEngine;

impl ObserveEngine {
    pub async fn observe_stream<R: AsyncRead>(
        &self,
        reader: R,
        config: ObserveConfig,
        store: &mut BrainStore,
    ) -> ObserveReport {
        // Rolling window: embed every N chars, compare with prev
        // On shift: encode buffered chunk as new memory
        // Tag with observation_source, observation_chunk_id
    }

    pub async fn watch_directory(
        &self,
        path: &Path,
        config: ObserveConfig,
        store: &mut BrainStore,
    ) {}
}

pub struct ObserveReport {
    pub memories_created: usize,
    pub bytes_processed: usize,
    pub topic_shifts_detected: usize,
    pub duration_ms: u64,
}
```

**Dependencies**

```toml
notify = "6"   # cross-platform file watching (inotify/kqueue/FSEvents)
```

**CLI / MCP**

```bash
# Pipe mode
cat conversation.txt | membrain observe
echo "user prefers dark mode" | membrain observe --context "preferences"
claude --output-format stream 2>&1 | membrain observe --context "coding session"

# Watch mode
membrain observe --watch ~/.claude/conversations/
membrain observe --watch ./logs/ --pattern "*.jsonl"

# Options
membrain observe --chunk-size 300 --topic-threshold 0.4 --context "project-x"
membrain observe --dry-run   # show what would be encoded, don't write
```

```
MCP tool: observe(content, context?, chunk_size?, source_label?)
  → { memories_created: n, topic_shifts: n }
```

**Milestone Placement**

Implement in **Milestone 9 (Daemon + MCP)**. Core observe logic can be added to
`membrain-core` as `engine::observe`. File watching requires `notify` crate — add
to Cargo.toml. `--dry-run` flag makes this safe to test.

---

### 46.7 Memory Confidence Intervals

> Cross-ref: Extends Section 10.9 (Uncertainty Surface) with concrete confidence scoring and corroboration mechanics.

**Concept**

Each memory carries a `confidence: f32` separate from `strength`.
Strength = how consolidated. Confidence = how reliable/certain.
Confidence decreases when a memory is reconsolidated many times (unstable belief)
or conflicts with newer memories. Confidence increases when multiple independent
memories corroborate the same fact. Agents can filter by confidence threshold.

**Schema Changes**

```sql
ALTER TABLE memories ADD COLUMN confidence REAL NOT NULL DEFAULT 1.0;
ALTER TABLE memories ADD COLUMN corroboration_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memories ADD COLUMN reconsolidation_count INTEGER NOT NULL DEFAULT 0;

ALTER TABLE cold_memories ADD COLUMN confidence REAL NOT NULL DEFAULT 1.0;
```

**Core Logic**

```rust
fn update_confidence_on_reconsolidate(m: &mut Memory) {
    m.reconsolidation_count += 1;
    let penalty = 0.05 * (m.reconsolidation_count as f32).sqrt();
    m.confidence = (m.confidence - penalty).max(0.1);
}

fn update_confidence_on_corroborate(m: &mut Memory) {
    m.corroboration_count += 1;
    let bonus = 0.1 / (m.corroboration_count as f32).sqrt();
    m.confidence = (m.confidence + bonus).min(1.0);
}

fn update_confidence_on_conflict(m: &mut Memory) {
    m.confidence = (m.confidence - 0.2).max(0.1);
}

pub struct RecallFilters {
    pub min_confidence: Option<f32>,
    pub min_strength: f32,
}
```

**CLI / MCP**

```bash
membrain recall "user preferences" --min-confidence 0.8
membrain uncertain                   # list memories with confidence < 0.5
membrain uncertain --top 20
membrain inspect <uuid>              # now shows confidence + reconsolidation_count
membrain stats                       # now shows avg_confidence, low_confidence_count
```

```
MCP tool: recall(query, min_confidence?: f32, ...)
  → memories now include confidence field

MCP tool: uncertain(top_k?)
  → { memories: [{id, content, confidence, reconsolidation_count}] }
```

**BrainStats Extension**

```rust
pub struct BrainStats {
    // existing fields...
    pub avg_confidence: f32,
    pub low_confidence_count: usize,
    pub high_confidence_count: usize,
}
```

**Milestone Placement**

Schema columns in **Milestone 1**. Update rules in **Milestone 3 (LTP/LTD)** and
**Milestone 5 (Reconsolidation)**. Corroboration check in **Milestone 2 (Encoding)**.
CLI/MCP surface in **Milestone 10**.

---

### 46.8 Skill Extraction from Episodic Clusters

> Cross-ref: Extends Section 10.4 (Reflection Compiler) with a concrete no-LLM extraction pipeline.

**Concept**

When an engram reaches a member_count threshold, membrain inspects whether the
episodic memories share a consistent pattern (same MemoryKind, overlapping keywords,
similar embedding centroid). If so, it synthesizes a single `Procedural` memory
from the cluster — an abstract skill/pattern distilled from repeated episodes.
No LLM required: uses TF-IDF on cluster members + centroid content.

**Schema Changes**

```sql
ALTER TABLE engrams ADD COLUMN extraction_attempted INTEGER NOT NULL DEFAULT 0;
ALTER TABLE engrams ADD COLUMN extracted_procedural_id TEXT REFERENCES memories(id);

ALTER TABLE memories ADD COLUMN distilled_from_engram TEXT REFERENCES engrams(id);
ALTER TABLE memories ADD COLUMN distilled_at INTEGER;
```

**Core Logic**

```rust
pub struct SkillExtractor {
    min_cluster_size: usize,     // default: 15
    min_episode_consistency: f32, // default: 0.6
    keyword_top_k: usize,        // default: 10
}

impl SkillExtractor {
    pub fn evaluate_engram(&self, engram: &Engram, store: &BrainStore) -> Option<ProceduralDraft> {
        if engram.member_count < self.min_cluster_size { return None; }
        if engram.extraction_attempted != 0 { return None; }

        let members = store.get_engram_members(engram.id);

        // 1. Check kind consistency: most members are Episodic?
        let episodic_ratio = members.iter().filter(|m| m.kind == MemoryKind::Episodic).count()
            as f32 / members.len() as f32;
        if episodic_ratio < 0.7 { return None; }

        // 2. Check centroid coherence: avg pairwise sim > threshold
        // 3. Extract top keywords via TF-IDF across all member contents
        // 4. Synthesize procedural content: "When [context], [action pattern]"
        // 5. Return ProceduralDraft for insertion as MemoryKind::Procedural
    }
}

pub struct ProceduralDraft {
    pub content: String,
    pub keywords: Vec<String>,
    pub source_engram_id: Uuid,
    pub confidence: f32,
}
```

**CLI / MCP**

```bash
membrain skills                          # list all extracted procedural memories
membrain skills --extract                # manually trigger extraction pass
membrain inspect <uuid> --show-source   # if procedural: show source engram
membrain engram <uuid> --extract        # trigger extraction for specific engram
```

```
MCP tool: skills()
  → { procedures: [{id, content, source_engram_id, confidence, member_count}] }

MCP tool: extract_skills()
  → { extracted: n, skipped: n }
```

**Milestone Placement**

Implement after **Milestone 7 (Engram Graph)**. The extraction pass runs as part
of the **Milestone 6 (Consolidation Engine)** background cycle — natural fit
alongside NREM/REM equivalents. Add `extracted_procedural_id` to schema in Milestone 1.

---

### 46.9 Cross-Agent Memory Sharing

> Cross-ref: Extends Section 10.8 (Namespace Lenses / Role Capsules) with concrete multi-agent visibility and sharing mechanics.

**Concept**

Memories can be marked with a visibility level: `Private` (default), `Shared`
(accessible within a namespace), or `Public` (global). Agents in the same namespace
can recall each other's shared memories. One agent learns → shares → all agents know.
Transforms membrain from single-agent memory into collective intelligence layer.

**Schema Changes**

```sql
ALTER TABLE memories ADD COLUMN namespace_id TEXT NOT NULL DEFAULT 'default';
ALTER TABLE memories ADD COLUMN agent_id TEXT NOT NULL DEFAULT 'default';
ALTER TABLE memories ADD COLUMN visibility TEXT NOT NULL DEFAULT 'private';

CREATE TABLE shared_memories (
  memory_id      TEXT PRIMARY KEY,
  namespace_id   TEXT NOT NULL,
  shared_at      INTEGER NOT NULL,
  shared_by      TEXT NOT NULL,
  access_count   INTEGER DEFAULT 0
);

CREATE INDEX idx_shared_ns ON shared_memories(namespace_id);
```

**Core Logic**

```rust
pub struct AgentContext {
    pub agent_id: String,
    pub namespace_id: String,
}

pub trait BrainStore: Send + Sync {
    fn remember(&mut self, input: EncodeInput, ctx: &AgentContext) -> Result<Uuid>;
    fn recall(&self, query: RecallQuery, ctx: &AgentContext) -> Result<RetrievalResult>;
    fn share(&mut self, id: Uuid, namespace: &str) -> Result<()>;
    fn unshare(&mut self, id: Uuid) -> Result<()>;
}

fn namespace_filter(ctx: &AgentContext) -> String {
    format!(
        "(agent_id = '{aid}' OR visibility = 'shared' AND namespace_id = '{ns}' OR visibility = 'public')",
        aid = ctx.agent_id,
        ns = ctx.namespace_id
    )
}
```

**Config**

```toml
[agent]
agent_id     = "default"
namespace_id = "default"
```

**CLI / MCP**

```bash
membrain remember "deploy steps for project-x" --share --namespace project-x
membrain recall "deploy steps" --namespace project-x
membrain share <uuid> --namespace project-x
membrain unshare <uuid>
membrain namespace list
membrain namespace stats project-x
```

```
MCP tool: remember(content, visibility?, namespace_id?, ...)
MCP tool: recall(query, namespace_id?, include_public?, ...)
MCP tool: share(id, namespace_id)
```

**Milestone Placement**

`namespace_id` and `agent_id` columns in **Milestone 1** (critical — cannot add later).
`visibility` column in **Milestone 1** as well. Full sharing API in **Milestone 9**.

---

### 46.10 Brain Health Dashboard

**Concept**

`membrain health` renders a full terminal dashboard: tier utilization, decay curve
health, top engrams, landmark count, conflict count, confidence distribution,
dream engine status, and recent activity. Makes the brain feel alive and debuggable.
`--watch` mode refreshes live. The single best demo/showcase command.

**Schema Changes**

None. Pure read queries over existing tables.

**Core Logic**

```rust
pub struct BrainHealthReport {
    pub hot_memories: usize,
    pub hot_capacity: usize,
    pub cold_memories: usize,
    pub hot_utilization_pct: f32,

    pub avg_strength: f32,
    pub avg_confidence: f32,
    pub low_confidence_count: usize,

    pub decay_rate: f32,
    pub archive_count: usize,

    pub total_engrams: usize,
    pub avg_cluster_size: f32,
    pub top_engrams: Vec<(String, usize)>,

    pub landmark_count: usize,
    pub unresolved_conflicts: usize,
    pub uncertain_count: usize,
    pub dream_links_total: usize,
    pub last_dream_tick: Option<u64>,

    pub total_recalls: u64,
    pub total_encodes: u64,
    pub current_tick: u64,
    pub daemon_uptime_ticks: u64,
}

pub fn render_health_dashboard(report: &BrainHealthReport) -> String {
    // ASCII progress bars, aligned columns
    // Uses only std — no terminal dependency required
    // Color via ANSI escape codes (disabled if NO_COLOR env set)
}
```

**Output Example**

```
╔══════════════════════════════════════════════════════╗
║  membrain — Brain Health                            ║
╠══════════════════════════════════════════════════════╣
║  TIER UTILIZATION                                   ║
║  Hot   [████████████░░░░░░░░] 38k / 50k  76%       ║
║  Cold  [██░░░░░░░░░░░░░░░░░░] 120k        —         ║
╠══════════════════════════════════════════════════════╣
║  QUALITY                                            ║
║  Avg strength    0.71   ▓▓▓▓▓▓▓░░░                 ║
║  Avg confidence  0.84   ▓▓▓▓▓▓▓▓░░                 ║
║  Decay rate      1.2%/1k ticks  ✓ healthy          ║
╠══════════════════════════════════════════════════════╣
║  ENGRAMS                                            ║
║  Total: 312   Avg size: 8.4                        ║
║  Top: [rust-debugging:42] [project-x:31] [prefs:18]║
╠══════════════════════════════════════════════════════╣
║  SIGNALS                                            ║
║  Landmarks       7 temporal anchors                ║
║  Conflicts       3 unresolved ⚠                    ║
║  Uncertain       12 low-confidence memories        ║
║  Dream links     1,847 total  last: 2h ago         ║
╠══════════════════════════════════════════════════════╣
║  ACTIVITY                                           ║
║  Tick: 48,291   Encodes: 2,104   Recalls: 9,871    ║
║  Daemon uptime: 48,291 ticks                       ║
╚══════════════════════════════════════════════════════╝
```

**CLI / MCP**

```bash
membrain health                   # full dashboard, one-shot
membrain health --watch           # refresh every 2 seconds
membrain health --watch --interval 5
membrain health --json            # machine-readable for scripting
membrain health --brief           # one-line summary
```

```
MCP tool: health()
  → BrainHealthReport as JSON
```

**Milestone Placement**

Basic version (tiers + engrams + activity) in **Milestone 6**.
Full version including all feature signals in **Milestone 10**.
`--watch` mode in **Milestone 10**. No dependencies beyond existing queries.

---

### 46.11 Batch 1 Summary Table

| # | Feature | Schema Changes | Milestone | Effort | Section 10 Cross-ref |
|---|---------|---------------|-----------|--------|----------------------|
| 1 | Dream Mode | `dream_links` table | After M7 | Medium | — |
| 2 | Belief Versioning | `superseded_by`, `belief_version`, `belief_chain_id`, `belief_conflicts` | M1 schema + M3 logic | Medium | 10.2 Belief Ledger |
| 3 | Query-by-Example | None | M4 | Very Low | — |
| 4 | Context Budget API | None | M9 | Low | — |
| 5 | Temporal Landmarks | `is_landmark`, `era_id`, `landmarks` table | M1 schema + M2 logic | Low | — |
| 6 | Passive Observation | `observation_source`, `observation_chunk_id` | M9 | Medium | — |
| 7 | Confidence Intervals | `confidence`, `corroboration_count`, `reconsolidation_count` | M1 schema + M3 logic | Low | 10.9 Uncertainty Surface |
| 8 | Skill Extraction | `distilled_from_engram`, `extracted_procedural_id` | After M7 | Medium | 10.4 Reflection Compiler |
| 9 | Cross-Agent Sharing | `namespace_id`, `agent_id`, `visibility`, `shared_memories` | M1 schema (critical) + M9 API | Medium | 10.8 Namespace Lenses |
| 10 | Health Dashboard | None | M6 basic + M10 full | Very Low | — |

### 46.12 Critical M1 Schema Additions (must be in first migration)

These columns must be present in Milestone 1's initial schema to avoid costly ALTER TABLE
migrations later. Features 2, 5, 7, 8, and 9 all require columns that affect query filters
or are referenced by foreign keys across the system.

```sql
-- Feature 2: Belief Versioning
ALTER TABLE memories ADD COLUMN superseded_by TEXT;
ALTER TABLE memories ADD COLUMN belief_version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE memories ADD COLUMN belief_chain_id TEXT;

-- Feature 5: Temporal Landmarks
ALTER TABLE memories ADD COLUMN is_landmark INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memories ADD COLUMN landmark_label TEXT;
ALTER TABLE memories ADD COLUMN era_id TEXT;

-- Feature 7: Confidence Intervals
ALTER TABLE memories ADD COLUMN confidence REAL NOT NULL DEFAULT 1.0;
ALTER TABLE memories ADD COLUMN corroboration_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memories ADD COLUMN reconsolidation_count INTEGER NOT NULL DEFAULT 0;

-- Feature 8: Skill Extraction
ALTER TABLE memories ADD COLUMN distilled_from_engram TEXT;
ALTER TABLE memories ADD COLUMN distilled_at INTEGER;

-- Feature 9: Cross-Agent Sharing (MOST CRITICAL — affects all query filters)
ALTER TABLE memories ADD COLUMN namespace_id TEXT NOT NULL DEFAULT 'default';
ALTER TABLE memories ADD COLUMN agent_id TEXT NOT NULL DEFAULT 'default';
ALTER TABLE memories ADD COLUMN visibility TEXT NOT NULL DEFAULT 'private';

-- Model version (base plan fix)
ALTER TABLE memory_embeddings ADD COLUMN model_version TEXT NOT NULL DEFAULT 'all-MiniLM-L6-v2';
```

---

## 47. Feature Implementation Specs (Batch 2)

> 10 additional implementation-ready feature specs (Features 11–20).
> Each section covers: concept, schema changes, core logic, API/CLI surface, and milestone placement.

---

### 47.1 Causal Chain Tracking ("Why do I believe this?")

**Concept**

When a memory is created *from* another memory — via reconsolidation update,
skill extraction, schema synthesis, or explicit agent reasoning — track the
directed causal provenance: `derived_from: Vec<Uuid>`. This is a typed causality
link. The agent can trace any belief back to its root evidence. If root evidence
is invalidated, the entire derived chain receives an automatic confidence penalty.

**Schema Changes**

```sql
CREATE TABLE causal_links (
  src_memory_id  TEXT NOT NULL REFERENCES memories(id),
  dst_memory_id  TEXT NOT NULL REFERENCES memories(id),
  link_type      TEXT NOT NULL,  -- 'derived'|'reconsolidated'|'extracted'|'inferred'
  created_at     INTEGER NOT NULL,
  agent_id       TEXT,
  PRIMARY KEY (src_memory_id, dst_memory_id)
);

CREATE INDEX idx_causal_src ON causal_links(src_memory_id);
CREATE INDEX idx_causal_dst ON causal_links(dst_memory_id);

ALTER TABLE memories ADD COLUMN has_causal_parents INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memories ADD COLUMN has_causal_children INTEGER NOT NULL DEFAULT 0;
```

**Core Logic**

```rust
pub enum CausalLinkType {
    Derived,
    Reconsolidated,
    Extracted,
    Inferred,
}

pub struct CausalLink {
    pub src: Uuid,
    pub dst: Uuid,
    pub link_type: CausalLinkType,
    pub created_at: u64,
}

impl BrainStore {
    pub fn link_causal(
        &mut self,
        child_id: Uuid,
        parent_ids: &[Uuid],
        link_type: CausalLinkType,
    ) -> Result<()>;

    pub fn trace_causality(&self, id: Uuid) -> Result<CausalTrace>;

    pub fn invalidate_causal_chain(&mut self, root_id: Uuid) -> Result<InvalidationReport>;
}

pub struct CausalTrace {
    pub root_id: Uuid,
    pub chain: Vec<CausalStep>,
    pub depth: usize,
    pub all_roots_valid: bool,
}

pub struct CausalStep {
    pub memory_id: Uuid,
    pub content: String,
    pub link_type: CausalLinkType,
    pub tick: u64,
    pub confidence: f32,
    pub strength: f32,
}

pub struct InvalidationReport {
    pub root_id: Uuid,
    pub chain_length: usize,
    pub memories_penalized: usize,
    pub avg_confidence_delta: f32,
}

fn cascade_penalty(depth: usize) -> f32 {
    match depth {
        1 => 0.20,
        2 => 0.10,
        _ => 0.05,
    }
}
```

**CLI / MCP**

```bash
membrain why <uuid>                    # trace causal chain to root evidence
membrain why <uuid> --depth 5         # limit trace depth
membrain why <uuid> --json            # machine-readable chain

membrain invalidate <uuid>             # mark memory as wrong, cascade penalty
membrain invalidate <uuid> --dry-run  # show what would be penalized

membrain causal-graph <uuid>           # show full forward+backward causal subgraph
```

```
MCP tool: why(id)
  → { chain: [{memory_id, content, link_type, tick, confidence}], depth, all_roots_valid }

MCP tool: invalidate(id, dry_run?)
  → { memories_penalized: n, avg_confidence_delta: f32 }
```

**Milestone Placement**

Schema in **Milestone 1**. `link_causal()` called in **Milestone 5 (Reconsolidation)**
and **Milestone 8 (Skill Extraction)**. `trace_causality()` CLI in **Milestone 10**.
`invalidate_causal_chain()` in **Milestone 8**. Low risk — purely additive.

---

### 47.2 Memory Snapshots + Time Travel Recall

> Cross-ref: Extends Section 10.10 (Deterministic Journal + Doctor + Time Travel) with concrete snapshot-based time travel.

**Concept**

`membrain snapshot --name "before-refactor"` records the current brain state tick
as a named checkpoint — zero data copy, just metadata. Any future recall can be
scoped to that snapshot: memories created after the snapshot tick are excluded,
and effective_strength is recomputed using the snapshot tick as `now`. Allows
querying "what did the agent know at point X in time?"

**Schema Changes**

```sql
CREATE TABLE snapshots (
  name         TEXT PRIMARY KEY,
  tick         INTEGER NOT NULL,
  created_at   INTEGER NOT NULL,
  note         TEXT,
  memory_count INTEGER NOT NULL,
  namespace_id TEXT NOT NULL DEFAULT 'default'
);
```

**Core Logic**

```rust
pub struct Snapshot {
    pub name: String,
    pub tick: u64,
    pub created_at: u64,
    pub note: Option<String>,
    pub memory_count: usize,
    pub namespace_id: String,
}

impl BrainStore {
    pub fn create_snapshot(&self, name: &str, note: Option<&str>) -> Result<Snapshot>;

    pub fn recall_at_snapshot(
        &self,
        query: RecallQuery,
        snapshot_name: &str,
    ) -> Result<RetrievalResult> {
        let snap = self.get_snapshot(snapshot_name)?;
        // Filter: WHERE created_at <= snap.tick AND state != 'Archived'
        // effective_strength computed with delta = snap.tick - last_accessed
    }

    pub fn list_snapshots(&self) -> Result<Vec<Snapshot>>;
    pub fn delete_snapshot(&self, name: &str) -> Result<()>;
}

fn effective_strength_at(m: &Memory, snap_tick: u64) -> f32 {
    let effective_last = m.last_accessed.min(snap_tick);
    let elapsed = snap_tick.saturating_sub(effective_last);
    if m.bypass_decay {
        m.strength
    } else {
        (-(elapsed as f32) / m.stability).exp() * m.strength
    }
}
```

**CLI / MCP**

```bash
membrain snapshot --name before-refactor
membrain snapshot --name v1-launch --note "Day we shipped v1"

membrain snapshot list
membrain snapshot delete before-refactor

membrain recall "architecture decision" --at before-refactor
membrain recall "user preferences" --at v1-launch --top 5 --json

membrain stats --at before-refactor
membrain health --at before-refactor
```

```
MCP tool: snapshot(name, note?)
  → { name, tick, memory_count }

MCP tool: recall(query, at_snapshot?: string, ...)
  → RetrievalResult scoped to snapshot

MCP tool: list_snapshots()
  → { snapshots: [{name, tick, note, memory_count}] }
```

**Milestone Placement**

Schema in **Milestone 1** (zero cost). Core `create_snapshot()` in **Milestone 4 (Retrieval)**.
`recall_at_snapshot()` in **Milestone 4**. CLI in **Milestone 10**. Near-zero risk.

---

### 47.3 Attention Heatmap + Adaptive Cache Pre-warming

**Concept**

Track which memories are actually retrieved across sessions — the full recall event
log with query patterns. Derive two outcomes:
(1) `membrain hot-paths` and `membrain dead-zones` for observability;
(2) automatic Tier1 cache pre-warming on daemon start using historical hot-path data.

**Schema Changes**

```sql
CREATE TABLE recall_log (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  tick            INTEGER NOT NULL,
  query_hash      TEXT NOT NULL,
  query_preview   TEXT,
  retrieved_ids   TEXT NOT NULL,
  tier            TEXT NOT NULL,
  latency_us      INTEGER NOT NULL,
  namespace_id    TEXT NOT NULL DEFAULT 'default'
);

CREATE INDEX idx_recall_tick ON recall_log(tick DESC);
CREATE INDEX idx_recall_query ON recall_log(query_hash);

CREATE TABLE hot_path_cache (
  memory_id       TEXT PRIMARY KEY REFERENCES memories(id),
  retrieve_count  INTEGER NOT NULL DEFAULT 0,
  last_retrieved  INTEGER NOT NULL,
  avg_rank        REAL NOT NULL,
  score           REAL NOT NULL
);
```

**Core Logic**

```rust
pub struct AttentionHeatmap {
    log_cap: usize,           // default: 50_000
    rebuild_interval: u64,    // default: every 500 ticks
    prewarm_top_n: usize,     // default: 256
}

impl AttentionHeatmap {
    pub async fn log_recall_event(&self, event: RecallEvent, store: &BrainStore);
    pub fn rebuild_hot_paths(&self, store: &mut BrainStore) -> HeatmapReport;
    pub async fn prewarm_tier1(&self, store: &mut BrainStore) -> PrewarmReport;
    pub fn hot_paths(&self, top_n: usize, store: &BrainStore) -> Vec<HotPathEntry>;
    pub fn dead_zones(&self, min_age_ticks: u64, store: &BrainStore) -> Vec<DeadZoneEntry>;
}

pub struct RecallEvent {
    pub tick: u64,
    pub query: String,
    pub retrieved_ids: Vec<Uuid>,
    pub tier: RetrievalPath,
    pub latency_us: u64,
    pub namespace_id: String,
}

pub struct HotPathEntry {
    pub memory_id: Uuid,
    pub content_preview: String,
    pub retrieve_count: usize,
    pub avg_rank: f32,
    pub score: f32,
}

pub struct DeadZoneEntry {
    pub memory_id: Uuid,
    pub content_preview: String,
    pub age_ticks: u64,
    pub strength: f32,
    pub retrieve_count: usize,
}

pub struct PrewarmReport {
    pub loaded: usize,
    pub skipped_archived: usize,
    pub duration_ms: u64,
}
```

**CLI / MCP**

```bash
membrain hot-paths                     # top 20 most-retrieved memories
membrain hot-paths --top 50 --json
membrain dead-zones                    # memories never retrieved since encoding
membrain dead-zones --min-age 1000
membrain dead-zones --forget-all      # archive all dead zones (with confirmation)

membrain recall-patterns
membrain heatmap --since 5000
```

```
MCP tool: hot_paths(top_n?)
  → { entries: [{memory_id, content_preview, retrieve_count, score}] }

MCP tool: dead_zones(min_age_ticks?)
  → { entries: [{memory_id, content_preview, age_ticks, strength}] }
```

**Milestone Placement**

Schema in **Milestone 1**. Logging in **Milestone 4 (Retrieval Engine)**.
`rebuild_hot_paths()` in **Milestone 6 (Consolidation)**. `prewarm_tier1()` in
**Milestone 9 (Daemon)**. CLI in **Milestone 10**.

---

### 47.4 Semantic Diff ("What changed in my brain?")

**Concept**

`membrain diff --since <snapshot_or_tick>` produces a human-readable semantic
summary of brain changes between two points: new beliefs, strengthened memories,
archived memories, resolved conflicts, and newly formed engrams. Requires the
Snapshot system (Feature 12 / 47.2) for named checkpoints, but also works with
raw tick numbers.

**Schema Changes**

None beyond snapshots (47.2). Diff is computed from existing tables using tick-range queries.

**Core Logic**

```rust
pub struct DiffRequest {
    pub since: DiffAnchor,
    pub until: DiffAnchor,
    pub namespace_id: String,
    pub top_n: usize,
}

pub enum DiffAnchor {
    SnapshotName(String),
    Tick(u64),
    Current,
}

pub struct BrainDiff {
    pub since_tick: u64,
    pub until_tick: u64,
    pub new_memories: Vec<DiffEntry>,
    pub strengthened: Vec<DiffEntry>,
    pub weakened: Vec<DiffEntry>,
    pub archived: Vec<DiffEntry>,
    pub conflicts_resolved: usize,
    pub new_engrams: Vec<EngramDiff>,
    pub landmarks_added: Vec<DiffEntry>,
    pub skills_extracted: usize,
    pub dream_links_added: usize,
}

pub struct DiffEntry {
    pub memory_id: Uuid,
    pub content_preview: String,
    pub before_strength: Option<f32>,
    pub after_strength: Option<f32>,
    pub delta: f32,
    pub kind: MemoryKind,
}

pub struct EngramDiff {
    pub engram_id: Uuid,
    pub member_count: usize,
    pub top_preview: String,
}
```

**Output Example**

```
BRAIN DIFF  tick 4000 → 8291  (4291 ticks)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

NEW BELIEFS  (+47)
  + [Semantic] Switched architecture to microservices
  + [Episodic] User prefers async patterns over callbacks
  + [Procedural] Deploy workflow: build → test → stage → prod
  ... and 44 more

STRENGTHENED  (12 memories recalled 5+ times)
  ↑ "Rust borrow checker rules"        0.41 → 0.89  (+0.48)
  ↑ "User prefers concise responses"   0.55 → 0.91  (+0.36)

WEAKENED / ARCHIVED  (23)
  ↓ "Old Python scraper approach"      0.62 → archived
  ↓ "Initial monolith design"          0.71 → archived

CONFLICTS RESOLVED  (3)
  ✓ dark-mode vs light-mode    → dark mode preferred
  ✓ sync vs async preference   → async confirmed

NEW ENGRAMS  (2)
  ✦ "deployment-workflow"  18 members
  ✦ "user-communication"   11 members

LANDMARKS ADDED  (1)
  ⚑ "Switched to microservices"  tick 6201

SKILLS EXTRACTED  2  |  DREAM LINKS ADDED  847
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**CLI / MCP**

```bash
membrain diff --since before-refactor
membrain diff --since 4000
membrain diff --since before-refactor --until v1-launch
membrain diff --since 4000 --top 5
membrain diff --since before-refactor --json
membrain diff --since before-refactor --brief
```

```
MCP tool: diff(since, until?, top_n?)
  → BrainDiff as structured JSON
```

**Milestone Placement**

Raw tick diff works from **Milestone 3** onwards. Full diff with all categories in
**Milestone 10**. Audit log (Feature 19 / 47.9) strengthens the strength-delta queries.

---

### 47.5 Fork + Merge Brain States

**Concept**

`membrain fork --name agent-b` creates a new brain namespace that inherits all
`public` (and optionally `shared`) memories from the parent — by reference, not
by copy. The fork gets its own private namespace for new writes. Forks can diverge
independently. `membrain merge agent-b --into default` harvests new memories from
the fork back into the parent, using the belief versioning system (Feature 2 / 46.2)
to handle conflicts. Analogous to git branch + merge for memory state.

**Schema Changes**

```sql
CREATE TABLE brain_forks (
  name                TEXT PRIMARY KEY,
  parent_namespace    TEXT NOT NULL,
  forked_at_tick      INTEGER NOT NULL,
  inherited_vis       TEXT NOT NULL DEFAULT 'public',
  status              TEXT NOT NULL DEFAULT 'active',
  merged_at_tick      INTEGER,
  note                TEXT
);

CREATE TABLE fork_merge_log (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  fork_name       TEXT NOT NULL,
  target_ns       TEXT NOT NULL,
  tick            INTEGER NOT NULL,
  memories_merged INTEGER NOT NULL,
  conflicts_found INTEGER NOT NULL,
  conflicts_auto_resolved INTEGER NOT NULL,
  conflicts_pending INTEGER NOT NULL
);
```

**Core Logic**

```rust
pub struct ForkConfig {
    pub name: String,
    pub parent_namespace: String,
    pub inherit_visibility: ForkInheritance,
    pub note: Option<String>,
}

pub enum ForkInheritance {
    PublicOnly,
    SharedToo,
    All,
}

pub struct MergeConfig {
    pub fork_name: String,
    pub target_namespace: String,
    pub conflict_strategy: ConflictStrategy,
    pub dry_run: bool,
}

pub enum ConflictStrategy {
    ForkWins,
    ParentWins,
    RecencyWins,
    Manual,
}

impl BrainStore {
    pub fn fork(&mut self, config: ForkConfig) -> Result<ForkInfo>;
    pub fn merge_fork(&mut self, config: MergeConfig) -> Result<MergeReport>;
}

pub struct MergeReport {
    pub memories_merged: usize,
    pub conflicts_found: usize,
    pub conflicts_auto_resolved: usize,
    pub conflicts_pending: usize,
    pub engrams_merged: usize,
}
```

**CLI / MCP**

```bash
membrain fork --name agent-specialist --inherit public
membrain fork --name experiment --inherit shared --note "testing new approach"

membrain fork list
membrain fork status agent-specialist

membrain merge agent-specialist --into default
membrain merge agent-specialist --into default --conflict recency-wins
membrain merge agent-specialist --into default --dry-run

membrain fork abandon experiment
```

```
MCP tool: fork(name, parent_namespace?, inherit?, note?)
  → { name, forked_at_tick, inherited_count }

MCP tool: merge_fork(fork_name, target_namespace, conflict_strategy?, dry_run?)
  → MergeReport
```

**Milestone Placement**

Depends on namespace system (Feature 9 / 46.9). Schema in **Milestone 1**.
Core fork/merge logic in **Milestone 9**. Requires belief versioning (46.2) for conflict handling.

---

### 47.6 Predictive Pre-recall (Speculative Execution)

**Concept**

Track sequential recall patterns within sessions: if query A is frequently followed
by query B, learn this association. When A is recalled, proactively embed and
pre-load the predicted B results into Tier1 cache — before the agent asks.
Analogous to CPU branch prediction applied to memory retrieval.

**Schema Changes**

```sql
CREATE TABLE recall_sequences (
  query_hash_a   TEXT NOT NULL,
  query_hash_b   TEXT NOT NULL,
  count          INTEGER NOT NULL DEFAULT 1,
  last_seen      INTEGER NOT NULL,
  avg_gap_ticks  REAL NOT NULL DEFAULT 10.0,
  namespace_id   TEXT NOT NULL DEFAULT 'default',
  PRIMARY KEY (query_hash_a, query_hash_b, namespace_id)
);

CREATE INDEX idx_seq_a ON recall_sequences(query_hash_a, namespace_id);
```

**Core Logic**

```rust
pub struct PredictiveEngine {
    min_count_threshold: usize,   // default: 3
    max_gap_ticks: u64,           // default: 50
    top_predictions: usize,       // default: 3
    prewarm_top_k: usize,         // default: 5
}

impl PredictiveEngine {
    pub async fn record_transition(
        &self,
        prev_query_hash: u64,
        current_query_hash: u64,
        gap_ticks: u64,
        store: &BrainStore,
    );

    pub async fn trigger_prewarm(
        &self,
        query_hash: u64,
        store: &mut BrainStore,
        tier1: &mut LruCache<u64, Vec<ScoredMemory>>,
    ) -> PredictionResult;
}

pub struct PredictionResult {
    pub predictions: Vec<PredictedQuery>,
    pub prewarmed: usize,
}

pub struct PredictedQuery {
    pub query_hash: u64,
    pub confidence: f32,
    pub prewarmed_ids: Vec<Uuid>,
}
```

**Config**

```toml
[predictive]
enabled              = true
min_count_threshold  = 3
max_gap_ticks        = 50
top_predictions      = 3
prewarm_top_k        = 5
```

**CLI / MCP**

```bash
membrain recall-patterns               # show learned A→B sequences
membrain recall-patterns --top 20
membrain recall-patterns --reset       # clear learned sequences

membrain stats                         # now includes prewarm_hit_rate
membrain health                        # shows predictive cache hit/miss
```

**Milestone Placement**

Depends on recall_log (47.3, Milestone 4). Core sequence tracking in **Milestone 6**.
Async pre-warming in **Milestone 9 (Daemon)**. Stats in **Milestone 10**.

---

### 47.7 Memory Schema Compression

**Concept**

When many episodic memories accumulate around the same *type of situation*
(not necessarily the same engram), membrain runs a schema extraction pass
that synthesizes them into a single `Schema` memory. Original episodics have
their strength reduced (not deleted). The schema memory bypasses decay.
Distinct from Skill Extraction (46.8) which targets action patterns within a
single engram — Schema Compression operates across engrams and targets
*situation patterns*.

**Schema Changes**

```sql
ALTER TABLE memories ADD COLUMN compressed_into TEXT REFERENCES memories(id);
ALTER TABLE memories ADD COLUMN compression_tick INTEGER;

CREATE TABLE compression_log (
  id                  INTEGER PRIMARY KEY AUTOINCREMENT,
  schema_memory_id    TEXT NOT NULL REFERENCES memories(id),
  source_memory_count INTEGER NOT NULL,
  tick                INTEGER NOT NULL,
  namespace_id        TEXT NOT NULL DEFAULT 'default',
  keyword_summary     TEXT
);
```

**Core Logic**

```rust
pub struct SchemaCompressor {
    min_episode_count: usize,       // default: 20
    centroid_coherence_min: f32,    // default: 0.55
    strength_reduction_factor: f32, // default: 0.5
    keyword_top_k: usize,           // default: 15
    min_keyword_frequency: f32,     // default: 0.4
}

impl SchemaCompressor {
    pub fn run_compression_pass(&self, store: &mut BrainStore) -> CompressionReport;
    fn find_compressible_clusters(&self, store: &BrainStore) -> Vec<EpisodicCluster>;
    fn synthesize_schema(&self, cluster: &EpisodicCluster, store: &BrainStore) -> Result<SchemaDraft>;
    fn apply_compression(&self, draft: SchemaDraft, store: &mut BrainStore) -> Result<Uuid>;
}

pub struct EpisodicCluster {
    pub representative_ids: Vec<Uuid>,
    pub centroid_embedding: Vec<f32>,
    pub coherence_score: f32,
    pub dominant_keywords: Vec<String>,
}

pub struct SchemaDraft {
    pub content: String,
    pub keywords: Vec<String>,
    pub source_ids: Vec<Uuid>,
    pub confidence: f32,
    pub strength: f32,
}

pub struct CompressionReport {
    pub schemas_created: usize,
    pub episodes_compressed: usize,
    pub storage_reduction_pct: f32,
}
```

**Compression Trigger Conditions**

```
Trigger compression pass when:
  - consolidation cycle runs AND
  - total episodic memories > SOFT_CAP * 0.7 AND
  - at least one cluster has > min_episode_count members

Per-cluster trigger:
  - member_count > min_episode_count
  - centroid_coherence > centroid_coherence_min
  - not already compressed (compressed_into IS NULL)
  - majority MemoryKind::Episodic
```

**CLI / MCP**

```bash
membrain compress                      # manually trigger compression pass
membrain compress --dry-run           # show what would be compressed
membrain schemas                       # list all schema memories
membrain schemas --top 10
membrain inspect <uuid> --show-source
membrain uncompress <schema-uuid>     # restore strength to source episodes
```

```
MCP tool: compress(dry_run?)
  → { schemas_created, episodes_compressed, storage_reduction_pct }

MCP tool: schemas(top_n?)
  → { schemas: [{id, content, source_count, confidence, keywords}] }
```

**Milestone Placement**

Implement during **Milestone 6 (Consolidation Engine)** as a compression sub-pass.
Add `compressed_into` column to schema in **Milestone 1**.

---

### 47.8 Emotional Trajectory Tracking

**Concept**

Track the aggregate emotional state (valence + arousal) of memories encoded per era.
Creates a mood timeline across sessions. Two outcomes:
(1) `membrain mood` for observability;
(2) optional mood-congruent retrieval boost — memories encoded in a similar emotional
state are ranked slightly higher, mirroring state-dependent memory in neuroscience.

**Schema Changes**

```sql
CREATE TABLE emotional_timeline (
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  era_id         TEXT,
  tick_start     INTEGER NOT NULL,
  tick_end       INTEGER,
  avg_valence    REAL NOT NULL,
  avg_arousal    REAL NOT NULL,
  memory_count   INTEGER NOT NULL DEFAULT 0,
  namespace_id   TEXT NOT NULL DEFAULT 'default'
);

CREATE INDEX idx_emotion_tick ON emotional_timeline(tick_start);

ALTER TABLE memories ADD COLUMN encoding_valence REAL;
ALTER TABLE memories ADD COLUMN encoding_arousal REAL;
```

**Core Logic**

```rust
pub struct EmotionalTracker {
    window_size_ticks: u64,         // default: 500
    congruence_weight: f32,         // default: 0.1
    congruence_enabled: bool,       // default: false (opt-in)
}

impl EmotionalTracker {
    pub fn snapshot_encoding_mood(&self, store: &BrainStore) -> MoodSnapshot;
    pub fn emit_timeline_row(&self, store: &mut BrainStore) -> Result<()>;
    pub fn mood_congruence_score(
        &self,
        candidate: &Memory,
        current_mood: &MoodSnapshot,
    ) -> f32;
}

pub struct MoodSnapshot {
    pub avg_valence: f32,
    pub avg_arousal: f32,
    pub sample_size: usize,
    pub tick: u64,
}

pub enum MoodState {
    CalmPositive,     // valence > 0.2, arousal < 0.4
    ActivePositive,   // valence > 0.2, arousal >= 0.4
    CalmNegative,     // valence < -0.2, arousal < 0.4
    Stressed,         // valence < -0.2, arousal >= 0.6
    Neutral,          // |valence| <= 0.2
}
```

**CLI / MCP**

```bash
membrain mood                          # current mood state
membrain mood --history               # full timeline
membrain mood --history --since 5000
membrain mood --history --json

membrain recall "debugging session" --mood-congruent
```

```
MCP tool: mood_history(since_tick?, namespace_id?)
  → { timeline: [{tick_start, tick_end, avg_valence, avg_arousal, state, memory_count}] }

MCP tool: recall(query, mood_congruent?: bool, ...)
  → existing RetrievalResult with mood_boost_applied flag
```

**Milestone Placement**

Schema columns in **Milestone 1**. `snapshot_encoding_mood()` in **Milestone 2 (Encoding)**.
`emit_timeline_row()` in **Milestone 6 (Consolidation)**. Mood-congruent retrieval in
**Milestone 4** as opt-in. CLI in **Milestone 10**.

---

### 47.9 Write-Ahead Memory Audit Log

**Concept**

Every mutation on the memory system — encode, recall (LTP), strengthen, archive,
reconsolidate, forget, interference penalty, dream link creation — is appended to
an immutable audit log for forensics: "why does this memory have strength 0.2
when it started at 0.8?" `membrain audit <uuid>` produces a full operation history.

**Schema Changes**

```sql
CREATE TABLE memory_audit_log (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  op              TEXT NOT NULL,
  memory_id       TEXT NOT NULL,
  tick            INTEGER NOT NULL,
  before_strength REAL,
  after_strength  REAL,
  before_conf     REAL,
  after_conf      REAL,
  triggered_by    TEXT NOT NULL,
  delta_note      TEXT,
  namespace_id    TEXT NOT NULL DEFAULT 'default'
);

CREATE INDEX idx_audit_memory ON memory_audit_log(memory_id, tick);
CREATE INDEX idx_audit_tick   ON memory_audit_log(tick DESC);
CREATE INDEX idx_audit_op     ON memory_audit_log(op);
```

**Core Logic**

```rust
pub struct AuditLogger {
    max_rows: usize,         // default: 200_000
    purge_batch: usize,      // default: 10_000
    enabled: bool,           // default: true
}

pub struct AuditEntry {
    pub op: AuditOp,
    pub memory_id: Uuid,
    pub tick: u64,
    pub before_strength: Option<f32>,
    pub after_strength: Option<f32>,
    pub before_confidence: Option<f32>,
    pub after_confidence: Option<f32>,
    pub triggered_by: AuditTrigger,
    pub delta_note: Option<String>,
    pub namespace_id: String,
}

pub enum AuditOp {
    Encode, Recall, Strengthen, Weaken, Archive, Reconsolidate,
    Interference, Consolidate, DreamLink, Invalidate, Compress,
}

pub enum AuditTrigger {
    User, Agent, Decay, Interference, Consolidation, Dream, Ltp, System,
}

impl AuditLogger {
    pub fn log(&self, entry: AuditEntry, db: &Connection) -> Result<()>;
    pub fn maybe_purge(&self, db: &Connection) -> Result<()>;
}

impl BrainStore {
    pub fn audit_memory(&self, id: Uuid) -> Result<Vec<AuditEntry>>;
    pub fn audit_range(&self, since_tick: u64, op: Option<AuditOp>) -> Result<Vec<AuditEntry>>;
}
```

**CLI / MCP**

```bash
membrain audit <uuid>                       # full history for one memory
membrain audit <uuid> --since 5000
membrain audit <uuid> --op recall

membrain audit --since 5000 --op archive
membrain audit --recent 100
membrain audit --json
```

```
MCP tool: audit(memory_id?, since_tick?, op?, limit?)
  → { entries: [{op, memory_id, tick, before_strength, after_strength, triggered_by, note}] }
```

**Milestone Placement**

Schema in **Milestone 1**. `AuditLogger` integration in **Milestone 2** (encode path)
and **Milestone 3** (LTP/LTD, decay, interference). By **Milestone 6**, all mutation
paths instrumented. CLI in **Milestone 10**. Overhead: one INSERT per mutation, ~microseconds.

---

### 47.10 Query Intent Classification + Auto-routing

**Concept**

`membrain ask "natural language question"` automatically classifies query intent
using fast keyword pattern matching (no LLM, no ML model) and routes to the
appropriate retrieval configuration. One unified entry point that does the right thing.

**Schema Changes**

None.

**Core Logic**

```rust
pub enum QueryIntent {
    SemanticBroad,        // "what do I know about X"
    ExistenceCheck,       // "did I / have I / do I know"
    RecentFirst,          // "recently / lately / today / last time"
    StrengthWeighted,     // "important / critical / key / essential"
    UncertaintyFocused,   // "uncertain / not sure / might / possibly"
    CausalTrace,          // "why do I believe / how did I learn / origin of"
    TemporalAnchor,       // "before X / after X / when did"
    DiverseSample,        // "different / varied / alternatives / counterexample"
    ProceduralLookup,     // "how to / steps for / procedure"
    EmotionalFilter,      // "worried / frustrated / excited about"
}

pub struct IntentClassifier;

impl IntentClassifier {
    pub fn classify(query: &str) -> (QueryIntent, f32);

    pub fn intent_to_recall_config(intent: QueryIntent, query: &str) -> RecallQuery {
        match intent {
            SemanticBroad       => RecallQuery { top_k: 10, min_strength: 0.1, ..default() },
            ExistenceCheck      => RecallQuery { top_k: 1, min_strength: 0.05, include_archived: true, ..default() },
            RecentFirst         => RecallQuery { top_k: 5, sort_by: SortBy::Recency, ..default() },
            StrengthWeighted    => RecallQuery { top_k: 5, min_strength: 0.6, sort_by: SortBy::Strength, ..default() },
            UncertaintyFocused  => RecallQuery { top_k: 10, max_confidence: Some(0.5), ..default() },
            CausalTrace         => RecallQuery { top_k: 1, follow_causal: true, ..default() },
            ProceduralLookup    => RecallQuery { top_k: 5, kind_filter: Some(MemoryKind::Procedural), ..default() },
            DiverseSample       => RecallQuery { top_k: 10, diversity_penalty: 0.3, ..default() },
            TemporalAnchor      => RecallQuery { top_k: 5, era_filter: parse_era(query), ..default() },
            EmotionalFilter     => RecallQuery { top_k: 10, mood_congruent: true, ..default() },
        }
    }
}

const PATTERNS: &[(&str, QueryIntent)] = &[
    ("did i|have i|do i know|have i ever|did i ever", ExistenceCheck),
    ("why do i believe|how did i learn|where did i|origin of", CausalTrace),
    ("how to|steps for|procedure for|workflow for", ProceduralLookup),
    ("recently|lately|today|last time|most recent", RecentFirst),
    ("important|critical|key|essential|must know", StrengthWeighted),
    ("uncertain|not sure|might|possibly|unsure", UncertaintyFocused),
    ("before|after|when did|timeline|era", TemporalAnchor),
    ("different|varied|alternative|counterexample|unlike", DiverseSample),
    ("worried|frustrated|stressed|excited|anxious", EmotionalFilter),
];
```

**Response Formatting by Intent**

```rust
pub fn format_ask_response(result: RetrievalResult, intent: QueryIntent) -> String {
    match intent {
        ExistenceCheck   => format_existence(result),
        CausalTrace      => format_causal_chain(result),
        ProceduralLookup => format_steps(result),
        _                => format_standard(result),
    }
}
```

**CLI / MCP**

```bash
membrain ask "what do I know about Rust lifetimes?"
membrain ask "did I ever encounter a borrow checker error with async?"
membrain ask "what's most important about the deploy process?"
membrain ask "what was I uncertain about last week?"
membrain ask "why do I believe microservices are better here?"
membrain ask "how to deploy the service?"

membrain ask "..." --explain-intent     # show classified intent + config used
membrain ask "..." --override-intent semantic-broad
```

```
MCP tool: ask(query, explain_intent?)
  → {
      intent: string,
      intent_confidence: f32,
      result: RetrievalResult,
      formatted_response: string
    }
```

**Milestone Placement**

Fully self-contained — no new storage. Implement as a thin wrapper in **Milestone 9 (MCP)**
or **Milestone 10 (CLI Polish)**. `ask` becomes the primary MCP tool agents use,
with `remember/recall/forget` as power-user tools.

---

### 47.11 Batch 2 Summary Table

| #  | Feature | Schema Changes | Key Dependency | Milestone | Effort |
|----|---------|---------------|----------------|-----------|--------|
| 11 | Causal Chain Tracking | `causal_links` table, 2 columns | Reconsolidation (M5) | M5+M10 | Low |
| 12 | Snapshots + Time Travel | `snapshots` table | None | M4 | Very Low |
| 13 | Attention Heatmap | `recall_log`, `hot_path_cache` | Retrieval (M4) | M4+M9 | Low |
| 14 | Semantic Diff | None (uses snapshots) | Feature 12 | M4+M10 | Low |
| 15 | Fork + Merge | `brain_forks`, `fork_merge_log` | Namespace (46.9) | M9 | Medium |
| 16 | Predictive Pre-recall | `recall_sequences` | Feature 13 | M6+M9 | Low |
| 17 | Schema Compression | `compression_log`, 2 columns | Engrams (M7) | M6 | Medium |
| 18 | Emotional Trajectory | `emotional_timeline`, 2 columns | Encoding (M2) | M2+M6 | Low |
| 19 | Write-Ahead Audit Log | `memory_audit_log` | None | M2 | Very Low |
| 20 | Query Intent Routing | None | Full retrieval stack | M9/M10 | Very Low |

### 47.12 Critical M1 Schema Additions (Batch 2)

All columns that must be present from the first migration:

```sql
-- Feature 11: Causal tracking
ALTER TABLE memories ADD COLUMN has_causal_parents  INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memories ADD COLUMN has_causal_children INTEGER NOT NULL DEFAULT 0;

-- Feature 17: Compression
ALTER TABLE memories ADD COLUMN compressed_into   TEXT REFERENCES memories(id);
ALTER TABLE memories ADD COLUMN compression_tick  INTEGER;

-- Feature 18: Emotional trajectory
ALTER TABLE memories ADD COLUMN encoding_valence REAL;
ALTER TABLE memories ADD COLUMN encoding_arousal REAL;

-- New tables (safe to create at M1, used later)
CREATE TABLE causal_links ( ... );       -- Feature 11
CREATE TABLE snapshots ( ... );          -- Feature 12
CREATE TABLE recall_log ( ... );         -- Feature 13
CREATE TABLE hot_path_cache ( ... );     -- Feature 13
CREATE TABLE recall_sequences ( ... );   -- Feature 16
CREATE TABLE compression_log ( ... );    -- Feature 17
CREATE TABLE emotional_timeline ( ... ); -- Feature 18
CREATE TABLE memory_audit_log ( ... );   -- Feature 19
CREATE TABLE brain_forks ( ... );        -- Feature 15
CREATE TABLE fork_merge_log ( ... );     -- Feature 15
```

> Full CREATE TABLE statements are in each feature section above.
> Tables created at M1 but unused until their respective milestone are zero-cost.
