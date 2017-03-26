// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

use error::RazberryError;
use rustc_serialize::json::Json;
use std::collections::HashMap;

/**
 * A single update from a single device. There may be many per refresh.
 *
 * Given a key of the form,
 *   devices.14.instances.0.commandClasses.32.data.srcNodeId
 *
 * We parse out,
 *   device_id = "14"
 *   path = (instances, 0, commandClasses, 32, data, srcNodeId)
 *   data = JSON that lives under that key.
 */
pub struct DeviceUpdate<'a> {
  pub path: Vec<&'a str>,
  pub data: &'a Json,
}

impl <'a> DeviceUpdate<'a> {
  /// Parse the update JSON payload and return a list of device updates, grouped
  /// by device identifier.
  pub fn parse_updates(json: &'a Json)
      -> Result<HashMap<String, Vec<DeviceUpdate<'a>>>, RazberryError> {
    let json = json.as_object()
        .ok_or(RazberryError::BadResponse)?;

    let mut all_updates = HashMap::new();

    for (update_key, update_value) in json {
      if !update_key.starts_with("devices.") {
        // Everything was dumped into the top-level keyspace.
        // We only want device updates.
        continue;
      }

      // Split by dot-component, skip "devices" and keep the device number
      let mut split = update_key.split(".");
      let _r = split.next().ok_or(RazberryError::BadResponse)?; // skip
      let device_id = split.next().ok_or(RazberryError::BadResponse)?;
      let path = split.collect::<Vec<&str>>();

      let update = DeviceUpdate {
        path: path,
        data: update_value,
      };

      let device_updates = all_updates.entry(device_id.to_string())
          .or_insert_with(|| Vec::new());

      device_updates.push(update);
    }

    Ok(all_updates)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse() {
    let json = Json::from_str(r#"
      {
        "devices.1.instances.0.commandClasses.32.data.level": {
          "value": 255,
          "type": "int",
          "invalidateTime": 1491741863,
          "updateTime": 1492409902
        },
        "devices.1.instances.0.commandClasses.32.data.srcNodeId": {
          "value": 9,
          "type": "int",
          "invalidateTime": 1491741863,
          "updateTime": 1492409902
        },
        "devices.1.instances.0.commandClasses.32.data.srcInstanceId": {
          "value": 0,
          "type": "int",
          "invalidateTime": 1491741863,
          "updateTime": 1492409902
        },
        "devices.9.data.lastReceived": {
          "value": 0,
          "type": "int",
          "invalidateTime": 1465266282,
          "updateTime": 1492409902
        },
        "devices.9.instances.0.commandClasses.48.data.1": {
          "value": null,
          "type": "empty",
          "sensorTypeString": {
            "value": "General purpose",
            "type": "string",
            "invalidateTime": 1487401953,
            "updateTime": 1487401954
          },
          "level": {
            "value": true,
            "type": "bool",
            "invalidateTime": 1489636200,
            "updateTime": 1492409902
          },
          "invalidateTime": 1487401953,
          "updateTime": 1492409902
        },
        "updateTime": 1492409924
      }
    "#).unwrap();

    let updates = DeviceUpdate::parse_updates(&json).unwrap();

    // Assert devices within the update
    assert!(updates.contains_key("1"));
    assert!(updates.contains_key("9"));

    // Assert updates for first device.
    assert_eq!(3, updates.get("1").unwrap().len());

    assert_eq!(vec!["instances", "0", "commandClasses", "32", "data", "level"],
      updates.get("1").unwrap().get(0).unwrap().path);

    // Assert updates for second device.
    assert_eq!(2, updates.get("9").unwrap().len());

    assert_eq!(vec!["data", "lastReceived"],
      updates.get("9").unwrap().get(0).unwrap().path);

    assert_eq!(vec!["instances", "0", "commandClasses", "48", "data", "1"],
      updates.get("9").unwrap().get(1).unwrap().path);

    // TODO: Assert JSON payloads
  }
}
