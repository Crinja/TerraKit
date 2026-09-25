//! Registered storage access contract definitions.

use crate::resource::StorageAccessId;

/// Authoritative definition of one registered storage access contract.
///
/// Definitions are created only by the owning TerraKit registry universe.
/// Core intentionally does not interpret the concrete interface semantics
/// associated with the ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageAccessDefinition {
    id: StorageAccessId,
}

impl StorageAccessDefinition {
    pub(crate) const fn new(id: StorageAccessId) -> Self {
        Self { id }
    }

    /// Returns the stable versioned access contract ID.
    pub const fn id(&self) -> &StorageAccessId {
        &self.id
    }
}
