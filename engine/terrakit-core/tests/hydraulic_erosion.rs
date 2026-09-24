mod common;

use common::{capability_id, capability_registry, hydraulic_schema, resource_type_id};
use terrakit_core::resource::{
    CapabilityBinding, MetadataKind, MetadataValidationError, MetadataValue, ResourceMetadata,
    ResourceTypeDefinition, ResourceView, SchemaPath,
};

#[test]
fn resource_type_can_advertise_specialised_and_generic_capabilities() {
    let registry = capability_registry();
    let hydraulic = capability_id("domain.hydraulic-erosion@1");
    let erosion_flow = capability_id("terrakit.erosion-flow@1");
    let vector_field = capability_id("terrakit.vector-field-2d@1");

    let definition = ResourceTypeDefinition::new(
        resource_type_id("domain.hydraulic-erosion-result@1"),
        hydraulic_schema(),
        vec![
            CapabilityBinding::root(hydraulic.clone()),
            CapabilityBinding::view(
                erosion_flow.clone(),
                ResourceView::Path(SchemaPath::field("flow")),
            ),
            CapabilityBinding::view(
                vector_field.clone(),
                ResourceView::Path(SchemaPath::field("flow")),
            ),
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
        &ResourceView::Path(SchemaPath::field("flow"))
    );

    let metadata = ResourceMetadata::new();

    assert_eq!(
        definition.validate_metadata(&metadata),
        Err(MetadataValidationError::MissingRequired {
            key: "coordinate-space".into(),
            expected: MetadataKind::Identifier,
        })
    );

    let mut metadata = ResourceMetadata::new();

    metadata.insert(
        "coordinate-space",
        MetadataValue::Identifier("world".into()),
    );

    assert_eq!(
        definition.validate_metadata(&metadata),
        Ok(())
    );
}
