use std::fmt;

use super::Schema;

/// One navigation step within a structural schema.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SchemaPathSegment {
    /// Select a named field from a struct.
    Field(Box<str>),

    /// Select the element schema of an array-like structure.
    Element,

    /// Select the contained schema of an optional value.
    OptionalValue,

    /// Select one named case from a variant.
    Variant(Box<str>),
}

/// Stable path from a resource root to a nested structural view.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemaPath {
    segments: Vec<SchemaPathSegment>,
}

impl SchemaPath {
    /// Creates the root path.
    pub const fn root() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Returns whether this path selects the resource root.
    pub fn is_root(&self) -> bool {
        self.segments.is_empty()
    }

    /// Returns the immediate parent path.
    pub fn parent(&self) -> Option<Self> {
        if self.segments.is_empty() {
            return None;
        }

        let mut segments = self.segments.clone();
        segments.pop();

        Some(Self { segments })
    }

    /// Creates a path selecting one root field.
    pub fn field(name: impl Into<Box<str>>) -> Self {
        Self {
            segments: vec![SchemaPathSegment::Field(name.into())],
        }
    }

    /// Appends another field selection.
    pub fn then_field(mut self, name: impl Into<Box<str>>) -> Self {
        self.segments.push(SchemaPathSegment::Field(name.into()));
        self
    }

    /// Appends an element selection.
    pub fn then_element(mut self) -> Self {
        self.segments.push(SchemaPathSegment::Element);
        self
    }

    /// Appends an optional value selection.
    pub fn then_optional_value(mut self) -> Self {
        self.segments.push(SchemaPathSegment::OptionalValue);
        self
    }

    /// Appends a variant selection.
    pub fn then_variant(mut self, name: impl Into<Box<str>>) -> Self {
        self.segments.push(SchemaPathSegment::Variant(name.into()));
        self
    }

    /// Returns the path segments.
    pub fn segments(&self) -> &[SchemaPathSegment] {
        &self.segments
    }

    /// Resolves this path against a root schema.
    pub fn resolve<'a>(&self, root: &'a Schema) -> Result<&'a Schema, ViewError> {
        let mut current = root;

        for segment in &self.segments {
            match segment {
                SchemaPathSegment::Field(name) => {
                    if !matches!(current, Schema::Struct(_)) {
                        return Err(ViewError::FieldOnNonStruct);
                    }

                    current = current
                        .field(name)
                        .ok_or_else(|| ViewError::UnknownField(name.clone()))?;
                }
                SchemaPathSegment::Element => {
                    current = current.element().ok_or(ViewError::ElementOnNonArray)?;
                }
                SchemaPathSegment::OptionalValue => {
                    current = current
                        .optional_value()
                        .ok_or(ViewError::OptionalValueOnNonOptional)?;
                }
                SchemaPathSegment::Variant(name) => {
                    if !matches!(current, Schema::Variant(_)) {
                        return Err(ViewError::VariantOnNonVariant);
                    }

                    current = current
                        .variant_schema(name)
                        .ok_or_else(|| ViewError::UnknownVariant(name.clone()))?;
                }
            }
        }

        Ok(current)
    }
}

/// Which logical part of a resource backs a capability.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ResourceView {
    /// The complete resource schema satisfies the capability.
    Root,
    /// A nested path satisfies the capability.
    Path(SchemaPath),
}

impl ResourceView {
    /// Resolves the view to its actual schema.
    pub fn resolve<'a>(&self, root: &'a Schema) -> Result<&'a Schema, ViewError> {
        match self {
            Self::Root => Ok(root),
            Self::Path(path) => path.resolve(root),
        }
    }

    /// Returns the canonical form of this resource view.
    pub fn canonical(&self) -> Self {
        match self {
            Self::Root => Self::Root,
            Self::Path(path) if path.is_root() => Self::Root,
            Self::Path(path) => Self::Path(path.clone()),
        }
    }

    /// Returns the immediate parent resource view.
    pub fn parent(&self) -> Option<Self> {
        match self.canonical() {
            Self::Root => None,
            Self::Path(path) => {
                let parent = path.parent()?;

                if parent.is_root() {
                    Some(Self::Root)
                } else {
                    Some(Self::Path(parent))
                }
            }
        }
    }
}

/// Error resolving a resource view.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ViewError {
    /// A field path referenced a field not present at that point in the schema.
    UnknownField(Box<str>),
    /// A field path was used on a schema that was not a struct.
    FieldOnNonStruct,
    /// An element path was used on a schema that was not array-like.
    ElementOnNonArray,
    /// An optional value path was used on a schema that was not optional.
    OptionalValueOnNonOptional,
    /// A variant path referenced a case not present at that point in the schema.
    UnknownVariant(Box<str>),
    /// A variant path was used on a schema that was not a variant.
    VariantOnNonVariant,
}

impl fmt::Display for ViewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownField(field) => {
                write!(f, "resource view references unknown field '{field}'")
            }
            Self::FieldOnNonStruct => write!(f, "resource view field selection requires a struct"),
            Self::ElementOnNonArray => write!(
                f,
                "resource view element selection requires an array-like schema"
            ),
            Self::OptionalValueOnNonOptional => write!(
                f,
                "resource view optional value selection requires an optional schema"
            ),
            Self::UnknownVariant(variant) => {
                write!(f, "resource view references unknown variant '{variant}'")
            }
            Self::VariantOnNonVariant => write!(
                f,
                "resource view variant selection requires a variant schema"
            ),
        }
    }
}

impl std::error::Error for ViewError {}
