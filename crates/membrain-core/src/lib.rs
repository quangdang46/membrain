//! Canonical core crate boundaries for the initial workspace bootstrap.

/// Stable top-level core facade and module composition.
pub mod api;
pub mod brain_store;
/// Shared runtime configuration and bounded-work budgets.
pub mod config;
/// Compile-time constants for canonical runtime defaults.
pub mod constants;
/// Embedding and generation-aware vector mechanics owned by the core crate.
pub mod embed;
/// Request-path and maintenance orchestration owned by the core crate.
pub mod engine;
/// Bounded lineage and neighborhood expansion primitives.
pub mod graph;
/// Shared health aggregation and operator-report types.
pub mod health;
/// Candidate-generation and index maintenance seams.
pub mod index;
/// Durable schema migration and compatibility surfaces.
pub mod migrate;
/// Shared trace and outcome vocabulary for all wrappers.
pub mod observability;
/// Centralized policy evaluation and early-denial surfaces.
pub mod policy;
/// Namespace-aware hot, warm, and cold storage primitives.
pub mod store;
/// Shared canonical data shapes exported to wrappers.
pub mod types;

/// Stable top-level core facade for downstream wrappers.
pub use brain_store::BrainStore;
/// Shared runtime configuration carried by the core facade.
pub use config::RuntimeConfig;
/// Shared API version exposed to wrapper crates.
pub use types::CoreApiVersion;
