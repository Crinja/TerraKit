use terrakit_core::resource::{
    MetadataKind, MetadataRequirement, MetadataValidationError, MetadataValue, ResourceMetadata,
    ResourceView, SchemaPath, ScopedMetadataRequirement,
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
fn resource_metadata_reports_scoped_values_and_kinds() {
    let flow = flow_view();
    let mut metadata = ResourceMetadata::new();

    assert!(metadata.is_empty());

    metadata.insert_root(
        "coordinate-space",
        MetadataValue::Identifier("world".into()),
    );

    metadata.insert(
        flow.clone(),
        "units",
        MetadataValue::Identifier("metres-per-second".into()),
    );

    assert!(metadata.contains(&flow, "coordinate-space"));
    assert!(metadata.contains_exact(&flow, "units"));
    assert!(!metadata.contains_exact(&flow, "coordinate-space"));

    assert_eq!(metadata.len(), 2);

    assert_eq!(
        metadata
            .get(&flow, "coordinate-space")
            .map(MetadataValue::kind),
        Some(MetadataKind::Identifier)
    );

    assert_eq!(
        metadata.get_exact(&flow, "units").map(MetadataValue::kind),
        Some(MetadataKind::Identifier)
    );
}

#[test]
fn resource_metadata_inherits_from_the_closest_scope() {
    let flow = flow_view();
    let direction = flow_direction_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root("units", MetadataValue::Identifier("root-units".into()));

    metadata.insert(
        flow.clone(),
        "units",
        MetadataValue::Identifier("flow-units".into()),
    );

    metadata.insert(
        direction.clone(),
        "units",
        MetadataValue::Identifier("direction-units".into()),
    );

    assert_eq!(
        metadata.resolve(&direction, "units"),
        Some((
            direction.clone(),
            &MetadataValue::Identifier("direction-units".into()),
        ))
    );

    assert_eq!(
        metadata.resolve(&flow, "units"),
        Some((
            flow.clone(),
            &MetadataValue::Identifier("flow-units".into()),
        ))
    );
}

#[test]
fn resource_metadata_inherits_through_multiple_scopes() {
    let direction = flow_direction_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(
        "coordinate-space",
        MetadataValue::Identifier("world".into()),
    );

    assert_eq!(
        metadata.get(&direction, "coordinate-space"),
        Some(&MetadataValue::Identifier("world".into()))
    );
}

#[test]
fn resource_metadata_does_not_inherit_from_siblings() {
    let flow = flow_view();
    let water = water_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert(
        flow,
        "units",
        MetadataValue::Identifier("metres-per-second".into()),
    );

    assert_eq!(metadata.get(&water, "units"), None);
}

#[test]
fn resource_metadata_effective_entries_use_closest_values() {
    let flow = flow_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(
        "coordinate-space",
        MetadataValue::Identifier("world".into()),
    );

    metadata.insert_root("units", MetadataValue::Identifier("root-units".into()));

    metadata.insert(
        flow.clone(),
        "units",
        MetadataValue::Identifier("flow-units".into()),
    );

    let effective = metadata.effective_entries(&flow);

    assert_eq!(
        effective.get("coordinate-space"),
        Some(&&MetadataValue::Identifier("world".into()))
    );

    assert_eq!(
        effective.get("units"),
        Some(&&MetadataValue::Identifier("flow-units".into()))
    );
}

#[test]
fn resource_metadata_rejects_missing_required_values() {
    let flow = flow_view();
    let metadata = ResourceMetadata::new();

    let requirements = vec![ScopedMetadataRequirement::new(
        flow.clone(),
        MetadataRequirement::required("coordinate-space", MetadataKind::Identifier),
    )];

    assert_eq!(
        metadata.validate(&requirements),
        Err(MetadataValidationError::MissingRequired {
            scope: flow,
            key: "coordinate-space".into(),
            expected: MetadataKind::Identifier,
        })
    );
}

#[test]
fn resource_metadata_rejects_the_closest_wrong_value_kind() {
    let flow = flow_view();
    let direction = flow_direction_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root(
        "coordinate-space",
        MetadataValue::Identifier("world".into()),
    );

    metadata.insert(
        flow.clone(),
        "coordinate-space",
        MetadataValue::String("local".into()),
    );

    let requirements = vec![ScopedMetadataRequirement::new(
        direction.clone(),
        MetadataRequirement::required("coordinate-space", MetadataKind::Identifier),
    )];

    assert_eq!(
        metadata.validate(&requirements),
        Err(MetadataValidationError::KindMismatch {
            scope: direction,
            resolved_scope: flow,
            key: "coordinate-space".into(),
            expected: MetadataKind::Identifier,
            actual: MetadataKind::String,
        })
    );
}

#[test]
fn resource_metadata_allows_child_override_with_different_kind() {
    let flow = flow_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root("units", MetadataValue::Identifier("world-units".into()));

    metadata.insert(flow.clone(), "units", MetadataValue::String("m/s".into()));

    let requirements = vec![
        ScopedMetadataRequirement::new(
            ResourceView::Root,
            MetadataRequirement::required("units", MetadataKind::Identifier),
        ),
        ScopedMetadataRequirement::new(
            flow,
            MetadataRequirement::required("units", MetadataKind::String),
        ),
    ];

    assert_eq!(metadata.validate(&requirements), Ok(()));
}

#[test]
fn resource_metadata_allows_missing_optional_and_unknown_values() {
    let flow = flow_view();
    let mut metadata = ResourceMetadata::new();

    metadata.insert_root("producer", MetadataValue::String("test".into()));

    let requirements = vec![ScopedMetadataRequirement::new(
        flow,
        MetadataRequirement::optional("coordinate-space", MetadataKind::Identifier),
    )];

    assert_eq!(metadata.validate(&requirements), Ok(()));
}
