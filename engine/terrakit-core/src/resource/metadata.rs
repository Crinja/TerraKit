//! Resource metadata contracts

use std::collections::BTreeMap;
use std::fmt;

use super::ResourceView;

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

/// One metadata requirement declared by a capability contract.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetadataRequirement {
    key: Box<str>,
    kind: MetadataKind,
    required: bool,
}

impl MetadataRequirement {
    /// Creates a required metadata entry.
    pub fn required(key: impl Into<Box<str>>, kind: MetadataKind) -> Self {
        Self {
            key: key.into(),
            kind,
            required: true,
        }
    }

    /// Creates an optional metadata entry.
    pub fn optional(key: impl Into<Box<str>>, kind: MetadataKind) -> Self {
        Self {
            key: key.into(),
            kind,
            required: false,
        }
    }

    /// Returns the metadata key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the required metadata kind.
    pub const fn kind(&self) -> MetadataKind {
        self.kind
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
    scopes: BTreeMap<ResourceView, BTreeMap<Box<str>, MetadataValue>>,
}

impl ResourceMetadata {
    /// Creates empty metadata.
    pub fn new() -> Self {
        Self {
            scopes: BTreeMap::new(),
        }
    }

    /// Inserts or replaces a metadata value on one resource view.
    pub fn insert(&mut self, scope: ResourceView, key: impl Into<Box<str>>, value: MetadataValue) {
        self.scopes
            .entry(scope.canonical())
            .or_default()
            .insert(key.into(), value);
    }

    /// Inserts or replaces metadata on the complete resource.
    pub fn insert_root(&mut self, key: impl Into<Box<str>>, value: MetadataValue) {
        self.insert(ResourceView::Root, key, value);
    }

    /// Returns one metadata value from exactly one resource view.
    pub fn get_exact(&self, scope: &ResourceView, key: &str) -> Option<&MetadataValue> {
        self.scopes
            .get(&scope.canonical())
            .and_then(|entries| entries.get(key))
    }

    /// Returns one effective metadata value using inheritance.
    pub fn get(&self, scope: &ResourceView, key: &str) -> Option<&MetadataValue> {
        self.resolve(scope, key).map(|(_, value)| value)
    }

    /// Resolves one metadata value using closest-scope inheritance.
    pub fn resolve(
        &self,
        scope: &ResourceView,
        key: &str,
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
    pub fn contains(&self, scope: &ResourceView, key: &str) -> bool {
        self.get(scope, key).is_some()
    }

    /// Returns whether one metadata key exists exactly on a scope.
    pub fn contains_exact(&self, scope: &ResourceView, key: &str) -> bool {
        self.get_exact(scope, key).is_some()
    }

    /// Returns all effective metadata for one resource view.
    pub fn effective_entries(&self, scope: &ResourceView) -> BTreeMap<&str, &MetadataValue> {
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
                    effective.insert(key.as_ref(), value);
                }
            }
        }

        effective
    }

    /// Validates metadata against a set of scoped requirements.
    pub fn validate(
        &self,
        requirements: &[ScopedMetadataRequirement],
    ) -> Result<(), MetadataValidationError> {
        for scoped in requirements {
            let requirement = scoped.requirement();

            match self.resolve(scoped.scope(), requirement.key()) {
                Some((resolved_scope, value)) if value.kind() != requirement.kind() => {
                    return Err(MetadataValidationError::KindMismatch {
                        scope: scoped.scope().clone(),
                        resolved_scope,
                        key: requirement.key().into(),
                        expected: requirement.kind(),
                        actual: value.kind(),
                    });
                }
                Some(_) => {}
                None if requirement.is_required() => {
                    return Err(MetadataValidationError::MissingRequired {
                        scope: scoped.scope().clone(),
                        key: requirement.key().into(),
                        expected: requirement.kind(),
                    });
                }
                None => {}
            }
        }

        Ok(())
    }

    /// Iterates over all explicitly stored metadata entries.
    pub fn iter(&self) -> impl Iterator<Item = (&ResourceView, &str, &MetadataValue)> {
        self.scopes.iter().flat_map(|(scope, entries)| {
            entries
                .iter()
                .map(move |(key, value)| (scope, key.as_ref(), value))
        })
    }

    /// Iterates over metadata stored exactly on one scope.
    pub fn iter_scope(&self, scope: &ResourceView) -> impl Iterator<Item = (&str, &MetadataValue)> {
        let scope = scope.canonical();

        self.scopes
            .get(&scope)
            .into_iter()
            .flat_map(|entries| entries.iter().map(|(key, value)| (key.as_ref(), value)))
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
    /// A required metadata entry was not present.
    MissingRequired {
        /// Resource view requiring the metadata.
        scope: ResourceView,
        /// Missing metadata key.
        key: Box<str>,
        /// Kind required by the contract.
        expected: MetadataKind,
    },
    /// A metadata entry was present with the wrong kind.
    KindMismatch {
        /// Resource view requiring the metadata.
        scope: ResourceView,
        /// Scope that provided the closest metadata value.
        resolved_scope: ResourceView,
        /// Metadata key whose value had the wrong kind.
        key: Box<str>,
        /// Kind required by the contract.
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
                resolved_scope,
                key,
                expected,
                actual,
            } => write!(
                f,
                "metadata '{key}' for scope {scope:?} resolves from {resolved_scope:?} with kind mismatch: expected {expected:?}, got {actual:?}"
            ),
        }
    }
}

impl std::error::Error for MetadataValidationError {}
