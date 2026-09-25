//! Resource storage interfaces.

use crate::resource::{Schema, StorageAccessId};

use super::{
    ResolvedResourceView, StorageAccessGuard, StorageAccessRequest, StorageAccessSupport,
    StorageError, StorageViewState,
};

/// Immutable physical realization of one complete resource value.
///
/// A storage instance has one schema for its entire lifetime. Implementing this
/// trait asserts that the complete logical value is initialized, structurally
/// conforms to [`Self::schema`], and is readable through every access contract
/// reported as supported. Concrete physical representation is entirely owned by
/// the implementation and is not part of the resource schema.
///
/// Published storage is `Send + Sync` so a validated resource can cross runtime
/// worker-thread boundaries. Implementations wrapping thread-affine APIs must
/// provide their own synchronization, proxy, or dispatch mechanism.
pub trait ResourceStorage: Send + Sync {
    /// Returns the immutable logical schema represented by this storage.
    fn schema(&self) -> &Schema;

    /// Returns runtime structural state for one resolved view.
    fn view_state(&self, view: &ResolvedResourceView) -> Result<StorageViewState, StorageError>;

    /// Returns stable support for one access contract on a resolved view.
    ///
    /// This describes a stable property of the storage implementation. Temporary
    /// runtime failures belong to [`Self::open_access`].
    fn access_support(
        &self,
        view: &ResolvedResourceView,
        access: &StorageAccessId,
    ) -> StorageAccessSupport;

    /// Opens one access contract for a resolved view.
    ///
    /// The returned guard owns the complete access lifetime. Any opaque interface
    /// exposed by the guard is interpreted by the crate that owns the requested
    /// [`StorageAccessId`], never by core.
    fn open_access<'a>(
        &'a self,
        view: &ResolvedResourceView,
        request: &StorageAccessRequest<'_>,
    ) -> Result<Box<dyn StorageAccessGuard + 'a>, StorageError>;
}

/// Mutable construction boundary for resource storage.
///
/// Writers may contain incomplete logical data and are never themselves public
/// resources. The writer is runtime-erased so factories and plugin loaders can
/// construct and finish storage without knowing its concrete implementation.
/// `finish` must only produce a [`ResourceStorage`] whose complete logical value
/// is initialized and readable.
pub trait ResourceStorageWriter: Send {
    /// Returns the immutable schema being constructed.
    fn schema(&self) -> &Schema;

    /// Returns runtime structural state for one resolved view.
    fn view_state(&self, view: &ResolvedResourceView) -> Result<StorageViewState, StorageError>;

    /// Returns stable support for one construction access contract.
    fn access_support(
        &self,
        view: &ResolvedResourceView,
        access: &StorageAccessId,
    ) -> StorageAccessSupport;

    /// Opens one construction access contract for a resolved view.
    ///
    /// The mutable writer borrow provides exclusive construction access while
    /// the returned guard is alive.
    fn open_access<'a>(
        &'a mut self,
        view: &ResolvedResourceView,
        request: &StorageAccessRequest<'_>,
    ) -> Result<Box<dyn StorageAccessGuard + 'a>, StorageError>;

    /// Finalizes construction into complete readable resource storage.
    fn finish(self: Box<Self>) -> Result<Box<dyn ResourceStorage>, StorageError>;
}
