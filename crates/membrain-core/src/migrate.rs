/// Stable migration boundary owned by `membrain-core`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MigrationModule;

impl MigrationModule {
    /// Returns the stable component identifier for this migration surface.
    pub const fn component_name(&self) -> &'static str {
        "migrate"
    }
}
