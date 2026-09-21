// Scatter placement resources.
// These types describe objects that should be placed on generated terrain.
// They contain no engine-specific information such as Unity GameObjects or
// Unreal actors.

/// A single generated scatter placement.

/// A scatter point describes where an object should be placed, how it should
/// be rotated and scaled, and which object prototype should be used.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScatterPoint {
    /// World-space position of the object.
    pub position: [f32; 3],

    /// Object rotation represented as an XYZW quaternion.
    pub rotation: [f32; 4],

    /// Object scale represented as XYZ values.
    pub scale: [f32; 3],

    /// Identifier used to select the object prototype.

    /// The meaning of this ID is decided by the host engine or application.
    pub prototype_id: u32,
}

/// A collection of generated scatter placements.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScatterPoints {
    points: Vec<ScatterPoint>,
}

impl ScatterPoints {
    /// Creates an empty scatter-point collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates scatter points from an existing vector.
    pub fn from_vec(points: Vec<ScatterPoint>) -> Self {
        Self { points }
    }

    /// Returns all scatter points.
    pub fn points(&self) -> &[ScatterPoint] {
        &self.points
    }

    /// Returns the number of scatter points.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Returns true when there are no scatter points.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Consumes the collection and returns the underlying vector.
    pub fn into_vec(self) -> Vec<ScatterPoint> {
        self.points
    }
}