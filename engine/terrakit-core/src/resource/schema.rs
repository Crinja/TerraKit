//! Structural schema language
//!
//! This should be kept intentionally small. New domain concepts should be
//! implemented as compositions of existing primatives
//!
//! If you are modifying this, ask yourself whether you genuinely need a
//! new primative or are leaking domain concepts

use std::collections::HashSet;
use std::fmt;

/// Numeric primitives supported by the schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumericType {
    /// Signed 8-bit integer.
    I8,
    /// Signed 16-bit integer.
    I16,
    /// Signed 32-bit integer.
    I32,
    /// Signed 64-bit integer.
    I64,
    /// Unsigned 8-bit integer.
    U8,
    /// Unsigned 16-bit integer.
    U16,
    /// Unsigned 32-bit integer.
    U32,
    /// Unsigned 64-bit integer.
    U64,
    /// 32-bit float.
    F32,
    /// 64-bit float.
    F64,
}

/// Named field in a structural schema.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaField {
    name: Box<str>,
    schema: Schema,
}

impl SchemaField {
    /// Create a named field.
    pub fn new(name: impl Into<Box<str>>, schema: Schema) -> Result<Self, SchemaError> {
        let name = name.into();

        if name.trim().is_empty() {
            return Err(SchemaError::EmptyFieldName);
        }

        Ok(Self { name, schema })
    }

    /// Return the field name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the schema structure.
    pub fn schema(&self) -> &Schema {
        &self.schema
    }
}

/// Named case in a variant schema.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaVariant {
    name: Box<str>,
    schema: Schema,
}

impl SchemaVariant {
    /// Create a named variant.
    pub fn new(name: impl Into<Box<str>>, schema: Schema) -> Result<Self, SchemaError> {
        let name = name.into();

        if name.trim().is_empty() {
            return Err(SchemaError::EmptyVariantName);
        }

        Ok(Self { name, schema })
    }

    /// Return the variant name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the variant schema.
    pub fn schema(&self) -> &Schema {
        &self.schema
    }
}

/// Logical structure of data.
///
/// This is **not** a runtime value and does not describe physical memory layout.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Schema {
    /// Boolean.
    Bool,

    /// Numeric.
    Numeric(NumericType),

    /// UTF-8 text.
    String,

    /// Stable symbolic identifier (prototype ID, asset ID, tag, etc.).
    Identifier,

    /// Fixed-lane numeric vector. `lanes` must be non-zero.
    Vector {
        /// Numeric type.
        element: NumericType,

        /// Number of components.
        lanes: u8,
    },

    /// Sequence of zero or more elements.
    Array(Box<Schema>),

    /// Fixed-size sequence of elements.
    FixedArray {
        /// Element type.
        element: Box<Schema>,

        /// Number of elements.
        length: u32,
    },

    /// N-dimensional dense array of zero or more elements.
    DenseArray {
        /// Element type.
        element: Box<Schema>,

        /// Number of dimensions.
        rank: u8,
    },

    /// Optional value of one schema.
    Optional(Box<Schema>),

    /// Named heterogeneous fields.
    Struct(Vec<SchemaField>),

    /// Value containing one of multiple possible schemas.
    Variant(Vec<SchemaVariant>),
}

impl Schema {
    /// Construct a boolean schema.
    pub const fn bool() -> Self {
        Self::Bool
    }

    /// Construct a signed 8-bit integer schema.
    pub const fn i8() -> Self {
        Self::Numeric(NumericType::I8)
    }

    /// Construct a signed 16-bit integer schema.
    pub const fn i16() -> Self {
        Self::Numeric(NumericType::I16)
    }

    /// Construct a signed 32-bit integer schema.
    pub const fn i32() -> Self {
        Self::Numeric(NumericType::I32)
    }

    /// Construct a signed 64-bit integer schema.
    pub const fn i64() -> Self {
        Self::Numeric(NumericType::I64)
    }

    /// Construct an unsigned 8-bit integer schema.
    pub const fn u8() -> Self {
        Self::Numeric(NumericType::U8)
    }

    /// Construct an unsigned 16-bit integer schema.
    pub const fn u16() -> Self {
        Self::Numeric(NumericType::U16)
    }

    /// Construct an unsigned 32-bit integer schema.
    pub const fn u32() -> Self {
        Self::Numeric(NumericType::U32)
    }

    /// Construct an unsigned 64-bit integer schema.
    pub const fn u64() -> Self {
        Self::Numeric(NumericType::U64)
    }

    /// Construct a 32-bit floating-point schema.
    pub const fn f32() -> Self {
        Self::Numeric(NumericType::F32)
    }

    /// Construct a 64-bit floating-point schema.
    pub const fn f64() -> Self {
        Self::Numeric(NumericType::F64)
    }

    /// Construct a numeric schema from an explicit numeric type.
    pub const fn numeric(numeric_type: NumericType) -> Self {
        Self::Numeric(numeric_type)
    }

    /// Construct a UTF-8 string schema.
    pub const fn string() -> Self {
        Self::String
    }

    /// Construct a symbolic identifier schema.
    pub const fn identifier() -> Self {
        Self::Identifier
    }

    /// Construct a fixed-lane numeric vector.
    ///
    /// `lanes` describes the number of numeric components and must be non-zero.
    pub fn vector(element: NumericType, lanes: u8) -> Result<Self, SchemaError> {
        if lanes == 0 {
            return Err(SchemaError::ZeroVectorLanes);
        }

        Ok(Self::Vector { element, lanes })
    }

    /// Construct a two-component `f32` vector.
    pub const fn vec2_f32() -> Self {
        Self::Vector {
            element: NumericType::F32,
            lanes: 2,
        }
    }

    /// Construct a three-component `f32` vector.
    pub const fn vec3_f32() -> Self {
        Self::Vector {
            element: NumericType::F32,
            lanes: 3,
        }
    }

    /// Construct a four-component `f32` vector.
    pub const fn vec4_f32() -> Self {
        Self::Vector {
            element: NumericType::F32,
            lanes: 4,
        }
    }

    /// Construct a variable-length sequence of values.
    pub fn array(element: Schema) -> Self {
        Self::Array(Box::new(element))
    }

    /// Construct a fixed-size sequence of values.
    pub fn fixed_array(element: Schema, length: u32) -> Self {
        Self::FixedArray {
            element: Box::new(element),
            length,
        }
    }

    /// Construct an N-dimensional dense array.
    pub fn dense_array(element: Schema, rank: u8) -> Result<Self, SchemaError> {
        if rank == 0 {
            return Err(SchemaError::ZeroDenseArrayRank);
        }

        Ok(Self::DenseArray {
            element: Box::new(element),
            rank,
        })
    }

    /// Construct an optional value.
    pub fn optional(schema: Schema) -> Self {
        Self::Optional(Box::new(schema))
    }

    /// Construct a heterogeneous structure from named fields.
    ///
    /// Field names must be unique within the structure.
    pub fn structure(fields: Vec<SchemaField>) -> Result<Self, SchemaError> {
        let mut names = HashSet::with_capacity(fields.len());

        for field in &fields {
            if !names.insert(field.name()) {
                return Err(SchemaError::DuplicateFieldName(
                    field.name().into(),
                ));
            }
        }

        Ok(Self::Struct(fields))
    }

    /// Construct a value containing one of multiple possible schemas.
    ///
    /// Variant names must be unique.
    pub fn variant(variants: Vec<SchemaVariant>) -> Result<Self, SchemaError> {
        if variants.is_empty() {
            return Err(SchemaError::EmptyVariant);
        }

        let mut names = HashSet::with_capacity(variants.len());

        for variant in &variants {
            if !names.insert(variant.name()) {
                return Err(SchemaError::DuplicateVariantName(
                    variant.name().into(),
                ));
            }
        }

        Ok(Self::Variant(variants))
    }

    /// Construct an empty structure.
    ///
    /// Useful for variants which do not contain additional data.
    pub fn unit() -> Self {
        Self::Struct(Vec::new())
    }

    /// Validate this schema and all nested schemas.
    pub fn validate(&self) -> Result<(), SchemaError> {
        match self {
            Self::Bool | Self::Numeric(_) | Self::String | Self::Identifier => Ok(()),
            Self::Vector { lanes, .. } => {
                if *lanes == 0 {
                    return Err(SchemaError::ZeroVectorLanes);
                }

                Ok(())
            }
            Self::Array(element) | Self::Optional(element) => element.validate(),
            Self::FixedArray { element, .. } => element.validate(),
            Self::DenseArray { element, rank } => {
                if *rank == 0 {
                    return Err(SchemaError::ZeroDenseArrayRank);
                }

                element.validate()
            }
            Self::Struct(fields) => {
                let mut names = HashSet::with_capacity(fields.len());

                for field in fields {
                    if field.name().trim().is_empty() {
                        return Err(SchemaError::EmptyFieldName);
                    }

                    if !names.insert(field.name()) {
                        return Err(SchemaError::DuplicateFieldName(field.name().into()));
                    }

                    field.schema().validate()?;
                }

                Ok(())
            }
            Self::Variant(variants) => {
                if variants.is_empty() {
                    return Err(SchemaError::EmptyVariant);
                }

                let mut names = HashSet::with_capacity(variants.len());

                for variant in variants {
                    if variant.name().trim().is_empty() {
                        return Err(SchemaError::EmptyVariantName);
                    }

                    if !names.insert(variant.name()) {
                        return Err(SchemaError::DuplicateVariantName(variant.name().into()));
                    }

                    variant.schema().validate()?;
                }

                Ok(())
            }
        }
    }

    /// Return whether this schema accepts another schema.
    pub fn accepts(&self, other: &Schema) -> bool {
        self == other
    }

    /// Return one named struct field schema.
    pub fn field(&self, name: &str) -> Option<&Schema> {
        match self {
            Self::Struct(fields) => fields
                .iter()
                .find(|field| field.name() == name)
                .map(SchemaField::schema),
            _ => None,
        }
    }

    /// Return the element schema of an array-like structure.
    pub fn element(&self) -> Option<&Schema> {
        match self {
            Self::Array(element)
            | Self::FixedArray { element, .. }
            | Self::DenseArray { element, .. } => Some(element),
            _ => None,
        }
    }

    /// Return the contained schema of an optional value.
    pub fn optional_value(&self) -> Option<&Schema> {
        match self {
            Self::Optional(schema) => Some(schema),
            _ => None,
        }
    }

    /// Return one named variant schema.
    pub fn variant_schema(&self, name: &str) -> Option<&Schema> {
        match self {
            Self::Variant(variants) => variants
                .iter()
                .find(|variant| variant.name() == name)
                .map(SchemaVariant::schema),
            _ => None,
        }
    }
}

/// Errors produced when constructing an invalid schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaError {
    /// Field name was empty.
    EmptyFieldName,

    /// Multiple fields used the same name.
    DuplicateFieldName(Box<str>),

    /// Variant name was empty.
    EmptyVariantName,

    /// Multiple variants used the same name.
    DuplicateVariantName(Box<str>),

    /// Variant contained no possible values.
    EmptyVariant,

    /// Vector contained zero numeric components.
    ZeroVectorLanes,

    /// Dense array contained zero dimensions.
    ZeroDenseArrayRank,
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFieldName => {
                write!(f, "schema field name cannot be empty")
            }

            Self::DuplicateFieldName(name) => {
                write!(f, "duplicate schema field name '{name}'")
            }

            Self::EmptyVariantName => {
                write!(f, "schema variant name cannot be empty")
            }

            Self::DuplicateVariantName(name) => {
                write!(f, "duplicate schema variant name '{name}'")
            }

            Self::EmptyVariant => {
                write!(f, "schema variant must contain at least one case")
            }

            Self::ZeroVectorLanes => {
                write!(f, "vector lane count must be greater than zero")
            }

            Self::ZeroDenseArrayRank => {
                write!(f, "dense array rank must be greater than zero")
            }
        }
    }
}

impl std::error::Error for SchemaError {}
