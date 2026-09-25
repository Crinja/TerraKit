//! Resource metadata contracts

use crate::resource::{MetadataKeyId, ResourceView};

/// Primitive metadata value kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetadataKind {
    /// Boolean flag.
    Bool,
    /// Signed integer.
    I64,
    /// Unsigned integer.
    U64,
    /// Floating-point scalar.
    F64,
    /// UTF-8 string.
    String,
    /// Symbolic identifier.
    Identifier,
}

/// Unresolved metadata key registration input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataKeySpec {
    id: MetadataKeyId,
    kind: MetadataKind,
}

impl MetadataKeySpec {
    /// Creates a metadata key spec.
    pub fn new(id: MetadataKeyId, kind: MetadataKind) -> Self {
        Self { id, kind }
    }

    /// Returns the versioned metadata key ID.
    pub fn id(&self) -> &MetadataKeyId {
        &self.id
    }

    /// Returns the value kind requested by this metadata key.
    pub const fn kind(&self) -> MetadataKind {
        self.kind
    }

    pub(crate) fn into_parts(self) -> (MetadataKeyId, MetadataKind) {
        (self.id, self.kind)
    }
}

/// Definition of one reusable metadata key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataKeyDefinition {
    id: MetadataKeyId,
    kind: MetadataKind,
}

impl MetadataKeyDefinition {
    pub(crate) fn new(id: MetadataKeyId, kind: MetadataKind) -> Self {
        Self { id, kind }
    }

    /// Returns the versioned metadata key ID.
    pub fn id(&self) -> &MetadataKeyId {
        &self.id
    }

    /// Returns the value kind used by this metadata key.
    pub const fn kind(&self) -> MetadataKind {
        self.kind
    }
}

/// One metadata requirement declared by a capability contract.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetadataRequirement {
    key: MetadataKeyId,
    required: bool,
}

impl MetadataRequirement {
    /// Creates a required metadata entry.
    pub fn required(key: MetadataKeyId) -> Self {
        Self {
            key,
            required: true,
        }
    }

    /// Creates an optional metadata entry.
    pub fn optional(key: MetadataKeyId) -> Self {
        Self {
            key,
            required: false,
        }
    }

    /// Returns the metadata key.
    pub fn key(&self) -> &MetadataKeyId {
        &self.key
    }

    /// Returns whether the metadata entry must be present.
    pub const fn is_required(&self) -> bool {
        self.required
    }
}

/// Metadata requirement resolved against one registry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolvedMetadataRequirement {
    requirement: MetadataRequirement,
    kind: MetadataKind,
}

impl ResolvedMetadataRequirement {
    pub(crate) fn new(requirement: MetadataRequirement, kind: MetadataKind) -> Self {
        Self { requirement, kind }
    }

    /// Returns the metadata requirement.
    pub fn requirement(&self) -> &MetadataRequirement {
        &self.requirement
    }

    /// Returns the metadata key.
    pub fn key(&self) -> &MetadataKeyId {
        self.requirement.key()
    }

    /// Returns whether the metadata entry must be present.
    pub const fn is_required(&self) -> bool {
        self.requirement.is_required()
    }

    /// Returns the metadata kind resolved when the contract was registered.
    pub const fn kind(&self) -> MetadataKind {
        self.kind
    }
}

/// Unresolved metadata requirement attached to one resource view.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopedMetadataRequirementSpec {
    scope: ResourceView,
    requirement: MetadataRequirement,
}

impl ScopedMetadataRequirementSpec {
    /// Creates a metadata requirement for one resource view.
    pub fn new(scope: ResourceView, requirement: MetadataRequirement) -> Self {
        Self {
            scope: scope.canonical(),
            requirement,
        }
    }

    /// Creates a metadata requirement for the complete resource.
    pub fn root(requirement: MetadataRequirement) -> Self {
        Self::new(ResourceView::Root, requirement)
    }

    /// Returns the resource view this requirement applies to.
    pub fn scope(&self) -> &ResourceView {
        &self.scope
    }

    /// Returns the metadata requirement.
    pub fn requirement(&self) -> &MetadataRequirement {
        &self.requirement
    }
}

/// Metadata requirement attached to one resource view.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopedMetadataRequirement {
    scope: ResourceView,
    requirement: ResolvedMetadataRequirement,
}

impl ScopedMetadataRequirement {
    /// Creates a resolved metadata requirement for one resource view.
    pub(crate) fn new(
        scope: ResourceView,
        requirement: MetadataRequirement,
        kind: MetadataKind,
    ) -> Self {
        Self {
            scope: scope.canonical(),
            requirement: ResolvedMetadataRequirement::new(requirement, kind),
        }
    }

    /// Returns the resource view this requirement applies to.
    pub fn scope(&self) -> &ResourceView {
        &self.scope
    }

    /// Returns the metadata requirement.
    pub fn requirement(&self) -> &MetadataRequirement {
        self.requirement.requirement()
    }

    /// Returns the resolved metadata requirement.
    pub fn resolved_requirement(&self) -> &ResolvedMetadataRequirement {
        &self.requirement
    }

    /// Returns the metadata kind resolved when the resource type was registered.
    pub const fn kind(&self) -> MetadataKind {
        self.requirement.kind()
    }
}
