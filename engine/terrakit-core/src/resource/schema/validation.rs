use std::fmt;

use super::{Schema, SchemaField, SchemaVariant};

impl Schema {
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
                for field in fields {
                    if field.name().is_empty() {
                        return Err(SchemaError::EmptyFieldName);
                    }
                }

                for fields in fields.windows(2) {
                    match fields[0].name().cmp(fields[1].name()) {
                        std::cmp::Ordering::Equal => {
                            return Err(SchemaError::DuplicateFieldName(fields[0].name().into()));
                        }
                        std::cmp::Ordering::Greater => {
                            return Err(SchemaError::NonCanonicalFieldOrder);
                        }
                        std::cmp::Ordering::Less => {}
                    }
                }

                for field in fields {
                    field.schema().validate()?;
                }

                Ok(())
            }
            Self::Variant(variants) => {
                if variants.is_empty() {
                    return Err(SchemaError::EmptyVariant);
                }

                for variant in variants {
                    if variant.name().is_empty() {
                        return Err(SchemaError::EmptyVariantName);
                    }
                }

                for variants in variants.windows(2) {
                    match variants[0].name().cmp(variants[1].name()) {
                        std::cmp::Ordering::Equal => {
                            return Err(SchemaError::DuplicateVariantName(
                                variants[0].name().into(),
                            ));
                        }
                        std::cmp::Ordering::Greater => {
                            return Err(SchemaError::NonCanonicalVariantOrder);
                        }
                        std::cmp::Ordering::Less => {}
                    }
                }

                for variant in variants {
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
#[non_exhaustive]
pub enum SchemaError {
    /// Field name was empty.
    EmptyFieldName,

    /// Multiple fields used the same name.
    DuplicateFieldName(Box<str>),

    /// Struct fields were not ordered canonically by name.
    NonCanonicalFieldOrder,

    /// Variant name was empty.
    EmptyVariantName,

    /// Variant cases were not ordered canonically by name.
    NonCanonicalVariantOrder,

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

            Self::NonCanonicalFieldOrder => {
                write!(f, "schema struct fields are not in canonical name order")
            }

            Self::EmptyVariantName => {
                write!(f, "schema variant name cannot be empty")
            }

            Self::DuplicateVariantName(name) => {
                write!(f, "duplicate schema variant name '{name}'")
            }

            Self::NonCanonicalVariantOrder => {
                write!(f, "schema variants are not in canonical name order")
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
