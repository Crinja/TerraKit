//! Height-field terrain stages.

mod flat;
mod noise;
mod temperature;

pub use flat::FlatHeightStage;
pub use noise::{HeightNoiseMode, NoiseHeightStage};
pub use temperature::TemperatureMapStage;
