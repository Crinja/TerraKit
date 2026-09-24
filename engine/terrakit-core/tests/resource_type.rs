mod common;

use common::{
    capability_id, hydraulic_schema, metadata_key_id, resource_registry, resource_type_id,
};
use terrakit_core::resource::{
    CapabilityBinding, MetadataRequirement, ResourceCapabilityDefinition, ResourceRegistry,
    ResourceRegistryError, ResourceTypeDefinition, ResourceTypeError, ResourceView, Schema,
    SchemaPath,
};

#[test]
fn resource_registry_rejects_unknown_capability() {
    let mut registry = ResourceRegistry::new();
    let unknown = capability_id("domain.unknown@1");

    let definition = ResourceTypeDefinition::new(
        resource_type_id("domain.test@1"),
        Schema::f32(),
        vec![CapabilityBinding::root(unknown.clone())],
    )
    .unwrap();

    assert_eq!(
        registry.register_resource_type(definition),
        Err(ResourceRegistryError::InvalidResourceType(
            ResourceTypeError::UnknownCapability(unknown)
        ))
    );
}

#[test]
fn resource_type_rejects_duplicate_capability_binding() {
    let erosion_flow = capability_id("terrakit.erosion-flow@1");

    assert_eq!(
        ResourceTypeDefinition::new(
            resource_type_id("domain.test@1"),
            hydraulic_schema(),
            vec![
                CapabilityBinding::view(
                    erosion_flow.clone(),
                    ResourceView::Path(SchemaPath::field("flow")),
                ),
                CapabilityBinding::view(
                    erosion_flow.clone(),
                    ResourceView::Path(SchemaPath::field("flow")),
                ),
            ],
        ),
        Err(ResourceTypeError::DuplicateCapabilityBinding(erosion_flow))
    );
}

#[test]
fn resource_type_rejects_invalid_capability_view() {
    let erosion_flow = capability_id("terrakit.erosion-flow@1");

    assert!(matches!(
        ResourceTypeDefinition::new(
            resource_type_id("domain.test@1"),
            hydraulic_schema(),
            vec![CapabilityBinding::view(
                erosion_flow,
                ResourceView::Path(SchemaPath::field("missing")),
            )],
        ),
        Err(ResourceTypeError::InvalidView { .. })
    ));
}

#[test]
fn resource_registry_rejects_capability_schema_mismatch() {
    let mut registry = resource_registry();
    let erosion_flow = capability_id("terrakit.erosion-flow@1");

    let definition = ResourceTypeDefinition::new(
        resource_type_id("domain.test@1"),
        hydraulic_schema(),
        vec![CapabilityBinding::view(
            erosion_flow,
            ResourceView::Path(SchemaPath::field("sediment")),
        )],
    )
    .unwrap();

    assert!(matches!(
        registry.register_resource_type(definition),
        Err(ResourceRegistryError::InvalidResourceType(
            ResourceTypeError::CapabilitySchemaMismatch { .. }
        ))
    ));
}

#[test]
fn resource_registry_rejects_duplicate_resource_type_ids() {
    let mut registry = resource_registry();
    let id = resource_type_id("domain.hydraulic-erosion-result@1");
    let definition = ResourceTypeDefinition::new(id.clone(), hydraulic_schema(), vec![]).unwrap();

    registry.register_resource_type(definition.clone()).unwrap();

    assert_eq!(registry.resource_type(&id), Some(&definition));
    assert_eq!(
        registry.register_resource_type(definition),
        Err(ResourceRegistryError::DuplicateResourceType(id))
    );
}

#[test]
fn resource_type_iteration_is_deterministic() {
    let mut registry = resource_registry();

    for id in ["domain.zeta@1", "domain.alpha@1", "domain.middle@1"] {
        registry
            .register_resource_type(
                ResourceTypeDefinition::new(resource_type_id(id), Schema::f32(), vec![]).unwrap(),
            )
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
            ResourceCapabilityDefinition::new(
                first.clone(),
                Schema::f32(),
                vec![MetadataRequirement::optional(coordinate_space.clone())],
            )
            .unwrap(),
        )
        .unwrap();

    registry
        .register_capability(
            ResourceCapabilityDefinition::new(
                second.clone(),
                Schema::f32(),
                vec![MetadataRequirement::required(coordinate_space.clone())],
            )
            .unwrap(),
        )
        .unwrap();

    let id = resource_type_id("domain.test@1");

    registry
        .register_resource_type(
            ResourceTypeDefinition::new(
                id.clone(),
                Schema::f32(),
                vec![
                    CapabilityBinding::root(first),
                    CapabilityBinding::root(second),
                ],
            )
            .unwrap(),
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
