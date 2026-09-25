//! Structural schema language
//!
//! This should be kept intentionally small. New domain concepts should be
//! implemented as compositions of existing primatives
//!
//! If you are modifying this, ask yourself whether you genuinely need a
//! new primative or are leaking domain concepts

mod types;
mod validation;
mod view;

pub use types::{NumericType, Schema, SchemaField, SchemaVariant};
pub use validation::SchemaError;
pub use view::{
    ResourcePath, ResourcePathSegment, ResourceView, SchemaPath, SchemaPathSegment, ViewError,
};
