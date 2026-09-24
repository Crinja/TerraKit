use terrakit_core::resource::{IdError, MetadataKeyId, ResourceCapabilityId, ResourceTypeId};

#[test]
fn versioned_ids_are_validated() {
    assert!(ResourceTypeId::new("terrakit.mesh@1").is_ok());
    assert!(ResourceCapabilityId::new("terrakit.vector-field@2").is_ok());
    assert!(MetadataKeyId::new("terrakit.coordinate-space@1").is_ok());

    assert_eq!(
        ResourceTypeId::new("terrakit.mesh"),
        Err(IdError::MissingMajorVersion)
    );
    assert_eq!(
        ResourceTypeId::new("mesh@1"),
        Err(IdError::MissingNamespace)
    );
    assert_eq!(
        ResourceTypeId::new("terrakit.mesh@x"),
        Err(IdError::InvalidMajorVersion)
    );
    assert_eq!(
        ResourceTypeId::new(" terrakit.mesh@1"),
        Err(IdError::SurroundingWhitespace)
    );
    assert_eq!(
        MetadataKeyId::new("units@1"),
        Err(IdError::MissingNamespace)
    );
}
