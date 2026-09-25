//! Runtime resource descriptor.

use std::fmt;

use crate::resource::registry::ResourceRegistryId;
use crate::resource::{
    MetadataValidationError, ResourceId, ResourceMetadata, ResourceRegistry, ResourceTypeId,
};

/// Description of one runtime resource instance.
///
/// The descriptor identifies the runtime value, references its resource type,
/// and owns the metadata associated with that instance.
///
/// Storage is deliberately not part of the descriptor.
/// The descriptor remains bound to the registry that validated it.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceDescriptor {
    id: ResourceId,
    resource_type: ResourceTypeId,
    metadata: ResourceMetadata,
    registry_id: ResourceRegistryId,
}

impl ResourceDescriptor {
    /// Creates and validates a resource descriptor.
    pub fn new(
        id: ResourceId,
        resource_type: ResourceTypeId,
        metadata: ResourceMetadata,
        registry: &ResourceRegistry,
    ) -> Result<Self, ResourceDescriptorError> {
        let definition = registry
            .resource_type(&resource_type)
            .ok_or_else(|| ResourceDescriptorError::UnknownResourceType(resource_type.clone()))?;

        registry
            .validate_metadata(definition, &metadata)
            .map_err(ResourceDescriptorError::InvalidMetadata)?;

        Ok(Self {
            id,
            resource_type,
            metadata,
            registry_id: registry.id(),
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

    pub(crate) const fn registry_id(&self) -> ResourceRegistryId {
        self.registry_id
    }
}

/// Resource descriptor validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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
