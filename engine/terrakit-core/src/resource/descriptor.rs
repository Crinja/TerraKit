//! Runtime resource descriptor.

use std::fmt;

use super::{
    MetadataKeyRegistry, MetadataValidationError, ResourceId, ResourceMetadata, ResourceTypeId,
    ResourceTypeRegistry,
};

/// Description of one runtime resource instance.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceDescriptor {
    id: ResourceId,
    resource_type: ResourceTypeId,
    metadata: ResourceMetadata,
}

impl ResourceDescriptor {
    /// Creates and validates a resource descriptor.
    pub fn new(
        id: ResourceId,
        resource_type: ResourceTypeId,
        metadata: ResourceMetadata,
        type_registry: &ResourceTypeRegistry,
        metadata_registry: &MetadataKeyRegistry,
    ) -> Result<Self, ResourceDescriptorError> {
        let definition = type_registry
            .get(&resource_type)
            .ok_or_else(|| ResourceDescriptorError::UnknownResourceType(resource_type.clone()))?;

        definition
            .validate_metadata(&metadata, metadata_registry)
            .map_err(ResourceDescriptorError::InvalidMetadata)?;

        Ok(Self {
            id,
            resource_type,
            metadata,
        })
    }

    /// Returns the runtime resource ID.
    pub const fn id(&self) -> &ResourceId {
        &self.id
    }

    /// Returns the resource type ID.
    pub fn resource_type(&self) -> &ResourceTypeId {
        &self.resource_type
    }

    /// Returns the runtime metadata.
    pub fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }
}

/// Resource descriptor validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceDescriptorError {
    /// The referenced resource type is not registered.
    UnknownResourceType(ResourceTypeId),
    /// Runtime metadata does not satisfy the resource type.
    InvalidMetadata(MetadataValidationError),
}

impl fmt::Display for ResourceDescriptorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownResourceType(id) => {
                write!(f, "unknown resource type '{id}'")
            }
            Self::InvalidMetadata(error) => {
                write!(f, "invalid resource metadata: {error}")
            }
        }
    }
}

impl std::error::Error for ResourceDescriptorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnknownResourceType(_) => None,
            Self::InvalidMetadata(error) => Some(error),
        }
    }
}
