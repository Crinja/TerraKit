#![allow(dead_code)]

use terrakit_core::resource::{
    CapabilityRegistry, MetadataKeyDefinition, MetadataKeyId, MetadataKeyRegistry, MetadataKind,
    MetadataRequirement, ResourceCapabilityDefinition, ResourceCapabilityId, ResourceTypeId,
    Schema, SchemaField,
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

pub fn metadata_key_registry() -> MetadataKeyRegistry {
    let mut registry = MetadataKeyRegistry::new();

    registry
        .register(MetadataKeyDefinition::new(
            metadata_key_id("terrakit.coordinate-space@1"),
            MetadataKind::Identifier,
        ))
        .unwrap();

    registry
        .register(MetadataKeyDefinition::new(
            metadata_key_id("terrakit.units@1"),
            MetadataKind::Identifier,
        ))
        .unwrap();

    registry
        .register(MetadataKeyDefinition::new(
            metadata_key_id("terrakit.producer@1"),
            MetadataKind::String,
        ))
        .unwrap();

    registry
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

pub fn capability_registry(metadata_registry: &MetadataKeyRegistry) -> CapabilityRegistry {
    let hydraulic_schema = hydraulic_schema();
    let mut registry = CapabilityRegistry::new();

    registry
        .register(
            ResourceCapabilityDefinition::new(
                capability_id("domain.hydraulic-erosion@1"),
                hydraulic_schema,
                vec![],
                metadata_registry,
            )
            .unwrap(),
        )
        .unwrap();

    registry
        .register(
            ResourceCapabilityDefinition::new(
                capability_id("terrakit.erosion-flow@1"),
                vector_field_schema(),
                vec![MetadataRequirement::required(metadata_key_id(
                    "terrakit.coordinate-space@1",
                ))],
                metadata_registry,
            )
            .unwrap(),
        )
        .unwrap();

    registry
        .register(
            ResourceCapabilityDefinition::new(
                capability_id("terrakit.vector-field-2d@1"),
                vector_field_schema(),
                vec![],
                metadata_registry,
            )
            .unwrap(),
        )
        .unwrap();

    registry
}
