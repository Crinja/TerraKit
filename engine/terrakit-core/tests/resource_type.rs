mod common;

use common::{
    capability_id, hydraulic_schema, metadata_key_id, resource_registry, resource_type_id,
};
use terrakit_core::resource::{
    CapabilityBinding, MetadataRequirement, ResourceCapabilitySpec, ResourceRegistry,
    ResourceRegistryError, ResourceTypeError, ResourceTypeSpec, ResourceView, Schema, SchemaPath,
    ScopedMetadataRequirementSpec,
};

#[test]
fn resource_registry_rejects_unknown_capability() {
    let mut registry = ResourceRegistry::new();
    let unknown = capability_id("domain.unknown@1");

    let spec = ResourceTypeSpec::new(resource_type_id("domain.test@1"), Schema::f32())
        .with_capability(CapabilityBinding::root(unknown.clone()));

    assert_eq!(
        registry.register_resource_type(spec),
        Err(ResourceRegistryError::InvalidResourceType(
            ResourceTypeError::UnknownCapability(unknown)
        ))
    );
}

#[test]
fn resource_registry_rejects_duplicate_capability_binding() {
    let mut registry = resource_registry();
    let erosion_flow = capability_id("terrakit.erosion-flow@1");

    let spec = ResourceTypeSpec::new(resource_type_id("domain.test@1"), hydraulic_schema())
        .with_capability(CapabilityBinding::view(
            erosion_flow.clone(),
            ResourceView::Path(SchemaPath::field("flow")),
        ))
        .with_capability(CapabilityBinding::view(
            erosion_flow.clone(),
            ResourceView::Path(SchemaPath::field("flow")),
        ));

    assert_eq!(
        registry.register_resource_type(spec),
        Err(ResourceRegistryError::InvalidResourceType(
            ResourceTypeError::DuplicateCapabilityBinding(erosion_flow)
        ))
    );
}

#[test]
fn resource_registry_rejects_invalid_capability_view() {
    let mut registry = resource_registry();
    let erosion_flow = capability_id("terrakit.erosion-flow@1");

    let spec = ResourceTypeSpec::new(resource_type_id("domain.test@1"), hydraulic_schema())
        .with_capability(CapabilityBinding::view(
            erosion_flow,
            ResourceView::Path(SchemaPath::field("missing")),
        ));

    assert!(matches!(
        registry.register_resource_type(spec),
        Err(ResourceRegistryError::InvalidResourceType(
            ResourceTypeError::InvalidView { .. }
        ))
    ));
}

#[test]
fn resource_registry_rejects_capability_schema_mismatch() {
    let mut registry = resource_registry();
    let erosion_flow = capability_id("terrakit.erosion-flow@1");

    let spec = ResourceTypeSpec::new(resource_type_id("domain.test@1"), hydraulic_schema())
        .with_capability(CapabilityBinding::view(
            erosion_flow,
            ResourceView::Path(SchemaPath::field("sediment")),
        ));

    assert!(matches!(
        registry.register_resource_type(spec),
        Err(ResourceRegistryError::InvalidResourceType(
            ResourceTypeError::CapabilitySchemaMismatch { .. }
        ))
    ));
}

#[test]
fn resource_registry_rejects_duplicate_resource_type_ids() {
    let mut registry = resource_registry();
    let id = resource_type_id("domain.hydraulic-erosion-result@1");
    let spec = ResourceTypeSpec::new(id.clone(), hydraulic_schema());

    registry.register_resource_type(spec.clone()).unwrap();

    let definition = registry.resource_type(&id).unwrap();
    assert_eq!(definition.id(), &id);
    assert_eq!(definition.schema(), &hydraulic_schema());

    assert_eq!(
        registry.register_resource_type(spec),
        Err(ResourceRegistryError::DuplicateResourceType(id))
    );
}

#[test]
fn resource_type_iteration_is_deterministic() {
    let mut registry = resource_registry();

    for id in ["domain.zeta@1", "domain.alpha@1", "domain.middle@1"] {
        registry
            .register_resource_type(ResourceTypeSpec::new(resource_type_id(id), Schema::f32()))
            .unwrap();
    }

    let ids: Vec<_> = registry
        .resource_types()
        .map(|definition| definition.id().as_str())
        .collect();

    assert_eq!(
        ids,
        vec!["domain.alpha@1", "domain.middle@1", "domain.zeta@1"]
    );
}

#[test]
fn resource_type_merges_compatible_capability_metadata_requirements() {
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");
    let first = capability_id("domain.first@1");
    let second = capability_id("domain.second@1");
    let mut registry = resource_registry();

    registry
        .register_capability(
            ResourceCapabilitySpec::new(first.clone(), Schema::f32())
                .with_metadata(MetadataRequirement::optional(coordinate_space.clone())),
        )
        .unwrap();

    registry
        .register_capability(
            ResourceCapabilitySpec::new(second.clone(), Schema::f32())
                .with_metadata(MetadataRequirement::required(coordinate_space.clone())),
        )
        .unwrap();

    let id = resource_type_id("domain.test@1");

    registry
        .register_resource_type(
            ResourceTypeSpec::new(id.clone(), Schema::f32())
                .with_capability(CapabilityBinding::root(first))
                .with_capability(CapabilityBinding::root(second)),
        )
        .unwrap();

    let definition = registry.resource_type(&id).unwrap();

    assert_eq!(definition.metadata_requirements().len(), 1);
    assert_eq!(
        definition.metadata_requirements()[0].scope(),
        &ResourceView::Root
    );
    assert_eq!(
        definition.metadata_requirements()[0].requirement(),
        &MetadataRequirement::required(coordinate_space)
    );
    assert_eq!(
        definition.metadata_requirements()[0].kind(),
        terrakit_core::resource::MetadataKind::Identifier
    );
}

#[test]
fn resource_type_can_require_metadata_without_a_capability() {
    let units = metadata_key_id("terrakit.units@1");
    let id = resource_type_id("domain.encoded-value@1");
    let mut registry = resource_registry();

    registry
        .register_resource_type(
            ResourceTypeSpec::new(id.clone(), Schema::f32()).with_metadata(
                ScopedMetadataRequirementSpec::root(MetadataRequirement::required(units.clone())),
            ),
        )
        .unwrap();

    let definition = registry.resource_type(&id).unwrap();

    assert_eq!(definition.metadata_requirements().len(), 1);
    assert_eq!(
        definition.metadata_requirements()[0].requirement(),
        &MetadataRequirement::required(units)
    );
}

#[test]
fn resource_registry_rejects_unknown_type_owned_metadata_key() {
    let unknown = metadata_key_id("domain.unknown@1");
    let scope = ResourceView::Root;
    let mut registry = resource_registry();

    let spec = ResourceTypeSpec::new(resource_type_id("domain.test@1"), Schema::f32())
        .with_metadata(ScopedMetadataRequirementSpec::new(
            scope.clone(),
            MetadataRequirement::required(unknown.clone()),
        ));

    assert_eq!(
        registry.register_resource_type(spec),
        Err(ResourceRegistryError::InvalidResourceType(
            ResourceTypeError::UnknownMetadataKey {
                scope,
                key: unknown,
            }
        ))
    );
}

#[test]
fn resource_registry_rejects_invalid_type_owned_metadata_view() {
    let units = metadata_key_id("terrakit.units@1");
    let mut registry = resource_registry();

    let spec = ResourceTypeSpec::new(resource_type_id("domain.test@1"), Schema::f32())
        .with_metadata(ScopedMetadataRequirementSpec::new(
            ResourceView::Path(SchemaPath::field("missing")),
            MetadataRequirement::required(units),
        ));

    assert!(matches!(
        registry.register_resource_type(spec),
        Err(ResourceRegistryError::InvalidResourceType(
            ResourceTypeError::InvalidMetadataView { .. }
        ))
    ));
}
