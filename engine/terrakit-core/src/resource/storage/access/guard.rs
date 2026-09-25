//! Storage access lifetime guards.

use crate::resource::StorageAccessInterface;

/// Lifetime-owning result of opening one storage access contract.
///
/// Implementations own any locks, mappings, staging allocations, plugin
/// lifetime pins, or native handles needed to keep the exposed interface valid.
/// Dropping the guard closes the access operation.
pub trait StorageAccessGuard {
    /// Returns the opaque contract-specific interface.
    ///
    /// The returned pointers remain valid only while this guard remains alive.
    fn interface(&self) -> StorageAccessInterface<'_>;
}
