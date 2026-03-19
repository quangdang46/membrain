use crate::store::Tier2StoreApi;

/// Durable Tier2 indexed store boundary owned by `membrain-core`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Tier2Store;

impl Tier2StoreApi for Tier2Store {
    fn component_name(&self) -> &'static str {
        "store.tier2"
    }
}
