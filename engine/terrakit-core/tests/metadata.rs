mod common;

use common::{metadata_key_id, resource_registry};
use terrakit_core::resource::{
    MetadataInheritance, MetadataKeySpec, MetadataKind, MetadataValue, ResourceMetadata,
    ResourceRegistry, ResourceRegistryError, ResourceView, SchemaPath,
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
    assert_eq!(definition.inheritance(), MetadataInheritance::Exact);

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
fn resource_metadata_reports_explicit_scoped_values_and_kinds() {
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

    assert!(metadata.contains_exact(&flow, &units));
    assert!(!metadata.contains_exact(&flow, &coordinate_space));
    assert_eq!(metadata.len(), 2);
    assert_eq!(
        metadata.get_exact(&flow, &units).map(MetadataValue::kind),
        Some(MetadataKind::Identifier)
    );
}

#[test]
fn inherited_metadata_resolves_from_the_closest_scope() {
    let registry = resource_registry();
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");
    let flow = flow_view();
    let direction = flow_direction_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(
        coordinate_space.clone(),
        MetadataValue::Identifier("root".into()),
    );

    metadata.insert(
        flow.clone(),
        coordinate_space.clone(),
        MetadataValue::Identifier("flow".into()),
    );

    assert_eq!(
        registry.resolve_metadata(&metadata, &direction, &coordinate_space),
        Some((flow, &MetadataValue::Identifier("flow".into()),))
    );
}

#[test]
fn exact_metadata_does_not_inherit() {
    let registry = resource_registry();
    let units = metadata_key_id("terrakit.units@1");
    let flow = flow_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(units.clone(), MetadataValue::Identifier("metres".into()));

    assert_eq!(registry.metadata_value(&metadata, &flow, &units), None);

    assert_eq!(
        metadata.get_exact(&ResourceView::Root, &units),
        Some(&MetadataValue::Identifier("metres".into()))
    );
}

#[test]
fn inherited_metadata_does_not_inherit_from_siblings() {
    let registry = resource_registry();
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");
    let flow = flow_view();
    let water = water_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert(
        flow,
        coordinate_space.clone(),
        MetadataValue::Identifier("flow-space".into()),
    );

    assert_eq!(
        registry.metadata_value(&metadata, &water, &coordinate_space),
        None
    );
}

#[test]
fn effective_metadata_uses_each_keys_inheritance_policy() {
    let registry = resource_registry();
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

    let effective = registry.effective_metadata(&metadata, &flow);

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
    let coordinate_space = metadata_key_id("terrakit.coordinate-space@1");
    let units = metadata_key_id("terrakit.units@1");

    assert_eq!(
        registry
            .metadata_key(&coordinate_space)
            .map(|definition| definition.inheritance()),
        Some(MetadataInheritance::Inherited)
    );

    assert_eq!(
        registry
            .metadata_key(&units)
            .map(|definition| definition.inheritance()),
        Some(MetadataInheritance::Exact)
    );
}
