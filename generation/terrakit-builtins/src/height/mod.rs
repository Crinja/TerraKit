//! Height-field terrain stages.

mod amplify;
mod flat;
mod noise;
mod temperature;

pub use flat::FlatHeightStage;
pub use noise::{HeightNoiseMode, NoiseHeightStage};
pub use amplify::{HeightAmplifyMode, AmplifyHeightStage};
pub use temperature::TemperatureMapStage;
