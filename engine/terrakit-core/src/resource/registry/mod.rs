//! Resource contract registry.
//!
//! All cross-contract validation is performed through this registry so
//! metadata, capabilities and resource types belong to one contract universe.

mod capability;
mod metadata;
mod resource_type;

use std::fmt;

use capability::CapabilityRegistry;
use metadata::MetadataKeyRegistry;
use resource_type::ResourceTypeRegistry;

use crate::resource::{
    CapabilityError, MetadataKeyDefinition, MetadataKeyId, ResourceCapabilityDefinition,
    ResourceCapabilityId, ResourceTypeDefinition, ResourceTypeError, ResourceTypeId,
};

/// Registry of resource contracts.
#[derive(Debug, Clone, Default)]
pub struct ResourceRegistry {
    metadata_keys: MetadataKeyRegistry,
    capabilities: CapabilityRegistry,
    resource_types: ResourceTypeRegistry,
}

impl ResourceRegistry {
    /// Creates an empty resource registry.
    pub fn new() -> Self {
        Self {
            metadata_keys: MetadataKeyRegistry::new(),
            capabilities: CapabilityRegistry::new(),
            resource_types: ResourceTypeRegistry::new(),
        }
    }

    /// Returns one metadata key definition by ID.
    pub fn metadata_key(&self, id: &MetadataKeyId) -> Option<&MetadataKeyDefinition> {
        self.metadata_keys.get(id)
    }

    /// Returns one capability definition by ID.
    pub fn capability(&self, id: &ResourceCapabilityId) -> Option<&ResourceCapabilityDefinition> {
        self.capabilities.get(id)
    }

    /// Returns one resource type definition by ID.
    pub fn resource_type(&self, id: &ResourceTypeId) -> Option<&ResourceTypeDefinition> {
        self.resource_types.get(id)
    }

    /// Iterates over all registered metadata key definitions.
    pub fn metadata_keys(&self) -> impl Iterator<Item = &MetadataKeyDefinition> {
        self.metadata_keys.iter()
    }

    /// Iterates over all registered capability definitions.
    pub fn capabilities(&self) -> impl Iterator<Item = &ResourceCapabilityDefinition> {
        self.capabilities.iter()
    }

    /// Iterates over all registered resource type definitions.
    pub fn resource_types(&self) -> impl Iterator<Item = &ResourceTypeDefinition> {
        self.resource_types.iter()
    }
}

/// Resource registry error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceRegistryError {
    /// A metadata key with the same stable ID is already registered.
    DuplicateMetadataKey(MetadataKeyId),
    /// A capability with the same stable ID is already registered.
    DuplicateCapability(ResourceCapabilityId),
    /// A resource type with the same stable ID is already registered.
    DuplicateResourceType(ResourceTypeId),
    /// A capability references a metadata key not registered in this registry.
    UnknownMetadataKey {
        /// Capability containing the requirement.
        capability: ResourceCapabilityId,
        /// Missing metadata key.
        key: MetadataKeyId,
    },
    /// A capability spec could not be registered.
    InvalidCapability(CapabilityError),
    /// A resource type spec could not be resolved against this registry.
    InvalidResourceType(ResourceTypeError),
}

impl fmt::Display for ResourceRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateMetadataKey(id) => {
                write!(f, "metadata key '{id}' is already registered")
            }
            Self::DuplicateCapability(id) => {
                write!(f, "capability '{id}' is already registered")
            }
            Self::DuplicateResourceType(id) => {
                write!(f, "resource type '{id}' is already registered")
            }
            Self::UnknownMetadataKey { capability, key } => write!(
                f,
                "capability '{capability}' references unknown metadata key '{key}'"
            ),
            Self::InvalidCapability(error) => {
                write!(f, "invalid capability: {error}")
            }
            Self::InvalidResourceType(error) => {
                write!(f, "invalid resource type: {error}")
            }
        }
    }
}

impl std::error::Error for ResourceRegistryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidCapability(error) => Some(error),
            Self::InvalidResourceType(error) => Some(error),
            _ => None,
        }
    }
}
