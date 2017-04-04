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

  /// Process the updates from the client.
  /// Should not be publicly used.
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


#[cfg(test)]
mod tests {
  use super::*;
  use chrono::UTC;

  #[test]
  fn test_initialize_from_json() {
    // A subset of the JSON for a binary sensor.
    let json = r#"
      {
        "name": "SensorBinary",
        "data": {
          "value": null,
          "1": {
            "value": null,
            "type": "empty",
            "sensorTypeString": {
              "value": "General purpose",
              "type": "string",
              "invalidateTime": 1456552384,
              "updateTime": 1456552385
            },
            "level": {
              "value": false,
              "type": "bool",
              "invalidateTime": 1456552384,
              "updateTime": 1465265727
            },
            "invalidateTime": 1456552384,
            "updateTime": 1465265727
          },
          "invalidateTime": 1456552382,
          "updateTime": 1456552383
        }
      }
    "#;

    let json = Json::from_str(json).unwrap();
    let sensor = SensorBinary::initialize_from_json(&json).unwrap();

    assert_eq!(false, sensor.get_level())
  }

  #[test]
  fn test_process_update() {
    let mut sensor = SensorBinary {
      level: true,
      level_updated: UTC::now(),
    };

    assert_eq!(true, sensor.get_level());

    let json = r#"
      {
        "value": null,
        "type": "empty",
        "sensorTypeString": {
          "value": "General purpose",
          "type": "string",
          "invalidateTime": 1487401953,
          "updateTime": 1487401954
        },
        "level": {
          "value": false,
          "type": "bool",
          "invalidateTime": 1489636200,
          "updateTime": 1491289442
        },
        "invalidateTime": 1487401953,
        "updateTime": 1491289442
      }
    "#;

    let json = Json::from_str(json).unwrap();

    let update = DeviceUpdate {
      path: vec!["instances", "0", "commandClasses", "48", "data", "1"],
      data: &json,
    };

    sensor.process_update(&update);

    assert_eq!(false, sensor.get_level());
  }
}
