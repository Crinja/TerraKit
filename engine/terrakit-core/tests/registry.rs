mod common;

use common::{capability_id, metadata_key_id, resource_type_id};
use terrakit_core::resource::{
    CapabilityBinding, MetadataKeyDefinition, MetadataKind, MetadataRequirement, MetadataValue,
    ResourceCapabilityDefinition, ResourceDescriptor, ResourceDescriptorError, ResourceId,
    ResourceMetadata, ResourceRegistry, ResourceTypeDefinition, Schema,
};

fn registry_with_coordinate_space(kind: MetadataKind, include_type: bool) -> ResourceRegistry {
    let key = metadata_key_id("terrakit.coordinate-space@1");
    let capability = capability_id("domain.value@1");
    let resource_type = resource_type_id("domain.value-resource@1");
    let mut registry = ResourceRegistry::new();

    registry
        .register_metadata_key(MetadataKeyDefinition::new(key.clone(), kind))
        .unwrap();

    registry
        .register_capability(
            ResourceCapabilityDefinition::new(
                capability.clone(),
                Schema::f32(),
                vec![MetadataRequirement::required(key)],
            )
            .unwrap(),
        )
        .unwrap();

    if include_type {
        registry
            .register_resource_type(
                ResourceTypeDefinition::new(
                    resource_type,
                    Schema::f32(),
                    vec![CapabilityBinding::root(capability)],
                )
                .unwrap(),
            )
            .unwrap();
    }

    registry
}

#[test]
fn resource_type_freezes_metadata_kind_from_its_registry() {
    let registry = registry_with_coordinate_space(MetadataKind::Identifier, true);
    let resource_type = resource_type_id("domain.value-resource@1");
    let definition = registry.resource_type(&resource_type).unwrap();

    assert_eq!(definition.metadata_requirements().len(), 1);
    assert_eq!(
        definition.metadata_requirements()[0].kind(),
        MetadataKind::Identifier
    );
}

#[test]
fn imported_resource_type_is_resolved_against_the_destination_registry() {
    let registry_a = registry_with_coordinate_space(MetadataKind::Identifier, true);
    let resource_type = resource_type_id("domain.value-resource@1");
    let definition_from_a = registry_a.resource_type(&resource_type).unwrap().clone();

    let mut registry_b = registry_with_coordinate_space(MetadataKind::String, false);
    registry_b
        .register_resource_type(definition_from_a)
        .unwrap();

    let definition_in_b = registry_b.resource_type(&resource_type).unwrap();

    assert_eq!(
        definition_in_b.metadata_requirements()[0].kind(),
        MetadataKind::String
    );
}

#[test]
fn descriptor_validation_uses_one_contract_universe() {
    let registry_a = registry_with_coordinate_space(MetadataKind::Identifier, true);
    let registry_b = registry_with_coordinate_space(MetadataKind::String, true);
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
