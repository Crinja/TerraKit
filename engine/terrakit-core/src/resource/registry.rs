//! Resource contract registry.
//!
//! All cross-contract validation is performed through this registry so
//! metadata, capabilities and resource types belong to one contract universe.

use std::fmt;

use super::capability::{CapabilityError, CapabilityRegistry};
use super::metadata::{MetadataKeyRegistry, MetadataKeyRegistryError};
use super::resource_type::{ResourceTypeRegistry, ResourceTypeRegistryError};
use super::{
    MetadataKeyDefinition, MetadataKeyId, MetadataValidationError, ResourceCapabilityDefinition,
    ResourceCapabilityId, ResourceMetadata, ResourceTypeDefinition, ResourceTypeError,
    ResourceTypeId,
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

    /// Registers a metadata key definition.
    pub fn register_metadata_key(
        &mut self,
        definition: MetadataKeyDefinition,
    ) -> Result<(), ResourceRegistryError> {
        self.metadata_keys
            .register(definition)
            .map_err(|error| match error {
                MetadataKeyRegistryError::DuplicateKey(id) => {
                    ResourceRegistryError::DuplicateMetadataKey(id)
                }
            })
    }

    /// Registers a capability definition.
    ///
    /// All metadata keys referenced by the capability must already exist in
    /// this registry.
    pub fn register_capability(
        &mut self,
        definition: ResourceCapabilityDefinition,
    ) -> Result<(), ResourceRegistryError> {
        if self.capabilities.contains(definition.id()) {
            return Err(ResourceRegistryError::DuplicateCapability(
                definition.id().clone(),
            ));
        }

        for requirement in definition.metadata_requirements() {
            if !self.metadata_keys.contains(requirement.key()) {
                return Err(ResourceRegistryError::UnknownMetadataKey {
                    capability: definition.id().clone(),
                    key: requirement.key().clone(),
                });
            }
        }

        self.capabilities
            .register(definition)
            .map_err(|error| match error {
                CapabilityError::DuplicateCapability(id) => {
                    ResourceRegistryError::DuplicateCapability(id)
                }
                CapabilityError::InvalidSchema(message) => {
                    ResourceRegistryError::InvalidCapability(message)
                }
                CapabilityError::DuplicateMetadataRequirement(key) => {
                    ResourceRegistryError::InvalidCapability(
                        format!("metadata requirement '{key}' is declared more than once").into(),
                    )
                }
            })
    }

    /// Registers and resolves one resource type definition.
    ///
    /// Capability contracts and metadata kinds are resolved from this registry
    /// before the type is stored.
    pub fn register_resource_type(
        &mut self,
        definition: ResourceTypeDefinition,
    ) -> Result<(), ResourceRegistryError> {
        if self.resource_types.contains(definition.id()) {
            return Err(ResourceRegistryError::DuplicateResourceType(
                definition.id().clone(),
            ));
        }

        let definition = definition
            .resolve_contracts(&self.capabilities, &self.metadata_keys)
            .map_err(ResourceRegistryError::InvalidResourceType)?;

        self.resource_types
            .register(definition)
            .map_err(|error| match error {
                ResourceTypeRegistryError::DuplicateType(id) => {
                    ResourceRegistryError::DuplicateResourceType(id)
                }
            })
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

    /// Validates metadata for one registered resource type.
    pub(crate) fn validate_metadata(
        &self,
        definition: &ResourceTypeDefinition,
        metadata: &ResourceMetadata,
    ) -> Result<(), MetadataValidationError> {
        definition.validate_metadata(metadata, &self.metadata_keys)
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
    /// A capability definition could not be registered.
    InvalidCapability(Box<str>),
    /// A resource type could not be resolved against this registry.
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
            Self::InvalidCapability(message) => {
                write!(f, "invalid capability: {message}")
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
            Self::InvalidResourceType(error) => Some(error),
            _ => None,
        }
    }
}
