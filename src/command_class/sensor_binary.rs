// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

use error::RazberryError;
use rustc_serialize::json::Json;

/**
 * Represents a sensor with a binary state.
 */
pub struct SensorBinary {
  // TODO: Keep more data, such as sensor update time, previous value, etc.
  level: bool,
}

impl SensorBinary {
  /// Construct a SensorBinary command class.
  pub fn initialize_from_json(json: &Json)
      -> Result<SensorBinary, RazberryError> {

    let sensor = SensorBinary {
      level: true,
    };

    Ok(sensor)
  }

  /// Get the sensor's state.
  pub fn get_level(&self) -> bool {
    self.level
  }

  pub fn process_updates(&self, json: &Json) -> Result<(), RazberryError> {
    Ok(())
  }
}
