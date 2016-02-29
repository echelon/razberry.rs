// Copyright (c) 2016 Brandon Thomas <bt@brand.io>

use rustc_serialize::json::BuilderError;
use rustc_serialize::json::Json;
use rustc_serialize::json::Object;
use sensors::BurglarAlarmData;
use sensors::GeneralPurposeBinaryData;

pub type Timestamp = i64;
pub type ParseError = BuilderError;

/// A response from Razberry's /ZWaveAPI/Data endpoint.
/// These responses contain the entire state of the gateway at the time
/// of the request.
#[derive(Clone)]
pub struct GatewayState {
  /// Internal data.
  json: Json,

  /// The end of the state change range (gateway server time when the
  /// response was returned).
  end_timestamp: Timestamp,
}

/// A response from Razberry's /ZWaveAPI/Data/{timestamp} endpoint.
/// These responses contain only updates that ocurred after the
/// requested timestamp.
#[derive(Clone)]
pub struct PartialGatewayState {
  /// Internal data.
  json: Json,

  /// The start of the state change range (timestamp sent in the
  /// request URL).
  start_timestamp: Timestamp,

  /// The end of the state change range (gateway server time when the
  /// response was returned).
  end_timestamp: Timestamp,
}

/// Possible error with the response.
#[derive(Debug)]
pub enum ResponseError {
  /// There was a problem parsing the JSON received.
  ParseError,

  /// The JSON we received did not match the schema we expected.
  MalformedResponse,

  /// The response was missing a timestamp.
  MissingTimestamp,

  /// The PartialGatewayState cannot be merged since it could result in
  /// missing events. This is the result of a gap between queries to
  /// the '/Data' and '/Data/{timestamp}' endpoints.
  PossibleMissingEvents,
}

impl GatewayState {
  /// Build from a raw JSON string.
  pub fn build(raw_json: &str) -> Result<GatewayState, ResponseError> {
    let json = try!(parse_json(raw_json));

    let timestamp = try!(json.find("updateTime")
                         .and_then(|t| t.as_i64())
                         .ok_or(ResponseError::MissingTimestamp));

    Ok(GatewayState {
      json: json,
      end_timestamp: timestamp,
    })
  }

  /// Get a reference to the underlying Json.
  pub fn get_json(&self) -> &Json {
    &self.json
  }

  /// Get the end of the state change range.
  pub fn get_end_timestamp(&self) -> Timestamp {
    self.end_timestamp
  }

  /// TODO DOC
  pub fn merge(&mut self, partial_state: &PartialGatewayState)
      -> Result<(), ResponseError> {
    if partial_state.get_start_timestamp() > self.end_timestamp {
      // Chance of missing events.
      return Err(ResponseError::PossibleMissingEvents);
    } else if partial_state.get_end_timestamp() <= self.end_timestamp {
      // No new events to pick up.
      return Ok(());
    }

    let updated_values_obj = match partial_state.get_json().as_object() {
      None => { return Err(ResponseError::MalformedResponse); },
      Some(j) => j,
    };

    for (key, updated_json_val) in updated_values_obj.iter() {
      if key != "updateTime" {
        self.merge_updated_values(key, updated_json_val);
      }
    }

    self.end_timestamp = partial_state.get_end_timestamp();

    Ok(())
  }

  // TODO: This seriously needs testing.
  /// Merge updates into the JSON tree.
  ///
  /// The 'device_key' encodes a tree traversal path of the form,
  ///
  ///   "devices.1.instances.0.commandClasses.32.data.level"
  ///
  /// The 'updated_json' contains updated keys (not a full leaf
  /// replacement) at that subtree, eg.
  ///
  ///   { "value": 255, "updateTime": 1456647251 }
  ///
  fn merge_updated_values(&mut self, device_key: &str, updated_json: &Json) {
    let updated_object = match updated_json.as_object() {
      None => { return; }, // Malformed update payload.
      Some(j) => j,
    };

    let split = device_key.split(".");
    let search_path = split.collect::<Vec<&str>>();

    let root_object = self.json.as_object_mut();
    let maybe_subtree = GatewayState::get_subtree_mut(root_object, &search_path);

    let mut subtree = match maybe_subtree {
      None => {
        // We received a reference to a part of the tree that doesn't
        // exist, which probably means there was a new device added.
        // TODO: Synthesize new nodes in the device tree instead of
        // ignoring new devices outright.
        return;
      },
      Some(stree) => stree,
    };

    for (updated_key, updated_val) in updated_object.iter() {
      subtree.insert(updated_key.to_string(), updated_val.clone());
    }
  }

  // TODO: This seriously needs testing.
  /// Search the JSON tree for the subtree we want to update, and return
  /// a mutable reference. This is a hack since there is nothing along
  /// the lines of 'Json::find_path_mut(path)'.
  ///
  /// 'maybe_json_object' is the tree node we want to search.
  /// 'path' is a slice containing the key traversal, in DFS order.
  fn get_subtree_mut<'a>(maybe_json_object: Option<&'a mut Object>, path: &[&str])
      -> Option<&'a mut Object> {
    let object = match maybe_json_object {
      None => { return None; }, // Couldn't recursively find the node we wanted.
      Some(o) => o,
    };

    match path.split_first() {
      None => {
        return Some(object); // Our search is done, pop off the stack.
      },
      Some((first, remaining_path)) => {
        // Recursively search...
        let child : Option<&mut Json> = object.get_mut(&(first.to_string()));
        let child_json = match child {
          None => { return None; }, // Couldn't find child.
          Some(j) => j,
        };
        GatewayState::get_subtree_mut(child_json.as_object_mut(), remaining_path)
      },
    }
  }

  // TODO: Fix this API. It's really obtuse.
  /// Get "burglar alarm" sensor data from the results, if present.
  pub fn get_burglar_alarm(&self, device: u8, instance: u8) ->
      Option<BurglarAlarmData> {
    let name = format!(
        "devices.{}.instances.{}.commandClasses.113.data.7",
        device, instance);

    let path = DataResponse::path_query_parts(&name);
    let alarm_data = self.json.find_path(&path);

    alarm_data.and_then(|data| Some(BurglarAlarmData::new(&data)))
  }

  // TODO: Fix this API. It's really obtuse.
  /// Get the "general purpose" (0x01) binary sensor (0x30) data, if present.
  pub fn get_general_purpose_binary(&self, device: u8, instance: u8) ->
    Option<GeneralPurposeBinaryData> {
    let name = format!(
        "devices.{}.instances.{}.commandClasses.48.data.1",
        device, instance);

    let path = DataResponse::path_query_parts(&name);
    let sensor_data = self.json.find_path(&path);

    sensor_data.and_then(
        |data| Some(GeneralPurposeBinaryData::new(&data)))
  }
}

impl PartialGatewayState {
  /// Build from a raw JSON string.
  pub fn build(raw_json: &str, request_time: Timestamp) ->
      Result<PartialGatewayState, ResponseError> {
    let json = try!(parse_json(raw_json));

    let timestamp = try!(json.find("updateTime")
                         .and_then(|t| t.as_i64())
                         .ok_or(ResponseError::MissingTimestamp));

    Ok(PartialGatewayState {
      json: json,
      start_timestamp: request_time,
      end_timestamp: timestamp,
    })
  }

  /// Get a reference to the underlying Json.
  pub fn get_json(&self) -> &Json {
    &self.json
  }

  /// Get the start of the state change range (URL parameter)
  pub fn get_start_timestamp(&self) -> Timestamp {
    self.start_timestamp
  }

  /// Get the end of the state change range (time payload was generated)
  pub fn get_end_timestamp(&self) -> Timestamp {
    self.end_timestamp
  }
}

/// Parse a raw string into JSON.
fn parse_json(raw_string: &str) -> Result<Json, ResponseError> {
  return Json::from_str(raw_string).map_err(|_| ResponseError::ParseError)
}

/* ========================== DEPRECATED ========================== */

/// XXX: DEPRECATED.
/// A response from Razberry's /ZWaveAPI/Data endpoint.
pub struct DataResponse {
  json: Json,
}

/// XXX: DEPRECATED.
// TODO: This API is a little obtuse.
impl DataResponse {
  pub fn new(json: Json) -> DataResponse {
    DataResponse { json: json }
  }

  pub fn from_str(raw_response: &str) -> Result<DataResponse, ParseError> {
    let json = try!(Json::from_str(raw_response));
    Ok(DataResponse::new(json))
  }

  /// Get the timestamp when the response was generated.
  pub fn get_timestamp(&self) -> Option<Timestamp> {
    // TODO: Check that timestamps are valid Unix timestamps
    self.json.find("updateTime").and_then(|t| t.as_i64())
  }

  /// Whether the response payload is "full", ie. contains all state
  /// information. When the endpoint is called without a timestamp, it
  /// returns a full state dump response.
  pub fn is_full_response(&self) -> bool {
    self.json.find("devices").is_some()
  }

  /// Get "burglar alarm" sensor data from the results, if present.
  pub fn get_burglar_alarm(&self, device: u8, instance: u8) ->
      Option<BurglarAlarmData> {
    let name = format!(
        "devices.{}.instances.{}.commandClasses.113.data.7",
        device, instance);

    let alarm_data = if self.is_full_response() {
      let path = DataResponse::path_query_parts(&name);
      self.json.find_path(&path)
    } else {
      self.json.find(&name)
    };

    alarm_data.and_then(|data| Some(BurglarAlarmData::new(&data)))
  }

  /// Get the "general purpose" (0x01) binary sensor (0x30) data, if present.
  pub fn get_general_purpose_binary(&self, device: u8, instance: u8) ->
    Option<GeneralPurposeBinaryData> {
    let name = format!(
        "devices.{}.instances.{}.commandClasses.48.data.1",
        device, instance);

    let sensor_data = if self.is_full_response() {
      let path = DataResponse::path_query_parts(&name);
      self.json.find_path(&path)
    } else {
      self.json.find(&name)
    };

    sensor_data.and_then(
        |data| Some(GeneralPurposeBinaryData::new(&data)))
  }

  /// Get a reference to the underlying Json.
  pub fn get_json(&self) -> &Json {
    &self.json
  }

  /// Convert a query, eg. 'devices.1.instances.1.*', into its parts.
  fn path_query_parts(query: &str) -> Vec<&str> {
    query.split(".").collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn get_timestamp_present() {
    let json = "{ \"updateTime\": 1456036584 }";
    let response = DataResponse::from_str(json).unwrap();
    assert_eq!(1456036584i64, response.get_timestamp().unwrap());
  }

  #[test]
  fn get_timestamp_absent() {
    let json = "{}";
    let response = DataResponse::from_str(json).unwrap();
    assert!(response.get_timestamp().is_none());
  }

  #[test]
  fn get_timestamp_invalid() {
    let json = "{\"updateTime\": \"invalid\" }";
    let response = DataResponse::from_str(json).unwrap();
    assert!(response.get_timestamp().is_none());
  }

  #[test]
  fn path_query_parts() {
    let expected = vec!["devices", "1", "instances"];
    let result = DataResponse::path_query_parts("devices.1.instances");
    assert_eq!(expected, result);
  }

  // From hitting /ZWaveAPI/Data without a timestamp.
  const FULL_JSON : &'static str = "\
    { \
      \"devices\": { \
        \"4\": { \
          \"instances\": { \
            \"0\": { \
              \"commandClasses\": {
                \"113\": { \
                  \"name\": \"Alarm\", \
                  \"data\": { \
                  \"7\": { \
                    \"value\": null, \
                    \"type\": \"empty\", \
                    \"typeString\": { \
                      \"value\": \"Burglar\", \
                      \"type\": \"string\", \
                      \"invalidateTime\": 1455606541, \
                      \"updateTime\": 1455606542 \
                    }, \
                    \"status\": { \
                      \"value\": 0, \
                      \"type\": \"int\", \
                      \"invalidateTime\": 1455606541, \
                      \"updateTime\": 1456014517 \
                    }, \
                    \"eventMask\": { \
                      \"value\": 128, \
                      \"type\": \"int\", \
                      \"invalidateTime\": 1455606541, \
                      \"updateTime\": 1455606542 \
                    }, \
                    \"event\": { \
                      \"value\": 7, \
                      \"type\": \"int\", \
                      \"invalidateTime\": 1455606541, \
                      \"updateTime\": 1456014517 \
                    }, \
                    \"eventString\": { \
                      \"value\": \"Motion detected\", \
                      \"type\": \"string\", \
                      \"invalidateTime\": 1455606541, \
                      \"updateTime\": 1456014517 \
                    }, \
                    \"eventParameters\": { \
                      \"value\": [ 7 ], \
                      \"type\": \"binary\", \
                      \"invalidateTime\": 1455606541, \
                      \"updateTime\": 1456014517 \
                    }, \
                    \"eventSequence\": { \
                      \"value\": null, \
                      \"type\": \"empty\", \
                      \"invalidateTime\": 1455606541, \
                      \"updateTime\": 1455606542 \
                    }, \
                    \"invalidateTime\": 1455606541, \
                    \"updateTime\": 1456014517 \
                  }, \
                  \"invalidateTime\": 1455606416, \
                  \"updateTime\": 1455606417 \
                  } \
                } \
              } \
            } \
          } \
        }, \
        \"5\": { \
          \"instances\": { \
            \"0\": { \
              \"commandClasses\": { \
                \"48\": { \
                  \"name\": \"SensorBinary\", \
                  \"data\": { \
                  \"1\": { \
                    \"value\": null, \
                    \"type\": \"empty\", \
                    \"sensorTypeString\": { \
                      \"value\": \"General purpose\", \
                      \"type\": \"string\", \
                      \"invalidateTime\": 1456552384, \
                      \"updateTime\": 1456552385 \
                    }, \
                    \"level\": { \
                      \"value\": true, \
                      \"type\": \"bool\", \
                      \"invalidateTime\": 1456552384, \
                      \"updateTime\": 1456569899 \
                    }, \
                    \"invalidateTime\": 1456552384, \
                    \"updateTime\": 1456569899 \
                  }, \
                  \"invalidateTime\": 1456552382, \
                  \"updateTime\": 1456552383 \
                  } \
                } \
              } \
            } \
          } \
        } \
      }, \
      \"updateTime\": 1456036584 \
    } \
  ";

  #[test]
  fn is_full_response_on_full_payload() {
    let response = DataResponse::from_str(FULL_JSON).unwrap();
    assert!(response.is_full_response());
  }

  #[test]
  fn get_burglar_alarm_on_full_payload() {
    let response = DataResponse::from_str(FULL_JSON).unwrap();
    let alarm = response.get_burglar_alarm(4, 0);
    assert!(alarm.is_some());
  }

  #[test]
  fn get_general_purpose_binary_data_on_full_payload() {
    let response = DataResponse::from_str(FULL_JSON).unwrap();
    let binary = response.get_general_purpose_binary(5, 0);
    assert!(binary.is_some());
    assert!(binary.unwrap().get_status().unwrap());
  }

  #[test]
  fn get_timestamp_on_full_payload() {
    let response = DataResponse::from_str(FULL_JSON).unwrap();
    assert_eq!(1456036584i64, response.get_timestamp().unwrap());
  }

  // From hitting /ZWaveAPI/Data with a timestamp.
  const PARTIAL_JSON : &'static str = "\
    { \
      \"devices.4.instances.0.commandClasses.113.data.7\": { \
        \"value\": null, \
        \"type\": \"empty\", \
        \"typeString\": { \
          \"value\": \"Burglar\", \
          \"type\": \"string\", \
          \"invalidateTime\": 1455606541, \
          \"updateTime\": 1455606542 \
        }, \
        \"status\": { \
          \"value\": 0, \
          \"type\": \"int\", \
          \"invalidateTime\": 1455606541, \
          \"updateTime\": 1456014517 \
        }, \
        \"eventMask\": { \
          \"value\": 128, \
          \"type\": \"int\", \
          \"invalidateTime\": 1455606541, \
          \"updateTime\": 1455606542 \
        }, \
        \"event\": { \
          \"value\": 7, \
          \"type\": \"int\", \
          \"invalidateTime\": 1455606541, \
          \"updateTime\": 1456014517 \
        }, \
        \"eventString\": { \
          \"value\": \"Motion detected\", \
          \"type\": \"string\", \
          \"invalidateTime\": 1455606541, \
          \"updateTime\": 1456014517 \
        }, \
        \"eventParameters\": { \
          \"value\": [ 7 ], \
          \"type\": \"binary\", \
          \"invalidateTime\": 1455606541, \
          \"updateTime\": 1456014517 \
        }, \
        \"eventSequence\": { \
          \"value\": null, \
          \"type\": \"empty\", \
          \"invalidateTime\": 1455606541, \
          \"updateTime\": 1455606542 \
        }, \
        \"invalidateTime\": 1455606541, \
        \"updateTime\": 1456014517 \
      }, \
      \"devices.5.instances.0.commandClasses.48.data.1\": { \
        \"value\": null, \
        \"type\": \"empty\", \
        \"sensorTypeString\": { \
          \"value\": \"General purpose\", \
          \"type\": \"string\", \
          \"invalidateTime\": 1456552384, \
          \"updateTime\": 1456552385 \
        }, \
        \"level\": { \
          \"value\": false, \
          \"type\": \"bool\", \
          \"invalidateTime\": 1456552384, \
          \"updateTime\": 1456553060 \
        }, \
        \"invalidateTime\": 1456552384, \
        \"updateTime\": 1456553060 \
      }, \
    \"updateTime\": 1456036584 \
    }";

  #[test]
  fn is_full_response_on_partial_payload() {
    let response = DataResponse::from_str(PARTIAL_JSON).unwrap();
    assert!(!response.is_full_response());
  }

  #[test]
  fn get_burglar_alarm_on_partial_payload() {
    let response = DataResponse::from_str(PARTIAL_JSON).unwrap();
    let alarm = response.get_burglar_alarm(4, 0);
    assert!(alarm.is_some());
  }

  #[test]
  fn get_general_purpose_binary_data_on_partial_payload() {
    let response = DataResponse::from_str(PARTIAL_JSON).unwrap();
    let binary = response.get_general_purpose_binary(5, 0);
    assert!(binary.is_some());
    assert!(!binary.unwrap().get_status().unwrap());
  }

  #[test]
  fn get_timestamp_on_partial_payload() {
    let response = DataResponse::from_str(PARTIAL_JSON).unwrap();
    assert_eq!(1456036584i64, response.get_timestamp().unwrap());
  }
}

