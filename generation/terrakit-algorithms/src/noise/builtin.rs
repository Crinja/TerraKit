//! Built-in deterministic noise algorithm selectors.

use terrakit_core::GenerationSeed;

use super::{Noise2D, Noise3D, ValueNoise2D, ValueNoise3D};

/// First-party 2D noise implementations shipped by TerraKit.
///
/// This enum is for built-in deterministic algorithms only. It does not model
/// external plugins or dynamic algorithm loading.
#[derive(Debug, Clone)]
pub enum BuiltinNoise2D {
    /// Coherent value noise.
    Value(ValueNoise2D),
}

impl Noise2D for BuiltinNoise2D {
    fn sample(&self, x: f64, y: f64) -> f32 {
        match self {
            Self::Value(noise) => noise.sample(x, y),
        }
    }
}

/// First-party 3D noise implementations shipped by TerraKit.
///
/// This enum is for built-in deterministic algorithms only. It does not model
/// external plugins or dynamic algorithm loading.
#[derive(Debug, Clone)]
pub enum BuiltinNoise3D {
    /// Coherent value noise.
    Value(ValueNoise3D),
}

impl Noise3D for BuiltinNoise3D {
    fn sample(&self, x: f64, y: f64, z: f64) -> f32 {
        match self {
            Self::Value(noise) => noise.sample(x, y, z),
        }
    }
}

/// Identifier for first-party noise algorithms available in TerraKit.
///
/// This enum is configuration-friendly and intentionally covers only built-in
/// algorithms. External plugins should use their own identifiers outside this
/// crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoiseAlgorithm {
    /// Coherent value noise.
    Value,
}

impl NoiseAlgorithm {
    /// Creates a 2D built-in noise sampler using the supplied deterministic
    /// generation seed.
    pub fn create_2d(self, seed: GenerationSeed) -> BuiltinNoise2D {
        match self {
            Self::Value => BuiltinNoise2D::Value(ValueNoise2D::new(seed)),
        }
    }

    /// Creates a 3D built-in noise sampler using the supplied deterministic
    /// generation seed.
    pub fn create_3d(self, seed: GenerationSeed) -> BuiltinNoise3D {
        match self {
            Self::Value => BuiltinNoise3D::Value(ValueNoise3D::new(seed)),
        }
    }
}
