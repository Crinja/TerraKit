//! Height-field terrain stages.

mod flat;
mod noise;

pub use flat::FlatHeightStage;
pub use noise::{HeightNoiseMode, NoiseHeightStage};
