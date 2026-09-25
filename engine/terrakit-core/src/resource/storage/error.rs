//! Storage errors.

use std::fmt;

use crate::resource::{Schema, SchemaError, StorageAccessId};

/// Resource storage error.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StorageError {
    /// The supplied logical schema is invalid.
    InvalidSchema(SchemaError),
    /// The storage backend does not support the requested schema.
    UnsupportedSchema(Schema),
    /// The storage backend does not support the requested access contract.
    UnsupportedAccess(StorageAccessId),
    /// The resolved view belongs to another resource schema.
    ViewSchemaMismatch,
    /// Storage construction has not produced a complete readable value.
    Incomplete,
    /// Backend-specific storage failure.
    Backend(Box<str>),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSchema(error) => write!(f, "invalid storage schema: {error}"),
            Self::UnsupportedSchema(schema) => {
                write!(f, "storage backend does not support schema {schema:?}")
            }
            Self::UnsupportedAccess(access) => {
                write!(
                    f,
                    "storage backend does not support access contract '{access}'"
                )
            }
            Self::ViewSchemaMismatch => {
                write!(
                    f,
                    "resolved resource view belongs to another storage schema"
                )
            }
            Self::Incomplete => write!(f, "storage is not fully initialized"),
            Self::Backend(message) => write!(f, "storage backend error: {message}"),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidSchema(error) => Some(error),
            _ => None,
        }
    }
}
