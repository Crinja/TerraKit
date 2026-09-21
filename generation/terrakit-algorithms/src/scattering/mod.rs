// TerraKit object scattering algorithms.

// Scattering is responsible for generating engine-independent object placement data.

// The scattering system does NOT create Unity GameObjects, Unreal Actors, Godot Nodes, etc.

// Instead it produces `ScatterPoint` values that can be consumed by an engine integration layer.

mod masks;
mod poisson;
mod random;
mod types;

pub use masks::*;
pub use poisson::*;
pub use random::*;
pub use types::*;