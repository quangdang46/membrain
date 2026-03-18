#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PolicyDecision {
    Allow,
    Deny,
}
