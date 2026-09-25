//! Core-resolved runtime resource views.

use std::fmt;

use crate::resource::{ResourceView, Schema, ViewError};

/// Resource view already resolved against one immutable resource schema.
///
/// Direct construction is private. Callers may request resolution through
/// [`ResolvedResourceView::resolve`], which proves that the addressable view
/// exists in the supplied schema before constructing this owned token. Owning a
/// schema snapshot lets writers resolve a view and then take mutable access
/// without retaining an immutable borrow of the writer itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedResourceView {
    view: ResourceView,
    resource_schema: Schema,
}

impl ResolvedResourceView {
    /// Resolves and canonicalizes one runtime resource view against a schema.
    pub fn resolve(
        view: &ResourceView,
        resource_schema: &Schema,
    ) -> Result<Self, ResourceViewResolveError> {
        view.resolve(resource_schema)
            .map_err(|error| ResourceViewResolveError::InvalidView {
                view: view.clone(),
                error,
            })?;

        Ok(Self {
            view: view.canonical(),
            resource_schema: resource_schema.clone(),
        })
    }

    /// Returns the logical resource view.
    pub fn view(&self) -> &ResourceView {
        &self.view
    }

    /// Returns the complete resource schema this view was resolved against.
    pub fn resource_schema(&self) -> &Schema {
        &self.resource_schema
    }

    /// Returns the schema exposed by this resource view.
    pub fn schema(&self) -> &Schema {
        self.view
            .resolve(&self.resource_schema)
            .expect("resolved resource view must remain valid for its immutable schema")
    }
}

/// Error resolving one runtime resource view.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResourceViewResolveError {
    /// The requested resource view does not exist in the resource schema.
    InvalidView {
        /// Invalid resource view.
        view: ResourceView,
        /// Typed schema-navigation failure.
        error: ViewError,
    },
}

impl fmt::Display for ResourceViewResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidView { view, error } => {
                write!(f, "invalid resource view {view:?}: {error}")
            }
        }
    }
}

impl std::error::Error for ResourceViewResolveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidView { error, .. } => Some(error),
        }
    }
}
