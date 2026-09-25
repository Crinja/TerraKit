//! Identifers used by resource model/
//!
//! - ['ResourceId'] identifies a runtime value
//! - ['ResourceTypeId'] / ['ResourceCapabilityId'] / ['MetadataKeyId'] idenifies versioned contracts
//!
//! Contract IDs use lowercase dot-separated names and a canonical decimal major version.

use std::fmt;

/// Identity for one runtime resource value.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId(pub u64);

/// Versioned identifier for one complete resource representation.
///
/// For example 'terrakit.erosion-flow-result@1'
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceTypeId(Box<str>);

/// Versioned identifier for one reusable resource capability contract.
///
// For example `terrakit.erosion-flow@1`.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceCapabilityId(Box<str>);

/// Versioned identifier for one metadata key.
///
/// For example `terrakit.coordinate-space@1`.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MetadataKeyId(Box<str>);

impl ResourceTypeId {
    /// Creates a resource type ID.
    pub fn new(value: impl Into<Box<str>>) -> Result<Self, IdError> {
        Ok(Self(validate_versioned_id(value.into())?))
    }

    /// Returns the original canonical spelling.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ResourceCapabilityId {
    /// Creates a capability ID.
    pub fn new(value: impl Into<Box<str>>) -> Result<Self, IdError> {
        Ok(Self(validate_versioned_id(value.into())?))
    }

    /// Returns the original canonical spelling.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl MetadataKeyId {
    /// Creates a metadata key ID.
    pub fn new(value: impl Into<Box<str>>) -> Result<Self, IdError> {
        Ok(Self(validate_versioned_id(value.into())?))
    }

    /// Returns the original canonical spelling.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn validate_versioned_id(value: Box<str>) -> Result<Box<str>, IdError> {
    let text = value.as_ref();

    if text.trim() != text {
        return Err(IdError::SurroundingWhitespace);
    }

    let Some((name, major)) = text.rsplit_once('@') else {
        return Err(IdError::MissingMajorVersion);
    };

    if name.contains('@') {
        return Err(IdError::InvalidQualifiedName);
    }

    let mut segments = name.split('.');
    let Some(first) = segments.next() else {
        return Err(IdError::MissingNamespace);
    };
    let Some(second) = segments.next() else {
        return Err(IdError::MissingNamespace);
    };

    if !valid_name_segment(first)
        || !valid_name_segment(second)
        || segments.any(|segment| !valid_name_segment(segment))
    {
        return Err(IdError::InvalidQualifiedName);
    }

    if major.is_empty() || !major.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(IdError::InvalidMajorVersion);
    }

    if major.len() > 1 && major.starts_with('0') {
        return Err(IdError::NonCanonicalMajorVersion);
    }

    if major.parse::<u32>().is_err() {
        return Err(IdError::InvalidMajorVersion);
    }

    Ok(value)
}

fn valid_name_segment(segment: &str) -> bool {
    let mut bytes = segment.bytes();

    matches!(bytes.next(), Some(b'a'..=b'z'))
        && bytes.all(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'-'))
}

/// Identifier validation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum IdError {
    /// No `@major` suffix was provided.
    MissingMajorVersion,
    /// The qualified name had no namespace separator.
    MissingNamespace,
    /// The qualified name was not canonical lowercase ASCII.
    InvalidQualifiedName,
    /// The major version was not an unsigned integer.
    InvalidMajorVersion,
    /// The major version contained leading zeroes.
    NonCanonicalMajorVersion,
    /// IDs are canonical strings and may not contain outer whitespace.
    SurroundingWhitespace,
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingMajorVersion => "ID must end in @<major>",
            Self::MissingNamespace => "ID must include a namespace, e.g. terrakit.name@1",
            Self::InvalidQualifiedName => {
                "ID name must use lowercase ASCII segments separated by dots"
            }
            Self::InvalidMajorVersion => "ID major version must be an unsigned integer",
            Self::NonCanonicalMajorVersion => "ID major version must not contain leading zeroes",
            Self::SurroundingWhitespace => "ID must not contain surrounding whitespace",
        };
        f.write_str(message)
    }
}

impl std::error::Error for IdError {}

impl fmt::Debug for ResourceTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ResourceTypeId")
            .field(&self.as_str())
            .finish()
    }
}

impl fmt::Debug for ResourceCapabilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ResourceCapabilityId")
            .field(&self.as_str())
            .finish()
    }
}

impl fmt::Debug for MetadataKeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MetadataKeyId")
            .field(&self.as_str())
            .finish()
    }
}

impl fmt::Display for ResourceTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for ResourceCapabilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for MetadataKeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
