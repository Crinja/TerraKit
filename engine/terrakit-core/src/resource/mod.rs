//! Generic resource description system
//!
//! Schema =        what shape exists?
//! ResourceType =  what is the complete representation?
//! Capability =    what interface can a view of it satisfy?
//! ResourceId =    which runtime value is this?

mod contract;
mod id;
mod instance;
mod registry;
mod schema;
mod storage;

pub use contract::{
    CapabilityBinding, CapabilityError, MetadataInheritance, MetadataKeyDefinition,
    MetadataKeySpec, MetadataKind, MetadataRequirement, ResolvedMetadataRequirement,
    ResourceCapabilityDefinition, ResourceCapabilitySpec, ResourceTypeDefinition,
    ResourceTypeError, ResourceTypeSpec, ScopedMetadataRequirement, ScopedMetadataRequirementSpec,
};

pub use id::{
    IdError, MetadataKeyId, ResourceCapabilityId, ResourceId, ResourceTypeId, StorageAccessId,
};

pub use instance::{
    MetadataValidationError, MetadataValue, Resource, ResourceAccess, ResourceAccessError,
    ResourceDescriptor, ResourceDescriptorError, ResourceError, ResourceMetadata,
};

pub use registry::{MetadataLookupError, ResourceRegistry, ResourceRegistryError};

pub use schema::{
    NumericType, ResourcePath, ResourcePathSegment, ResourceView, Schema, SchemaError, SchemaField,
    SchemaPath, SchemaPathSegment, SchemaVariant, ViewError,
};

pub use storage::{
    ResolvedResourceView, ResourceStorage, ResourceStorageWriter, ResourceViewResolveError,
    StorageAccessDefinition, StorageAccessGuard, StorageAccessInterface, StorageAccessRequest,
    StorageAccessSpec, StorageAccessSupport, StorageError, StorageViewState, StorageViewStateError,
};
