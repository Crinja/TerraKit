use terrakit_core::resource::{MetadataKind, MetadataValue, ResourceMetadata};

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
