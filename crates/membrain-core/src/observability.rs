#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutcomeClass {
    Accepted,
    Rejected,
    Partial,
    Preview,
    Blocked,
    Degraded,
}
