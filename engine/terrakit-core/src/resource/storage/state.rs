//! Runtime structural state exposed by resource storage.

use std::fmt;

use crate::resource::Schema;

/// Runtime structural state for one resolved resource view.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StorageViewState {
    /// The selected view itself has no runtime structural state beyond its schema.
    ///
    /// This says nothing about descendant views. For example, a struct root may
    /// be `Static` while one of its fields is an `Array` with a runtime length.
    Static,
    /// Runtime length of one variable-length array.
    Array {
        /// Number of elements currently stored.
        length: usize,
    },
    /// Runtime extents of one dense array.
    DenseArray {
        /// One extent per dense-array dimension.
        extents: Box<[usize]>,
    },
    /// Runtime presence state of one optional value.
    Optional {
        /// Whether the optional currently contains a value.
        present: bool,
    },
    /// Runtime active case of one variant value.
    Variant {
        /// Active variant name.
        variant: Box<str>,
    },
}

impl StorageViewState {
    /// Returns dense array extents when this view is a dense array.
    pub fn dense_array(&self) -> Option<&[usize]> {
        match self {
            Self::DenseArray { extents } => Some(extents),
            _ => None,
        }
    }

    /// Validates this runtime state against one resolved logical schema.
    pub fn validate_for(&self, schema: &Schema) -> Result<(), StorageViewStateError> {
        match schema {
            Schema::Bool
            | Schema::Numeric(_)
            | Schema::String
            | Schema::Identifier
            | Schema::Vector { .. }
            | Schema::FixedArray { .. }
            | Schema::Struct(_) => match self {
                Self::Static => Ok(()),
                actual => Err(StorageViewStateError::ExpectedStatic {
                    actual: actual.clone(),
                }),
            },
            Schema::Array(_) => match self {
                Self::Array { .. } => Ok(()),
                actual => Err(StorageViewStateError::ExpectedArray {
                    actual: actual.clone(),
                }),
            },
            Schema::DenseArray { rank, .. } => match self {
                Self::DenseArray { extents } if extents.len() == *rank as usize => Ok(()),
                Self::DenseArray { extents } => {
                    Err(StorageViewStateError::DenseArrayRankMismatch {
                        expected: *rank as usize,
                        actual: extents.len(),
                    })
                }
                actual => Err(StorageViewStateError::ExpectedDenseArray {
                    actual: actual.clone(),
                }),
            },
            Schema::Optional(_) => match self {
                Self::Optional { .. } => Ok(()),
                actual => Err(StorageViewStateError::ExpectedOptional {
                    actual: actual.clone(),
                }),
            },
            Schema::Variant(variants) => match self {
                Self::Variant { variant }
                    if variants
                        .iter()
                        .any(|candidate| candidate.name() == variant.as_ref()) =>
                {
                    Ok(())
                }
                Self::Variant { variant } => Err(StorageViewStateError::UnknownVariant {
                    variant: variant.clone(),
                }),
                actual => Err(StorageViewStateError::ExpectedVariant {
                    actual: actual.clone(),
                }),
            },
        }
    }
}

/// Runtime structural state that does not satisfy a logical schema.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StorageViewStateError {
    /// A statically described view returned dynamic state.
    ExpectedStatic {
        /// State returned by the storage implementation.
        actual: StorageViewState,
    },
    /// An array view did not return array state.
    ExpectedArray {
        /// State returned by the storage implementation.
        actual: StorageViewState,
    },
    /// A dense array view did not return dense-array state.
    ExpectedDenseArray {
        /// State returned by the storage implementation.
        actual: StorageViewState,
    },
    /// Dense-array runtime rank differed from the schema rank.
    DenseArrayRankMismatch {
        /// Rank required by the schema.
        expected: usize,
        /// Rank reported by storage.
        actual: usize,
    },
    /// An optional view did not return optional state.
    ExpectedOptional {
        /// State returned by the storage implementation.
        actual: StorageViewState,
    },
    /// A variant view did not return variant state.
    ExpectedVariant {
        /// State returned by the storage implementation.
        actual: StorageViewState,
    },
    /// Storage selected a variant case that does not exist in the schema.
    UnknownVariant {
        /// Unknown active variant name.
        variant: Box<str>,
    },
}

impl fmt::Display for StorageViewStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpectedStatic { actual } => {
                write!(f, "expected static runtime state, got {actual:?}")
            }
            Self::ExpectedArray { actual } => {
                write!(f, "expected array runtime state, got {actual:?}")
            }
            Self::ExpectedDenseArray { actual } => {
                write!(f, "expected dense-array runtime state, got {actual:?}")
            }
            Self::DenseArrayRankMismatch { expected, actual } => write!(
                f,
                "dense-array runtime rank mismatch: expected {expected}, got {actual}"
            ),
            Self::ExpectedOptional { actual } => {
                write!(f, "expected optional runtime state, got {actual:?}")
            }
            Self::ExpectedVariant { actual } => {
                write!(f, "expected variant runtime state, got {actual:?}")
            }
            Self::UnknownVariant { variant } => {
                write!(f, "storage selected unknown variant '{variant}'")
            }
        }
    }
}

impl std::error::Error for StorageViewStateError {}
