// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

use chrono::NaiveDateTime;
use chrono::UTC;
use chrono::datetime::DateTime;
use command_classes::CommandClasses;
use error::RazberryError;
use rustc_serialize::json::Json;
use std::collections::HashMap;
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

    // TODO: Multiple device instances.
    // Multiple instances are probably pretty rare, and muddy up the API a bit.
    let cc_json = json.find_path(&["instances", "0", "commandClasses"])
        .and_then(|c| c.as_object())
        .ok_or(RazberryError::BadResponse)?;

    //let mut command_classes = HashMap::new();

    // TODO: Multiple command class instances.
    // Multiple instances here are probably more common, but I want to get a
    // simple working implementation first. The API is subject to change when
    // support is added.
    for (command_class_id, command_class) in cc_json {
      println!("Found command class ID: {}", command_class_id);
      let cc = CommandClasses::from_str(command_class_id);
      println!("Found command class: {:?}", cc);
      //let device = Device::initialize_from_json(device_id, &device_json)?;
      //devices.push(device);
    }

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
