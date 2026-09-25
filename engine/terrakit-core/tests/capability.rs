mod common;

use common::{capability_id, metadata_key_id, resource_registry};
use terrakit_core::resource::{
    CapabilityError, MetadataRequirement, NumericType, ResourceCapabilitySpec, ResourceRegistry,
    ResourceRegistryError, Schema,
};

#[test]
fn resource_registry_validates_capability_schema_and_duplicate_requirements() {
    let invalid_schema = Schema::Vector {
        element: NumericType::F32,
        lanes: 0,
    };

    let mut registry = ResourceRegistry::new();

    assert!(matches!(
        registry.register_capability(ResourceCapabilitySpec::new(
            capability_id("terrakit.invalid@1"),
            invalid_schema,
        )),
        Err(ResourceRegistryError::InvalidCapability(
            CapabilityError::InvalidSchema(_)
        ))
    ));

    let mut registry = resource_registry();
    let units = metadata_key_id("terrakit.units@1");

    assert_eq!(
        registry.register_capability(
            ResourceCapabilitySpec::new(
                capability_id("terrakit.duplicate-metadata@1"),
                Schema::f32(),
            )
            .with_metadata(MetadataRequirement::required(units.clone()))
            .with_metadata(MetadataRequirement::optional(units.clone())),
        ),
        Err(ResourceRegistryError::InvalidCapability(
            CapabilityError::DuplicateMetadataRequirement(units)
        ))
    );
}

#[test]
fn resource_registry_rejects_capability_with_unknown_metadata_key() {
    let mut registry = resource_registry();
    let unknown = metadata_key_id("domain.unknown@1");
    let capability = capability_id("domain.unknown-metadata@1");

    let spec = ResourceCapabilitySpec::new(capability.clone(), Schema::f32())
        .with_metadata(MetadataRequirement::required(unknown.clone()));

    assert_eq!(
        registry.register_capability(spec),
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
    let spec = ResourceCapabilitySpec::new(id.clone(), Schema::f32());

    registry.register_capability(spec.clone()).unwrap();

    let definition = registry.capability(&id).unwrap();
    assert_eq!(definition.id(), &id);
    assert_eq!(definition.view_schema(), &Schema::f32());

    assert_eq!(
        registry.register_capability(spec),
        Err(ResourceRegistryError::DuplicateCapability(id))
    );
}

#[test]
fn capability_definition_freezes_metadata_kind() {
    let mut registry = resource_registry();
    let units = metadata_key_id("terrakit.units@1");
    let id = capability_id("domain.units@1");

    registry
        .register_capability(
            ResourceCapabilitySpec::new(id.clone(), Schema::f32())
                .with_metadata(MetadataRequirement::required(units.clone())),
        )
        .unwrap();

    let definition = registry.capability(&id).unwrap();
    let requirement = &definition.metadata_requirements()[0];

    assert_eq!(requirement.key(), &units);
    assert!(requirement.is_required());
    assert_eq!(
        requirement.kind(),
        terrakit_core::resource::MetadataKind::Identifier
    );
}

#[test]
fn capability_iteration_is_deterministic() {
    let mut registry = resource_registry();

    for id in ["domain.zeta@1", "domain.alpha@1", "domain.middle@1"] {
        registry
            .register_capability(ResourceCapabilitySpec::new(
                capability_id(id),
                Schema::f32(),
            ))
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
