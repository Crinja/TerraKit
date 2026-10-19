//! Height-field terrain stages.

mod amplify;
mod flat;
mod humidity;
mod noise;
mod temperature;

pub use amplify::{AmplifyHeightStage, HeightAmplifyMode};
pub use flat::FlatHeightStage;
pub use humidity::HumidityMapStage;
pub use noise::{HeightNoiseMode, NoiseHeightStage};
pub use temperature::TemperatureMapStage;
