mod common;

use common::{capability_id, metadata_key_id, metadata_key_registry};
use terrakit_core::resource::{
    CapabilityError, CapabilityRegistry, MetadataKeyDefinition, MetadataKeyRegistry, MetadataKind,
    MetadataRequirement, NumericType, ResourceCapabilityDefinition, Schema,
};

#[test]
fn capability_definition_validates_schema_and_metadata_requirements() {
    let metadata_registry = metadata_key_registry();

    let invalid_schema = Schema::Vector {
        element: NumericType::F32,
        lanes: 0,
    };

    assert!(matches!(
        ResourceCapabilityDefinition::new(
            capability_id("terrakit.invalid@1"),
            invalid_schema,
            vec![],
            &metadata_registry,
        ),
        Err(CapabilityError::InvalidSchema(_))
    ));

    let unknown = metadata_key_id("domain.unknown@1");

    assert_eq!(
        ResourceCapabilityDefinition::new(
            capability_id("terrakit.unknown-metadata@1"),
            Schema::f32(),
            vec![MetadataRequirement::required(unknown.clone())],
            &metadata_registry,
        ),
        Err(CapabilityError::UnknownMetadataKey(unknown))
    );

    let units = metadata_key_id("terrakit.units@1");

    assert_eq!(
        ResourceCapabilityDefinition::new(
            capability_id("terrakit.duplicate-metadata@1"),
            Schema::f32(),
            vec![
                MetadataRequirement::required(units.clone()),
                MetadataRequirement::optional(units.clone()),
            ],
            &metadata_registry,
        ),
        Err(CapabilityError::DuplicateMetadataRequirement(units))
    );
}

#[test]
fn capability_definition_accepts_registered_metadata_keys() {
    let mut metadata_registry = MetadataKeyRegistry::new();
    let key = metadata_key_id("domain.temperature-scale@1");

    metadata_registry
        .register(MetadataKeyDefinition::new(
            key.clone(),
            MetadataKind::Identifier,
        ))
        .unwrap();

    let definition = ResourceCapabilityDefinition::new(
        capability_id("domain.temperature@1"),
        Schema::f32(),
        vec![MetadataRequirement::required(key.clone())],
        &metadata_registry,
    )
    .unwrap();

    assert_eq!(definition.metadata_requirements().len(), 1);
    assert_eq!(definition.metadata_requirements()[0].key(), &key);
}

#[test]
fn capability_registry_rejects_duplicate_ids() {
    let metadata_registry = metadata_key_registry();
    let id = capability_id("terrakit.test@1");
    let definition =
        ResourceCapabilityDefinition::new(id.clone(), Schema::f32(), vec![], &metadata_registry)
            .unwrap();

    let mut registry = CapabilityRegistry::new();
    registry.register(definition.clone()).unwrap();

    assert!(registry.contains(&id));
    assert_eq!(registry.len(), 1);
    assert!(!registry.is_empty());
    assert_eq!(registry.iter().count(), 1);
    assert_eq!(
        registry.register(definition),
        Err(CapabilityError::DuplicateCapability(id))
    );
}

#[test]
fn capability_registry_iteration_is_deterministic() {
    let metadata_registry = metadata_key_registry();
    let mut registry = CapabilityRegistry::new();

    for id in ["terrakit.zeta@1", "terrakit.alpha@1", "terrakit.middle@1"] {
        registry
            .register(
                ResourceCapabilityDefinition::new(
                    capability_id(id),
                    Schema::f32(),
                    vec![],
                    &metadata_registry,
                )
                .unwrap(),
            )
            .unwrap();
    }

    let ids: Vec<_> = registry
        .iter()
        .map(|definition| definition.id().as_str())
        .collect();

    assert_eq!(
        ids,
        vec!["terrakit.alpha@1", "terrakit.middle@1", "terrakit.zeta@1",]
    );
}
