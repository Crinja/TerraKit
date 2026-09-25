//! Storage access support discovery.

/// Stable support for one storage access contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StorageAccessSupport {
    /// The backend supports this access contract.
    Supported,
    /// The backend does not support this access contract.
    Unsupported,
}

impl StorageAccessSupport {
    /// Returns whether the access contract is supported.
    pub const fn is_supported(self) -> bool {
        matches!(self, Self::Supported)
    }
}
