use crate::config::RuntimeConfig;
use crate::types::CoreApiVersion;

/// Stable top-level core facade for the initial workspace bootstrap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrainStore {
    config: RuntimeConfig,
}

impl BrainStore {
    /// Builds a new core facade from the shared runtime configuration.
    pub fn new(config: RuntimeConfig) -> Self {
        Self { config }
    }

    /// Returns the runtime configuration carried by this facade.
    pub fn config(&self) -> RuntimeConfig {
        self.config
    }

    /// Returns the shared core API version expected by wrapper crates.
    pub const fn api_version() -> CoreApiVersion {
        CoreApiVersion::current()
    }
}
