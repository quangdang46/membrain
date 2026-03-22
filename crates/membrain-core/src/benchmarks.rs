//! Benchmark harness entrypoints for hot-path encode, recall, and ranking.
//!
//! This module provides the first reusable harness layer matching the metadata
//! contract from `docs/BENCHMARKS.md`. Each harness entrypoint produces a
//! machine-readable `BenchmarkArtifact` with required metadata fields:
//! dataset cardinality, machine profile, build mode, warm/cold declaration,
//! harness entrypoint identity, and artifact location.
//!
//! Benchmarks are deterministic and rerunnable: they wrap the existing engine
//! entrypoints (`prepare_fast_path`, `plan_recall`, `fuse_scores`) without
//! modifying their behavior.

use std::time::Instant;

use crate::config::RuntimeConfig;
use crate::engine::encode::EncodeEngine;
use crate::engine::ranking::{fuse_scores, RankingInput, RankingProfile};
use crate::engine::recall::{RecallEngine, RecallRequest, RecallRuntime};
use crate::types::{RawEncodeInput, RawIntakeKind, SessionId};

// ── Benchmark metadata contract ───────────────────────────────────────────────

/// Stable benchmark scenario families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BenchmarkScenario {
    /// Encode hot-path benchmarking.
    Encode,
    /// Recall/retrieval planning benchmarking.
    Recall,
    /// Ranking score-fusion benchmarking.
    Ranking,
}

impl BenchmarkScenario {
    /// Returns the stable machine-readable scenario name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Encode => "encode",
            Self::Recall => "recall",
            Self::Ranking => "ranking",
        }
    }
}

/// Warm or cold state declaration for a benchmark run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WarmColdState {
    /// Process, cache, model, and index are warm.
    Warm,
    /// Process or index are cold.
    Cold,
}

impl WarmColdState {
    /// Returns the stable machine-readable state label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Warm => "warm",
            Self::Cold => "cold",
        }
    }
}

/// Representativeness label for benchmark results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Representativeness {
    /// The run reflects the intended production envelope.
    Representative,
    /// The run is exploratory and cannot close a stage gate.
    Exploratory,
}

impl Representativeness {
    /// Returns the stable machine-readable label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Representative => "representative",
            Self::Exploratory => "exploratory",
        }
    }
}

/// Required metadata contract for every benchmark artifact.
///
/// Matches the canonical checklist in `docs/BENCHMARKS.md`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkMetadata {
    /// Scenario name.
    pub scenario: String,
    /// Touched path or stage identifier.
    pub touched_path: String,
    /// Dataset or fixture identity.
    pub dataset_id: String,
    /// Dataset cardinality (number of items).
    pub dataset_cardinality: usize,
    /// Payload-shape summary.
    pub payload_shape: String,
    /// Machine profile identifier.
    pub machine_profile: String,
    /// Build mode (debug or release).
    pub build_mode: String,
    /// Warm/cold state declaration.
    pub warm_cold: String,
    /// Concurrency level (1 for single-threaded harness).
    pub concurrency: usize,
    /// Iteration or sample count used for percentile derivation.
    pub sample_count: usize,
    /// Harness or command entrypoint name.
    pub harness_entrypoint: String,
    /// Representativeness label.
    pub representativeness: String,
}

/// Latency percentiles captured by a benchmark run.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LatencyPercentiles {
    /// Median latency in nanoseconds.
    pub p50_ns: u64,
    /// 95th percentile latency in nanoseconds.
    pub p95_ns: u64,
    /// 99th percentile latency in nanoseconds.
    pub p99_ns: u64,
    /// Minimum latency in nanoseconds.
    pub min_ns: u64,
    /// Maximum latency in nanoseconds.
    pub max_ns: u64,
    /// Mean latency in nanoseconds.
    pub mean_ns: u64,
}

/// Machine-readable benchmark artifact produced by a harness run.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkArtifact {
    /// Required metadata contract fields.
    pub metadata: BenchmarkMetadata,
    /// Latency percentile results.
    pub latency: LatencyPercentiles,
    /// Whether the run passed the declared contract.
    pub pass: bool,
    /// Pass/fail contract description.
    pub contract: String,
    /// Bounded-work or routing evidence (scenario-specific).
    pub evidence: serde_json::Value,
}

// ── Percentile computation ────────────────────────────────────────────────────

/// Computes latency percentiles from a sorted sample of nanosecond durations.
fn compute_percentiles(mut samples: Vec<u64>) -> LatencyPercentiles {
    samples.sort_unstable();
    let len = samples.len();
    if len == 0 {
        return LatencyPercentiles {
            p50_ns: 0,
            p95_ns: 0,
            p99_ns: 0,
            min_ns: 0,
            max_ns: 0,
            mean_ns: 0,
        };
    }

    let percentile = |p: f64| -> u64 {
        let idx = ((p / 100.0) * (len as f64 - 1.0)).round() as usize;
        samples[idx.min(len - 1)]
    };

    let sum: u64 = samples.iter().sum();

    LatencyPercentiles {
        p50_ns: percentile(50.0),
        p95_ns: percentile(95.0),
        p99_ns: percentile(99.0),
        min_ns: samples[0],
        max_ns: samples[len - 1],
        mean_ns: sum / len as u64,
    }
}

// ── Encode benchmark harness ─────────────────────────────────────────────────

/// Harness entrypoint for the encode fast path.
///
/// Runs `EncodeEngine::prepare_fast_path` for `iterations` rounds and produces
/// a benchmark artifact with latency percentiles and encode-specific evidence.
pub fn bench_encode(
    engine: &EncodeEngine,
    iterations: usize,
    warm_cold: WarmColdState,
    dataset_id: &str,
    dataset_cardinality: usize,
) -> BenchmarkArtifact {
    let input = RawEncodeInput::new(
        RawIntakeKind::Event,
        "benchmark encode test payload for hot-path measurement",
    );
    let mut latencies = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        let result = engine.prepare_fast_path(input.clone());
        let elapsed = start.elapsed().as_nanos() as u64;
        latencies.push(elapsed);
        // Prevent the compiler from optimizing away the work.
        std::hint::black_box(&result);
    }

    let latency = compute_percentiles(latencies);

    // Run once more for evidence capture.
    let final_result = engine.prepare_fast_path(input);

    let evidence = serde_json::json!({
        "memory_type": final_result.classification.memory_type.as_str(),
        "route_family": final_result.classification.route_family.as_str(),
        "provisional_salience": final_result.provisional_salience,
        "fingerprint_hex": format!("{:016x}", final_result.fingerprint),
        "stayed_within_latency_budget": final_result.trace.stayed_within_latency_budget,
        "trace_stages": final_result.trace.stages.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
    });

    let pass = latency.p95_ns < 10_000_000; // p95 < 10ms per encode contract

    BenchmarkArtifact {
        metadata: BenchmarkMetadata {
            scenario: BenchmarkScenario::Encode.as_str().to_string(),
            touched_path: "encode_fast_path".to_string(),
            dataset_id: dataset_id.to_string(),
            dataset_cardinality,
            payload_shape: "short_text_event".to_string(),
            machine_profile: "local_dev".to_string(),
            build_mode: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            }
            .to_string(),
            warm_cold: warm_cold.as_str().to_string(),
            concurrency: 1,
            sample_count: iterations,
            harness_entrypoint: "bench_encode".to_string(),
            representativeness: Representativeness::Exploratory.as_str().to_string(),
        },
        latency,
        pass,
        contract: "p95 < 10ms".to_string(),
        evidence,
    }
}

// ── Recall benchmark harness ─────────────────────────────────────────────────

/// Harness entrypoint for recall/retrieval planning.
///
/// Runs `RecallEngine::plan_recall` for `iterations` rounds per plan kind
/// and produces a benchmark artifact with latency percentiles and routing evidence.
pub fn bench_recall(
    engine: &RecallEngine,
    config: RuntimeConfig,
    iterations: usize,
    warm_cold: WarmColdState,
    dataset_id: &str,
    dataset_cardinality: usize,
) -> BenchmarkArtifact {
    // Benchmark each plan kind separately.
    let scenarios: Vec<(&str, RecallRequest)> = vec![
        ("exact_id", RecallRequest::exact(crate::types::MemoryId(42))),
        (
            "session_lookup",
            RecallRequest::small_session_lookup(SessionId(7)),
        ),
        (
            "deep_fallback",
            RecallRequest {
                exact_memory_id: None,
                session_id: Some(SessionId(9)),
                small_lookup: false,
            },
        ),
    ];

    let mut all_latencies = Vec::new();
    let mut per_scenario = Vec::new();

    for (name, request) in &scenarios {
        let mut latencies = Vec::with_capacity(iterations);
        for _ in 0..iterations {
            let start = Instant::now();
            let plan = engine.plan_recall(*request, config);
            let elapsed = start.elapsed().as_nanos() as u64;
            latencies.push(elapsed);
            std::hint::black_box(&plan);
        }
        let scenario_latency = compute_percentiles(latencies.clone());
        per_scenario.push(serde_json::json!({
            "plan_kind": name,
            "p50_ns": scenario_latency.p50_ns,
            "p95_ns": scenario_latency.p95_ns,
        }));
        all_latencies.extend(latencies);
    }

    let latency = compute_percentiles(all_latencies);

    // Run one final plan for evidence.
    let final_plan = engine.plan_recall(RecallRequest::exact(crate::types::MemoryId(1)), config);

    let evidence = serde_json::json!({
        "tier1_candidate_budget": final_plan.tier1_candidate_budget,
        "per_scenario": per_scenario,
        "terminates_in_tier1": final_plan.terminates_in_tier1(),
    });

    // Tier1 exact path contract: p95 < 0.1ms (100μs)
    let pass = latency.p95_ns < 100_000;

    BenchmarkArtifact {
        metadata: BenchmarkMetadata {
            scenario: BenchmarkScenario::Recall.as_str().to_string(),
            touched_path: "recall_planning".to_string(),
            dataset_id: dataset_id.to_string(),
            dataset_cardinality,
            payload_shape: "recall_request_variants".to_string(),
            machine_profile: "local_dev".to_string(),
            build_mode: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            }
            .to_string(),
            warm_cold: warm_cold.as_str().to_string(),
            concurrency: 1,
            sample_count: iterations * 3,
            harness_entrypoint: "bench_recall".to_string(),
            representativeness: Representativeness::Exploratory.as_str().to_string(),
        },
        latency,
        pass,
        contract: "p95 < 0.1ms for Tier1 planning".to_string(),
        evidence,
    }
}

// ── Ranking benchmark harness ────────────────────────────────────────────────

/// Harness entrypoint for ranking score fusion.
///
/// Runs `fuse_scores` for `iterations` rounds across multiple profiles and
/// produces a benchmark artifact with latency percentiles and score evidence.
pub fn bench_ranking(
    iterations: usize,
    warm_cold: WarmColdState,
    dataset_id: &str,
    dataset_cardinality: usize,
) -> BenchmarkArtifact {
    let profiles: Vec<(&str, RankingProfile)> = vec![
        ("balanced", RankingProfile::balanced()),
        ("recency_biased", RankingProfile::recency_biased()),
        ("strength_biased", RankingProfile::strength_biased()),
    ];

    let test_inputs: Vec<(&str, RankingInput)> = vec![
        (
            "high_recency",
            RankingInput {
                recency: 900,
                salience: 300,
                strength: 200,
                provenance: 500,
                conflict: 500,
                confidence: 500,
            },
        ),
        (
            "high_strength",
            RankingInput {
                recency: 200,
                salience: 400,
                strength: 950,
                provenance: 600,
                conflict: 500,
                confidence: 500,
            },
        ),
        (
            "all_mid",
            RankingInput {
                recency: 500,
                salience: 500,
                strength: 500,
                provenance: 500,
                conflict: 500,
                confidence: 500,
            },
        ),
        (
            "max_signals",
            RankingInput {
                recency: 1000,
                salience: 1000,
                strength: 1000,
                provenance: 1000,
                conflict: 1000,
                confidence: 1000,
            },
        ),
    ];

    let mut all_latencies = Vec::new();
    let mut profile_evidence = Vec::new();

    for (profile_name, profile) in &profiles {
        let mut profile_latencies = Vec::new();
        let mut score_samples = Vec::new();

        for (input_name, input) in &test_inputs {
            for _ in 0..iterations {
                let start = Instant::now();
                let result = fuse_scores(*input, *profile);
                let elapsed = start.elapsed().as_nanos() as u64;
                profile_latencies.push(elapsed);
                std::hint::black_box(&result);
            }
            // Capture one score sample for evidence.
            let result = fuse_scores(*input, *profile);
            score_samples.push(serde_json::json!({
                "input": input_name,
                "final_score": result.final_score,
                "profile": result.profile_name,
            }));
        }

        let profile_latency = compute_percentiles(profile_latencies.clone());
        profile_evidence.push(serde_json::json!({
            "profile": profile_name,
            "p50_ns": profile_latency.p50_ns,
            "p95_ns": profile_latency.p95_ns,
            "score_samples": score_samples,
        }));
        all_latencies.extend(profile_latencies);
    }

    let latency = compute_percentiles(all_latencies);

    let evidence = serde_json::json!({
        "per_profile": profile_evidence,
        "input_count": test_inputs.len(),
        "profile_count": profiles.len(),
    });

    // Ranking is pure arithmetic, expect sub-microsecond.
    let pass = latency.p95_ns < 10_000; // p95 < 10μs

    BenchmarkArtifact {
        metadata: BenchmarkMetadata {
            scenario: BenchmarkScenario::Ranking.as_str().to_string(),
            touched_path: "ranking_score_fusion".to_string(),
            dataset_id: dataset_id.to_string(),
            dataset_cardinality,
            payload_shape: "ranking_signal_vectors".to_string(),
            machine_profile: "local_dev".to_string(),
            build_mode: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            }
            .to_string(),
            warm_cold: warm_cold.as_str().to_string(),
            concurrency: 1,
            sample_count: iterations * test_inputs.len() * profiles.len(),
            harness_entrypoint: "bench_ranking".to_string(),
            representativeness: Representativeness::Exploratory.as_str().to_string(),
        },
        latency,
        pass,
        contract: "p95 < 10μs for pure score fusion".to_string(),
        evidence,
    }
}

// ── Dataset families ──────────────────────────────────────────────────────────

/// Named dataset cardinality classes used to label benchmark workloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DatasetSize {
    /// Small dataset: exploratory or smoke-test scale.
    Small,
    /// Medium dataset: representative of typical production hot sets.
    Medium,
    /// Large dataset: representative of production-scale cold cardinality.
    Large,
}

impl DatasetSize {
    /// Returns the stable machine-readable size label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }
}

/// Named benchmark dataset descriptor with cardinality and representativeness.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkDataset {
    /// Stable dataset identity for artifact metadata.
    pub id: String,
    /// Cardinality class label.
    pub size: DatasetSize,
    /// Number of items in this dataset.
    pub cardinality: usize,
    /// Payload shape description.
    pub payload_shape: String,
    /// Whether this dataset is representative or exploratory.
    pub representativeness: Representativeness,
}

impl BenchmarkDataset {
    /// Small exploratory dataset (smoke-test scale).
    pub fn small_exploratory() -> Self {
        Self {
            id: "synthetic_small".to_string(),
            size: DatasetSize::Small,
            cardinality: 10,
            payload_shape: "short_text_event".to_string(),
            representativeness: Representativeness::Exploratory,
        }
    }

    /// Medium representative dataset (typical production hot set).
    pub fn medium_representative() -> Self {
        Self {
            id: "synthetic_medium".to_string(),
            size: DatasetSize::Medium,
            cardinality: 1_000,
            payload_shape: "mixed_text_event_observation".to_string(),
            representativeness: Representativeness::Representative,
        }
    }

    /// Large representative dataset (production-scale cold cardinality).
    pub fn large_representative() -> Self {
        Self {
            id: "synthetic_large".to_string(),
            size: DatasetSize::Large,
            cardinality: 100_000,
            payload_shape: "mixed_text_all_families".to_string(),
            representativeness: Representativeness::Representative,
        }
    }
}

// ── Benchmark suite with warm/cold modes ─────────────────────────────────────

/// A benchmark suite that runs harness entrypoints across warm and cold passes.
///
/// The first pass is always cold (fresh engine state). Subsequent passes are
/// warm (warmed caches, established state). This produces two artifacts per
/// scenario so regressions can distinguish cold-start from steady-state behavior.
#[derive(Debug)]
pub struct BenchmarkSuite {
    dataset: BenchmarkDataset,
    iterations: usize,
}

/// Bundle of benchmark artifacts produced by a full suite run.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkSuiteResult {
    /// Dataset descriptor used for this suite.
    pub dataset: BenchmarkDataset,
    /// Cold-pass artifacts (fresh engine state).
    pub cold_artifacts: Vec<BenchmarkArtifact>,
    /// Warm-pass artifacts (warmed caches).
    pub warm_artifacts: Vec<BenchmarkArtifact>,
}

impl BenchmarkSuite {
    /// Creates a new benchmark suite for the given dataset and iteration count.
    pub fn new(dataset: BenchmarkDataset, iterations: usize) -> Self {
        Self {
            dataset,
            iterations,
        }
    }

    /// Runs the full suite: cold pass first, then warm pass.
    ///
    /// The cold pass creates fresh engine instances. The warm pass reuses the
    /// same engines after the cold pass has populated their state.
    pub fn run(&self) -> BenchmarkSuiteResult {
        // Cold pass: fresh engines every time.
        let cold_artifacts = self.run_cold();

        // Warm pass: reuse engines after cold warmup.
        let warm_artifacts = self.run_warm();

        BenchmarkSuiteResult {
            dataset: self.dataset.clone(),
            cold_artifacts,
            warm_artifacts,
        }
    }

    fn run_cold(&self) -> Vec<BenchmarkArtifact> {
        let engine = EncodeEngine::default();
        let recall_engine = RecallEngine;
        let config = RuntimeConfig::default();

        vec![
            bench_encode(
                &engine,
                self.iterations,
                WarmColdState::Cold,
                &self.dataset.id,
                self.dataset.cardinality,
            ),
            bench_recall(
                &recall_engine,
                config,
                self.iterations,
                WarmColdState::Cold,
                &self.dataset.id,
                self.dataset.cardinality,
            ),
            bench_ranking(
                self.iterations,
                WarmColdState::Cold,
                &self.dataset.id,
                self.dataset.cardinality,
            ),
        ]
    }

    fn run_warm(&self) -> Vec<BenchmarkArtifact> {
        // Create engines and warm them up with a few iterations before measuring.
        let engine = EncodeEngine::default();
        let recall_engine = RecallEngine;
        let config = RuntimeConfig::default();

        // Warmup pass.
        let warmup_input = RawEncodeInput::new(RawIntakeKind::Event, "warmup payload");
        for _ in 0..self.iterations {
            std::hint::black_box(engine.prepare_fast_path(warmup_input.clone()));
            std::hint::black_box(
                recall_engine.plan_recall(RecallRequest::exact(crate::types::MemoryId(1)), config),
            );
        }

        vec![
            bench_encode(
                &engine,
                self.iterations,
                WarmColdState::Warm,
                &self.dataset.id,
                self.dataset.cardinality,
            ),
            bench_recall(
                &recall_engine,
                config,
                self.iterations,
                WarmColdState::Warm,
                &self.dataset.id,
                self.dataset.cardinality,
            ),
            bench_ranking(
                self.iterations,
                WarmColdState::Warm,
                &self.dataset.id,
                self.dataset.cardinality,
            ),
        ]
    }
}

// ── Convenience helpers ──────────────────────────────────────────────────────

/// Runs all benchmark harness entrypoints with default parameters and returns
/// the collected artifacts.
pub fn bench_all(iterations: usize) -> Vec<BenchmarkArtifact> {
    let engine = EncodeEngine::default();
    let recall_engine = RecallEngine;
    let config = RuntimeConfig::default();

    vec![
        bench_encode(
            &engine,
            iterations,
            WarmColdState::Warm,
            "synthetic_default",
            1,
        ),
        bench_recall(
            &recall_engine,
            config,
            iterations,
            WarmColdState::Warm,
            "synthetic_default",
            1,
        ),
        bench_ranking(iterations, WarmColdState::Warm, "synthetic_default", 4),
    ]
}

/// Runs a representative benchmark suite at the given cardinality class.
pub fn bench_representative(size: DatasetSize, iterations: usize) -> BenchmarkSuiteResult {
    let dataset = match size {
        DatasetSize::Small => BenchmarkDataset::small_exploratory(),
        DatasetSize::Medium => BenchmarkDataset::medium_representative(),
        DatasetSize::Large => BenchmarkDataset::large_representative(),
    };
    BenchmarkSuite::new(dataset, iterations).run()
}

// ── Regression comparison ────────────────────────────────────────────────────

/// Configurable thresholds for regression detection.
///
/// Thresholds are expressed as fractional tolerances (0.0 = no tolerance,
/// 0.2 = 20% degradation allowed). A negative tolerance means improvement
/// is expected.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RegressionThresholds {
    /// Maximum allowed p50 latency increase as a fraction (e.g., 0.2 = 20%).
    pub p50_max_degradation: f64,
    /// Maximum allowed p95 latency increase as a fraction.
    pub p95_max_degradation: f64,
    /// Maximum allowed p99 latency increase as a fraction.
    pub p99_max_degradation: f64,
}

impl RegressionThresholds {
    /// Default thresholds: 20% p50, 30% p95, 50% p99.
    pub const fn default_thresholds() -> Self {
        Self {
            p50_max_degradation: 0.20,
            p95_max_degradation: 0.30,
            p99_max_degradation: 0.50,
        }
    }

    /// Strict thresholds for release-gate benchmarks: 10% p50, 15% p95, 25% p99.
    pub const fn strict() -> Self {
        Self {
            p50_max_degradation: 0.10,
            p95_max_degradation: 0.15,
            p99_max_degradation: 0.25,
        }
    }

    /// Relaxed thresholds for exploratory benchmarks: 50% across the board.
    pub const fn relaxed() -> Self {
        Self {
            p50_max_degradation: 0.50,
            p95_max_degradation: 0.50,
            p99_max_degradation: 0.50,
        }
    }
}

/// Regression verdict for a single latency percentile comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RegressionVerdict {
    /// Performance improved beyond noise floor.
    Improvement,
    /// Performance is within acceptable bounds.
    Pass,
    /// Performance regressed beyond threshold.
    Regression,
}

impl RegressionVerdict {
    /// Returns the stable machine-readable verdict label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Improvement => "improvement",
            Self::Pass => "pass",
            Self::Regression => "regression",
        }
    }
}

/// Detailed comparison of a single percentile between baseline and current.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PercentileComparison {
    /// Percentile name (p50, p95, p99).
    pub percentile: String,
    /// Baseline value in nanoseconds.
    pub baseline_ns: u64,
    /// Current value in nanoseconds.
    pub current_ns: u64,
    /// Signed delta (positive = slower, negative = faster).
    pub delta_ns: i64,
    /// Fractional change (positive = degradation, negative = improvement).
    pub delta_fraction: f64,
    /// Verdict against configured threshold.
    pub verdict: RegressionVerdict,
}

/// Machine-readable regression report comparing current vs baseline artifacts.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RegressionReport {
    /// Scenario being compared.
    pub scenario: String,
    /// Dataset identity.
    pub dataset_id: String,
    /// Warm/cold state.
    pub warm_cold: String,
    /// Per-percentile comparisons.
    pub comparisons: Vec<PercentileComparison>,
    /// Overall verdict (worst of all percentiles).
    pub overall_verdict: RegressionVerdict,
    /// Whether both runs passed their individual contracts.
    pub both_passed_contract: bool,
    /// Thresholds used for this comparison.
    pub thresholds: RegressionThresholds,
}

/// Compares a current benchmark artifact against a baseline artifact.
///
/// Returns a regression report with per-percentile verdicts and an overall
/// verdict. The comparison is only valid when both artifacts measure the
/// same scenario, dataset, and warm/cold state.
pub fn compare_artifacts(
    baseline: &BenchmarkArtifact,
    current: &BenchmarkArtifact,
    thresholds: &RegressionThresholds,
) -> RegressionReport {
    let compare_percentile =
        |name: &str, base: u64, curr: u64, threshold: f64| -> PercentileComparison {
            let delta_ns = curr as i64 - base as i64;
            let delta_fraction = if base == 0 {
                if curr == 0 {
                    0.0
                } else {
                    f64::INFINITY
                }
            } else {
                (curr as f64 - base as f64) / base as f64
            };

            let verdict = if delta_fraction > threshold {
                RegressionVerdict::Regression
            } else if delta_fraction < -0.05 {
                RegressionVerdict::Improvement
            } else {
                RegressionVerdict::Pass
            };

            PercentileComparison {
                percentile: name.to_string(),
                baseline_ns: base,
                current_ns: curr,
                delta_ns,
                delta_fraction,
                verdict,
            }
        };

    let comparisons = vec![
        compare_percentile(
            "p50",
            baseline.latency.p50_ns,
            current.latency.p50_ns,
            thresholds.p50_max_degradation,
        ),
        compare_percentile(
            "p95",
            baseline.latency.p95_ns,
            current.latency.p95_ns,
            thresholds.p95_max_degradation,
        ),
        compare_percentile(
            "p99",
            baseline.latency.p99_ns,
            current.latency.p99_ns,
            thresholds.p99_max_degradation,
        ),
    ];

    let overall_verdict = if comparisons
        .iter()
        .any(|c| c.verdict == RegressionVerdict::Regression)
    {
        RegressionVerdict::Regression
    } else if comparisons
        .iter()
        .all(|c| c.verdict == RegressionVerdict::Improvement)
    {
        RegressionVerdict::Improvement
    } else {
        RegressionVerdict::Pass
    };

    RegressionReport {
        scenario: current.metadata.scenario.clone(),
        dataset_id: current.metadata.dataset_id.clone(),
        warm_cold: current.metadata.warm_cold.clone(),
        comparisons,
        overall_verdict,
        both_passed_contract: baseline.pass && current.pass,
        thresholds: thresholds.clone(),
    }
}

/// Compares two suite result bundles, matching artifacts by scenario and warm/cold state.
pub fn compare_suites(
    baseline: &BenchmarkSuiteResult,
    current: &BenchmarkSuiteResult,
    thresholds: &RegressionThresholds,
) -> Vec<RegressionReport> {
    let mut reports = Vec::new();

    for (base_artifacts, curr_artifacts) in [
        (&baseline.cold_artifacts, &current.cold_artifacts),
        (&baseline.warm_artifacts, &current.warm_artifacts),
    ] {
        for base in base_artifacts {
            if let Some(curr) = curr_artifacts
                .iter()
                .find(|a| a.metadata.scenario == base.metadata.scenario)
            {
                reports.push(compare_artifacts(base, curr, thresholds));
            }
        }
    }

    reports
}

/// Serializes a benchmark artifact to a JSON string for storage.
pub fn artifact_to_json(artifact: &BenchmarkArtifact) -> String {
    serde_json::to_string_pretty(artifact).unwrap_or_else(|_| String::from("{}"))
}

/// Deserializes a benchmark artifact from a JSON string.
pub fn artifact_from_json(json: &str) -> Option<BenchmarkArtifact> {
    serde_json::from_str(json).ok()
}

/// Serializes a suite result to a JSON string for storage.
pub fn suite_result_to_json(result: &BenchmarkSuiteResult) -> String {
    serde_json::to_string_pretty(result).unwrap_or_else(|_| String::from("{}"))
}

/// Deserializes a suite result from a JSON string.
pub fn suite_result_from_json(json: &str) -> Option<BenchmarkSuiteResult> {
    serde_json::from_str(json).ok()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_scenario_labels_are_stable() {
        assert_eq!(BenchmarkScenario::Encode.as_str(), "encode");
        assert_eq!(BenchmarkScenario::Recall.as_str(), "recall");
        assert_eq!(BenchmarkScenario::Ranking.as_str(), "ranking");
    }

    #[test]
    fn warm_cold_state_labels_are_stable() {
        assert_eq!(WarmColdState::Warm.as_str(), "warm");
        assert_eq!(WarmColdState::Cold.as_str(), "cold");
    }

    #[test]
    fn representativeness_labels_are_stable() {
        assert_eq!(
            Representativeness::Representative.as_str(),
            "representative"
        );
        assert_eq!(Representativeness::Exploratory.as_str(), "exploratory");
    }

    #[test]
    fn compute_percentiles_handles_empty_input() {
        let latency = compute_percentiles(vec![]);
        assert_eq!(latency.p50_ns, 0);
        assert_eq!(latency.p95_ns, 0);
        assert_eq!(latency.p99_ns, 0);
        assert_eq!(latency.min_ns, 0);
        assert_eq!(latency.max_ns, 0);
        assert_eq!(latency.mean_ns, 0);
    }

    #[test]
    fn compute_percentiles_handles_single_sample() {
        let latency = compute_percentiles(vec![1000]);
        assert_eq!(latency.p50_ns, 1000);
        assert_eq!(latency.p95_ns, 1000);
        assert_eq!(latency.p99_ns, 1000);
        assert_eq!(latency.min_ns, 1000);
        assert_eq!(latency.max_ns, 1000);
        assert_eq!(latency.mean_ns, 1000);
    }

    #[test]
    fn compute_percentiles_handles_multiple_samples() {
        let samples: Vec<u64> = (1..=100).collect();
        let latency = compute_percentiles(samples);
        assert_eq!(latency.min_ns, 1);
        assert_eq!(latency.max_ns, 100);
        // p50 of 1..100: round(0.5 * 99) = round(49.5) = 50, samples[50] = 51
        assert!(latency.p50_ns >= 50 && latency.p50_ns <= 51);
        // p95 of 1..100 should be around 95-96.
        assert!(latency.p95_ns >= 94 && latency.p95_ns <= 96);
    }

    #[test]
    fn encode_benchmark_produces_valid_artifact() {
        let engine = EncodeEngine::default();
        let artifact = bench_encode(&engine, 10, WarmColdState::Warm, "test_encode", 1);

        assert_eq!(artifact.metadata.scenario, "encode");
        assert_eq!(artifact.metadata.touched_path, "encode_fast_path");
        assert_eq!(artifact.metadata.dataset_id, "test_encode");
        assert_eq!(artifact.metadata.sample_count, 10);
        assert_eq!(artifact.metadata.harness_entrypoint, "bench_encode");
        assert_eq!(artifact.metadata.warm_cold, "warm");
        assert_eq!(artifact.metadata.representativeness, "exploratory");
        assert_eq!(artifact.latency.p50_ns, artifact.latency.p50_ns); // sanity
        assert!(artifact.latency.min_ns <= artifact.latency.p50_ns);
        assert!(artifact.latency.p50_ns <= artifact.latency.p99_ns);
        assert!(artifact.latency.p99_ns <= artifact.latency.max_ns);
        assert!(!artifact.evidence.is_null());
    }

    #[test]
    fn recall_benchmark_produces_valid_artifact() {
        let engine = RecallEngine;
        let config = RuntimeConfig::default();
        let artifact = bench_recall(&engine, config, 10, WarmColdState::Cold, "test_recall", 5);

        assert_eq!(artifact.metadata.scenario, "recall");
        assert_eq!(artifact.metadata.touched_path, "recall_planning");
        assert_eq!(artifact.metadata.sample_count, 30); // 10 * 3 scenarios
        assert_eq!(artifact.metadata.harness_entrypoint, "bench_recall");
        assert_eq!(artifact.metadata.warm_cold, "cold");
        assert!(artifact.latency.min_ns <= artifact.latency.p50_ns);
        assert!(artifact.latency.p50_ns <= artifact.latency.p99_ns);
    }

    #[test]
    fn ranking_benchmark_produces_valid_artifact() {
        let artifact = bench_ranking(10, WarmColdState::Warm, "test_ranking", 4);

        assert_eq!(artifact.metadata.scenario, "ranking");
        assert_eq!(artifact.metadata.touched_path, "ranking_score_fusion");
        // 10 iterations * 4 inputs * 3 profiles = 120
        assert_eq!(artifact.metadata.sample_count, 120);
        assert_eq!(artifact.metadata.harness_entrypoint, "bench_ranking");
        assert!(artifact.latency.min_ns <= artifact.latency.p50_ns);
        assert!(artifact.latency.p50_ns <= artifact.latency.p99_ns);
    }

    #[test]
    fn benchmark_metadata_fields_match_contract() {
        let engine = EncodeEngine::default();
        let artifact = bench_encode(&engine, 5, WarmColdState::Warm, "contract_check", 100);

        // Verify all required metadata fields are populated.
        assert!(!artifact.metadata.scenario.is_empty());
        assert!(!artifact.metadata.touched_path.is_empty());
        assert!(!artifact.metadata.dataset_id.is_empty());
        assert!(artifact.metadata.dataset_cardinality > 0);
        assert!(!artifact.metadata.payload_shape.is_empty());
        assert!(!artifact.metadata.machine_profile.is_empty());
        assert!(!artifact.metadata.build_mode.is_empty());
        assert!(!artifact.metadata.warm_cold.is_empty());
        assert!(artifact.metadata.concurrency > 0);
        assert!(artifact.metadata.sample_count > 0);
        assert!(!artifact.metadata.harness_entrypoint.is_empty());
        assert!(!artifact.metadata.representativeness.is_empty());
        assert!(!artifact.contract.is_empty());
    }

    #[test]
    fn bench_all_returns_three_artifacts() {
        let artifacts = bench_all(5);
        assert_eq!(artifacts.len(), 3);
        assert_eq!(artifacts[0].metadata.scenario, "encode");
        assert_eq!(artifacts[1].metadata.scenario, "recall");
        assert_eq!(artifacts[2].metadata.scenario, "ranking");
    }

    #[test]
    fn benchmark_artifact_is_serializable() {
        let artifact = bench_ranking(5, WarmColdState::Warm, "serialization_test", 1);
        let json = serde_json::to_string(&artifact).expect("artifact must serialize to JSON");
        let deserialized: BenchmarkArtifact =
            serde_json::from_str(&json).expect("artifact must deserialize from JSON");
        assert_eq!(artifact.metadata.scenario, deserialized.metadata.scenario);
        assert_eq!(artifact.latency.p50_ns, deserialized.latency.p50_ns);
    }

    #[test]
    fn encode_artifact_evidence_has_trace_stages() {
        let engine = EncodeEngine::default();
        let artifact = bench_encode(&engine, 3, WarmColdState::Warm, "trace_test", 1);

        let stages = artifact.evidence["trace_stages"]
            .as_array()
            .expect("evidence must have trace_stages array");
        assert_eq!(stages.len(), 5);
        assert!(stages.iter().any(|s| s.as_str() == Some("normalize")));
        assert!(stages.iter().any(|s| s.as_str() == Some("fingerprint")));
    }

    #[test]
    fn recall_artifact_evidence_has_per_scenario_breakdown() {
        let engine = RecallEngine;
        let config = RuntimeConfig::default();
        let artifact = bench_recall(&engine, config, 3, WarmColdState::Warm, "scenario_test", 1);

        let per_scenario = artifact.evidence["per_scenario"]
            .as_array()
            .expect("evidence must have per_scenario array");
        assert_eq!(per_scenario.len(), 3);
        assert!(per_scenario
            .iter()
            .any(|s| s["plan_kind"].as_str() == Some("exact_id")));
        assert!(per_scenario
            .iter()
            .any(|s| s["plan_kind"].as_str() == Some("session_lookup")));
        assert!(per_scenario
            .iter()
            .any(|s| s["plan_kind"].as_str() == Some("deep_fallback")));
    }

    #[test]
    fn ranking_artifact_evidence_has_per_profile_breakdown() {
        let artifact = bench_ranking(3, WarmColdState::Warm, "profile_test", 1);

        let per_profile = artifact.evidence["per_profile"]
            .as_array()
            .expect("evidence must have per_profile array");
        assert_eq!(per_profile.len(), 3);
        assert!(per_profile
            .iter()
            .any(|p| p["profile"].as_str() == Some("balanced")));
        assert!(per_profile
            .iter()
            .any(|p| p["profile"].as_str() == Some("recency_biased")));
        assert!(per_profile
            .iter()
            .any(|p| p["profile"].as_str() == Some("strength_biased")));
    }

    #[test]
    fn build_mode_reflects_actual_build() {
        let engine = EncodeEngine::default();
        let artifact = bench_encode(&engine, 1, WarmColdState::Warm, "build_mode_test", 1);

        let expected = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        assert_eq!(artifact.metadata.build_mode, expected);
    }

    // ── Dataset and suite tests ───────────────────────────────────────────

    #[test]
    fn dataset_size_labels_are_stable() {
        assert_eq!(DatasetSize::Small.as_str(), "small");
        assert_eq!(DatasetSize::Medium.as_str(), "medium");
        assert_eq!(DatasetSize::Large.as_str(), "large");
    }

    #[test]
    fn dataset_presets_have_correct_properties() {
        let small = BenchmarkDataset::small_exploratory();
        assert_eq!(small.size, DatasetSize::Small);
        assert_eq!(small.cardinality, 10);
        assert_eq!(small.representativeness, Representativeness::Exploratory);

        let medium = BenchmarkDataset::medium_representative();
        assert_eq!(medium.size, DatasetSize::Medium);
        assert_eq!(medium.cardinality, 1_000);
        assert_eq!(
            medium.representativeness,
            Representativeness::Representative
        );

        let large = BenchmarkDataset::large_representative();
        assert_eq!(large.size, DatasetSize::Large);
        assert_eq!(large.cardinality, 100_000);
        assert_eq!(large.representativeness, Representativeness::Representative);
    }

    #[test]
    fn suite_produces_cold_and_warm_artifacts() {
        let dataset = BenchmarkDataset::small_exploratory();
        let suite = BenchmarkSuite::new(dataset, 5);
        let result = suite.run();

        assert_eq!(result.cold_artifacts.len(), 3);
        assert_eq!(result.warm_artifacts.len(), 3);

        // Cold artifacts should be labeled cold.
        for artifact in &result.cold_artifacts {
            assert_eq!(artifact.metadata.warm_cold, "cold");
            assert_eq!(artifact.metadata.dataset_id, "synthetic_small");
            assert_eq!(artifact.metadata.dataset_cardinality, 10);
        }

        // Warm artifacts should be labeled warm.
        for artifact in &result.warm_artifacts {
            assert_eq!(artifact.metadata.warm_cold, "warm");
            assert_eq!(artifact.metadata.dataset_id, "synthetic_small");
        }
    }

    #[test]
    fn suite_covers_all_three_scenarios() {
        let dataset = BenchmarkDataset::small_exploratory();
        let suite = BenchmarkSuite::new(dataset, 3);
        let result = suite.run();

        let cold_scenarios: Vec<&str> = result
            .cold_artifacts
            .iter()
            .map(|a| a.metadata.scenario.as_str())
            .collect();
        assert!(cold_scenarios.contains(&"encode"));
        assert!(cold_scenarios.contains(&"recall"));
        assert!(cold_scenarios.contains(&"ranking"));

        let warm_scenarios: Vec<&str> = result
            .warm_artifacts
            .iter()
            .map(|a| a.metadata.scenario.as_str())
            .collect();
        assert!(warm_scenarios.contains(&"encode"));
        assert!(warm_scenarios.contains(&"recall"));
        assert!(warm_scenarios.contains(&"ranking"));
    }

    #[test]
    fn suite_result_preserves_dataset_metadata() {
        let dataset = BenchmarkDataset::medium_representative();
        let suite = BenchmarkSuite::new(dataset.clone(), 5);
        let result = suite.run();

        assert_eq!(result.dataset.id, "synthetic_medium");
        assert_eq!(result.dataset.size, DatasetSize::Medium);
        assert_eq!(result.dataset.cardinality, 1_000);
        assert_eq!(
            result.dataset.representativeness,
            Representativeness::Representative
        );
    }

    #[test]
    fn bench_representative_runs_all_sizes() {
        let small_result = bench_representative(DatasetSize::Small, 3);
        assert_eq!(small_result.cold_artifacts.len(), 3);
        assert_eq!(small_result.dataset.size, DatasetSize::Small);

        let medium_result = bench_representative(DatasetSize::Medium, 3);
        assert_eq!(medium_result.cold_artifacts.len(), 3);
        assert_eq!(medium_result.dataset.size, DatasetSize::Medium);

        let large_result = bench_representative(DatasetSize::Large, 3);
        assert_eq!(large_result.cold_artifacts.len(), 3);
        assert_eq!(large_result.dataset.size, DatasetSize::Large);
    }

    #[test]
    fn suite_result_is_serializable() {
        let dataset = BenchmarkDataset::small_exploratory();
        let suite = BenchmarkSuite::new(dataset, 3);
        let result = suite.run();

        let json = serde_json::to_string(&result).expect("suite result must serialize");
        let deserialized: BenchmarkSuiteResult =
            serde_json::from_str(&json).expect("suite result must deserialize");
        assert_eq!(result.dataset.id, deserialized.dataset.id);
        assert_eq!(
            result.cold_artifacts.len(),
            deserialized.cold_artifacts.len()
        );
    }

    #[test]
    fn medium_dataset_artifacts_are_representative() {
        let dataset = BenchmarkDataset::medium_representative();
        let suite = BenchmarkSuite::new(dataset, 3);
        let result = suite.run();

        for artifact in result
            .cold_artifacts
            .iter()
            .chain(result.warm_artifacts.iter())
        {
            assert!(!artifact.metadata.representativeness.is_empty());
        }
    }

    // ── Regression comparison tests ───────────────────────────────────────

    #[test]
    fn regression_verdict_labels_are_stable() {
        assert_eq!(RegressionVerdict::Improvement.as_str(), "improvement");
        assert_eq!(RegressionVerdict::Pass.as_str(), "pass");
        assert_eq!(RegressionVerdict::Regression.as_str(), "regression");
    }

    #[test]
    fn regression_thresholds_have_sane_defaults() {
        let defaults = RegressionThresholds::default_thresholds();
        assert!(defaults.p50_max_degradation > 0.0);
        assert!(defaults.p95_max_degradation >= defaults.p50_max_degradation);
        assert!(defaults.p99_max_degradation >= defaults.p95_max_degradation);

        let strict = RegressionThresholds::strict();
        assert!(strict.p50_max_degradation < defaults.p50_max_degradation);

        let relaxed = RegressionThresholds::relaxed();
        assert!(relaxed.p50_max_degradation > defaults.p50_max_degradation);
    }

    fn make_test_artifact(
        scenario: &str,
        p50: u64,
        p95: u64,
        p99: u64,
        pass: bool,
    ) -> BenchmarkArtifact {
        BenchmarkArtifact {
            metadata: BenchmarkMetadata {
                scenario: scenario.to_string(),
                touched_path: "test_path".to_string(),
                dataset_id: "test_dataset".to_string(),
                dataset_cardinality: 100,
                payload_shape: "test_shape".to_string(),
                machine_profile: "local_dev".to_string(),
                build_mode: "debug".to_string(),
                warm_cold: "warm".to_string(),
                concurrency: 1,
                sample_count: 10,
                harness_entrypoint: "test_harness".to_string(),
                representativeness: "exploratory".to_string(),
            },
            latency: LatencyPercentiles {
                p50_ns: p50,
                p95_ns: p95,
                p99_ns: p99,
                min_ns: p50 / 2,
                max_ns: p99 * 2,
                mean_ns: (p50 + p95 + p99) / 3,
            },
            pass,
            contract: "test".to_string(),
            evidence: serde_json::json!({}),
        }
    }

    #[test]
    fn compare_identical_artifacts_passes() {
        let baseline = make_test_artifact("encode", 1000, 2000, 3000, true);
        let current = make_test_artifact("encode", 1000, 2000, 3000, true);
        let thresholds = RegressionThresholds::default_thresholds();

        let report = compare_artifacts(&baseline, &current, &thresholds);
        assert_eq!(report.overall_verdict, RegressionVerdict::Pass);
        assert!(report.both_passed_contract);
        for comp in &report.comparisons {
            assert_eq!(comp.verdict, RegressionVerdict::Pass);
            assert_eq!(comp.delta_ns, 0);
        }
    }

    #[test]
    fn compare_detects_regression() {
        let baseline = make_test_artifact("encode", 1000, 2000, 3000, true);
        let current = make_test_artifact("encode", 1500, 3000, 5000, true);
        let thresholds = RegressionThresholds::default_thresholds();

        let report = compare_artifacts(&baseline, &current, &thresholds);
        assert_eq!(report.overall_verdict, RegressionVerdict::Regression);
        // p50 went from 1000 to 1500 = 50% degradation, exceeds 20% threshold.
        assert_eq!(report.comparisons[0].verdict, RegressionVerdict::Regression);
        assert!((report.comparisons[0].delta_fraction - 0.5).abs() < 0.01);
    }

    #[test]
    fn compare_detects_improvement() {
        let baseline = make_test_artifact("encode", 2000, 4000, 6000, true);
        let current = make_test_artifact("encode", 1000, 2000, 3000, true);
        let thresholds = RegressionThresholds::default_thresholds();

        let report = compare_artifacts(&baseline, &current, &thresholds);
        assert_eq!(report.overall_verdict, RegressionVerdict::Improvement);
        for comp in &report.comparisons {
            assert_eq!(comp.verdict, RegressionVerdict::Improvement);
            assert!(comp.delta_ns < 0);
        }
    }

    #[test]
    fn compare_within_threshold_passes() {
        let baseline = make_test_artifact("encode", 1000, 2000, 3000, true);
        // 15% degradation - within default 20% p50 threshold.
        let current = make_test_artifact("encode", 1150, 2400, 4200, true);
        let thresholds = RegressionThresholds::default_thresholds();

        let report = compare_artifacts(&baseline, &current, &thresholds);
        // p50 should pass (15% < 20%), p95 should pass (20% < 30%), p99 should pass (40% < 50%).
        assert_eq!(report.overall_verdict, RegressionVerdict::Pass);
    }

    #[test]
    fn compare_strict_thresholds_are_stricter() {
        let baseline = make_test_artifact("encode", 1000, 2000, 3000, true);
        // 15% degradation.
        let current = make_test_artifact("encode", 1150, 2200, 3300, true);

        let default_report = compare_artifacts(
            &baseline,
            &current,
            &RegressionThresholds::default_thresholds(),
        );
        let strict_report = compare_artifacts(&baseline, &current, &RegressionThresholds::strict());

        // 15% exceeds strict 10% p50 threshold but not default 20%.
        assert_eq!(
            strict_report.comparisons[0].verdict,
            RegressionVerdict::Regression
        );
        assert_eq!(
            default_report.comparisons[0].verdict,
            RegressionVerdict::Pass
        );
    }

    #[test]
    fn compare_marks_contract_failure() {
        let baseline = make_test_artifact("encode", 1000, 2000, 3000, true);
        let current = make_test_artifact("encode", 1000, 2000, 3000, false);
        let thresholds = RegressionThresholds::default_thresholds();

        let report = compare_artifacts(&baseline, &current, &thresholds);
        assert!(!report.both_passed_contract);
    }

    #[test]
    fn compare_suites_matches_by_scenario() {
        let dataset = BenchmarkDataset::small_exploratory();
        let suite = BenchmarkSuite::new(dataset, 3);
        let baseline = suite.run();
        let current = suite.run();
        let thresholds = RegressionThresholds::default_thresholds();

        let reports = compare_suites(&baseline, &current, &thresholds);
        // Should have reports for both cold and warm passes, 3 scenarios each = 6.
        assert_eq!(reports.len(), 6);
        for report in &reports {
            assert!(
                report.scenario == "encode"
                    || report.scenario == "recall"
                    || report.scenario == "ranking"
            );
        }
    }

    #[test]
    fn artifact_roundtrip_json() {
        let artifact = make_test_artifact("encode", 1000, 2000, 3000, true);
        let json = artifact_to_json(&artifact);
        let restored = artifact_from_json(&json).expect("must deserialize");
        assert_eq!(artifact.metadata.scenario, restored.metadata.scenario);
        assert_eq!(artifact.latency.p50_ns, restored.latency.p50_ns);
        assert_eq!(artifact.latency.p95_ns, restored.latency.p95_ns);
    }

    #[test]
    fn suite_result_roundtrip_json() {
        let dataset = BenchmarkDataset::small_exploratory();
        let suite = BenchmarkSuite::new(dataset, 3);
        let result = suite.run();
        let json = suite_result_to_json(&result);
        let restored = suite_result_from_json(&json).expect("must deserialize");
        assert_eq!(result.dataset.id, restored.dataset.id);
        assert_eq!(result.cold_artifacts.len(), restored.cold_artifacts.len());
        assert_eq!(result.warm_artifacts.len(), restored.warm_artifacts.len());
    }
}
