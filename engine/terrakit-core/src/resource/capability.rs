//! Capability contracts and registry.
//!
//! Capabilities are named, versioned interpretation whose exposed view satisifies a schema.
//! A resource may advertise any amount of capabilities

use std::collections::{BTreeMap, HashSet, btree_map::Entry};
use std::fmt;

use super::{
    MetadataKeyId, MetadataKeyRegistry, MetadataRequirement, ResourceCapabilityId, ResourceView,
    Schema,
};

/// Definition of one reusable resource capability
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceCapabilityDefinition {
    id: ResourceCapabilityId,
    view_schema: Schema,
    metadata: Vec<MetadataRequirement>,
}

impl ResourceCapabilityDefinition {
    /// Creates a capability definition
    pub fn new(
        id: ResourceCapabilityId,
        view_schema: Schema,
        metadata: Vec<MetadataRequirement>,
        metadata_registry: &MetadataKeyRegistry,
    ) -> Result<Self, CapabilityError> {
        view_schema
            .validate()
            .map_err(|error| CapabilityError::InvalidSchema(error.to_string().into()))?;

        let mut keys = HashSet::with_capacity(metadata.len());

        for requirement in &metadata {
            if !metadata_registry.contains(requirement.key()) {
                return Err(CapabilityError::UnknownMetadataKey(
                    requirement.key().clone(),
                ));
            }

            if !keys.insert(requirement.key().clone()) {
                return Err(CapabilityError::DuplicateMetadataRequirement(
                    requirement.key().clone(),
                ));
            }
        }

        Ok(Self {
            id,
            view_schema,
            metadata,
        })
    }

    /// Returns the stable versioned capability ID
    pub fn id(&self) -> &ResourceCapabilityId {
        &self.id
    }

    /// Returns the schema a resource view must satisfy to claim this capability.
    pub fn view_schema(&self) -> &Schema {
        &self.view_schema
    }

    /// Returns metadata requirements associated with this capability
    pub fn metadata_requirements(&self) -> &[MetadataRequirement] {
        &self.metadata
    }
}

/// Declares that one view of a resource type satisfies a capability.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityBinding {
    capability: ResourceCapabilityId,
    view: ResourceView,
}

impl CapabilityBinding {
    /// Binds a capability to the complete resource.
    pub fn root(capability: ResourceCapabilityId) -> Self {
        Self {
            capability,
            view: ResourceView::Root,
        }
    }

    /// Binds a capability to one nested resource view.
    pub fn view(capability: ResourceCapabilityId, view: ResourceView) -> Self {
        Self {
            capability,
            view: view.canonical(),
        }
    }

    /// Returns the claimed capability ID.
    pub fn capability(&self) -> &ResourceCapabilityId {
        &self.capability
    }

    /// Returns the resource view that provides the capability.
    pub fn resource_view(&self) -> &ResourceView {
        &self.view
    }
}

/// Registry of known resource capability contracts.
///
/// The registry defines which capability IDs exist and what structural
/// contract each capability requires.
///
/// It does not decide which capability a consumer should use.
#[derive(Debug, Clone, Default)]
pub struct CapabilityRegistry {
    definitions: BTreeMap<ResourceCapabilityId, ResourceCapabilityDefinition>,
}

impl CapabilityRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a capability definition.
    ///
    /// Capability IDs must be unique.
    pub fn register(
        &mut self,
        definition: ResourceCapabilityDefinition,
    ) -> Result<(), CapabilityError> {
        let id = definition.id().clone();

        match self.definitions.entry(id) {
            Entry::Vacant(entry) => {
                entry.insert(definition);
                Ok(())
            }

            Entry::Occupied(entry) => {
                Err(CapabilityError::DuplicateCapability(entry.key().clone()))
            }
        }
    }

    /// Return a capability definition by ID.
    pub fn get(&self, id: &ResourceCapabilityId) -> Option<&ResourceCapabilityDefinition> {
        self.definitions.get(id)
    }

    /// Return whether a capability is registered.
    pub fn contains(&self, id: &ResourceCapabilityId) -> bool {
        self.definitions.contains_key(id)
    }

    /// Iterate over all registered capability definitions.
    pub fn iter(&self) -> impl Iterator<Item = &ResourceCapabilityDefinition> {
        self.definitions.values()
    }

    /// Return the number of registered capabilities.
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Return whether no capabilities are registered.
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}

/// Capability registration/definition error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityError {
    /// The capability schema itself was invalid.
    InvalidSchema(Box<str>),
    /// A capability ID was registered more than once.
    DuplicateCapability(ResourceCapabilityId),
    /// A capability referenced an unknown metadata key.
    UnknownMetadataKey(MetadataKeyId),
    /// A metadata key was required more than once.
    DuplicateMetadataRequirement(MetadataKeyId),
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSchema(message) => write!(f, "invalid capability schema: {message}"),
            Self::DuplicateCapability(id) => write!(f, "capability '{id}' is already registered"),
            Self::UnknownMetadataKey(id) => {
                write!(f, "metadata key '{id}' is not registered")
            }
            Self::DuplicateMetadataRequirement(id) => write!(
                f,
                "capability metadata requirement '{id}' is declared more than once"
            ),
        }
    }
}

impl std::error::Error for CapabilityError {}
