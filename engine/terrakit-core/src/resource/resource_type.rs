//! Complete resource type definition.
//!
//! Resource types are compositions of schema primatives
//! These can be validated without core requireing unique domain knowledge

use std::collections::{BTreeMap, HashSet, btree_map::Entry};
use std::fmt;

use super::capability::CapabilityRegistry;
use super::metadata::MetadataKeyRegistry;
use super::{
    CapabilityBinding, MetadataKeyId, MetadataKind, MetadataRequirement, MetadataValidationError,
    ResourceCapabilityId, ResourceMetadata, ResourceTypeId, ResourceView, Schema,
    ScopedMetadataRequirement,
};

/// Complete structural contract for one resource representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceTypeDefinition {
    id: ResourceTypeId,
    schema: Schema,
    capabilities: Vec<CapabilityBinding>,
    metadata: Vec<ScopedMetadataRequirement>,
}

impl ResourceTypeDefinition {
    /// Creates and validates a resource type definition.
    pub fn new(
        id: ResourceTypeId,
        schema: Schema,
        capabilities: Vec<CapabilityBinding>,
    ) -> Result<Self, ResourceTypeError> {
        schema
            .validate()
            .map_err(|error| ResourceTypeError::InvalidSchema(error.to_string().into()))?;

        let mut seen = HashSet::with_capacity(capabilities.len());

        for binding in &capabilities {
            if !seen.insert(binding.capability().clone()) {
                return Err(ResourceTypeError::DuplicateCapabilityBinding(
                    binding.capability().clone(),
                ));
            }

            binding.resource_view().resolve(&schema).map_err(|error| {
                ResourceTypeError::InvalidView {
                    capability: binding.capability().clone(),
                    message: error.to_string().into(),
                }
            })?;
        }

        Ok(Self {
            id,
            schema,
            capabilities,
            metadata: Vec::new(),
        })
    }

    /// Resolves cross-contract requirements before registration.
    pub(crate) fn resolve_contracts(
        mut self,
        capabilities: &CapabilityRegistry,
        metadata_registry: &MetadataKeyRegistry,
    ) -> Result<Self, ResourceTypeError> {
        let mut metadata = BTreeMap::<
            ResourceView,
            BTreeMap<MetadataKeyId, (MetadataRequirement, MetadataKind)>,
        >::new();

        for binding in &self.capabilities {
            let definition = capabilities.get(binding.capability()).ok_or_else(|| {
                ResourceTypeError::UnknownCapability(binding.capability().clone())
            })?;

            let actual = binding
                .resource_view()
                .resolve(&self.schema)
                .map_err(|error| ResourceTypeError::InvalidView {
                    capability: binding.capability().clone(),
                    message: error.to_string().into(),
                })?;

            if !definition.view_schema().accepts(actual) {
                return Err(ResourceTypeError::CapabilitySchemaMismatch {
                    capability: binding.capability().clone(),
                    expected: definition.view_schema().clone(),
                    actual: actual.clone(),
                });
            }

            let scope = binding.resource_view().canonical();
            let scoped = metadata.entry(scope).or_default();

            for requirement in definition.metadata_requirements() {
                let key_definition = metadata_registry.get(requirement.key()).ok_or_else(|| {
                    ResourceTypeError::UnknownMetadataKey {
                        capability: binding.capability().clone(),
                        key: requirement.key().clone(),
                    }
                })?;

                match scoped.entry(requirement.key().clone()) {
                    Entry::Vacant(entry) => {
                        entry.insert((requirement.clone(), key_definition.kind()));
                    }
                    Entry::Occupied(mut entry) => {
                        let (existing, kind) = entry.get();
                        let should_require = requirement.is_required() && !existing.is_required();
                        let kind = *kind;

                        if should_require {
                            entry.insert((
                                MetadataRequirement::required(requirement.key().clone()),
                                kind,
                            ));
                        }
                    }
                }
            }
        }

        self.metadata = metadata
            .into_iter()
            .flat_map(|(scope, requirements)| {
                requirements.into_values().map(move |(requirement, kind)| {
                    ScopedMetadataRequirement::new(scope.clone(), requirement, kind)
                })
            })
            .collect();

        Ok(self)
    }

    /// Returns the resource type's stable versioned ID.
    pub fn id(&self) -> &ResourceTypeId {
        &self.id
    }

    /// Returns the complete structural schema.
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    /// Returns all explicitly advertised capability bindings.
    pub fn capabilities(&self) -> &[CapabilityBinding] {
        &self.capabilities
    }

    /// Returns the effective metadata requirements for this resource type.
    pub fn metadata_requirements(&self) -> &[ScopedMetadataRequirement] {
        &self.metadata
    }

    /// Validates runtime metadata for one resource instance of this type.
    pub(crate) fn validate_metadata(
        &self,
        metadata: &ResourceMetadata,
        metadata_registry: &MetadataKeyRegistry,
    ) -> Result<(), MetadataValidationError> {
        for scope in metadata.scopes() {
            scope
                .resolve(&self.schema)
                .map_err(|error| MetadataValidationError::InvalidScope {
                    scope: scope.clone(),
                    message: error.to_string().into(),
                })?;
        }

        metadata.validate(&self.metadata, metadata_registry)
    }

    /// Returns whether this type explicitly advertises a capability.
    pub fn has_capability(&self, id: &ResourceCapabilityId) -> bool {
        self.capabilities
            .iter()
            .any(|binding| binding.capability() == id)
    }

    /// Returns the binding for one explicitly advertised capability.
    pub fn capability(&self, id: &ResourceCapabilityId) -> Option<&CapabilityBinding> {
        self.capabilities
            .iter()
            .find(|binding| binding.capability() == id)
    }
}

/// Registry of concrete resource type definitions.
#[derive(Debug, Clone, Default)]
pub(crate) struct ResourceTypeRegistry {
    definitions: BTreeMap<ResourceTypeId, ResourceTypeDefinition>,
}

impl ResourceTypeRegistry {
    /// Creates an empty resource type registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a concrete resource type, rejecting duplicate IDs.
    pub fn register(
        &mut self,
        definition: ResourceTypeDefinition,
    ) -> Result<(), ResourceTypeRegistryError> {
        let id = definition.id().clone();

        match self.definitions.entry(id) {
            Entry::Vacant(entry) => {
                entry.insert(definition);
                Ok(())
            }
            Entry::Occupied(entry) => Err(ResourceTypeRegistryError::DuplicateType(
                entry.key().clone(),
            )),
        }
    }

    /// Returns one concrete resource type by ID.
    pub fn get(&self, id: &ResourceTypeId) -> Option<&ResourceTypeDefinition> {
        self.definitions.get(id)
    }

    /// Iterates over all registered resource types.
    pub fn iter(&self) -> impl Iterator<Item = &ResourceTypeDefinition> {
        self.definitions.values()
    }

    /// Returns whether a resource type is registered.
    pub fn contains(&self, id: &ResourceTypeId) -> bool {
        self.definitions.contains_key(id)
    }
}

/// Resource type registry error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResourceTypeRegistryError {
    /// A resource type with the same stable ID is already registered.
    DuplicateType(ResourceTypeId),
}

impl fmt::Display for ResourceTypeRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateType(id) => write!(f, "resource type '{id}' is already registered"),
        }
    }
}

impl std::error::Error for ResourceTypeRegistryError {}

/// Error defining one concrete resource type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceTypeError {
    /// The resource's own schema was invalid.
    InvalidSchema(Box<str>),
    /// A claimed capability was not registered.
    UnknownCapability(ResourceCapabilityId),
    /// A registered capability references a metadata key missing from this registry.
    UnknownMetadataKey {
        /// Capability containing the metadata requirement.
        capability: ResourceCapabilityId,
        /// Missing metadata key.
        key: MetadataKeyId,
    },
    /// A resource claimed the same capability more than once.
    DuplicateCapabilityBinding(ResourceCapabilityId),
    /// The binding's view path did not exist in the resource schema.
    InvalidView {
        /// Capability whose view failed.
        capability: ResourceCapabilityId,
        /// Human-readable view-resolution error.
        message: Box<str>,
    },
    /// The resolved view exists but does not satisfy the capability schema.
    CapabilitySchemaMismatch {
        /// Capability being claimed.
        capability: ResourceCapabilityId,
        /// Contract schema declared by the capability.
        expected: Schema,
        /// Actual schema exposed by the resource view.
        actual: Schema,
    },
}

impl fmt::Display for ResourceTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSchema(message) => write!(f, "invalid resource schema: {message}"),
            Self::UnknownCapability(id) => write!(f, "unknown capability '{id}'"),
            Self::UnknownMetadataKey { capability, key } => write!(
                f,
                "capability '{capability}' references unknown metadata key '{key}'"
            ),
            Self::DuplicateCapabilityBinding(id) => {
                write!(f, "capability '{id}' is bound more than once")
            }
            Self::InvalidView {
                capability,
                message,
            } => write!(f, "invalid view for capability '{capability}': {message}"),
            Self::CapabilitySchemaMismatch {
                capability,
                expected,
                actual,
            } => write!(
                f,
                "capability '{capability}' schema mismatch: expected {expected:?}, got {actual:?}"
            ),
        }
    }
}

impl std::error::Error for ResourceTypeError {}
