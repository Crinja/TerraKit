use terrakit_core::resource::{
    MetadataKind, MetadataRequirement, MetadataValidationError, MetadataValue, ResourceMetadata,
};

#[test]
fn resource_metadata_reports_values_and_kinds() {
    let mut metadata = ResourceMetadata::new();
    assert!(metadata.is_empty());

    metadata.insert("units", MetadataValue::Identifier("metres".into()));
    metadata.insert("lod", MetadataValue::U64(2));

    assert!(metadata.contains("units"));
    assert_eq!(metadata.len(), 2);
    assert_eq!(
        metadata.get("units").map(MetadataValue::kind),
        Some(MetadataKind::Identifier)
    );
    assert_eq!(
        metadata.get("lod").map(MetadataValue::kind),
        Some(MetadataKind::U64)
    );

    let keys: Vec<_> = metadata.iter().map(|(key, _)| key).collect();
    assert_eq!(keys, vec!["lod", "units"]);
}

#[test]
fn resource_metadata_rejects_missing_required_values() {
    let metadata = ResourceMetadata::new();
    let requirements = vec![MetadataRequirement::required(
        "coordinate-space",
        MetadataKind::Identifier,
    )];

    assert_eq!(
        metadata.validate(&requirements),
        Err(MetadataValidationError::MissingRequired {
            key: "coordinate-space".into(),
            expected: MetadataKind::Identifier,
        })
    );
}

#[test]
fn resource_metadata_rejects_wrong_value_kind() {
    let mut metadata = ResourceMetadata::new();
    metadata.insert(
        "coordinate-space",
        MetadataValue::String("world".into()),
    );

    let requirements = vec![MetadataRequirement::required(
        "coordinate-space",
        MetadataKind::Identifier,
    )];

    assert_eq!(
        metadata.validate(&requirements),
        Err(MetadataValidationError::KindMismatch {
            key: "coordinate-space".into(),
            expected: MetadataKind::Identifier,
            actual: MetadataKind::String,
        })
    );
}

#[test]
fn resource_metadata_allows_missing_optional_and_unknown_values() {
    let mut metadata = ResourceMetadata::new();
    metadata.insert(
        "producer",
        MetadataValue::String("test".into()),
    );

    let requirements = vec![MetadataRequirement::optional(
        "coordinate-space",
        MetadataKind::Identifier,
    )];

    assert_eq!(metadata.validate(&requirements), Ok(()));
}
