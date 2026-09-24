mod common;

use common::capability_id;
use terrakit_core::resource::{
    CapabilityError, CapabilityRegistry, MetadataKind, MetadataRequirement, NumericType,
    ResourceCapabilityDefinition, Schema,
};

#[test]
fn capability_definition_validates_schema_and_metadata_requirements() {
    let invalid_schema = Schema::Vector {
        element: NumericType::F32,
        lanes: 0,
    };

    assert!(matches!(
        ResourceCapabilityDefinition::new(
            capability_id("terrakit.invalid@1"),
            invalid_schema,
            vec![],
        ),
        Err(CapabilityError::InvalidSchema(_))
    ));

    assert_eq!(
        ResourceCapabilityDefinition::new(
            capability_id("terrakit.empty-metadata@1"),
            Schema::f32(),
            vec![MetadataRequirement::required("", MetadataKind::Identifier)],
        ),
        Err(CapabilityError::EmptyMetadataKey)
    );

    assert_eq!(
        ResourceCapabilityDefinition::new(
            capability_id("terrakit.duplicate-metadata@1"),
            Schema::f32(),
            vec![
                MetadataRequirement::required("units", MetadataKind::Identifier),
                MetadataRequirement::optional("units", MetadataKind::String),
            ],
        ),
        Err(CapabilityError::DuplicateMetadataRequirement(
            "units".into()
        ))
    );
}

#[test]
fn capability_registry_rejects_duplicate_ids() {
    let id = capability_id("terrakit.test@1");
    let definition = ResourceCapabilityDefinition::new(id.clone(), Schema::f32(), vec![]).unwrap();

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
    let mut registry = CapabilityRegistry::new();

    for id in ["terrakit.zeta@1", "terrakit.alpha@1", "terrakit.middle@1"] {
        registry
            .register(
                ResourceCapabilityDefinition::new(capability_id(id), Schema::f32(), vec![])
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
