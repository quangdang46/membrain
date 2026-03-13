# membrain — Neuro Mapping

> Canonical source: PLAN.md Section 2 (Human Brain Analysis), Section 4 (Port to membrain).
> This maps human-memory-inspired functions into computational primitives.
> The goal is **functional translation** with performance constraints, not biological imitation.

## Brain Region Mapping

| Brain Region | Function | membrain Port |
|-------------|----------|---------------|
| **Hippocampus** | Index network — pointers to content, episodic formation, pattern completion from partial cue | `hot_store` (SQLite WAL) — fast index, episodic events. USearch HNSW hot index (~50k vectors in RAM, AVX2 SIMD) |
| **Neocortex** | Actual content storage, long-term semantic memory, no hippocampus needed after full consolidation | `cold_store` (SQLite + USearch mmap) — unlimited disk scale. int8 quantized vectors for speed. OS page cache |
| **Amygdala** | Emotional tagging, importance marking, strengthens consolidation when arousal is high | `EmotionalTag { valence, arousal }` — high arousal → `bypass_decay=true`, strength multiplier on encode |
| **Prefrontal Cortex** | Working memory, executive attention, 7±2 slots | `WorkingMemory` — 7-slot VecDeque, attention scoring, eviction → hot_store encode |
| **Cerebellum** | Procedural memory, motor learning, automaticity | `MemoryKind::Procedural` — extracted from episodic clusters (Feature 8), bypasses decay |

---

## 1. Unlimited Capacity

**Brain**: ~100T synaptic connections, ~2.5PB equivalent capacity.

**membrain**:
- Durable metadata and text retrieval in SQLite
- Lexical recall via SQLite FTS5
- Semantic recall via USearch hot/cold HNSW indexes
- Cold storage is disk-backed and mmap-friendly
- Architecture is disk-bounded, not RAM-bounded

---

## 2. Dual-Path Fast/Slow Retrieval

**Brain**: Fast path (neocortex familiarity/pattern matching) + Slow path (hippocampal reconstruction + cluster expansion).

**membrain**:

| Path | Mechanism | Latency |
|------|-----------|---------|
| **Fast** | Tier1 in-memory LRU cache, exact key lookups, working-set hits | <0.1ms |
| **Slow** | SQLite pre-filter → FTS5 lexical → USearch ANN → local reranker → engram BFS expansion → context re-ranking | <5ms (hot), <50ms (cold) |
| **Bridge** | If fast path is confident → return immediately; otherwise escalate to hybrid. Successful slow results update higher tiers | Adaptive |

---

## 3. LTP / LTD Engine (Long-Term Potentiation / Depression)

**Brain**: Recall → synapse strengthen → easier to fire again. Non-use → synapse weaken → harder to recall.

**membrain**:

```
on_recall(id):
  strength = min(strength + LTP_DELTA, MAX_STRENGTH)
  stability += STABILITY_INCREMENT × stability
  last_accessed = now_tick
  access_count += 1
  state = Labile { window: reconsolidation_window(age) }
  spread partial LTP to depth-1 engram neighbors (resonance)

effective_strength(memory, tick):
  if bypass_decay: return base_strength
  Δtick = tick - last_accessed
  return base_strength × e^(-Δtick / stability)
```

Key property: stability doubles after ~3.8 recalls — exponential memory strengthening.

---

## 4. Encoding Pipeline

**Brain**: Attention → Sensory register → Working memory → Encoding → LTM. With novelty detection, emotion tagging, context binding.

**membrain**:

```
encode(input, context, attention, emotional):
  1. attention_score < THRESHOLD → discard
  2. embedding = fastembed(input)
  3. context_embedding = fastembed(context)
  4. novelty_score = 1.0 - max_cosine_sim(embedding, existing)
  5. emotional_tag = { valence, arousal }
  6. initial_strength = BASE × novelty_mod × attention_mod × emotional_mod
  7. bypass_decay = arousal > AROUSAL_THRESHOLD && |valence| > VALENCE_THRESHOLD
  8. state = Labile
  9. INSERT into hot_store
  10. interference_check → weaken similar older memories
  11. engram_builder.try_cluster(new_memory)
  12. landmark_detector.evaluate(memory)  [Feature 5]
  13. confidence = 1.0, corroboration check  [Feature 7]
  14. snapshot_encoding_mood()  [Feature 18]
  15. audit_log(Encode)  [Feature 19]
```

---

## 5. Consolidation (Sleep Cycles)

**Brain**: Synaptic consolidation (~6h), Systems consolidation (days→years, hippocampus → neocortex), Sleep NREM (replay + migrate episodic→semantic).

**membrain** (event-triggered, not time-based):

| Phase | Brain Analog | Implementation |
|-------|-------------|----------------|
| **NREM** | Replay + migrate | Score hot memories → extract semantic patterns → cold_store.upsert → mark Consolidated |
| **REM** | Emotional processing + cross-links | Queue emotional memories → reduce emotional_weight (desensitization) → create cross-links in engram graph |
| **Homeostasis** | Sleep downscaling | If hot_store > MAX_LOAD → bulk_scale(HOMEOSTASIS_FACTOR) → archive strength < MIN |
| **Dream** (Feature 1) | REM association | Scan for similar but unlinked memories → create dream_links → trigger engram merges |
| **Compression** (Feature 17) | Schema formation | Synthesize abstract patterns from repeated episodes → create Schema memories |
| **Skill Extraction** (Feature 8) | Procedural memory | TF-IDF on mature engram clusters → synthesize Procedural memories |

**Triggers**: hot_store.len() > capacity, total_strength > pressure, explicit call, or idle threshold (dream).

---

## 6. Reconsolidation

**Brain**: Stable memory → recall → labile → mutable → reconsolidate (or update).

**membrain**:

```
recall → memory.state = Labile { since: now, window: reconsolidation_window(age) }

reconsolidation_window(age):
  base × (1.0 / (1.0 + age_in_days / 30.0))  // older = shorter window

reconsolidation_tick():
  for each Labile memory where window expired:
    if pending_update:
      content = merge(content, pending_update)
      embedding = re_embed(content)
      strength += RECONSOLIDATION_BONUS
      link_causal(updated_id, [original_id], Reconsolidated)  [Feature 11]
    state = Stable
```

---

## 7. Active Forgetting Engine

**Brain**: Don't forget randomly — remove non-predictive information. Signal/noise optimization. Sleep homeostasis.

**membrain**:

| Phase | Mechanism |
|-------|-----------|
| **Decay pruning** | `WHERE strength < MIN_STRENGTH AND NOT bypass_decay` → archive |
| **Interference resolution** | Find similar pairs (sim 0.7–0.99) → weaken older one |
| **Predictive pruning** | `WHERE access_count = 0 AND age > OLD_THRESHOLD` → strength × NON_PREDICTIVE_DECAY |
| **Capacity management** | If total > SOFT_CAP → sort by (strength × recency × emotional_weight) → archive bottom percentile |

---

## 8. Attention and Salience

**Brain**: Central executive coordinates attention, allocates resources to relevant stimuli.

**membrain**:
- `attention_score` gates encoding (below 0.2 → discard)
- `salience` influences retrieval ranking
- Working memory maintains 7-slot attention buffer
- `focus(id)` boosts attention score for executive control
- Context Budget API (Feature 4) manages attention-weighted injection into agent context

---

## 9. Engram Graph & Associative Recall

**Brain**: Engram = sparse distributed representation, pattern completion. One cue → activate cluster → reconstruction.

**membrain**:

```
Engram {
  id, memory_ids, centroid_embedding, formation_context, strength
}

Encoding:
  similar_engrams = engram_index.search(embedding, top=3)
  if max_sim > CLUSTER_THRESHOLD → add to existing, update centroid
  else → create new engram

Associative recall:
  1. Vector search → top K candidates
  2. For each → get engram
  3. Graph traverse: BFS via petgraph (hard depth cap)
  4. Collect all memory_ids in cluster
  5. Score and rank
  6. Reconstruct from fragments
```

---

## 10. Interference Handling

**Brain**: Proactive (old confuses new), Retroactive (new weakens old).

**membrain**:

| Type | Trigger | Effect |
|------|---------|--------|
| **Retroactive** | Encoding new memory | Similar older memories (sim 0.7–0.99): `strength -= interference_penalty(sim)` |
| **Proactive** | Recalling old memory | If has similar newer: `newer.retrieval_difficulty += PROACTIVE_PENALTY` |

Identical (sim > 0.99) is not interference — it is duplicate detection.

---

## 11. Context Reconstruction

**Brain**: Recall is reconstruction, not playback — context cues trigger pattern completion.

**membrain**:
- Context embedding stored alongside content embedding
- Recall query includes optional `--context` for task-aware retrieval
- Engram BFS provides cluster expansion (pattern completion)
- Era filtering (Feature 5) adds temporal context reconstruction
- Mood-congruent retrieval (Feature 18) mirrors state-dependent memory

---

## 12. Prediction-Linked Recall

**Brain**: Memory retrieval is biased by current goals and predictions.

**membrain**:
- Predictive Pre-recall (Feature 16): learn A→B query sequences → pre-warm Tier1 cache
- Query Intent Classification (Feature 20): route queries to optimal retrieval config
- Context Budget API (Feature 4): utility = relevance × strength × (1 − working_memory_overlap)
- Goal-based recall via `membrain recall --goal`

---

## Research Framing

The original plan is strongest when interpreted as a **functional translation** of neuroscience into systems design:

- hippocampus ↔ hot episodic index
- neocortex ↔ cold consolidated semantic store
- engram ↔ cluster with centroid and member pointers
- LTP ↔ strength increase on successful recall
- LTD ↔ decay applied lazily

Use "brain-inspired cognitive runtime" or "brain-inspired memory operating system" — avoid implying literal biological equivalence. Every biological metaphor must survive benchmarking and operational scrutiny.
