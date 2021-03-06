// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

use chrono::NaiveDateTime;
use chrono::UTC;
use chrono::datetime::DateTime;
use command_class::CommandClass;
use command_classes::CommandClasses;
use device_update::DeviceUpdate;
use error::RazberryError;
use rustc_serialize::json::Json;
use std::collections::HashMap;
use std::fmt;

/**
 * A Z-Wave Device.
 */
pub struct Device {
  // TODO: Change all the visibilities, and hide behind locks (interior mut)
  /// The string (integer?) ID of the device in Z Way.
  pub id: String,

  /// The user-defined name of the device; reported as "givenName".
  pub name: String,

  /// The last time the device was contacted by the Z Way controller.
  /// This is the value of "data.lastReceived.updateTime"
  pub last_contacted: DateTime<UTC>,

  /// Command classes associated with the device.
  pub command_classes: HashMap<CommandClasses, CommandClass>,
}

impl Device {
  // TODO: Test.
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

    let mut command_classes = HashMap::new();

    // TODO: Multiple command class instances.
    // Multiple instances here are probably more common, but I want to get a
    // simple working implementation first. The API is subject to change when
    // support is added.
    for (command_class_id, command_class_json) in cc_json {
      let command_class = match CommandClasses::from_str(command_class_id) {
        None => continue, // Unrecognized command class.
        Some(cc) => cc,
      };

      let cc_instance = CommandClass::initialize_from_json(command_class,
          command_class_json)?;

      match cc_instance {
        CommandClass::Unsupported => continue, // No support for this type yet.
        _ => {},
      }

      command_classes.insert(command_class, cc_instance);
    }

    let device = Device {
      id: device_id.to_string(),
      name: name.to_string(),
      last_contacted: last_contacted,
      command_classes: command_classes,
    };
    Ok(device)
  }

  /// Update the device from a JSON delta payload taken from the endpoint,
  /// '/ZWaveAPI/Data/{timestamp}'.
  pub fn process_updates(&mut self, updates: Vec<DeviceUpdate>)
      -> Result<(), RazberryError> {
    for update in updates {
      match update.path.get(0) {
        Some(&"data") => {
          // Device meta updates.
          if update.path.get(1) == Some(&"lastReceived") {
            let timestamp = update.data.find("updateTime")
                .and_then(|d| d.as_i64())
                .ok_or(RazberryError::BadResponse)?;
            let dt = NaiveDateTime::from_timestamp(timestamp, 0);
            self.last_contacted = DateTime::from_utc(dt, UTC);
          }
        },
        Some(&"instances") => {
          // Device command class updates.
          if update.path.get(2) == Some(&"commandClasses") {
            self.process_command_class_update(&update);
          }
        },
        _ => continue, // Unknown update
      }
    }
    Ok(())
  }

  fn process_command_class_update(&mut self, update: &DeviceUpdate)
      -> Result<(), RazberryError> {
    let command_class_id = update.path.get(3) // get the numeric value
        .ok_or(RazberryError::BadResponse)?;

    let command_class_id = match CommandClasses::from_str(command_class_id) {
      None => return Ok(()), // Unsupported command class
      Some(cc) => cc,
    };

    match self.command_classes.get_mut(&command_class_id) {
      None => Ok(()), // Not loaded at initialization. Could indicate problem.
      Some(cc) => cc.process_update(update),
    }
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
