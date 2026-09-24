use terrakit_core::resource::{
    ResourceView, Schema, SchemaField, SchemaPath, SchemaVariant, ViewError,
};

#[test]
fn schema_path_resolves_every_navigation_type() {
    let variant = Schema::variant(vec![
        SchemaVariant::new(
            "value",
            Schema::structure(vec![
                SchemaField::new(
                    "directions",
                    Schema::fixed_array(Schema::vec2_f32(), 2),
                )
                .unwrap(),
            ])
            .unwrap(),
        )
        .unwrap(),
    ])
    .unwrap();

    let schema = Schema::structure(vec![
        SchemaField::new(
            "records",
            Schema::array(Schema::optional(variant)),
        )
        .unwrap(),
    ])
    .unwrap();

    let path = SchemaPath::field("records")
        .then_element()
        .then_optional_value()
        .then_variant("value")
        .then_field("directions")
        .then_element();

    assert_eq!(path.resolve(&schema), Ok(&Schema::vec2_f32()));
    assert_eq!(
        ResourceView::Path(path).resolve(&schema),
        Ok(&Schema::vec2_f32())
    );
}

#[test]
fn schema_path_reports_navigation_errors() {
    assert_eq!(
        SchemaPath::field("missing").resolve(&Schema::f32()),
        Err(ViewError::FieldOnNonStruct)
    );

    let structure = Schema::structure(vec![]).unwrap();
    assert_eq!(
        SchemaPath::field("missing").resolve(&structure),
        Err(ViewError::UnknownField("missing".into()))
    );

    assert_eq!(
        SchemaPath::root().then_element().resolve(&Schema::f32()),
        Err(ViewError::ElementOnNonArray)
    );

    assert_eq!(
        SchemaPath::root()
            .then_optional_value()
            .resolve(&Schema::f32()),
        Err(ViewError::OptionalValueOnNonOptional)
    );

    assert_eq!(
        SchemaPath::root().then_variant("x").resolve(&Schema::f32()),
        Err(ViewError::VariantOnNonVariant)
    );
}
