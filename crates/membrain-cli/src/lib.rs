use membrain_core::{BrainStore, CoreApiVersion};

/// Returns the shared core API version used by the CLI wrapper.
pub fn core_api_version() -> CoreApiVersion {
    BrainStore::api_version()
}
