// Common types used by TerraKit object scattering.

// Configuration shared by scattering systems.
#[derive(Debug, Clone, Copy)]
pub struct ScatterSettings {
    pub seed: u64, // Random seed.
    pub prototype_id: u32, // Object prototype ID.
    pub min_scale: f32, // Minimum object scale.
    pub max_scale: f32, // Maximum object scale.
    pub random_rotation: bool, // Whether objects receive a random Y-axis rotation.
}

impl Default for ScatterSettings {
    fn default() -> Self {
        Self {
            seed: 0,
            prototype_id: 0,
            min_scale: 1.0,
            max_scale: 1.0,
            random_rotation: true,
        }
    }
}

impl ScatterSettings {
    // Validates the scattering settings.
    pub fn validate(&self) -> Result<(), ScatterError> {
        if !self.min_scale.is_finite() {
            return Err(ScatterError::InvalidSettings(
                "min_scale must be finite",
            ));
        }

        if !self.max_scale.is_finite() {
            return Err(ScatterError::InvalidSettings(
                "max_scale must be finite",
            ));
        }

        if self.min_scale < 0.0 {
            return Err(ScatterError::InvalidSettings(
                "min_scale cannot be negative",
            ));
        }

        if self.max_scale < self.min_scale {
            return Err(ScatterError::InvalidSettings(
                "max_scale must be greater than or equal to min_scale",
            ));
        }

        Ok(())
    }
}

// Defines a rectangular scattering area.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScatterArea {
    /// Min Max X coordinate.
    pub min_x: f32,
    pub max_x: f32,
    /// Min Max Z coordinate.
    pub min_z: f32,
    pub max_z: f32,
}

impl ScatterArea {
    // Creates a new rectangular area.
    pub fn new(min_x: f32, max_x: f32, min_z: f32, max_z: f32) -> Self {
        Self {
            min_x,
            max_x,
            min_z,
            max_z,
        }
    }

    // Returns the width of the area.
    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    // Returns the depth of the area.
    pub fn depth(&self) -> f32 {
        self.max_z - self.min_z
    }

    // Returns the area in square world units.
    pub fn area(&self) -> f32 {
        self.width().abs() * self.depth().abs()
    }

    // Validates the area.
    pub fn validate(&self) -> Result<(), ScatterError> {
        if !self.min_x.is_finite()
            || !self.max_x.is_finite()
            || !self.min_z.is_finite()
            || !self.max_z.is_finite()
        {
            return Err(ScatterError::InvalidArea);
        }

        if self.max_x <= self.min_x {
            return Err(ScatterError::InvalidArea);
        }

        if self.max_z <= self.min_z {
            return Err(ScatterError::InvalidArea);
        }

        Ok(())
    }

    // Converts a normalized coordinate into world space.
    pub fn point_from_normalized(&self, x: f32, z: f32) -> [f32; 3] {
        [
            self.min_x + x * self.width(),
            0.0,
            self.min_z + z * self.depth(),
        ]
    }
}

// Height information for a terrain location.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerrainSample {
    pub position: [f32; 3], // World-space position.
    pub normal: [f32; 3], // Surface normal.
}

impl TerrainSample {
    // Creates a terrain sample.
    pub fn new(position: [f32; 3], normal: [f32; 3]) -> Self {
        Self { position, normal }
    }

    // Calculates the surface slope in degrees.
    //
    // A flat surface has a slope of 0 degrees.
    // A vertical surface has a slope of 90 degrees.
    pub fn slope_degrees(&self) -> f32 {
        let normal_length =
            (self.normal[0] * self.normal[0]
                + self.normal[1] * self.normal[1]
                + self.normal[2] * self.normal[2])
                .sqrt();

        if normal_length <= f32::EPSILON {
            return 90.0;
        }

        let y = (self.normal[1] / normal_length).clamp(-1.0, 1.0);

        y.acos().to_degrees()
    }
}

// Errors that can occur during scattering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScatterError {
    InvalidSettings(&'static str), // Invalid scattering settings.
    InvalidArea, // Invalid scattering area.
    InvalidRadius, // Invalid Poisson radius.
    InvalidAttempts, // Invalid number of attempts.
    InvalidMask, // Invalid mask configuration.
}