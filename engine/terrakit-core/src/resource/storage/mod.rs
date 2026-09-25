//! Physical resource storage contracts.
//!
//! Core defines schema conformance, runtime structural state, view resolution,
//! and versioned access-contract registration/invocation. Concrete storage
//! backends and the concrete semantics of each access contract live outside
//! `terrakit-core`.

mod access;
mod error;
mod state;
mod traits;
mod view;

pub use access::{
    StorageAccessDefinition, StorageAccessGuard, StorageAccessInterface, StorageAccessRequest,
    StorageAccessSpec, StorageAccessSupport,
};
pub use error::StorageError;
pub use state::{StorageViewState, StorageViewStateError};
pub use traits::{ResourceStorage, ResourceStorageWriter};
pub use view::{ResolvedResourceView, ResourceViewResolveError};
