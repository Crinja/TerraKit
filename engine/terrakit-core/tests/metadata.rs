mod common;

use common::{metadata_key_id, metadata_key_registry};
use terrakit_core::resource::{
    MetadataKeyDefinition, MetadataKeyRegistry, MetadataKeyRegistryError, MetadataKind,
    MetadataRequirement, MetadataValidationError, MetadataValue, ResourceMetadata, ResourceView,
    SchemaPath, ScopedMetadataRequirement,
};

fn flow_view() -> ResourceView {
    ResourceView::Path(SchemaPath::field("flow"))
}

fn flow_direction_view() -> ResourceView {
    ResourceView::Path(SchemaPath::field("flow").then_field("direction"))
}

fn water_view() -> ResourceView {
    ResourceView::Path(SchemaPath::field("water"))
}

#[test]
fn metadata_key_registry_rejects_duplicate_ids() {
    let id = metadata_key_id("terrakit.units@1");
    let definition = MetadataKeyDefinition::new(id.clone(), MetadataKind::Identifier);

    let mut registry = MetadataKeyRegistry::new();
    registry.register(definition.clone()).unwrap();

    assert!(registry.contains(&id));
    assert_eq!(registry.get(&id), Some(&definition));
    assert_eq!(registry.len(), 1);
    assert!(!registry.is_empty());
    assert_eq!(
        registry.register(definition),
        Err(MetadataKeyRegistryError::DuplicateKey(id))
    );
}

#[test]
fn metadata_key_registry_iteration_is_deterministic() {
    let mut registry = MetadataKeyRegistry::new();

    for id in ["terrakit.zeta@1", "terrakit.alpha@1", "terrakit.middle@1"] {
        registry
            .register(MetadataKeyDefinition::new(
                metadata_key_id(id),
                MetadataKind::Identifier,
            ))
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

#[test]
fn resource_metadata_reports_scoped_values_and_kinds() {
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");
    let units = metadata_key_id("terrakit.units@1");
    let flow = flow_view();
    let mut metadata = ResourceMetadata::new();

    assert!(metadata.is_empty());

    metadata.insert_root(
        coordinate_space.clone(),
        MetadataValue::Identifier("world".into()),
    );

    metadata.insert(
        flow.clone(),
        units.clone(),
        MetadataValue::Identifier("metres-per-second".into()),
    );

    assert!(metadata.contains(&flow, &coordinate_space));
    assert!(metadata.contains_exact(&flow, &units));
    assert!(!metadata.contains_exact(&flow, &coordinate_space));

    assert_eq!(metadata.len(), 2);

    assert_eq!(
        metadata
            .get(&flow, &coordinate_space)
            .map(MetadataValue::kind),
        Some(MetadataKind::Identifier)
    );

    assert_eq!(
        metadata.get_exact(&flow, &units).map(MetadataValue::kind),
        Some(MetadataKind::Identifier)
    );
}

#[test]
fn resource_metadata_inherits_from_the_closest_scope() {
    let units = metadata_key_id("terrakit.units@1");
    let flow = flow_view();
    let direction = flow_direction_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(
        units.clone(),
        MetadataValue::Identifier("root-units".into()),
    );

    metadata.insert(
        flow.clone(),
        units.clone(),
        MetadataValue::Identifier("flow-units".into()),
    );

    metadata.insert(
        direction.clone(),
        units.clone(),
        MetadataValue::Identifier("direction-units".into()),
    );

    assert_eq!(
        metadata.resolve(&direction, &units),
        Some((
            direction.clone(),
            &MetadataValue::Identifier("direction-units".into()),
        ))
    );

    assert_eq!(
        metadata.resolve(&flow, &units),
        Some((
            flow.clone(),
            &MetadataValue::Identifier("flow-units".into()),
        ))
    );
}

#[test]
fn resource_metadata_inherits_through_multiple_scopes() {
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");
    let direction = flow_direction_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(
        coordinate_space.clone(),
        MetadataValue::Identifier("world".into()),
    );

    assert_eq!(
        metadata.get(&direction, &coordinate_space),
        Some(&MetadataValue::Identifier("world".into()))
    );
}

#[test]
fn resource_metadata_does_not_inherit_from_siblings() {
    let units = metadata_key_id("terrakit.units@1");
    let flow = flow_view();
    let water = water_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert(
        flow,
        units.clone(),
        MetadataValue::Identifier("metres-per-second".into()),
    );

    assert_eq!(metadata.get(&water, &units), None);
}

#[test]
fn resource_metadata_effective_entries_use_closest_values() {
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");
    let units = metadata_key_id("terrakit.units@1");
    let flow = flow_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(
        coordinate_space.clone(),
        MetadataValue::Identifier("world".into()),
    );

    metadata.insert_root(
        units.clone(),
        MetadataValue::Identifier("root-units".into()),
    );

    metadata.insert(
        flow.clone(),
        units.clone(),
        MetadataValue::Identifier("flow-units".into()),
    );

    let effective = metadata.effective_entries(&flow);

    let coordinate_space_value = effective
        .iter()
        .find(|(key, _)| key.as_str() == coordinate_space.as_str())
        .map(|(_, value)| *value);

    let units_value = effective
        .iter()
        .find(|(key, _)| key.as_str() == units.as_str())
        .map(|(_, value)| *value);

    assert_eq!(
        coordinate_space_value,
        Some(&MetadataValue::Identifier("world".into()))
    );

    assert_eq!(
        units_value,
        Some(&MetadataValue::Identifier("flow-units".into()))
    );
}

#[test]
fn resource_metadata_rejects_missing_required_values() {
    let metadata_registry = metadata_key_registry();
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");
    let flow = flow_view();
    let metadata = ResourceMetadata::new();

    let requirements = vec![ScopedMetadataRequirement::new(
        flow.clone(),
        MetadataRequirement::required(coordinate_space.clone()),
    )];

    assert_eq!(
        metadata.validate(&requirements, &metadata_registry),
        Err(MetadataValidationError::MissingRequired {
            scope: flow,
            key: coordinate_space,
            expected: MetadataKind::Identifier,
        })
    );
}

#[test]
fn resource_metadata_rejects_wrong_value_kind() {
    let metadata_registry = metadata_key_registry();
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");
    let flow = flow_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert(
        flow.clone(),
        coordinate_space.clone(),
        MetadataValue::String("world".into()),
    );

    let requirements = vec![ScopedMetadataRequirement::new(
        flow.clone(),
        MetadataRequirement::required(coordinate_space.clone()),
    )];

    assert_eq!(
        metadata.validate(&requirements, &metadata_registry),
        Err(MetadataValidationError::KindMismatch {
            scope: flow,
            key: coordinate_space,
            expected: MetadataKind::Identifier,
            actual: MetadataKind::String,
        })
    );
}

#[test]
fn resource_metadata_rejects_unknown_metadata_keys() {
    let metadata_registry = metadata_key_registry();
    let unknown = metadata_key_id("plugin.unknown@1");
    let flow = flow_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert(
        flow.clone(),
        unknown.clone(),
        MetadataValue::Identifier("value".into()),
    );

    assert_eq!(
        metadata.validate(&[], &metadata_registry),
        Err(MetadataValidationError::UnknownMetadataKey {
            scope: flow,
            key: unknown,
        })
    );
}

#[test]
fn resource_metadata_allows_missing_optional_and_extra_registered_values() {
    let metadata_registry = metadata_key_registry();
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");
    let producer = metadata_key_id("terrakit.producer@1");
    let flow = flow_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(producer, MetadataValue::String("test".into()));

    let requirements = vec![ScopedMetadataRequirement::new(
        flow,
        MetadataRequirement::optional(coordinate_space),
    )];

    assert_eq!(metadata.validate(&requirements, &metadata_registry), Ok(()));
}

#[test]
fn namespaced_metadata_keys_do_not_override_each_other() {
    let mut metadata_registry = metadata_key_registry();
    let terrakit_units = metadata_key_id("terrakit.units@1");
    let plugin_units = metadata_key_id("plugin.units@1");

    metadata_registry
        .register(MetadataKeyDefinition::new(
            plugin_units.clone(),
            MetadataKind::String,
        ))
        .unwrap();

    let flow = flow_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(
        terrakit_units.clone(),
        MetadataValue::Identifier("si".into()),
    );

    metadata.insert(
        flow.clone(),
        plugin_units.clone(),
        MetadataValue::String("m/s".into()),
    );

    assert_eq!(
        metadata.get(&flow, &terrakit_units),
        Some(&MetadataValue::Identifier("si".into()))
    );

    assert_eq!(
        metadata.get(&flow, &plugin_units),
        Some(&MetadataValue::String("m/s".into()))
    );

    assert_eq!(metadata.validate(&[], &metadata_registry), Ok(()));
}
