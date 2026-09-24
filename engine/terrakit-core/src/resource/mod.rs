//! Generic resource description system.
//!
//! Schema =       what shape exists?
//! ResourceType = what is the complete representation?
//! Capability =   what interface can a view of it satisfy?
//! ResourceId =   which runtime value is this?

mod capability;
mod descriptor;
mod id;
mod metadata;
mod registry;
mod resource_type;
mod schema;
mod view;

pub use capability::{CapabilityBinding, CapabilityError, ResourceCapabilityDefinition};

pub use descriptor::{ResourceDescriptor, ResourceDescriptorError};

pub use id::{IdError, MetadataKeyId, ResourceCapabilityId, ResourceId, ResourceTypeId};

pub use metadata::{
    MetadataKeyDefinition, MetadataKind, MetadataRequirement, MetadataValidationError,
    MetadataValue, ResourceMetadata, ScopedMetadataRequirement,
};

pub use registry::{ResourceRegistry, ResourceRegistryError};

pub use resource_type::{ResourceTypeDefinition, ResourceTypeError};

pub use schema::{NumericType, Schema, SchemaError, SchemaField, SchemaVariant};

pub use view::{ResourceView, SchemaPath, SchemaPathSegment, ViewError};
