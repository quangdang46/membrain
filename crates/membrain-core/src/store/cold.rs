use crate::store::ColdStoreApi;

/// Cold/archive store boundary owned by `membrain-core`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ColdStore;

impl ColdStoreApi for ColdStore {
    fn component_name(&self) -> &'static str {
        "store.cold"
    }
}
