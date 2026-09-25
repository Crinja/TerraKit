//! Runtime resource metadata.

use std::collections::BTreeMap;
use std::fmt;

use crate::resource::{MetadataInheritance, MetadataKeyId, MetadataKind, ResourceView};

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
    pub fn insert(&mut self, scope: ResourceView, key: MetadataKeyId, value: MetadataValue) {
        self.scopes
            .entry(scope.canonical())
            .or_default()
            .insert(key, value);
    }

    /// Inserts or replaces metadata on the complete resource.
    pub fn insert_root(&mut self, key: MetadataKeyId, value: MetadataValue) {
        self.insert(ResourceView::Root, key, value);
    }

    /// Returns one metadata value from exactly one resource view.
    pub fn get_exact(&self, scope: &ResourceView, key: &MetadataKeyId) -> Option<&MetadataValue> {
        self.scopes
            .get(&scope.canonical())
            .and_then(|entries| entries.get(key))
    }

    /// Returns whether one metadata key exists exactly on a scope.
    pub fn contains_exact(&self, scope: &ResourceView, key: &MetadataKeyId) -> bool {
        self.get_exact(scope, key).is_some()
    }

    /// Resolves one metadata value using the supplied inheritance behaviour.
    pub(crate) fn resolve(
        &self,
        scope: &ResourceView,
        key: &MetadataKeyId,
        inheritance: MetadataInheritance,
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

            if inheritance == MetadataInheritance::Exact {
                return None;
            }

            current = current.parent()?;
        }
    }

    /// Iterates over all explicitly stored metadata entries.
    pub fn iter(&self) -> impl Iterator<Item = (&ResourceView, &MetadataKeyId, &MetadataValue)> {
        self.scopes.iter().flat_map(|(scope, entries)| {
            entries.iter().map(move |(key, value)| (scope, key, value))
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
