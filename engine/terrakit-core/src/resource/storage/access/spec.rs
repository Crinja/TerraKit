//! Storage access registration input.

use crate::resource::StorageAccessId;

/// Unresolved storage access contract registration input.
///
/// Concrete semantics belong to the extension crate that owns the ID. Core
/// records only the stable identity needed for discovery and invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageAccessSpec {
    id: StorageAccessId,
}

impl StorageAccessSpec {
    /// Creates a storage access contract spec.
    pub const fn new(id: StorageAccessId) -> Self {
        Self { id }
    }

    /// Returns the stable versioned access contract ID.
    pub const fn id(&self) -> &StorageAccessId {
        &self.id
    }

    pub(crate) fn into_parts(self) -> StorageAccessId {
        self.id
    }
}
