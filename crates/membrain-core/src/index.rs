/// Stable index boundary owned by `membrain-core`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct IndexModule;

impl IndexModule {
    /// Returns the stable component identifier for this index surface.
    pub const fn component_name(&self) -> &'static str {
        "index"
    }
}
