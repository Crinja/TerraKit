use std::collections::{BTreeMap, btree_map::Entry};
use std::fmt;

use crate::resource::{MetadataKeyDefinition, MetadataKeyId, MetadataKeySpec};

use super::{ResourceRegistry, ResourceRegistryError};

/// Registry of known metadata keys.
#[derive(Debug, Clone, Default)]
pub(crate) struct MetadataKeyRegistry {
    definitions: BTreeMap<MetadataKeyId, MetadataKeyDefinition>,
}

impl MetadataKeyRegistry {
    /// Creates an empty metadata key registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a metadata key definition.
    pub fn register(
        &mut self,
        definition: MetadataKeyDefinition,
    ) -> Result<(), MetadataKeyRegistryError> {
        let id = definition.id().clone();

        match self.definitions.entry(id) {
            Entry::Vacant(entry) => {
                entry.insert(definition);
                Ok(())
            }
            Entry::Occupied(entry) => {
                Err(MetadataKeyRegistryError::DuplicateKey(entry.key().clone()))
            }
        }
    }

    /// Returns one metadata key definition by ID.
    pub fn get(&self, id: &MetadataKeyId) -> Option<&MetadataKeyDefinition> {
        self.definitions.get(id)
    }

    /// Returns whether a metadata key is registered.
    pub fn contains(&self, id: &MetadataKeyId) -> bool {
        self.definitions.contains_key(id)
    }

    /// Iterates over all registered metadata key definitions.
    pub fn iter(&self) -> impl Iterator<Item = &MetadataKeyDefinition> {
        self.definitions.values()
    }
}

/// Metadata key registry error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataKeyRegistryError {
    /// A metadata key with the same stable ID is already registered.
    DuplicateKey(MetadataKeyId),
}

impl fmt::Display for MetadataKeyRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateKey(id) => {
                write!(f, "metadata key '{id}' is already registered")
            }
        }
    }
}

impl std::error::Error for MetadataKeyRegistryError {}

impl ResourceRegistry {
    /// Registers a metadata key spec.
    pub fn register_metadata_key(
        &mut self,
        spec: MetadataKeySpec,
    ) -> Result<(), ResourceRegistryError> {
        if self.metadata_keys.contains(spec.id()) {
            return Err(ResourceRegistryError::DuplicateMetadataKey(
                spec.id().clone(),
            ));
        }

        let (id, kind) = spec.into_parts();
        let definition = MetadataKeyDefinition::new(id, kind);

        self.metadata_keys
            .register(definition)
            .map_err(|error| match error {
                MetadataKeyRegistryError::DuplicateKey(id) => {
                    ResourceRegistryError::DuplicateMetadataKey(id)
                }
            })
    }
}
