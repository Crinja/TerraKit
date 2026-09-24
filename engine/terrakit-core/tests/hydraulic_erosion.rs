mod common;

use common::{
    capability_id, capability_registry, hydraulic_schema, metadata_key_id, metadata_key_registry,
    resource_type_id,
};
use terrakit_core::resource::{
    CapabilityBinding, MetadataKind, MetadataValidationError, MetadataValue, ResourceMetadata,
    ResourceTypeDefinition, ResourceView, SchemaPath,
};

#[test]
fn resource_type_can_advertise_specialised_and_generic_capabilities() {
    let metadata_registry = metadata_key_registry();
    let registry = capability_registry(&metadata_registry);

    let hydraulic = capability_id("domain.hydraulic-erosion@1");
    let erosion_flow = capability_id("terrakit.erosion-flow@1");
    let vector_field = capability_id("terrakit.vector-field-2d@1");
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");

    let flow = ResourceView::Path(SchemaPath::field("flow"));

    let definition = ResourceTypeDefinition::new(
        resource_type_id("domain.hydraulic-erosion-result@1"),
        hydraulic_schema(),
        vec![
            CapabilityBinding::root(hydraulic.clone()),
            CapabilityBinding::view(erosion_flow.clone(), flow.clone()),
            CapabilityBinding::view(vector_field.clone(), flow.clone()),
        ],
        &registry,
    )
    .unwrap();

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

    let metadata = ResourceMetadata::new();

    assert_eq!(
        definition.validate_metadata(&metadata, &metadata_registry),
        Err(MetadataValidationError::MissingRequired {
            scope: flow.clone(),
            key: coordinate_space.clone(),
            expected: MetadataKind::Identifier,
        })
    );

    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(coordinate_space, MetadataValue::Identifier("world".into()));

    assert_eq!(
        definition.validate_metadata(&metadata, &metadata_registry),
        Ok(())
    );
}
