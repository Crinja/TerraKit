//! Storage access invocation requests.

use crate::resource::{StorageAccessId, StorageAccessInterface};

/// One request to open a versioned storage access contract.
///
/// Most contracts can begin with [`StorageAccessRequest::new`]. Contracts that
/// need request-specific operations may attach an opaque interface whose layout
/// is defined by the access-contract crate rather than by core. Storage
/// implementations may inspect that request interface only during `open_access`;
/// they must not retain its pointers after the call returns.
#[derive(Debug, Clone, Copy)]
pub struct StorageAccessRequest<'a> {
    access: &'a StorageAccessId,
    interface: Option<StorageAccessInterface<'a>>,
}

impl<'a> StorageAccessRequest<'a> {
    /// Creates an access request with no contract-specific request interface.
    pub const fn new(access: &'a StorageAccessId) -> Self {
        Self {
            access,
            interface: None,
        }
    }

    /// Creates an access request with one contract-specific request interface.
    ///
    /// The requested access contract is derived from the interface itself, so
    /// safe code cannot associate an interface with another access contract.
    ///
    /// ```compile_fail
    /// use std::ffi::c_void;
    /// use std::ptr::NonNull;
    /// use terrakit_core::resource::{
    ///     StorageAccessId, StorageAccessInterface, StorageAccessRequest,
    /// };
    ///
    /// let access_a = StorageAccessId::new("plugin.test.a@1").unwrap();
    /// let access_b = StorageAccessId::new("plugin.test.b@1").unwrap();
    /// let vtable = NonNull::<c_void>::dangling();
    ///
    /// // SAFETY: illustrative raw interface only.
    /// let interface = unsafe {
    ///     StorageAccessInterface::from_raw_parts(
    ///         &access_a,
    ///         std::ptr::null_mut(),
    ///         vtable,
    ///     )
    /// };
    ///
    /// // No safe constructor accepts a separate ID alongside the interface.
    /// let _ = StorageAccessRequest::with_interface(&access_b, interface);
    /// ```
    pub const fn with_interface(interface: StorageAccessInterface<'a>) -> Self {
        Self {
            access: interface.access_id(),
            interface: Some(interface),
        }
    }

    /// Returns the requested access contract ID.
    pub const fn access_id(&self) -> &'a StorageAccessId {
        self.access
    }

    /// Returns the optional contract-specific request interface.
    pub const fn interface(&self) -> Option<StorageAccessInterface<'a>> {
        self.interface
    }
}
