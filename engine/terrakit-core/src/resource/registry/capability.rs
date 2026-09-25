use std::collections::{BTreeMap, HashSet, btree_map::Entry};

use crate::resource::{
    CapabilityError, ResolvedMetadataRequirement, ResourceCapabilityDefinition,
    ResourceCapabilityId, ResourceCapabilitySpec,
};

use super::{ResourceRegistry, ResourceRegistryError};

/// Registry of known resource capability contracts.
///
/// The registry defines which capability IDs exist and what structural
/// contract each capability requires.
///
/// It does not decide which capability a consumer should use.
#[derive(Debug, Clone, Default)]
pub(crate) struct CapabilityRegistry {
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
}

impl ResourceRegistry {
    /// Registers and resolves one capability spec.
    ///
    /// All metadata keys referenced by the capability must already exist in
    /// this registry.
    pub fn register_capability(
        &mut self,
        spec: ResourceCapabilitySpec,
    ) -> Result<(), ResourceRegistryError> {
        if self.capabilities.contains(spec.id()) {
            return Err(ResourceRegistryError::DuplicateCapability(
                spec.id().clone(),
            ));
        }

        let (id, view_schema, metadata) = spec.into_parts();

        view_schema.validate().map_err(|error| {
            ResourceRegistryError::InvalidCapability(CapabilityError::InvalidSchema(
                error.to_string().into(),
            ))
        })?;

        let mut seen = HashSet::with_capacity(metadata.len());
        let mut resolved = Vec::with_capacity(metadata.len());

        for requirement in metadata {
            if !seen.insert(requirement.key().clone()) {
                return Err(ResourceRegistryError::InvalidCapability(
                    CapabilityError::DuplicateMetadataRequirement(requirement.key().clone()),
                ));
            }

            let key_definition = self.metadata_keys.get(requirement.key()).ok_or_else(|| {
                ResourceRegistryError::UnknownMetadataKey {
                    capability: id.clone(),
                    key: requirement.key().clone(),
                }
            })?;

            resolved.push(ResolvedMetadataRequirement::new(
                requirement,
                key_definition.kind(),
            ));
        }

        let definition = ResourceCapabilityDefinition::new(id, view_schema, resolved);

        self.capabilities
            .register(definition)
            .map_err(|error| match error {
                CapabilityError::DuplicateCapability(id) => {
                    ResourceRegistryError::DuplicateCapability(id)
                }
                error => ResourceRegistryError::InvalidCapability(error),
            })
    }
}
