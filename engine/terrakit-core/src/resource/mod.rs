//! Generic resource description system
//!
//! Schema =        what shape exists?
//! ResourceType =  what is the complete reporesentation?
//! Capability =    what interface can a view of it satisfy?
//! ResourceId =    which runtime value is this?

mod contract;
mod id;
mod instance;
mod registry;
mod schema;

pub use contract::{
    CapabilityBinding, CapabilityError, MetadataKeyDefinition, MetadataKeySpec, MetadataKind,
    MetadataRequirement, ResolvedMetadataRequirement, ResourceCapabilityDefinition,
    ResourceCapabilitySpec, ResourceTypeDefinition, ResourceTypeError, ResourceTypeSpec,
    ScopedMetadataRequirement, ScopedMetadataRequirementSpec,
};

pub use id::{IdError, MetadataKeyId, ResourceCapabilityId, ResourceId, ResourceTypeId};

pub use instance::{
    MetadataValidationError, MetadataValue, ResourceDescriptor, ResourceDescriptorError,
    ResourceMetadata,
};

pub use registry::{ResourceRegistry, ResourceRegistryError};

pub use schema::{
    NumericType, ResourceView, Schema, SchemaError, SchemaField, SchemaPath, SchemaPathSegment,
    SchemaVariant, ViewError,
};
