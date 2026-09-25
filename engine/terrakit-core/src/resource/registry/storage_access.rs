use std::collections::{BTreeMap, btree_map::Entry};
use std::fmt;

use crate::resource::{StorageAccessDefinition, StorageAccessId, StorageAccessSpec};

use super::{ResourceRegistry, ResourceRegistryError};

/// Registry of known storage access contracts.
#[derive(Debug, Clone, Default)]
pub(crate) struct StorageAccessRegistry {
    definitions: BTreeMap<StorageAccessId, StorageAccessDefinition>,
}

impl StorageAccessRegistry {
    /// Creates an empty storage access registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one finalized storage access definition.
    pub fn register(
        &mut self,
        definition: StorageAccessDefinition,
    ) -> Result<(), StorageAccessRegistryError> {
        let id = definition.id().clone();

        match self.definitions.entry(id) {
            Entry::Vacant(entry) => {
                entry.insert(definition);
                Ok(())
            }
            Entry::Occupied(entry) => Err(StorageAccessRegistryError::DuplicateAccess(
                entry.key().clone(),
            )),
        }
    }

    /// Returns one storage access definition by ID.
    pub fn get(&self, id: &StorageAccessId) -> Option<&StorageAccessDefinition> {
        self.definitions.get(id)
    }

    /// Returns whether a storage access contract is registered.
    pub fn contains(&self, id: &StorageAccessId) -> bool {
        self.definitions.contains_key(id)
    }

    /// Iterates over all registered storage access definitions.
    pub fn iter(&self) -> impl Iterator<Item = &StorageAccessDefinition> {
        self.definitions.values()
    }
}

/// Internal storage access registry error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageAccessRegistryError {
    /// A storage access contract with the same stable ID is already registered.
    DuplicateAccess(StorageAccessId),
}

impl fmt::Display for StorageAccessRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateAccess(id) => {
                write!(f, "storage access contract '{id}' is already registered")
            }
        }
    }
}

impl std::error::Error for StorageAccessRegistryError {}

impl ResourceRegistry {
    /// Registers one storage access contract spec.
    pub fn register_storage_access(
        &mut self,
        spec: StorageAccessSpec,
    ) -> Result<(), ResourceRegistryError> {
        if self.storage_accesses.contains(spec.id()) {
            return Err(ResourceRegistryError::DuplicateStorageAccess(
                spec.id().clone(),
            ));
        }

        let definition = StorageAccessDefinition::new(spec.into_parts());

        self.storage_accesses
            .register(definition)
            .map_err(|error| match error {
                StorageAccessRegistryError::DuplicateAccess(id) => {
                    ResourceRegistryError::DuplicateStorageAccess(id)
                }
            })
    }
}
