//! Resource contracts.

mod capability;
mod metadata;
mod resource_type;

pub use capability::{
    CapabilityBinding, CapabilityError, ResourceCapabilityDefinition, ResourceCapabilitySpec,
};
pub use metadata::{
    MetadataKeyDefinition, MetadataKeySpec, MetadataKind, MetadataRequirement,
    ResolvedMetadataRequirement, ScopedMetadataRequirement, ScopedMetadataRequirementSpec,
};
pub use resource_type::{ResourceTypeDefinition, ResourceTypeError, ResourceTypeSpec};
