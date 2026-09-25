//! Resource contract registry.
//!
//! All cross-contract validation is performed through this registry so
//! metadata, capabilities and resource types belong to one contract universe.

mod capability;
mod metadata;
mod resource_type;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use capability::CapabilityRegistry;
use metadata::MetadataKeyRegistry;
use resource_type::ResourceTypeRegistry;

use crate::resource::{
    CapabilityError, MetadataKeyDefinition, MetadataKeyId, MetadataValue,
    ResourceCapabilityDefinition, ResourceCapabilityId, ResourceDescriptor, ResourceTypeDefinition,
    ResourceTypeError, ResourceTypeId, ResourceView,
};

static NEXT_RESOURCE_REGISTRY_ID: AtomicU64 = AtomicU64::new(1);

/// Internal identity for one contract universe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ResourceRegistryId(u64);

impl ResourceRegistryId {
    fn next() -> Self {
        Self(NEXT_RESOURCE_REGISTRY_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Registry of resource contracts.
#[derive(Debug)]
pub struct ResourceRegistry {
    id: ResourceRegistryId,
    metadata_keys: MetadataKeyRegistry,
    capabilities: CapabilityRegistry,
    resource_types: ResourceTypeRegistry,
}

impl ResourceRegistry {
    /// Creates an empty resource registry.
    pub fn new() -> Self {
        Self {
            id: ResourceRegistryId::next(),
            metadata_keys: MetadataKeyRegistry::new(),
            capabilities: CapabilityRegistry::new(),
            resource_types: ResourceTypeRegistry::new(),
        }
    }

    pub(crate) const fn id(&self) -> ResourceRegistryId {
        self.id
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

    /// Resolves one metadata value for a validated resource descriptor.
    pub fn resolve_metadata<'a>(
        &self,
        descriptor: &'a ResourceDescriptor,
        scope: &ResourceView,
        key: &MetadataKeyId,
    ) -> Result<Option<(ResourceView, &'a MetadataValue)>, MetadataLookupError> {
        self.validate_descriptor_view(descriptor, scope)?;

        let definition = self
            .metadata_keys
            .get(key)
            .ok_or_else(|| MetadataLookupError::UnknownMetadataKey(key.clone()))?;

        Ok(descriptor
            .metadata()
            .resolve(scope, key, definition.inheritance()))
    }

    /// Returns one effective metadata value for a validated resource descriptor.
    pub fn metadata_value<'a>(
        &self,
        descriptor: &'a ResourceDescriptor,
        scope: &ResourceView,
        key: &MetadataKeyId,
    ) -> Result<Option<&'a MetadataValue>, MetadataLookupError> {
        Ok(self
            .resolve_metadata(descriptor, scope, key)?
            .map(|(_, value)| value))
    }

    /// Returns all effective metadata for one resource view.
    pub fn effective_metadata<'a>(
        &self,
        descriptor: &'a ResourceDescriptor,
        scope: &ResourceView,
    ) -> Result<BTreeMap<&'a MetadataKeyId, &'a MetadataValue>, MetadataLookupError> {
        self.validate_descriptor_view(descriptor, scope)?;

        let keys: BTreeSet<_> = descriptor
            .metadata()
            .iter()
            .map(|(_, key, _)| key)
            .collect();

        let mut effective = BTreeMap::new();

        for key in keys {
            if let Some(value) = self.metadata_value(descriptor, scope, key)? {
                effective.insert(key, value);
            }
        }

        Ok(effective)
    }

    fn validate_descriptor_view(
        &self,
        descriptor: &ResourceDescriptor,
        scope: &ResourceView,
    ) -> Result<&ResourceTypeDefinition, MetadataLookupError> {
        if descriptor.registry_id() != self.id {
            return Err(MetadataLookupError::ForeignDescriptor);
        }

        let definition = self
            .resource_types
            .get(descriptor.resource_type())
            .ok_or_else(|| {
                MetadataLookupError::UnknownResourceType(descriptor.resource_type().clone())
            })?;

        scope
            .resolve(definition.schema())
            .map_err(|error| MetadataLookupError::InvalidView {
                view: scope.clone(),
                message: error.to_string().into(),
            })?;

        Ok(definition)
    }
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata lookup error.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MetadataLookupError {
    /// The descriptor was validated by another resource registry.
    ForeignDescriptor,
    /// The descriptor references a resource type unavailable in this registry.
    UnknownResourceType(ResourceTypeId),
    /// The requested metadata key is not registered.
    UnknownMetadataKey(MetadataKeyId),
    /// The requested resource view does not exist in the descriptor's schema.
    InvalidView {
        /// Invalid resource view.
        view: ResourceView,
        /// Human-readable view-resolution error.
        message: Box<str>,
    },
}

impl fmt::Display for MetadataLookupError {
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
            Self::UnknownMetadataKey(id) => {
                write!(f, "metadata key '{id}' is not registered")
            }
            Self::InvalidView { view, message } => {
                write!(f, "invalid metadata lookup view {view:?}: {message}")
            }
        }
    }
}

impl std::error::Error for MetadataLookupError {}

/// Resource registry error.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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
