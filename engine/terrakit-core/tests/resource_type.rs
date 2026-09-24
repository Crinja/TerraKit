mod common;

use common::{capability_id, capability_registry, hydraulic_schema, resource_type_id};
use terrakit_core::resource::{
    CapabilityBinding, CapabilityRegistry, MetadataKind, MetadataRequirement,
    MetadataValidationError, MetadataValue, ResourceCapabilityDefinition, ResourceMetadata,
    ResourceTypeDefinition, ResourceTypeError, ResourceTypeRegistry, ResourceTypeRegistryError,
    ResourceView, Schema, SchemaField, SchemaPath,
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
                ResourceTypeDefinition::new(resource_type_id(id), Schema::f32(), vec![], &registry)
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
        vec!["domain.alpha@1", "domain.middle@1", "domain.zeta@1",]
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
                vec![MetadataRequirement::optional("units", MetadataKind::String)],
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
            scope: ResourceView::Root,
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

    assert_eq!(definition.metadata_requirements().len(), 1);

    assert_eq!(
        definition.metadata_requirements()[0].scope(),
        &ResourceView::Root
    );

    assert_eq!(
        definition.metadata_requirements()[0].requirement(),
        &MetadataRequirement::required("coordinate-space", MetadataKind::Identifier,)
    );
}

#[test]
fn resource_type_allows_different_metadata_kinds_at_parent_and_child_scopes() {
    let root_capability = capability_id("domain.root@1");
    let child_capability = capability_id("domain.child@1");

    let schema =
        Schema::structure(vec![SchemaField::new("value", Schema::f32()).unwrap()]).unwrap();

    let child = ResourceView::Path(SchemaPath::field("value"));

    let mut registry = CapabilityRegistry::new();

    registry
        .register(
            ResourceCapabilityDefinition::new(
                root_capability.clone(),
                schema.clone(),
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
                child_capability.clone(),
                Schema::f32(),
                vec![MetadataRequirement::required("units", MetadataKind::String)],
            )
            .unwrap(),
        )
        .unwrap();

    let definition = ResourceTypeDefinition::new(
        resource_type_id("domain.test@1"),
        schema,
        vec![
            CapabilityBinding::root(root_capability),
            CapabilityBinding::view(child_capability, child.clone()),
        ],
        &registry,
    )
    .unwrap();

    let mut metadata = ResourceMetadata::new();

    metadata.insert_root("units", MetadataValue::Identifier("world-units".into()));

    metadata.insert(child, "units", MetadataValue::String("child-units".into()));

    assert_eq!(definition.validate_metadata(&metadata), Ok(()));
}

#[test]
fn resource_type_rejects_invalid_runtime_metadata_scope() {
    let registry = capability_registry();

    let definition = ResourceTypeDefinition::new(
        resource_type_id("domain.test@1"),
        hydraulic_schema(),
        vec![],
        &registry,
    )
    .unwrap();

    let invalid = ResourceView::Path(SchemaPath::field("missing"));

    let mut metadata = ResourceMetadata::new();

    metadata.insert(
        invalid.clone(),
        "units",
        MetadataValue::Identifier("metres".into()),
    );

    assert!(matches!(
        definition.validate_metadata(&metadata),
        Err(MetadataValidationError::InvalidScope {
            scope,
            ..
        }) if scope == invalid
    ));
}

#[test]
fn resource_type_metadata_requirement_can_be_satisfied_by_parent_scope() {
    let capability = capability_id("domain.child@1");

    let schema =
        Schema::structure(vec![SchemaField::new("value", Schema::f32()).unwrap()]).unwrap();

    let child = ResourceView::Path(SchemaPath::field("value"));

    let mut registry = CapabilityRegistry::new();

    registry
        .register(
            ResourceCapabilityDefinition::new(
                capability.clone(),
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
        schema,
        vec![CapabilityBinding::view(capability, child)],
        &registry,
    )
    .unwrap();

    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(
        "coordinate-space",
        MetadataValue::Identifier("world".into()),
    );

    assert_eq!(definition.validate_metadata(&metadata), Ok(()));
}
