// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

use chrono::UTC;
use chrono::NaiveDateTime;
use chrono::datetime::DateTime;
use error::RazberryError;
use rustc_serialize::json::Json;
use std::fmt;

/// A ZWave device.
pub struct Device {
  /// The string (integer?) ID of the device in Z Way.
  pub id: String,
  /// The user-defined name of the device; reported as "givenName".
  pub name: String,
  /// The last time the device was contacted by the Z Way controller.
  pub last_contacted: DateTime<UTC>,
}

impl Device {
  // TODO(MERGE-BLOCKER): TEST
  /// Construct a device from a JSON subset taken from the full device
  /// payload endpoint, '/ZWaveAPI/Data/'. (This is not the delta update
  /// endpoint!)
  pub fn initialize_from_json(device_id: &str, json: &Json)
      -> Result<Device, RazberryError> {
    let name = Device::get_string_property(json)?;
    let last_contacted = Device::get_last_contacted(json)?;

    let device = Device {
      id: device_id.to_string(),
      name: name.to_string(),
      last_contacted: last_contacted,
    };
    Ok(device)
  }

  /// Update the device from a JSON delta payload taken from the endpoint,
  /// '/ZWaveAPI/Data/{timestamp}'.
  pub fn process_updates(&self, json: &Json) -> Result<(), RazberryError> {
    Ok(())
  }

  /// Get a string property on the device.
  fn get_string_property(json: &Json) -> Result<&str, RazberryError> {
    json.find_path(&["data", "givenName", "value"])
        .and_then(|d| d.as_string())
        .ok_or(RazberryError::BadResponse)
  }

  fn get_last_contacted(json: &Json) -> Result<DateTime<UTC>, RazberryError> {
    let timestamp = json.find_path(&["data", "lastReceived", "updateTime"])
        .and_then(|d| d.as_i64())
        .ok_or(RazberryError::BadResponse)?;

    let dt = NaiveDateTime::from_timestamp(timestamp, 0);
    Ok(DateTime::from_utc(dt, UTC))
  }
}

impl fmt::Display for Device {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Device({}, {})", self.id, self.name)
  }
}
