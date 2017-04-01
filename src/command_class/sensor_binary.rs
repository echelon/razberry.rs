// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

use chrono::NaiveDateTime;
use chrono::UTC;
use chrono::datetime::DateTime;
use device_update::DeviceUpdate;
use error::RazberryError;
use rustc_serialize::json::Json;
use std::fmt;

/**
 * Represents a sensor with a binary state.
 */
#[derive(Debug)]
pub struct SensorBinary {
  // TODO: Keep more data, such as sensor update time, previous value, etc.
  level: bool,
  level_updated: DateTime<UTC>,
}

impl SensorBinary {
  // TODO(MERGE-BLOCKER): Test.
  /// Construct a SensorBinary command class.
  pub fn initialize_from_json(json: &Json)
      -> Result<SensorBinary, RazberryError> {
    // TODO: Multiple instances within the command class.
    // Not sure what the various hardware support for these are. Perhaps they're
    // predefined indices, given that Aeotec seems to use consistent "IDs".
    // FIXME: Also, I'm not sure where Rust's json parser is getting "data" in
    // the path from. It doesn't even look like it's a key at this level!
    let level = json.find_path(&["data", "1", "level", "value"])
        .and_then(|j| j.as_boolean())
        .ok_or(RazberryError::BadResponse)?;

    let timestamp = json.find_path(&["data", "1", "level", "updateTime"])
        .and_then(|j| j.as_i64())
        .ok_or(RazberryError::BadResponse)?;

    let dt = NaiveDateTime::from_timestamp(timestamp, 0);
    let utc = DateTime::from_utc(dt, UTC);

    let sensor = SensorBinary {
      level: level,
      level_updated: utc,
    };

    Ok(sensor)
  }

  /// Get the sensor's state.
  pub fn get_level(&self) -> bool {
    self.level
  }

  pub fn process_update(&mut self, update: &DeviceUpdate)
      -> Result<(), RazberryError> {
    if update.path.get(4) != Some(&"data") {
      return Ok(()); // Irrelevant update.
    }

    let level = update.data.find_path(&["level", "value"])
        .and_then(|j| j.as_boolean())
        .ok_or(RazberryError::BadResponse)?;

    let timestamp = update.data.find_path(&["level", "updateTime"])
        .and_then(|j| j.as_i64())
        .ok_or(RazberryError::BadResponse)?;

    let dt = NaiveDateTime::from_timestamp(timestamp, 0);

    self.level = level;
    self.level_updated = DateTime::from_utc(dt, UTC);

    Ok(())
  }
}

impl fmt::Display for SensorBinary {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "SensorBinary(level: {}, updated:{})",
      self.level, self.level_updated)
  }
}
