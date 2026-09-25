//! Runtime resource instances.

mod descriptor;
mod metadata;

pub use descriptor::{ResourceDescriptor, ResourceDescriptorError};
pub use metadata::{MetadataValidationError, MetadataValue, ResourceMetadata};
