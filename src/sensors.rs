// Copyright (c) 2016 Brandon Thomas <bt@brand.io>

use response::Timestamp;
use rustc_serialize::json::Json;

// TODO: Doc, accessors
/// Command class 113, payload 7.
#[derive(Clone)]
pub struct BurglarAlarmData {
  json: Json,
}

// TODO: Doc, accessors
/// Command class 48, payload 1.
#[derive(Clone)]
pub struct GeneralPurposeBinaryData {
  json: Json,
}

impl BurglarAlarmData {
  /// Construct from JSON.
  pub fn new(json: &Json) -> BurglarAlarmData {
    BurglarAlarmData { json: json.clone() }
  }

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
  /// Construct from JSON.
  pub fn new(json: &Json) -> GeneralPurposeBinaryData {
    GeneralPurposeBinaryData { json: json.clone() }
  }

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
  mod burglar_alarm {
    use response::*;
    use sensors::*;
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

