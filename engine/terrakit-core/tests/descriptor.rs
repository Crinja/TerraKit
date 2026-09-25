mod common;

use common::{hydraulic_schema, metadata_key_id, resource_registry, resource_type_id};
use terrakit_core::resource::{
    CapabilityBinding, MetadataKind, MetadataValidationError, MetadataValue, ResourceDescriptor,
    ResourceDescriptorError, ResourceId, ResourceMetadata, ResourcePath, ResourceTypeSpec,
    ResourceView,
};

fn hydraulic_registry() -> (
    terrakit_core::resource::ResourceRegistry,
    terrakit_core::resource::ResourceTypeId,
) {
    let mut registry = resource_registry();
    let resource_type = resource_type_id("domain.hydraulic-erosion-result@1");

    registry
        .register_resource_type(
            ResourceTypeSpec::new(resource_type.clone(), hydraulic_schema()).with_capability(
                CapabilityBinding::view(
                    common::capability_id("terrakit.erosion-flow@1"),
                    ResourceView::Path(ResourcePath::field("flow")),
                ),
            ),
        )
        .unwrap();

    (registry, resource_type)
}

#[test]
fn resource_descriptor_accepts_valid_inherited_metadata() {
    let (registry, resource_type) = hydraulic_registry();
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");
    let flow = ResourceView::Path(ResourcePath::field("flow"));
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(
        coordinate_space.clone(),
        MetadataValue::Identifier("world".into()),
    );

    let descriptor =
        ResourceDescriptor::new(ResourceId(42), resource_type.clone(), metadata, &registry)
            .unwrap();

    assert_eq!(descriptor.id(), &ResourceId(42));
    assert_eq!(descriptor.resource_type(), &resource_type);
    assert_eq!(
        registry
            .metadata_value(&descriptor, &flow, &coordinate_space)
            .unwrap(),
        Some(&MetadataValue::Identifier("world".into()))
    );
}

#[test]
fn resource_descriptor_rejects_missing_required_metadata() {
    let (registry, resource_type) = hydraulic_registry();
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");
    let flow = ResourceView::Path(ResourcePath::field("flow"));

    assert_eq!(
        ResourceDescriptor::new(
            ResourceId(42),
            resource_type,
            ResourceMetadata::new(),
            &registry,
        ),
        Err(ResourceDescriptorError::InvalidMetadata(
            MetadataValidationError::MissingRequired {
                scope: flow,
                key: coordinate_space,
                expected: MetadataKind::Identifier,
            }
        ))
    );
}

#[test]
fn resource_descriptor_rejects_wrong_metadata_kind() {
    let (registry, resource_type) = hydraulic_registry();
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(
        coordinate_space.clone(),
        MetadataValue::String("world".into()),
    );

    assert_eq!(
        ResourceDescriptor::new(ResourceId(42), resource_type, metadata, &registry,),
        Err(ResourceDescriptorError::InvalidMetadata(
            MetadataValidationError::KindMismatch {
                scope: ResourceView::Root,
                key: coordinate_space,
                expected: MetadataKind::Identifier,
                actual: MetadataKind::String,
            }
        ))
    );
}

#[test]
fn resource_descriptor_rejects_unknown_metadata_key() {
    let (registry, resource_type) = hydraulic_registry();
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");
    let unknown = metadata_key_id("plugin.unknown@1");
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(coordinate_space, MetadataValue::Identifier("world".into()));
    metadata.insert_root(unknown.clone(), MetadataValue::Identifier("value".into()));

    assert_eq!(
        ResourceDescriptor::new(ResourceId(42), resource_type, metadata, &registry,),
        Err(ResourceDescriptorError::InvalidMetadata(
            MetadataValidationError::UnknownMetadataKey {
                scope: ResourceView::Root,
                key: unknown,
            }
        ))
    );
}

#[test]
fn resource_descriptor_rejects_invalid_metadata_scope() {
    let (registry, resource_type) = hydraulic_registry();
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");
    let units = metadata_key_id("terrakit.units@1");
    let invalid = ResourceView::Path(ResourcePath::field("missing"));
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(coordinate_space, MetadataValue::Identifier("world".into()));
    metadata.insert(
        invalid.clone(),
        units,
        MetadataValue::Identifier("metres".into()),
    );

    assert!(matches!(
        ResourceDescriptor::new(
            ResourceId(42),
            resource_type,
            metadata,
            &registry,
        ),
        Err(ResourceDescriptorError::InvalidMetadata(
            MetadataValidationError::InvalidScope { scope, .. }
        )) if scope == invalid
    ));
}

#[test]
fn resource_descriptor_rejects_unknown_resource_type() {
    let registry = resource_registry();
    let resource_type = resource_type_id("domain.unknown@1");

    assert_eq!(
        ResourceDescriptor::new(
            ResourceId(42),
            resource_type.clone(),
            ResourceMetadata::new(),
            &registry,
        ),
        Err(ResourceDescriptorError::UnknownResourceType(resource_type))
    );
}
