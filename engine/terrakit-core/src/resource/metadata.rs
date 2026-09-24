//! Resource metadata contracts

use std::collections::{BTreeMap, btree_map::Entry};
use std::fmt;

use super::{MetadataKeyId, ResourceView};

/// Primitive metadata value kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetadataKind {
    /// Boolean flag.
    Bool,
    /// Signed integer.
    I64,
    /// Unsigned integer.
    U64,
    /// Floating-point scalar.
    F64,
    /// UTF-8 string.
    String,
    /// Symbolic identifier.
    Identifier,
}

/// Definition of one reusable metadata key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataKeyDefinition {
    id: MetadataKeyId,
    kind: MetadataKind,
}

impl MetadataKeyDefinition {
    /// Creates a metadata key definition.
    pub fn new(id: MetadataKeyId, kind: MetadataKind) -> Self {
        Self { id, kind }
    }

    /// Returns the versioned metadata key ID.
    pub fn id(&self) -> &MetadataKeyId {
        &self.id
    }

    /// Returns the value kind used by this metadata key.
    pub const fn kind(&self) -> MetadataKind {
        self.kind
    }
}

/// Registry of known metadata keys.
#[derive(Debug, Clone, Default)]
pub struct MetadataKeyRegistry {
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
                Err(MetadataKeyRegistryError::DuplicateKey(
                    entry.key().clone(),
                ))
            }
        }
    }

    /// Returns one metadata key definition by ID.
    pub fn get(
        &self,
        id: &MetadataKeyId,
    ) -> Option<&MetadataKeyDefinition> {
        self.definitions.get(id)
    }

    /// Returns whether a metadata key is registered.
    pub fn contains(&self, id: &MetadataKeyId) -> bool {
        self.definitions.contains_key(id)
    }

    /// Iterates over all registered metadata key definitions.
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = &MetadataKeyDefinition> {
        self.definitions.values()
    }

    /// Returns the number of registered metadata keys.
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Returns whether no metadata keys are registered.
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}

/// One metadata requirement declared by a capability contract.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetadataRequirement {
    key: MetadataKeyId,
    required: bool,
}

impl MetadataRequirement {
    /// Creates a required metadata entry.
    pub fn required(key: MetadataKeyId) -> Self {
        Self {
            key,
            required: true,
        }
    }

    /// Creates an optional metadata entry.
    pub fn optional(key: MetadataKeyId) -> Self {
        Self {
            key,
            required: false,
        }
    }

    /// Returns the metadata key.
    pub fn key(&self) -> &MetadataKeyId {
        &self.key
    }

    /// Returns whether the metadata entry must be present.
    pub const fn is_required(&self) -> bool {
        self.required
    }
}

/// Metadata requirement attached to one resource view.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopedMetadataRequirement {
    scope: ResourceView,
    requirement: MetadataRequirement,
}

impl ScopedMetadataRequirement {
    /// Creates a metadata requirement for one resource view.
    pub fn new(scope: ResourceView, requirement: MetadataRequirement) -> Self {
        Self {
            scope: scope.canonical(),
            requirement,
        }
    }

    /// Returns the resource view this requirement applies to.
    pub fn scope(&self) -> &ResourceView {
        &self.scope
    }

    /// Returns the metadata requirement.
    pub fn requirement(&self) -> &MetadataRequirement {
        &self.requirement
    }
}

/// Runtime metadata values attached to one resource instance.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceMetadata {
    scopes: BTreeMap<ResourceView, BTreeMap<MetadataKeyId, MetadataValue>>,
}

impl ResourceMetadata {
    /// Creates empty metadata.
    pub fn new() -> Self {
        Self {
            scopes: BTreeMap::new(),
        }
    }

    /// Inserts or replaces a metadata value on one resource view.
    pub fn insert(
        &mut self,
        scope: ResourceView,
        key: MetadataKeyId,
        value: MetadataValue,
    ) {
        self.scopes
            .entry(scope.canonical())
            .or_default()
            .insert(key, value);
    }

    /// Inserts or replaces metadata on the complete resource.
    pub fn insert_root(
        &mut self,
        key: MetadataKeyId,
        value: MetadataValue,
    ) {
        self.insert(ResourceView::Root, key, value);
    }

    /// Returns one metadata value from exactly one resource view.
    pub fn get_exact(
        &self,
        scope: &ResourceView,
        key: &MetadataKeyId,
    ) -> Option<&MetadataValue> {
        self.scopes
            .get(&scope.canonical())
            .and_then(|entries| entries.get(key))
    }

    /// Returns one effective metadata value using inheritance.
    pub fn get(
        &self,
        scope: &ResourceView,
        key: &MetadataKeyId,
    ) -> Option<&MetadataValue> {
        self.resolve(scope, key).map(|(_, value)| value)
    }

    /// Resolves one metadata value using closest-scope inheritance.
    pub fn resolve(
        &self,
        scope: &ResourceView,
        key: &MetadataKeyId,
    ) -> Option<(ResourceView, &MetadataValue)> {
        let mut current = scope.canonical();
    
        loop {
            if let Some(value) = self
                .scopes
                .get(&current)
                .and_then(|entries| entries.get(key))
            {
                return Some((current, value));
            }
    
            current = current.parent()?;
        }
    }

    /// Returns whether one effective metadata key is present.
    pub fn contains(
        &self,
        scope: &ResourceView,
        key: &MetadataKeyId,
    ) -> bool {
        self.get(scope, key).is_some()
    }

    /// Returns whether one metadata key exists exactly on a scope.
    pub fn contains_exact(
        &self,
        scope: &ResourceView,
        key: &MetadataKeyId,
    ) -> bool {
        self.get_exact(scope, key).is_some()
    }

    /// Returns all effective metadata for one resource view.
    pub fn effective_entries(
        &self,
        scope: &ResourceView,
    ) -> BTreeMap<&MetadataKeyId, &MetadataValue> {
        let mut lineage = Vec::new();
        let mut current = Some(scope.canonical());
    
        while let Some(view) = current {
            current = view.parent();
            lineage.push(view);
        }
    
        let mut effective = BTreeMap::new();
    
        for view in lineage.into_iter().rev() {
            if let Some(entries) = self.scopes.get(&view) {
                for (key, value) in entries {
                    effective.insert(key, value);
                }
            }
        }
    
        effective
    }

    /// Validates metadata against a set of scoped requirements.
    pub fn validate(
        &self,
        requirements: &[ScopedMetadataRequirement],
        registry: &MetadataKeyRegistry,
    ) -> Result<(), MetadataValidationError> {
        for (scope, key, value) in self.iter() {
            let definition = registry
                .get(key)
                .ok_or_else(|| MetadataValidationError::UnknownMetadataKey {
                    scope: scope.clone(),
                    key: key.clone(),
                })?;
    
            if value.kind() != definition.kind() {
                return Err(MetadataValidationError::KindMismatch {
                    scope: scope.clone(),
                    key: key.clone(),
                    expected: definition.kind(),
                    actual: value.kind(),
                });
            }
        }
    
        for scoped in requirements {
            let requirement = scoped.requirement();
    
            let definition = registry
                .get(requirement.key())
                .ok_or_else(|| MetadataValidationError::UnknownMetadataKey {
                    scope: scoped.scope().clone(),
                    key: requirement.key().clone(),
                })?;
    
            if self
                .resolve(scoped.scope(), requirement.key())
                .is_none()
                && requirement.is_required()
            {
                return Err(MetadataValidationError::MissingRequired {
                    scope: scoped.scope().clone(),
                    key: requirement.key().clone(),
                    expected: definition.kind(),
                });
            }
        }
    
        Ok(())
    }

    /// Iterates over all explicitly stored metadata entries.
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (&ResourceView, &MetadataKeyId, &MetadataValue)> {
        self.scopes.iter().flat_map(|(scope, entries)| {
            entries
                .iter()
                .map(move |(key, value)| (scope, key, value))
        })
    }

    /// Iterates over metadata stored exactly on one scope.
    pub fn iter_scope(
        &self,
        scope: &ResourceView,
    ) -> impl Iterator<Item = (&MetadataKeyId, &MetadataValue)> {
        let scope = scope.canonical();
    
        self.scopes
            .get(&scope)
            .into_iter()
            .flat_map(|entries| entries.iter())
    }

    /// Iterates over all explicitly stored metadata scopes.
    pub fn scopes(&self) -> impl Iterator<Item = &ResourceView> {
        self.scopes.keys()
    }

    /// Returns the number of explicitly stored metadata entries.
    pub fn len(&self) -> usize {
        self.scopes.values().map(BTreeMap::len).sum()
    }

    /// Returns whether no metadata entries exist.
    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }
}

impl Default for ResourceMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Concrete metadata value.
#[derive(Debug, Clone, PartialEq)]
pub enum MetadataValue {
    /// Boolean metadata.
    Bool(bool),
    /// Signed integer metadata.
    I64(i64),
    /// Unsigned integer metadata.
    U64(u64),
    /// Floating-point metadata.
    F64(f64),
    /// UTF-8 text metadata.
    String(Box<str>),
    /// Stable identifier metadata.
    Identifier(Box<str>),
}

impl MetadataValue {
    /// Returns the metadata value kind.
    pub const fn kind(&self) -> MetadataKind {
        match self {
            Self::Bool(_) => MetadataKind::Bool,
            Self::I64(_) => MetadataKind::I64,
            Self::U64(_) => MetadataKind::U64,
            Self::F64(_) => MetadataKind::F64,
            Self::String(_) => MetadataKind::String,
            Self::Identifier(_) => MetadataKind::Identifier,
        }
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

/// Runtime metadata validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataValidationError {
    /// Metadata was attached to a view not present in the resource schema.
    InvalidScope {
        /// Invalid metadata scope.
        scope: ResourceView,
        /// Human-readable view-resolution error.
        message: Box<str>,
    },
    /// Metadata used a key that has not been registered.
    UnknownMetadataKey {
        /// Resource view containing the metadata.
        scope: ResourceView,
        /// Unknown metadata key.
        key: MetadataKeyId,
    },
    /// A required metadata entry was not present.
    MissingRequired {
        /// Resource view requiring the metadata.
        scope: ResourceView,
        /// Missing metadata key.
        key: MetadataKeyId,
        /// Kind required by the registered metadata key.
        expected: MetadataKind,
    },
    /// A metadata entry was present with the wrong kind.
    KindMismatch {
        /// Resource view containing the metadata.
        scope: ResourceView,
        /// Metadata key whose value had the wrong kind.
        key: MetadataKeyId,
        /// Kind defined by the metadata key.
        expected: MetadataKind,
        /// Kind provided by the resource instance.
        actual: MetadataKind,
    },
}

impl fmt::Display for MetadataValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidScope { scope, message } => {
                write!(f, "invalid metadata scope {scope:?}: {message}")
            }
            Self::UnknownMetadataKey { scope, key } => {
                write!(
                    f,
                    "metadata key '{key}' on scope {scope:?} is not registered"
                )
            }
            Self::MissingRequired {
                scope,
                key,
                expected,
            } => write!(
                f,
                "required metadata '{key}' with kind {expected:?} is missing for scope {scope:?}"
            ),
            Self::KindMismatch {
                scope,
                key,
                expected,
                actual,
            } => write!(
                f,
                "metadata '{key}' on scope {scope:?} has kind mismatch: expected {expected:?}, got {actual:?}"
            ),
        }
    }
}

impl std::error::Error for MetadataValidationError {}
