mod common;

use common::{capability_id, metadata_key_id, resource_registry};
use terrakit_core::resource::{
    CapabilityError, MetadataRequirement, NumericType, ResourceCapabilityDefinition,
    ResourceRegistryError, Schema,
};

#[test]
fn capability_definition_validates_schema_and_duplicate_requirements() {
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

    let units = metadata_key_id("terrakit.units@1");

    assert_eq!(
        ResourceCapabilityDefinition::new(
            capability_id("terrakit.duplicate-metadata@1"),
            Schema::f32(),
            vec![
                MetadataRequirement::required(units.clone()),
                MetadataRequirement::optional(units.clone()),
            ],
        ),
        Err(CapabilityError::DuplicateMetadataRequirement(units))
    );
}

#[test]
fn resource_registry_rejects_capability_with_unknown_metadata_key() {
    let mut registry = resource_registry();
    let unknown = metadata_key_id("domain.unknown@1");
    let capability = capability_id("domain.unknown-metadata@1");

    let definition = ResourceCapabilityDefinition::new(
        capability.clone(),
        Schema::f32(),
        vec![MetadataRequirement::required(unknown.clone())],
    )
    .unwrap();

    assert_eq!(
        registry.register_capability(definition),
        Err(ResourceRegistryError::UnknownMetadataKey {
            capability,
            key: unknown,
        })
    );
}

#[test]
fn resource_registry_rejects_duplicate_capability_ids() {
    let mut registry = resource_registry();
    let id = capability_id("terrakit.test@1");
    let definition = ResourceCapabilityDefinition::new(id.clone(), Schema::f32(), vec![]).unwrap();

    registry.register_capability(definition.clone()).unwrap();

    assert_eq!(registry.capability(&id), Some(&definition));
    assert_eq!(
        registry.register_capability(definition),
        Err(ResourceRegistryError::DuplicateCapability(id))
    );
}

#[test]
fn capability_iteration_is_deterministic() {
    let mut registry = resource_registry();

    for id in ["domain.zeta@1", "domain.alpha@1", "domain.middle@1"] {
        registry
            .register_capability(
                ResourceCapabilityDefinition::new(capability_id(id), Schema::f32(), vec![])
                    .unwrap(),
            )
            .unwrap();
    }

    let ids: Vec<_> = registry
        .capabilities()
        .map(|definition| definition.id().as_str())
        .collect();

    assert_eq!(
        ids,
        vec![
            "domain.alpha@1",
            "domain.hydraulic-erosion@1",
            "domain.middle@1",
            "domain.zeta@1",
            "terrakit.erosion-flow@1",
            "terrakit.vector-field-2d@1",
        ]
    );
}
