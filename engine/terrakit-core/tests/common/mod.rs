#![allow(dead_code)]

use terrakit_core::resource::{
    MetadataKeyId, MetadataKeySpec, MetadataKind, MetadataRequirement, ResourceCapabilityId,
    ResourceCapabilitySpec, ResourceRegistry, ResourceTypeId, Schema, SchemaField,
};

pub fn capability_id(value: &str) -> ResourceCapabilityId {
    ResourceCapabilityId::new(value).unwrap()
}

pub fn resource_type_id(value: &str) -> ResourceTypeId {
    ResourceTypeId::new(value).unwrap()
}

pub fn metadata_key_id(value: &str) -> MetadataKeyId {
    MetadataKeyId::new(value).unwrap()
}

pub fn vector_field_schema() -> Schema {
    Schema::dense_array(Schema::vec2_f32(), 2).unwrap()
}

pub fn hydraulic_schema() -> Schema {
    Schema::structure(vec![
        SchemaField::new("flow", vector_field_schema()).unwrap(),
        SchemaField::new("sediment", Schema::dense_array(Schema::f32(), 2).unwrap()).unwrap(),
        SchemaField::new("water", Schema::dense_array(Schema::f32(), 2).unwrap()).unwrap(),
    ])
    .unwrap()
}

pub fn resource_registry() -> ResourceRegistry {
    let mut registry = ResourceRegistry::new();

    registry
        .register_metadata_key(MetadataKeySpec::new(
            metadata_key_id("terrakit.coordinate-space@1"),
            MetadataKind::Identifier,
        ))
        .unwrap();

    registry
        .register_metadata_key(MetadataKeySpec::new(
            metadata_key_id("terrakit.units@1"),
            MetadataKind::Identifier,
        ))
        .unwrap();

    registry
        .register_metadata_key(MetadataKeySpec::new(
            metadata_key_id("terrakit.producer@1"),
            MetadataKind::String,
        ))
        .unwrap();

    registry
        .register_capability(ResourceCapabilitySpec::new(
            capability_id("domain.hydraulic-erosion@1"),
            hydraulic_schema(),
        ))
        .unwrap();

    registry
        .register_capability(
            ResourceCapabilitySpec::new(
                capability_id("terrakit.erosion-flow@1"),
                vector_field_schema(),
            )
            .with_metadata(MetadataRequirement::required(metadata_key_id(
                "terrakit.coordinate-space@1",
            ))),
        )
        .unwrap();

    registry
        .register_capability(ResourceCapabilitySpec::new(
            capability_id("terrakit.vector-field-2d@1"),
            vector_field_schema(),
        ))
        .unwrap();

    registry
}
