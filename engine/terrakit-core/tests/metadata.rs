mod common;

use common::{metadata_key_id, resource_registry};
use terrakit_core::resource::{
    MetadataKeySpec, MetadataKind, MetadataValue, ResourceMetadata, ResourceRegistry,
    ResourceRegistryError, ResourceView, SchemaPath,
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
fn resource_registry_rejects_duplicate_metadata_key_ids() {
    let id = metadata_key_id("terrakit.units@1");
    let spec = MetadataKeySpec::new(id.clone(), MetadataKind::Identifier);

    let mut registry = ResourceRegistry::new();
    registry.register_metadata_key(spec.clone()).unwrap();

    let definition = registry.metadata_key(&id).unwrap();
    assert_eq!(definition.id(), &id);
    assert_eq!(definition.kind(), MetadataKind::Identifier);

    assert_eq!(
        registry.register_metadata_key(spec),
        Err(ResourceRegistryError::DuplicateMetadataKey(id))
    );
}

#[test]
fn metadata_key_iteration_is_deterministic() {
    let mut registry = ResourceRegistry::new();

    for id in ["terrakit.zeta@1", "terrakit.alpha@1", "terrakit.middle@1"] {
        registry
            .register_metadata_key(MetadataKeySpec::new(
                metadata_key_id(id),
                MetadataKind::Identifier,
            ))
            .unwrap();
    }

    let ids: Vec<_> = registry
        .metadata_keys()
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

    assert_eq!(
        metadata.resolve(&direction, &units),
        Some((flow, &MetadataValue::Identifier("flow-units".into()),))
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

    assert_eq!(
        effective.get(&coordinate_space),
        Some(&&MetadataValue::Identifier("world".into()))
    );

    assert_eq!(
        effective.get(&units),
        Some(&&MetadataValue::Identifier("flow-units".into()))
    );
}

#[test]
fn existing_metadata_registry_contracts_are_unchanged() {
    let registry = resource_registry();
    let units = metadata_key_id("terrakit.units@1");

    assert_eq!(
        registry
            .metadata_key(&units)
            .map(|definition| definition.kind()),
        Some(MetadataKind::Identifier)
    );
}
