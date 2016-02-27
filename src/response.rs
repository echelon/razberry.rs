// Copyright (c) 2016 Brandon Thomas <bt@brand.io>

use rustc_serialize::json::BuilderError;
use rustc_serialize::json::Json;

pub type Timestamp = i64;
pub type ParseError = BuilderError;

/// A response from Razberry's /ZWaveAPI/Data endpoint.
pub struct DataResponse {
  json: Json,
}

// TODO: Doc, accessors
/// Command class 113, payload 7.
pub struct BurglarAlarmData {
  json: Json,
}

// TODO: Doc, accessors
/// Command class 48, payload 1.
pub struct GeneralPurposeBinaryData {
  json: Json,
}

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

    alarm_data.and_then(|data| Some(BurglarAlarmData { json: data.clone() }))
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
        |data| Some(GeneralPurposeBinaryData { json: data.clone() }))
  }

  /// Get a reference to the underlying Json.
  pub fn get_json(&self) -> &Json {
    &self.json
  }

  /// Convert a query, eg. 'devices.1.instances.1.*`, into its parts.
  fn path_query_parts(query: &str) -> Vec<&str> {
    query.split(".").collect()
  }
}

impl BurglarAlarmData {
  /// Get whether the alarm is triggered.
  pub fn get_status(&self) -> Option<bool> {
    self.json.find_path(&["status", "value"])
      .and_then(|j| j.as_i64())
      .and_then(|n| Some(n != 0))
  }

  /// Get when the sensor value was last updated
  pub fn get_status_updated(&self) -> Option<Timestamp> {
    self.json.find_path(&["status", "updateTime"])
      .and_then(|j| j.as_i64())
  }

  /// Get a reference to the underlying Json.
  pub fn get_json(&self) -> &Json {
    &self.json
  }
}

impl GeneralPurposeBinaryData {
  /// Get whether the sensor is triggered.
  pub fn get_status(&self) -> Option<bool> {
    self.json.find_path(&["level", "value"])
      .and_then(|j| j.as_boolean())
  }

  /// Get when the sensor value was last updated
  pub fn get_status_updated(&self) -> Option<Timestamp> {
    self.json.find_path(&["level", "updateTime"])
      .and_then(|j| j.as_i64())
  }

  /// Get a reference to the underlying Json.
  pub fn get_json(&self) -> &Json {
    &self.json
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

  mod burglar_alarm {
    use response::*;
    use rustc_serialize::json::Json;

    #[test]
    fn get_status_true() {
      let json = Json::from_str("{\"status\": {\"value\": 255}}").unwrap();
      let alarm = BurglarAlarmData { json: json.clone() };
      assert!(alarm.get_status().unwrap());
    }

    #[test]
    fn get_status_false() {
      let json = Json::from_str("{\"status\": {\"value\": 0}}").unwrap();
      let alarm = BurglarAlarmData { json: json.clone() };
      assert!(!alarm.get_status().unwrap());
    }

    #[test]
    fn get_status_invalid() {
      let json = Json::from_str("{\"status\": {\"value\": true}}").unwrap();
      let alarm = BurglarAlarmData { json: json.clone() };
      assert!(alarm.get_status().is_none());
    }

    #[test]
    fn get_status_not_present() {
      let json = Json::from_str("{}").unwrap();
      let alarm = BurglarAlarmData { json: json.clone() };
      assert!(alarm.get_status().is_none());
    }

    #[test]
    fn get_status_absent() {
      let response = DataResponse::from_str("{}").unwrap();
      let result = response.get_burglar_alarm(4, 1);
      assert!(result.is_none());
    }
  }
}

