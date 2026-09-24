mod common;

use common::{
    capability_id, capability_registry, hydraulic_schema, metadata_key_id, metadata_key_registry,
    resource_type_id,
};

use terrakit_core::resource::{
    CapabilityBinding, MetadataKind, MetadataValidationError, MetadataValue, ResourceDescriptor,
    ResourceDescriptorError, ResourceId, ResourceMetadata, ResourceTypeDefinition, ResourceTypeId,
    ResourceTypeRegistry, ResourceView, SchemaPath,
};

fn hydraulic_types(
    metadata_registry: &terrakit_core::resource::MetadataKeyRegistry,
) -> (ResourceTypeRegistry, ResourceTypeId) {
    let capabilities = capability_registry(metadata_registry);

    let hydraulic = capability_id("domain.hydraulic-erosion@1");

    let erosion_flow = capability_id("terrakit.erosion-flow@1");

    let vector_field = capability_id("terrakit.vector-field-2d@1");

    let flow = ResourceView::Path(SchemaPath::field("flow"));

    let id = resource_type_id("domain.hydraulic-erosion-result@1");

    let definition = ResourceTypeDefinition::new(
        id.clone(),
        hydraulic_schema(),
        vec![
            CapabilityBinding::root(hydraulic),
            CapabilityBinding::view(erosion_flow, flow.clone()),
            CapabilityBinding::view(vector_field, flow),
        ],
        &capabilities,
    )
    .unwrap();

    let mut types = ResourceTypeRegistry::new();

    types.register(definition).unwrap();

    (types, id)
}

#[test]
fn resource_descriptor_accepts_valid_metadata() {
    let metadata_registry = metadata_key_registry();

    let (types, resource_type) = hydraulic_types(&metadata_registry);

    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");

    let flow = ResourceView::Path(SchemaPath::field("flow"));

    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(
        coordinate_space.clone(),
        MetadataValue::Identifier("world".into()),
    );

    let descriptor = ResourceDescriptor::new(
        ResourceId(42),
        resource_type.clone(),
        metadata,
        &types,
        &metadata_registry,
    )
    .unwrap();

    assert_eq!(descriptor.id(), &ResourceId(42));

    assert_eq!(descriptor.resource_type(), &resource_type);

    assert_eq!(
        descriptor.metadata().get(&flow, &coordinate_space),
        Some(&MetadataValue::Identifier("world".into(),))
    );
}

#[test]
fn resource_descriptor_rejects_missing_required_metadata() {
    let metadata_registry = metadata_key_registry();

    let (types, resource_type) = hydraulic_types(&metadata_registry);

    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");

    let flow = ResourceView::Path(SchemaPath::field("flow"));

    assert_eq!(
        ResourceDescriptor::new(
            ResourceId(42),
            resource_type,
            ResourceMetadata::new(),
            &types,
            &metadata_registry,
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
    let metadata_registry = metadata_key_registry();

    let (types, resource_type) = hydraulic_types(&metadata_registry);

    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");

    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(
        coordinate_space.clone(),
        MetadataValue::String("world".into()),
    );

    assert_eq!(
        ResourceDescriptor::new(
            ResourceId(42),
            resource_type,
            metadata,
            &types,
            &metadata_registry,
        ),
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
fn resource_descriptor_rejects_unknown_resource_type() {
    let metadata_registry = metadata_key_registry();
    let types = ResourceTypeRegistry::new();

    let resource_type = resource_type_id("domain.unknown@1");

    assert_eq!(
        ResourceDescriptor::new(
            ResourceId(42),
            resource_type.clone(),
            ResourceMetadata::new(),
            &types,
            &metadata_registry,
        ),
        Err(ResourceDescriptorError::UnknownResourceType(resource_type,))
    );
}

#[test]
fn resource_descriptor_rejects_invalid_metadata_scope() {
    let metadata_registry = metadata_key_registry();

    let (types, resource_type) = hydraulic_types(&metadata_registry);

    let units = metadata_key_id("terrakit.units@1");

    let invalid = ResourceView::Path(SchemaPath::field("missing"));

    let mut metadata = ResourceMetadata::new();

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
            &types,
            &metadata_registry,
        ),
        Err(ResourceDescriptorError::InvalidMetadata(
            MetadataValidationError::InvalidScope {
                scope,
                ..
            }
        )) if scope == invalid
    ));
}

#[test]
fn resource_descriptor_rejects_unknown_metadata_key() {
    let metadata_registry = metadata_key_registry();

    let (types, resource_type) = hydraulic_types(&metadata_registry);

    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");

    let unknown = metadata_key_id("plugin.unknown@1");

    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(coordinate_space, MetadataValue::Identifier("world".into()));

    metadata.insert_root(unknown.clone(), MetadataValue::Identifier("value".into()));

    assert_eq!(
        ResourceDescriptor::new(
            ResourceId(42),
            resource_type,
            metadata,
            &types,
            &metadata_registry,
        ),
        Err(ResourceDescriptorError::InvalidMetadata(
            MetadataValidationError::UnknownMetadataKey {
                scope: ResourceView::Root,
                key: unknown,
            }
        ))
    );
}
