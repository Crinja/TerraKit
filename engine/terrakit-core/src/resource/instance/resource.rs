//! Validated runtime resource.

use std::fmt;

use crate::resource::{
    ResolvedResourceView, ResourceDescriptor, ResourceId, ResourceRegistry, ResourceStorage,
    ResourceTypeId, ResourceView, ResourceViewResolveError, Schema, StorageAccessGuard,
    StorageAccessId, StorageAccessInterface, StorageAccessRequest, StorageAccessSupport,
    StorageError, StorageViewState, StorageViewStateError,
};

/// One complete readable runtime resource.
///
/// A resource combines semantic identity with immutable physical storage.
/// Construction proves that both agree on the same resource schema. All normal
/// generic storage access is routed through this type so core can validate the
/// backend-neutral resource contract before exposing an access guard.
pub struct Resource {
    descriptor: ResourceDescriptor,
    storage: Box<dyn ResourceStorage>,
}

impl Resource {
    /// Creates a validated runtime resource.
    pub fn new(
        descriptor: ResourceDescriptor,
        storage: Box<dyn ResourceStorage>,
        registry: &ResourceRegistry,
    ) -> Result<Self, ResourceError> {
        if descriptor.registry_id() != registry.id() {
            return Err(ResourceError::ForeignDescriptor);
        }

        let definition = registry
            .resource_type(descriptor.resource_type())
            .ok_or_else(|| {
                ResourceError::UnknownResourceType(descriptor.resource_type().clone())
            })?;

        if storage.schema() != definition.schema() {
            return Err(ResourceError::StorageSchemaMismatch {
                expected: definition.schema().clone(),
                actual: storage.schema().clone(),
            });
        }

        Ok(Self {
            descriptor,
            storage,
        })
    }

    /// Returns the runtime resource ID.
    pub const fn id(&self) -> &ResourceId {
        self.descriptor.id()
    }

    /// Returns the resource type ID.
    pub fn resource_type(&self) -> &ResourceTypeId {
        self.descriptor.resource_type()
    }

    /// Returns the resource descriptor.
    pub fn descriptor(&self) -> &ResourceDescriptor {
        &self.descriptor
    }

    /// Returns the immutable schema realized by this resource.
    pub fn schema(&self) -> &Schema {
        self.storage.schema()
    }

    /// Resolves one addressable resource view against this resource's schema.
    pub fn resolve_view(
        &self,
        view: &ResourceView,
    ) -> Result<ResolvedResourceView, ResourceViewResolveError> {
        ResolvedResourceView::resolve(view, self.storage.schema())
    }

    /// Returns validated runtime structural state for one resource view.
    pub fn view_state(&self, view: &ResourceView) -> Result<StorageViewState, ResourceAccessError> {
        let resolved = self.resolve_view(view)?;
        self.resolved_view_state(&resolved)
    }

    /// Returns stable support for one versioned access contract.
    pub fn access_support(
        &self,
        view: &ResourceView,
        access: &StorageAccessId,
    ) -> Result<StorageAccessSupport, ResourceAccessError> {
        let resolved = self.resolve_view(view)?;
        self.resolved_view_state(&resolved)?;

        Ok(self.storage.access_support(&resolved, access))
    }

    /// Opens one versioned access contract through the validated resource boundary.
    ///
    /// Core resolves the logical view, validates its runtime structural state,
    /// checks advertised access support, and verifies that the returned guard is
    /// for the requested access contract. The access-contract crate then uses
    /// [`ResourceAccess::interface`] together with the resolved schema/state to
    /// perform any contract-specific validation and expose a safe typed API.
    pub fn open_access<'a>(
        &'a self,
        view: &ResourceView,
        request: &StorageAccessRequest<'_>,
    ) -> Result<ResourceAccess<'a>, ResourceAccessError> {
        let resolved = self.resolve_view(view)?;
        let state = self.resolved_view_state(&resolved)?;
        let requested = request.access_id();

        if !self
            .storage
            .access_support(&resolved, requested)
            .is_supported()
        {
            return Err(ResourceAccessError::UnsupportedAccess(requested.clone()));
        }

        let guard = self
            .storage
            .open_access(&resolved, request)
            .map_err(ResourceAccessError::Storage)?;

        let returned = guard.interface().access_id().clone();

        if &returned != requested {
            return Err(ResourceAccessError::AccessContractMismatch {
                requested: requested.clone(),
                returned,
            });
        }

        Ok(ResourceAccess {
            view: resolved,
            state,
            guard,
        })
    }

    fn resolved_view_state(
        &self,
        view: &ResolvedResourceView,
    ) -> Result<StorageViewState, ResourceAccessError> {
        let state = self
            .storage
            .view_state(view)
            .map_err(ResourceAccessError::Storage)?;

        state
            .validate_for(view.schema())
            .map_err(ResourceAccessError::InvalidStorageState)?;

        Ok(state)
    }
}

/// One opened, core-validated storage access operation.
///
/// The opaque interface is only meaningful to the crate that owns its
/// [`StorageAccessId`]. This object owns the underlying storage guard, so any
/// contract-specific context/vtable exposed through [`Self::interface`] remains
/// valid for the lifetime of this value.
pub struct ResourceAccess<'a> {
    view: ResolvedResourceView,
    state: StorageViewState,
    guard: Box<dyn StorageAccessGuard + 'a>,
}

impl<'a> ResourceAccess<'a> {
    /// Returns the core-resolved logical view for this operation.
    pub fn view(&self) -> &ResolvedResourceView {
        &self.view
    }

    /// Returns the validated runtime structural state for this operation.
    pub fn state(&self) -> &StorageViewState {
        &self.state
    }

    /// Returns the opaque access-contract interface.
    ///
    /// The interface carries its own authoritative [`StorageAccessId`]. The
    /// access-contract crate must validate and interpret the interface according
    /// to that ID. The returned pointers cannot outlive this `ResourceAccess`.
    pub fn interface(&self) -> StorageAccessInterface<'_> {
        self.guard.interface()
    }
}

/// Runtime resource construction error.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResourceError {
    /// The descriptor was validated by another resource registry.
    ForeignDescriptor,
    /// The descriptor references a resource type unavailable in this registry.
    UnknownResourceType(ResourceTypeId),
    /// Physical storage does not realize the resource type schema.
    StorageSchemaMismatch {
        /// Schema required by the resource type.
        expected: Schema,
        /// Schema declared by the storage instance.
        actual: Schema,
    },
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForeignDescriptor => {
                write!(f, "resource descriptor belongs to another registry")
            }
            Self::UnknownResourceType(id) => {
                write!(
                    f,
                    "resource descriptor references unknown resource type '{id}'"
                )
            }
            Self::StorageSchemaMismatch { expected, actual } => write!(
                f,
                "resource storage schema mismatch: expected {expected:?}, got {actual:?}"
            ),
        }
    }
}

impl std::error::Error for ResourceError {}

/// Runtime resource access error.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResourceAccessError {
    /// The requested logical resource view is invalid.
    InvalidView(ResourceViewResolveError),
    /// The storage backend returned structural state that violates the logical schema.
    InvalidStorageState(StorageViewStateError),
    /// The storage backend does not advertise the requested access contract.
    UnsupportedAccess(StorageAccessId),
    /// The storage backend opened a different access contract than requested.
    AccessContractMismatch {
        /// Requested access contract.
        requested: StorageAccessId,
        /// Contract reported by the returned guard.
        returned: StorageAccessId,
    },
    /// Storage access failed at runtime.
    Storage(StorageError),
}

impl fmt::Display for ResourceAccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidView(error) => write!(f, "{error}"),
            Self::InvalidStorageState(error) => {
                write!(f, "storage contract violation: {error}")
            }
            Self::UnsupportedAccess(access) => {
                write!(
                    f,
                    "resource storage does not support access contract '{access}'"
                )
            }
            Self::AccessContractMismatch {
                requested,
                returned,
            } => write!(
                f,
                "storage returned access contract '{returned}' for request '{requested}'"
            ),
            Self::Storage(error) => write!(f, "resource storage access failed: {error}"),
        }
    }
}

impl std::error::Error for ResourceAccessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidView(error) => Some(error),
            Self::InvalidStorageState(error) => Some(error),
            Self::Storage(error) => Some(error),
            Self::UnsupportedAccess(_) | Self::AccessContractMismatch { .. } => None,
        }
    }
}

impl From<ResourceViewResolveError> for ResourceAccessError {
    fn from(error: ResourceViewResolveError) -> Self {
        Self::InvalidView(error)
    }
}
