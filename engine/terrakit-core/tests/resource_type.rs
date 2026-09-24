mod common;

use common::{capability_id, capability_registry, hydraulic_schema, resource_type_id};
use terrakit_core::resource::{
    CapabilityBinding, CapabilityRegistry, MetadataKind, MetadataRequirement,
    ResourceCapabilityDefinition, ResourceTypeDefinition, ResourceTypeError,
    ResourceTypeRegistry, ResourceTypeRegistryError, ResourceView, Schema, SchemaPath,
};

#[test]
fn resource_type_rejects_unknown_capability() {
    let registry = CapabilityRegistry::new();
    let unknown = capability_id("domain.unknown@1");

    assert_eq!(
        ResourceTypeDefinition::new(
            resource_type_id("domain.test@1"),
            Schema::f32(),
            vec![CapabilityBinding::root(unknown.clone())],
            &registry,
        ),
        Err(ResourceTypeError::UnknownCapability(unknown))
    );
}

#[test]
fn resource_type_rejects_duplicate_capability_binding() {
    let registry = capability_registry();
    let erosion_flow = capability_id("terrakit.erosion-flow@1");

    assert_eq!(
        ResourceTypeDefinition::new(
            resource_type_id("domain.test@1"),
            hydraulic_schema(),
            vec![
                CapabilityBinding::view(
                    erosion_flow.clone(),
                    ResourceView::Path(SchemaPath::field("flow")),
                ),
                CapabilityBinding::view(
                    erosion_flow.clone(),
                    ResourceView::Path(SchemaPath::field("flow")),
                ),
            ],
            &registry,
        ),
        Err(ResourceTypeError::DuplicateCapabilityBinding(erosion_flow))
    );
}

#[test]
fn resource_type_rejects_invalid_capability_view() {
    let registry = capability_registry();
    let erosion_flow = capability_id("terrakit.erosion-flow@1");

    assert!(matches!(
        ResourceTypeDefinition::new(
            resource_type_id("domain.test@1"),
            hydraulic_schema(),
            vec![CapabilityBinding::view(
                erosion_flow,
                ResourceView::Path(SchemaPath::field("missing")),
            )],
            &registry,
        ),
        Err(ResourceTypeError::InvalidView { .. })
    ));
}

#[test]
fn resource_type_rejects_capability_schema_mismatch() {
    let registry = capability_registry();
    let erosion_flow = capability_id("terrakit.erosion-flow@1");

    assert!(matches!(
        ResourceTypeDefinition::new(
            resource_type_id("domain.test@1"),
            hydraulic_schema(),
            vec![CapabilityBinding::view(
                erosion_flow,
                ResourceView::Path(SchemaPath::field("sediment")),
            )],
            &registry,
        ),
        Err(ResourceTypeError::CapabilitySchemaMismatch { .. })
    ));
}

#[test]
fn resource_type_registry_rejects_duplicate_ids() {
    let registry = capability_registry();
    let definition = ResourceTypeDefinition::new(
        resource_type_id("domain.hydraulic-erosion-result@1"),
        hydraulic_schema(),
        vec![],
        &registry,
    )
    .unwrap();
    let id = definition.id().clone();

    let mut types = ResourceTypeRegistry::new();
    types.register(definition.clone()).unwrap();

    assert!(types.contains(&id));
    assert_eq!(types.len(), 1);
    assert!(!types.is_empty());
    assert_eq!(types.iter().count(), 1);
    assert_eq!(types.get(&id), Some(&definition));
    assert_eq!(
        types.register(definition),
        Err(ResourceTypeRegistryError::DuplicateType(id))
    );
}

#[test]
fn resource_type_registry_iteration_is_deterministic() {
    let registry = capability_registry();
    let mut types = ResourceTypeRegistry::new();

    for id in ["domain.zeta@1", "domain.alpha@1", "domain.middle@1"] {
        types
            .register(
                ResourceTypeDefinition::new(
                    resource_type_id(id),
                    Schema::f32(),
                    vec![],
                    &registry,
                )
                .unwrap(),
            )
            .unwrap();
    }

    let ids: Vec<_> = types
        .iter()
        .map(|definition| definition.id().as_str())
        .collect();

    assert_eq!(
        ids,
        vec![
            "domain.alpha@1",
            "domain.middle@1",
            "domain.zeta@1",
        ]
    );
}

#[test]
fn resource_type_rejects_conflicting_capability_metadata_requirements() {
    let first = capability_id("domain.first@1");
    let second = capability_id("domain.second@1");
    let mut registry = CapabilityRegistry::new();

    registry
        .register(
            ResourceCapabilityDefinition::new(
                first.clone(),
                Schema::f32(),
                vec![MetadataRequirement::required(
                    "units",
                    MetadataKind::Identifier,
                )],
            )
            .unwrap(),
        )
        .unwrap();

    registry
        .register(
            ResourceCapabilityDefinition::new(
                second.clone(),
                Schema::f32(),
                vec![MetadataRequirement::optional(
                    "units",
                    MetadataKind::String,
                )],
            )
            .unwrap(),
        )
        .unwrap();

    assert_eq!(
        ResourceTypeDefinition::new(
            resource_type_id("domain.test@1"),
            Schema::f32(),
            vec![
                CapabilityBinding::root(first),
                CapabilityBinding::root(second),
            ],
            &registry,
        ),
        Err(ResourceTypeError::ConflictingMetadataRequirement {
            key: "units".into(),
            existing: MetadataKind::Identifier,
            incoming: MetadataKind::String,
        })
    );
}

#[test]
fn resource_type_merges_compatible_capability_metadata_requirements() {
    let first = capability_id("domain.first@1");
    let second = capability_id("domain.second@1");
    let mut registry = CapabilityRegistry::new();

    registry
        .register(
            ResourceCapabilityDefinition::new(
                first.clone(),
                Schema::f32(),
                vec![MetadataRequirement::optional(
                    "coordinate-space",
                    MetadataKind::Identifier,
                )],
            )
            .unwrap(),
        )
        .unwrap();

    registry
        .register(
            ResourceCapabilityDefinition::new(
                second.clone(),
                Schema::f32(),
                vec![MetadataRequirement::required(
                    "coordinate-space",
                    MetadataKind::Identifier,
                )],
            )
            .unwrap(),
        )
        .unwrap();

    let definition = ResourceTypeDefinition::new(
        resource_type_id("domain.test@1"),
        Schema::f32(),
        vec![
            CapabilityBinding::root(first),
            CapabilityBinding::root(second),
        ],
        &registry,
    )
    .unwrap();

    assert_eq!(
        definition.metadata_requirements(),
        &[MetadataRequirement::required(
            "coordinate-space",
            MetadataKind::Identifier,
        )]
    );
}