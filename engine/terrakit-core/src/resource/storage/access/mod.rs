//! Versioned storage access contracts and opaque access handles.

mod definition;
mod guard;
mod interface;
mod request;
mod spec;
mod support;

pub use definition::StorageAccessDefinition;
pub use guard::StorageAccessGuard;
pub use interface::StorageAccessInterface;
pub use request::StorageAccessRequest;
pub use spec::StorageAccessSpec;
pub use support::StorageAccessSupport;
