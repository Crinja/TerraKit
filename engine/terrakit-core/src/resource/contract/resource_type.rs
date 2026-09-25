//! Complete resource type definition.
//!
//! Resource types are compositions of schema primatives
//! These can be validated without core requireing unique domain knowledge

use std::fmt;

use crate::resource::{MetadataKeyId, ResourceCapabilityId, ResourceTypeId, ResourceView, Schema};

use super::capability::CapabilityBinding;
use super::metadata::{ScopedMetadataRequirement, ScopedMetadataRequirementSpec};

/// Unresolved resource type registration input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceTypeSpec {
    id: ResourceTypeId,
    schema: Schema,
    capabilities: Vec<CapabilityBinding>,
    metadata: Vec<ScopedMetadataRequirementSpec>,
}

impl ResourceTypeSpec {
    /// Creates a resource type spec.
    pub fn new(id: ResourceTypeId, schema: Schema) -> Self {
        Self {
            id,
            schema,
            capabilities: Vec::new(),
            metadata: Vec::new(),
        }
    }

    /// Adds one capability binding.
    pub fn with_capability(mut self, binding: CapabilityBinding) -> Self {
        self.capabilities.push(binding);
        self
    }

    /// Adds one resource type metadata requirement.
    pub fn with_metadata(mut self, requirement: ScopedMetadataRequirementSpec) -> Self {
        self.metadata.push(requirement);
        self
    }

    /// Returns the resource type's stable versioned ID.
    pub fn id(&self) -> &ResourceTypeId {
        &self.id
    }

    /// Returns the complete structural schema.
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    /// Returns all requested capability bindings.
    pub fn capabilities(&self) -> &[CapabilityBinding] {
        &self.capabilities
    }

    /// Returns resource type metadata requirements.
    pub fn metadata_requirements(&self) -> &[ScopedMetadataRequirementSpec] {
        &self.metadata
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        ResourceTypeId,
        Schema,
        Vec<CapabilityBinding>,
        Vec<ScopedMetadataRequirementSpec>,
    ) {
        (self.id, self.schema, self.capabilities, self.metadata)
    }
}

/// Complete structural contract for one resource representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceTypeDefinition {
    id: ResourceTypeId,
    schema: Schema,
    capabilities: Vec<CapabilityBinding>,
    metadata: Vec<ScopedMetadataRequirement>,
}

impl ResourceTypeDefinition {
    pub(crate) fn new(
        id: ResourceTypeId,
        schema: Schema,
        capabilities: Vec<CapabilityBinding>,
        metadata: Vec<ScopedMetadataRequirement>,
    ) -> Self {
        Self {
            id,
            schema,
            capabilities,
            metadata,
        }
    }

    /// Returns the resource type's stable versioned ID.
    pub fn id(&self) -> &ResourceTypeId {
        &self.id
    }

    /// Returns the complete structural schema.
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    /// Returns all explicitly advertised capability bindings.
    pub fn capabilities(&self) -> &[CapabilityBinding] {
        &self.capabilities
    }

    /// Returns the effective metadata requirements for this resource type.
    pub fn metadata_requirements(&self) -> &[ScopedMetadataRequirement] {
        &self.metadata
    }

    /// Returns whether this type explicitly advertises a capability.
    pub fn has_capability(&self, id: &ResourceCapabilityId) -> bool {
        self.capabilities
            .iter()
            .any(|binding| binding.capability() == id)
    }

    /// Returns the binding for one explicitly advertised capability.
    pub fn capability(&self, id: &ResourceCapabilityId) -> Option<&CapabilityBinding> {
        self.capabilities
            .iter()
            .find(|binding| binding.capability() == id)
    }
}

/// Error defining one concrete resource type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceTypeError {
    /// The resource's own schema was invalid.
    InvalidSchema(Box<str>),
    /// A claimed capability was not registered.
    UnknownCapability(ResourceCapabilityId),
    /// A resource type metadata requirement references an unknown key.
    UnknownMetadataKey {
        /// Resource view containing the requirement.
        scope: ResourceView,
        /// Missing metadata key.
        key: MetadataKeyId,
    },
    /// A resource claimed the same capability more than once.
    DuplicateCapabilityBinding(ResourceCapabilityId),
    /// The binding's view path did not exist in the resource schema.
    InvalidView {
        /// Capability whose view failed.
        capability: ResourceCapabilityId,
        /// Human-readable view-resolution error.
        message: Box<str>,
    },
    /// A resource type metadata scope did not exist in the resource schema.
    InvalidMetadataView {
        /// Metadata key whose scope failed.
        key: MetadataKeyId,
        /// Human-readable view-resolution error.
        message: Box<str>,
    },
    /// The resolved view exists but does not satisfy the capability schema.
    CapabilitySchemaMismatch {
        /// Capability being claimed.
        capability: ResourceCapabilityId,
        /// Contract schema declared by the capability.
        expected: Schema,
        /// Actual schema exposed by the resource view.
        actual: Schema,
    },
}

impl fmt::Display for ResourceTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSchema(message) => write!(f, "invalid resource schema: {message}"),
            Self::UnknownCapability(id) => write!(f, "unknown capability '{id}'"),
            Self::UnknownMetadataKey { scope, key } => write!(
                f,
                "resource metadata requirement '{key}' on scope {scope:?} is not registered"
            ),
            Self::DuplicateCapabilityBinding(id) => {
                write!(f, "capability '{id}' is bound more than once")
            }
            Self::InvalidView {
                capability,
                message,
            } => write!(f, "invalid view for capability '{capability}': {message}"),
            Self::InvalidMetadataView { key, message } => {
                write!(
                    f,
                    "invalid view for metadata requirement '{key}': {message}"
                )
            }
            Self::CapabilitySchemaMismatch {
                capability,
                expected,
                actual,
            } => write!(
                f,
                "capability '{capability}' schema mismatch: expected {expected:?}, got {actual:?}"
            ),
        }
    }
}

impl std::error::Error for ResourceTypeError {}
