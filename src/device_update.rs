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
  // TODO TEST
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
