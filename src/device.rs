// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

use chrono::UTC;
use chrono::datetime::DateTime;
use error::RazberryError;
use rustc_serialize::json::Json;

/// A ZWave device.
pub struct Device {
  /// The string (integer?) ID of the device in Z Way.
  id: String,
  /// The user-defined name of the device; reported as "givenName".
  name: String,
  /// The last time the device was contacted by the Z Way controller.
  last_contacted: DateTime<UTC>,
}

impl Device {
  /// Construct a device from JSON.
  pub fn from_json(device_id: &str, json: &Json)
      -> Result<Device, RazberryError> {
    let device = Device {
      id: device_id.to_string(),
      name: "".to_string(),
      last_contacted: UTC::now(),
    };
    Ok(device)
  }
}