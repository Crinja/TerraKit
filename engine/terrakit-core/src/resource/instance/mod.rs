//! Runtime resource instances.

mod descriptor;
mod metadata;
mod resource;

pub use descriptor::{ResourceDescriptor, ResourceDescriptorError};
pub use metadata::{MetadataValidationError, MetadataValue, ResourceMetadata};
pub use resource::{Resource, ResourceAccess, ResourceAccessError, ResourceError};
