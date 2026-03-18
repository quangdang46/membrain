//! Canonical core crate boundaries for the initial workspace bootstrap.

mod brain_store;
mod config;
mod constants;
#[allow(dead_code)]
mod embed;
#[allow(dead_code)]
mod engine;
#[allow(dead_code)]
mod graph;
#[allow(dead_code)]
mod index;
#[allow(dead_code)]
mod migrate;
#[allow(dead_code)]
mod observability;
#[allow(dead_code)]
mod policy;
#[allow(dead_code)]
mod store;
mod types;

/// Stable top-level core facade for downstream wrappers.
pub use brain_store::BrainStore;
/// Shared runtime configuration carried by the core facade.
pub use config::RuntimeConfig;
/// Shared API version exposed to wrapper crates.
pub use types::CoreApiVersion;
