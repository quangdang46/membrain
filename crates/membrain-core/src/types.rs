/// Version of the shared core API consumed by wrapper crates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreApiVersion {
    /// Major version for breaking API changes.
    pub major: u16,
    /// Minor version for additive API changes.
    pub minor: u16,
}

impl CoreApiVersion {
    pub(crate) const fn current() -> Self {
        Self { major: 0, minor: 1 }
    }
}
