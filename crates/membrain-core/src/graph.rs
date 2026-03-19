/// Stable graph boundary owned by `membrain-core`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GraphModule;

impl GraphModule {
    /// Returns the stable component identifier for this graph surface.
    pub const fn component_name(&self) -> &'static str {
        "graph"
    }
}
