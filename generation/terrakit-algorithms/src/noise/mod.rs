//! Noise traits and first-party deterministic noise algorithms.
//!
//! Noise implementations sample algorithm-space coordinates, support negative
//! coordinates where applicable, and normally return finite values in roughly
//! `-1.0..=1.0`. Built-in selection enums represent only algorithms shipped by
//! TerraKit; external plugins are intentionally not modeled here.

mod builtin;
mod fractal;
mod traits;
mod value;

pub use builtin::*;
pub use fractal::*;
pub use traits::*;
pub use value::*;
