mod common;

use common::{capability_id, metadata_key_id, resource_type_id};
use terrakit_core::resource::{
    CapabilityBinding, MetadataInheritance, MetadataKeySpec, MetadataKind, MetadataLookupError,
    MetadataRequirement, MetadataValue, ResourceCapabilitySpec, ResourceDescriptor,
    ResourceDescriptorError, ResourceId, ResourceMetadata, ResourceRegistry, ResourceTypeSpec,
    ResourceView, Schema,
};

fn registry_with_coordinate_space(
    kind: MetadataKind,
    inheritance: MetadataInheritance,
    include_type: bool,
) -> ResourceRegistry {
    let key = metadata_key_id("terrakit.coordinate-space@1");
    let capability = capability_id("domain.value@1");
    let resource_type = resource_type_id("domain.value-resource@1");
    let mut registry = ResourceRegistry::new();

    registry
        .register_metadata_key(
            MetadataKeySpec::new(key.clone(), kind).with_inheritance(inheritance),
        )
        .unwrap();

    registry
        .register_capability(
            ResourceCapabilitySpec::new(capability.clone(), Schema::f32())
                .with_metadata(MetadataRequirement::required(key)),
        )
        .unwrap();

    if include_type {
        registry
            .register_resource_type(
                ResourceTypeSpec::new(resource_type, Schema::f32())
                    .with_capability(CapabilityBinding::root(capability)),
            )
            .unwrap();
    }

    registry
}

#[test]
fn capability_definition_freezes_metadata_kind_from_its_registry() {
    let registry = registry_with_coordinate_space(
        MetadataKind::Identifier,
        MetadataInheritance::Inherited,
        false,
    );
    let capability = capability_id("domain.value@1");
    let definition = registry.capability(&capability).unwrap();

    assert_eq!(definition.metadata_requirements().len(), 1);
    assert_eq!(
        definition.metadata_requirements()[0].kind(),
        MetadataKind::Identifier
    );
}

#[test]
fn capability_definition_freezes_metadata_inheritance_from_its_registry() {
    let key = metadata_key_id("domain.context@1");
    let capability = capability_id("domain.value@1");

    let spec = ResourceCapabilitySpec::new(capability.clone(), Schema::f32())
        .with_metadata(MetadataRequirement::required(key.clone()));

    let mut registry_a = ResourceRegistry::new();
    registry_a
        .register_metadata_key(
            MetadataKeySpec::new(key.clone(), MetadataKind::Identifier)
                .with_inheritance(MetadataInheritance::Inherited),
        )
        .unwrap();
    registry_a.register_capability(spec.clone()).unwrap();

    let mut registry_b = ResourceRegistry::new();
    registry_b
        .register_metadata_key(MetadataKeySpec::new(key, MetadataKind::Identifier))
        .unwrap();
    registry_b.register_capability(spec).unwrap();

    assert_eq!(
        registry_a
            .capability(&capability)
            .unwrap()
            .metadata_requirements()[0]
            .inheritance(),
        MetadataInheritance::Inherited
    );

    assert_eq!(
        registry_b
            .capability(&capability)
            .unwrap()
            .metadata_requirements()[0]
            .inheritance(),
        MetadataInheritance::Exact
    );
}

#[test]
fn resource_type_freezes_metadata_contract_from_its_registry() {
    let registry = registry_with_coordinate_space(
        MetadataKind::Identifier,
        MetadataInheritance::Inherited,
        true,
    );
    let resource_type = resource_type_id("domain.value-resource@1");
    let definition = registry.resource_type(&resource_type).unwrap();

    assert_eq!(definition.metadata_requirements().len(), 1);
    assert_eq!(
        definition.metadata_requirements()[0].kind(),
        MetadataKind::Identifier
    );
    assert_eq!(
        definition.metadata_requirements()[0].inheritance(),
        MetadataInheritance::Inherited
    );
}

#[test]
fn the_same_specs_are_resolved_against_each_destination_registry() {
    let key = metadata_key_id("terrakit.coordinate-space@1");
    let capability = capability_id("domain.value@1");
    let resource_type = resource_type_id("domain.value-resource@1");

    let capability_spec = ResourceCapabilitySpec::new(capability.clone(), Schema::f32())
        .with_metadata(MetadataRequirement::required(key.clone()));
    let resource_type_spec = ResourceTypeSpec::new(resource_type.clone(), Schema::f32())
        .with_capability(CapabilityBinding::root(capability));

    let mut registry_a = ResourceRegistry::new();
    registry_a
        .register_metadata_key(
            MetadataKeySpec::new(key.clone(), MetadataKind::Identifier)
                .with_inheritance(MetadataInheritance::Inherited),
        )
        .unwrap();
    registry_a
        .register_capability(capability_spec.clone())
        .unwrap();
    registry_a
        .register_resource_type(resource_type_spec.clone())
        .unwrap();

    let mut registry_b = ResourceRegistry::new();
    registry_b
        .register_metadata_key(MetadataKeySpec::new(key, MetadataKind::String))
        .unwrap();
    registry_b.register_capability(capability_spec).unwrap();
    registry_b
        .register_resource_type(resource_type_spec)
        .unwrap();

    let requirement_a = &registry_a
        .resource_type(&resource_type)
        .unwrap()
        .metadata_requirements()[0];
    let requirement_b = &registry_b
        .resource_type(&resource_type)
        .unwrap()
        .metadata_requirements()[0];

    assert_eq!(requirement_a.kind(), MetadataKind::Identifier);
    assert_eq!(requirement_a.inheritance(), MetadataInheritance::Inherited);
    assert_eq!(requirement_b.kind(), MetadataKind::String);
    assert_eq!(requirement_b.inheritance(), MetadataInheritance::Exact);
}

#[test]
fn descriptor_validation_uses_one_contract_universe() {
    let registry_a = registry_with_coordinate_space(
        MetadataKind::Identifier,
        MetadataInheritance::Inherited,
        true,
    );
    let registry_b =
        registry_with_coordinate_space(MetadataKind::String, MetadataInheritance::Exact, true);
    let resource_type = resource_type_id("domain.value-resource@1");
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");

    let mut string_metadata = ResourceMetadata::new();
    string_metadata.insert_root(
        coordinate_space.clone(),
        MetadataValue::String("world".into()),
    );

    assert!(
        ResourceDescriptor::new(
            ResourceId(1),
            resource_type.clone(),
            string_metadata.clone(),
            &registry_b,
        )
        .is_ok()
    );

    assert!(matches!(
        ResourceDescriptor::new(ResourceId(2), resource_type, string_metadata, &registry_a,),
        Err(ResourceDescriptorError::InvalidMetadata(_))
    ));
}

#[test]
fn metadata_lookup_rejects_descriptor_from_another_registry() {
    let registry_a = registry_with_coordinate_space(
        MetadataKind::Identifier,
        MetadataInheritance::Inherited,
        true,
    );
    let registry_b = registry_with_coordinate_space(
        MetadataKind::Identifier,
        MetadataInheritance::Inherited,
        true,
    );
    let resource_type = resource_type_id("domain.value-resource@1");
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(
        coordinate_space.clone(),
        MetadataValue::Identifier("world".into()),
    );

    let descriptor =
        ResourceDescriptor::new(ResourceId(3), resource_type, metadata, &registry_a).unwrap();

    assert_eq!(
        registry_b.metadata_value(&descriptor, &ResourceView::Root, &coordinate_space,),
        Err(MetadataLookupError::ForeignDescriptor)
    );
}
