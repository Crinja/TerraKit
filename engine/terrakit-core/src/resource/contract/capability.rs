//! Capability contracts and registry.
//!
//! Capabilities are named, versioned interpretation whose exposed view satisifies a schema.
//! A resource may advertise any amount of capabilities

use std::fmt;

use crate::resource::{ResourceCapabilityId, ResourceView, Schema};

use super::metadata::{MetadataRequirement, ResolvedMetadataRequirement};

/// Unresolved resource capability registration input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceCapabilitySpec {
    id: ResourceCapabilityId,
    view_schema: Schema,
    metadata: Vec<MetadataRequirement>,
}

impl ResourceCapabilitySpec {
    /// Creates a capability spec.
    pub fn new(id: ResourceCapabilityId, view_schema: Schema) -> Self {
        Self {
            id,
            view_schema,
            metadata: Vec::new(),
        }
    }

    /// Adds one metadata requirement.
    pub fn with_metadata(mut self, requirement: MetadataRequirement) -> Self {
        self.metadata.push(requirement);
        self
    }

    /// Returns the stable versioned capability ID.
    pub fn id(&self) -> &ResourceCapabilityId {
        &self.id
    }

    /// Returns the schema a resource view must satisfy to claim this capability.
    pub fn view_schema(&self) -> &Schema {
        &self.view_schema
    }

    /// Returns metadata requirements requested by this capability.
    pub fn metadata_requirements(&self) -> &[MetadataRequirement] {
        &self.metadata
    }

    pub(crate) fn into_parts(self) -> (ResourceCapabilityId, Schema, Vec<MetadataRequirement>) {
        (self.id, self.view_schema, self.metadata)
    }
}

/// Definition of one reusable resource capability
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceCapabilityDefinition {
    id: ResourceCapabilityId,
    view_schema: Schema,
    metadata: Vec<ResolvedMetadataRequirement>,
}

impl ResourceCapabilityDefinition {
    pub(crate) fn new(
        id: ResourceCapabilityId,
        view_schema: Schema,
        metadata: Vec<ResolvedMetadataRequirement>,
    ) -> Self {
        Self {
            id,
            view_schema,
            metadata,
        }
    }

    /// Returns the stable versioned capability ID
    pub fn id(&self) -> &ResourceCapabilityId {
        &self.id
    }

    /// Returns the schema a resource view must satisfy to claim this capability.
    pub fn view_schema(&self) -> &Schema {
        &self.view_schema
    }

    /// Returns metadata requirements associated with this capability
    pub fn metadata_requirements(&self) -> &[ResolvedMetadataRequirement] {
        &self.metadata
    }
}

/// Declares that one view of a resource type satisfies a capability.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityBinding {
    capability: ResourceCapabilityId,
    view: ResourceView,
}

impl CapabilityBinding {
    /// Binds a capability to the complete resource.
    pub fn root(capability: ResourceCapabilityId) -> Self {
        Self {
            capability,
            view: ResourceView::Root,
        }
    }

    /// Binds a capability to one nested resource view.
    pub fn view(capability: ResourceCapabilityId, view: ResourceView) -> Self {
        Self {
            capability,
            view: view.canonical(),
        }
    }

    /// Returns the claimed capability ID.
    pub fn capability(&self) -> &ResourceCapabilityId {
        &self.capability
    }

    /// Returns the resource view that provides the capability.
    pub fn resource_view(&self) -> &ResourceView {
        &self.view
    }
}

/// Capability registration/definition error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityError {
    /// The capability schema itself was invalid.
    InvalidSchema(Box<str>),
    /// A capability ID was registered more than once.
    DuplicateCapability(ResourceCapabilityId),
    /// A metadata key was required more than once.
    DuplicateMetadataRequirement(crate::resource::MetadataKeyId),
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSchema(message) => write!(f, "invalid capability schema: {message}"),
            Self::DuplicateCapability(id) => write!(f, "capability '{id}' is already registered"),
            Self::DuplicateMetadataRequirement(id) => write!(
                f,
                "capability metadata requirement '{id}' is declared more than once"
            ),
        }
    }
}

impl std::error::Error for CapabilityError {}
