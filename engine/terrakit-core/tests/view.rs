use terrakit_core::resource::{
    ResourcePath, ResourceView, Schema, SchemaField, SchemaPath, SchemaVariant, ViewError,
};

#[test]
fn schema_path_resolves_every_navigation_type() {
    let variant = Schema::variant(vec![
        SchemaVariant::new(
            "value",
            Schema::structure(vec![
                SchemaField::new("directions", Schema::fixed_array(Schema::vec2_f32(), 2)).unwrap(),
            ])
            .unwrap(),
        )
        .unwrap(),
    ])
    .unwrap();

    let schema = Schema::structure(vec![
        SchemaField::new("records", Schema::array(Schema::optional(variant))).unwrap(),
    ])
    .unwrap();

    let path = SchemaPath::field("records")
        .then_element()
        .then_optional_value()
        .then_variant("value")
        .then_field("directions")
        .then_element();

    assert_eq!(path.resolve(&schema), Ok(&Schema::vec2_f32()));
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

#[test]
fn resource_view_parent_walks_towards_root() {
    let leaf = ResourceView::Path(ResourcePath::field("result").then_field("position"));

    let parent = leaf.parent().unwrap();

    assert_eq!(parent, ResourceView::Path(ResourcePath::field("result")));
    assert_eq!(parent.parent(), Some(ResourceView::Root));
    assert_eq!(ResourceView::Root.parent(), None);
}

#[test]
fn empty_resource_path_canonicalises_to_root() {
    assert_eq!(
        ResourceView::Path(ResourcePath::root()).canonical(),
        ResourceView::Root
    );
}

#[test]
fn resource_path_resolves_addressable_struct_fields() {
    let schema = Schema::structure(vec![
        SchemaField::new(
            "result",
            Schema::structure(vec![
                SchemaField::new("position", Schema::vec3_f32()).unwrap(),
            ])
            .unwrap(),
        )
        .unwrap(),
    ])
    .unwrap();

    let path = ResourcePath::field("result").then_field("position");

    assert_eq!(path.resolve(&schema), Ok(&Schema::vec3_f32()));
    assert_eq!(
        ResourceView::Path(path).resolve(&schema),
        Ok(&Schema::vec3_f32())
    );
}
