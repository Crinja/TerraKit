mod common;

use common::{
    capability_id, hydraulic_schema, metadata_key_id, resource_registry, resource_type_id,
};
use terrakit_core::resource::{
    CapabilityBinding, MetadataInheritance, MetadataKind, MetadataValue, ResourceDescriptor,
    ResourceId, ResourceMetadata, ResourcePath, ResourceTypeSpec, ResourceView,
};

#[test]
fn resource_type_can_advertise_specialised_and_generic_capabilities() {
    let mut registry = resource_registry();

    let hydraulic = capability_id("domain.hydraulic-erosion@1");
    let erosion_flow = capability_id("terrakit.erosion-flow@1");
    let vector_field = capability_id("terrakit.vector-field-2d@1");
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");
    let resource_type = resource_type_id("domain.hydraulic-erosion-result@1");
    let flow = ResourceView::Path(ResourcePath::field("flow"));

    registry
        .register_resource_type(
            ResourceTypeSpec::new(resource_type.clone(), hydraulic_schema())
                .with_capability(CapabilityBinding::root(hydraulic.clone()))
                .with_capability(CapabilityBinding::view(erosion_flow.clone(), flow.clone()))
                .with_capability(CapabilityBinding::view(vector_field.clone(), flow.clone())),
        )
        .unwrap();

    let definition = registry.resource_type(&resource_type).unwrap();

    assert!(definition.has_capability(&hydraulic));
    assert!(definition.has_capability(&erosion_flow));
    assert!(definition.has_capability(&vector_field));
    assert_eq!(definition.capabilities().len(), 3);
    assert_eq!(
        definition
            .capability(&erosion_flow)
            .unwrap()
            .resource_view(),
        &flow
    );

    let requirement = definition
        .metadata_requirements()
        .iter()
        .find(|requirement| requirement.requirement().key() == &coordinate_space)
        .unwrap();

    assert_eq!(requirement.scope(), &flow);
    assert_eq!(requirement.kind(), MetadataKind::Identifier);
    assert_eq!(requirement.inheritance(), MetadataInheritance::Inherited);

    let mut metadata = ResourceMetadata::new();
    metadata.insert_root(coordinate_space, MetadataValue::Identifier("world".into()));

    assert!(ResourceDescriptor::new(ResourceId(1), resource_type, metadata, &registry,).is_ok());
}
