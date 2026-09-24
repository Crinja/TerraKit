//! Resource metadata contracts

use std::collections::BTreeMap;

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

/// Runtime metadata values attached to one resource instance.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceMetadata {
    entries: BTreeMap<Box<str>, MetadataValue>,
}

impl ResourceMetadata {
    /// Creates empty metadata.
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Inserts or replaces a metadata value.
    pub fn insert(&mut self, key: impl Into<Box<str>>, value: MetadataValue) {
        self.entries.insert(key.into(), value);
    }

    /// Returns one metadata value.
    pub fn get(&self, key: &str) -> Option<&MetadataValue> {
        self.entries.get(key)
    }

    /// Returns whether one metadata key is present.
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Iterates over all metadata entries.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &MetadataValue)> {
        self.entries
            .iter()
            .map(|(key, value)| (key.as_ref(), value))
    }

    /// Returns the number of metadata entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether no metadata entries exist.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
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
