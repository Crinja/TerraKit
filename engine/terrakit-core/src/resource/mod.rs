//! Generic resource description system.
//!
//! Schema =       what shape exists?
//! ResourceType = what is the complete representation?
//! Capability =   what interface can a view of it satisfy?
//! ResourceId =   which runtime value is this?

mod capability;
mod id;
mod metadata;
mod resource_type;
mod schema;
mod view;

pub use capability::{
    CapabilityBinding, CapabilityError, CapabilityRegistry, ResourceCapabilityDefinition,
};

pub use id::{IdError, MetadataKeyId, ResourceCapabilityId, ResourceId, ResourceTypeId};

pub use metadata::{
    MetadataKeyDefinition, MetadataKeyRegistry, MetadataKeyRegistryError, MetadataKind,
    MetadataRequirement, MetadataValidationError, MetadataValue, ResourceMetadata,
    ScopedMetadataRequirement,
};

pub use resource_type::{
    ResourceTypeDefinition, ResourceTypeError, ResourceTypeRegistry, ResourceTypeRegistryError,
};

pub use schema::{NumericType, Schema, SchemaError, SchemaField, SchemaVariant};

pub use view::{ResourceView, SchemaPath, SchemaPathSegment, ViewError};
