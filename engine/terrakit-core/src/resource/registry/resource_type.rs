use std::collections::{BTreeMap, HashSet, btree_map::Entry};
use std::fmt;

use crate::resource::{
    MetadataInheritance, MetadataKeyId, MetadataKind, MetadataRequirement, MetadataValidationError,
    ResourceMetadata, ResourceTypeDefinition, ResourceTypeError, ResourceTypeId, ResourceTypeSpec,
    ResourceView, ScopedMetadataRequirement,
};

use super::{ResourceRegistry, ResourceRegistryError};

type ResolvedMetadataMap = BTreeMap<
    ResourceView,
    BTreeMap<MetadataKeyId, (MetadataRequirement, MetadataKind, MetadataInheritance)>,
>;

/// Registry of concrete resource type definitions.
#[derive(Debug, Clone, Default)]
pub(crate) struct ResourceTypeRegistry {
    definitions: BTreeMap<ResourceTypeId, ResourceTypeDefinition>,
}

impl ResourceTypeRegistry {
    /// Creates an empty resource type registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a concrete resource type, rejecting duplicate IDs.
    pub fn register(
        &mut self,
        definition: ResourceTypeDefinition,
    ) -> Result<(), ResourceTypeRegistryError> {
        let id = definition.id().clone();

        match self.definitions.entry(id) {
            Entry::Vacant(entry) => {
                entry.insert(definition);
                Ok(())
            }
            Entry::Occupied(entry) => Err(ResourceTypeRegistryError::DuplicateType(
                entry.key().clone(),
            )),
        }
    }

    /// Returns one concrete resource type by ID.
    pub fn get(&self, id: &ResourceTypeId) -> Option<&ResourceTypeDefinition> {
        self.definitions.get(id)
    }

    /// Iterates over all registered resource types.
    pub fn iter(&self) -> impl Iterator<Item = &ResourceTypeDefinition> {
        self.definitions.values()
    }

    /// Returns whether a resource type is registered.
    pub fn contains(&self, id: &ResourceTypeId) -> bool {
        self.definitions.contains_key(id)
    }
}

/// Resource type registry error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResourceTypeRegistryError {
    /// A resource type with the same stable ID is already registered.
    DuplicateType(ResourceTypeId),
}

impl fmt::Display for ResourceTypeRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateType(id) => write!(f, "resource type '{id}' is already registered"),
        }
    }
}

impl std::error::Error for ResourceTypeRegistryError {}

impl ResourceRegistry {
    /// Registers and resolves one resource type spec.
    ///
    /// Capability contracts and metadata kinds are resolved from this registry
    /// before the type is stored.
    pub fn register_resource_type(
        &mut self,
        spec: ResourceTypeSpec,
    ) -> Result<(), ResourceRegistryError> {
        if self.resource_types.contains(spec.id()) {
            return Err(ResourceRegistryError::DuplicateResourceType(
                spec.id().clone(),
            ));
        }

        let (id, schema, mut capabilities, type_metadata) = spec.into_parts();

        schema.validate().map_err(|error| {
            ResourceRegistryError::InvalidResourceType(ResourceTypeError::InvalidSchema(
                error.to_string().into(),
            ))
        })?;

        let mut seen = HashSet::with_capacity(capabilities.len());
        let mut seen_type_metadata = HashSet::with_capacity(type_metadata.len());
        let mut metadata = ResolvedMetadataMap::new();

        for scoped in type_metadata {
            let scope = scoped.scope().canonical();
            let requirement = scoped.requirement().clone();

            if !seen_type_metadata.insert((scope.clone(), requirement.key().clone())) {
                return Err(ResourceRegistryError::InvalidResourceType(
                    ResourceTypeError::DuplicateMetadataRequirement {
                        scope,
                        key: requirement.key().clone(),
                    },
                ));
            }

            scope.resolve(&schema).map_err(|error| {
                ResourceRegistryError::InvalidResourceType(ResourceTypeError::InvalidMetadataView {
                    key: requirement.key().clone(),
                    message: error.to_string().into(),
                })
            })?;

            let key_definition = self.metadata_keys.get(requirement.key()).ok_or_else(|| {
                ResourceRegistryError::InvalidResourceType(ResourceTypeError::UnknownMetadataKey {
                    scope: scope.clone(),
                    key: requirement.key().clone(),
                })
            })?;

            merge_metadata_requirement(
                &mut metadata,
                scope,
                requirement,
                key_definition.kind(),
                key_definition.inheritance(),
            );
        }

        for binding in &capabilities {
            if !seen.insert(binding.capability().clone()) {
                return Err(ResourceRegistryError::InvalidResourceType(
                    ResourceTypeError::DuplicateCapabilityBinding(binding.capability().clone()),
                ));
            }

            let definition = self.capabilities.get(binding.capability()).ok_or_else(|| {
                ResourceRegistryError::InvalidResourceType(ResourceTypeError::UnknownCapability(
                    binding.capability().clone(),
                ))
            })?;

            let actual = binding.resource_view().resolve(&schema).map_err(|error| {
                ResourceRegistryError::InvalidResourceType(ResourceTypeError::InvalidView {
                    capability: binding.capability().clone(),
                    message: error.to_string().into(),
                })
            })?;

            if !definition.view_schema().accepts(actual) {
                return Err(ResourceRegistryError::InvalidResourceType(
                    ResourceTypeError::CapabilitySchemaMismatch {
                        capability: binding.capability().clone(),
                        expected: definition.view_schema().clone(),
                        actual: actual.clone(),
                    },
                ));
            }

            let scope = binding.resource_view().canonical();

            for requirement in definition.metadata_requirements() {
                merge_metadata_requirement(
                    &mut metadata,
                    scope.clone(),
                    requirement.requirement().clone(),
                    requirement.kind(),
                    requirement.inheritance(),
                );
            }
        }

        let metadata = metadata
            .into_iter()
            .flat_map(|(scope, requirements)| {
                requirements
                    .into_values()
                    .map(move |(requirement, kind, inheritance)| {
                        ScopedMetadataRequirement::new(
                            scope.clone(),
                            requirement,
                            kind,
                            inheritance,
                        )
                    })
            })
            .collect();

        capabilities.sort_by(|left, right| {
            left.capability()
                .cmp(right.capability())
                .then_with(|| left.resource_view().cmp(right.resource_view()))
        });

        let definition = ResourceTypeDefinition::new(id, schema, capabilities, metadata);

        self.resource_types
            .register(definition)
            .map_err(|error| match error {
                ResourceTypeRegistryError::DuplicateType(id) => {
                    ResourceRegistryError::DuplicateResourceType(id)
                }
            })
    }

    /// Validates metadata for one registered resource type.
    pub(crate) fn validate_metadata(
        &self,
        definition: &ResourceTypeDefinition,
        metadata: &ResourceMetadata,
    ) -> Result<(), MetadataValidationError> {
        for scope in metadata.scopes() {
            scope.resolve(definition.schema()).map_err(|error| {
                MetadataValidationError::InvalidScope {
                    scope: scope.clone(),
                    message: error.to_string().into(),
                }
            })?;
        }

        for (scope, key, value) in metadata.iter() {
            let key_definition = self.metadata_keys.get(key).ok_or_else(|| {
                MetadataValidationError::UnknownMetadataKey {
                    scope: scope.clone(),
                    key: key.clone(),
                }
            })?;

            if value.kind() != key_definition.kind() {
                return Err(MetadataValidationError::KindMismatch {
                    scope: scope.clone(),
                    key: key.clone(),
                    expected: key_definition.kind(),
                    actual: value.kind(),
                });
            }
        }

        for scoped in definition.metadata_requirements() {
            let requirement = scoped.requirement();

            match metadata.resolve(scoped.scope(), requirement.key(), scoped.inheritance()) {
                Some((resolved_scope, value)) if value.kind() != scoped.kind() => {
                    return Err(MetadataValidationError::KindMismatch {
                        scope: resolved_scope,
                        key: requirement.key().clone(),
                        expected: scoped.kind(),
                        actual: value.kind(),
                    });
                }
                Some(_) => {}
                None if requirement.is_required() => {
                    return Err(MetadataValidationError::MissingRequired {
                        scope: scoped.scope().clone(),
                        key: requirement.key().clone(),
                        expected: scoped.kind(),
                    });
                }
                None => {}
            }
        }

        Ok(())
    }
}

fn merge_metadata_requirement(
    metadata: &mut ResolvedMetadataMap,
    scope: ResourceView,
    requirement: MetadataRequirement,
    kind: MetadataKind,
    inheritance: MetadataInheritance,
) {
    let scoped = metadata.entry(scope).or_default();

    match scoped.entry(requirement.key().clone()) {
        Entry::Vacant(entry) => {
            entry.insert((requirement, kind, inheritance));
        }
        Entry::Occupied(mut entry) => {
            let (existing, existing_kind, existing_inheritance) = entry.get();

            debug_assert_eq!(*existing_kind, kind);
            debug_assert_eq!(*existing_inheritance, inheritance);

            let should_require = requirement.is_required() && !existing.is_required();
            let existing_kind = *existing_kind;
            let existing_inheritance = *existing_inheritance;

            if should_require {
                entry.insert((
                    MetadataRequirement::required(requirement.key().clone()),
                    existing_kind,
                    existing_inheritance,
                ));
            }
        }
    }
}
