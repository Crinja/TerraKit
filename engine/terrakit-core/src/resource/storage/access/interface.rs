//! Opaque storage access interfaces.

use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::resource::StorageAccessId;

/// Opaque contract-specific interface exposed by one access guard.
///
/// Core does not interpret these pointers. The crate that owns the matching
/// [`StorageAccessId`] defines the vtable layout and provides the safe wrapper
/// used by consumers. Both pointers are valid only for this interface's
/// lifetime, which is tied to the guard that exposed it.
///
/// The access ID is inseparable from the interface. Safe Rust can copy or
/// forward an interface, but cannot relabel it as a different access contract.
#[derive(Debug, Clone, Copy)]
pub struct StorageAccessInterface<'a> {
    access_id: &'a StorageAccessId,
    context: *mut c_void,
    vtable: NonNull<c_void>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> StorageAccessInterface<'a> {
    /// Creates an opaque access interface from contract-specific raw parts.
    ///
    /// # Safety
    ///
    /// `context` and `vtable` must implement the exact access contract
    /// identified by `access_id` and remain valid for `'a`. The vtable must use
    /// the layout defined by that contract. Core never dereferences either
    /// pointer.
    pub unsafe fn from_raw_parts(
        access_id: &'a StorageAccessId,
        context: *mut c_void,
        vtable: NonNull<c_void>,
    ) -> Self {
        Self {
            access_id,
            context,
            vtable,
            _lifetime: PhantomData,
        }
    }

    /// Returns the access contract implemented by this interface.
    pub const fn access_id(&self) -> &'a StorageAccessId {
        self.access_id
    }

    /// Returns the opaque contract-specific context pointer.
    pub const fn context(self) -> *mut c_void {
        self.context
    }

    /// Returns the opaque contract-specific vtable pointer.
    pub const fn vtable(self) -> NonNull<c_void> {
        self.vtable
    }
}
