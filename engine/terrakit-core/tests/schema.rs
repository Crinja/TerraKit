use terrakit_core::resource::{NumericType, Schema, SchemaError, SchemaField, SchemaVariant};

#[test]
fn schema_constructors_build_expected_structures() {
    assert_eq!(Schema::bool(), Schema::Bool);
    assert_eq!(Schema::i8(), Schema::Numeric(NumericType::I8));
    assert_eq!(Schema::i16(), Schema::Numeric(NumericType::I16));
    assert_eq!(Schema::i32(), Schema::Numeric(NumericType::I32));
    assert_eq!(Schema::i64(), Schema::Numeric(NumericType::I64));
    assert_eq!(Schema::u8(), Schema::Numeric(NumericType::U8));
    assert_eq!(Schema::u16(), Schema::Numeric(NumericType::U16));
    assert_eq!(Schema::u32(), Schema::Numeric(NumericType::U32));
    assert_eq!(Schema::u64(), Schema::Numeric(NumericType::U64));
    assert_eq!(Schema::f32(), Schema::Numeric(NumericType::F32));
    assert_eq!(Schema::f64(), Schema::Numeric(NumericType::F64));

    assert_eq!(
        Schema::vector(NumericType::F32, 3).unwrap(),
        Schema::vec3_f32()
    );

    assert_eq!(
        Schema::fixed_array(Schema::identifier(), 0),
        Schema::FixedArray {
            element: Box::new(Schema::Identifier),
            length: 0,
        }
    );
}

#[test]
fn schema_validation_rejects_invalid_direct_construction() {
    let zero_vector = Schema::Vector {
        element: NumericType::F32,
        lanes: 0,
    };
    assert_eq!(zero_vector.validate(), Err(SchemaError::ZeroVectorLanes));

    let zero_rank = Schema::DenseArray {
        element: Box::new(Schema::f32()),
        rank: 0,
    };
    assert_eq!(zero_rank.validate(), Err(SchemaError::ZeroDenseArrayRank));

    let empty_variant = Schema::Variant(vec![]);
    assert_eq!(empty_variant.validate(), Err(SchemaError::EmptyVariant));

    let field = SchemaField::new("value", Schema::f32()).unwrap();
    let duplicate_fields = Schema::Struct(vec![field.clone(), field]);
    assert_eq!(
        duplicate_fields.validate(),
        Err(SchemaError::DuplicateFieldName("value".into()))
    );

    let variant = SchemaVariant::new("some", Schema::unit()).unwrap();
    let duplicate_variants = Schema::Variant(vec![variant.clone(), variant]);
    assert_eq!(
        duplicate_variants.validate(),
        Err(SchemaError::DuplicateVariantName("some".into()))
    );
}

#[test]
fn schema_validation_checks_nested_schemas() {
    let schema = Schema::array(Schema::Optional(Box::new(Schema::Vector {
        element: NumericType::F32,
        lanes: 0,
    })));

    assert_eq!(schema.validate(), Err(SchemaError::ZeroVectorLanes));
}

#[test]
fn schema_acceptance_is_strict() {
    let f32_field = Schema::dense_array(Schema::f32(), 2).unwrap();
    let another_f32_field = Schema::dense_array(Schema::f32(), 2).unwrap();
    let f64_field = Schema::dense_array(Schema::f64(), 2).unwrap();

    assert!(f32_field.accepts(&another_f32_field));
    assert!(!f32_field.accepts(&f64_field));
}

#[test]
fn schema_introspection_returns_nested_structures() {
    let schema = Schema::structure(vec![
        SchemaField::new("values", Schema::array(Schema::f32())).unwrap(),
        SchemaField::new("optional", Schema::optional(Schema::identifier())).unwrap(),
        SchemaField::new(
            "variant",
            Schema::variant(vec![
                SchemaVariant::new("one", Schema::u32()).unwrap(),
                SchemaVariant::new("two", Schema::string()).unwrap(),
            ])
            .unwrap(),
        )
        .unwrap(),
    ])
    .unwrap();

    assert_eq!(
        schema.field("values").and_then(Schema::element),
        Some(&Schema::f32())
    );
    assert_eq!(
        schema.field("optional").and_then(Schema::optional_value),
        Some(&Schema::identifier())
    );
    assert_eq!(
        schema
            .field("variant")
            .and_then(|schema| schema.variant_schema("two")),
        Some(&Schema::string())
    );
}
