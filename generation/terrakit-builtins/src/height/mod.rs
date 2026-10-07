//! Height-field terrain stages.

mod flat;
mod noise;
mod amplify;

pub use flat::FlatHeightStage;
pub use noise::{HeightNoiseMode, NoiseHeightStage};
pub use amplify::{HeightAmplifyMode, AmplifyHeightStage};
mod temperature;

pub use flat::FlatHeightStage;
pub use noise::{HeightNoiseMode, NoiseHeightStage};
pub use temperature::TemperatureMapStage;
