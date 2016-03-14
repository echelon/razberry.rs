// Copyright (c) 2016 Brandon Thomas <bt@brand.io>

use response::Timestamp;
use rustc_serialize::json::Json;

// TODO: Doc, accessors
/// Command class 0x71 (113), payload 7.
#[derive(Clone)]
pub struct BurglarAlarmData {
  json: Json,
}

// TODO: Doc, accessors
/// Command class 0x30 (48), payload 1.
#[derive(Clone)]
pub struct GeneralPurposeBinaryData {
  json: Json,
}

impl BurglarAlarmData {
  /// Construct from JSON.
  pub fn new(json: &Json) -> BurglarAlarmData {
    BurglarAlarmData { json: json.clone() }
  }

  /// This is a huristic used to determine if the alarm has been
  /// activated. It works between Aeotec Gen5 and Gen6 alarm sensors,
  /// each of which reports "activation" differently. This may not work
  /// for other types of hardware. If I can find documentation on the
  /// alarm command class, I will update or deprecate this method as
  /// necessary.
  pub fn get_activated(&self) -> Option<bool> {
    let mask = match self.get_event_mask() {
      None => { return None; },
      Some(s) => s,
    };

    match mask {
      128i64 => self.get_status(), // Aeotec Gen5 sensor
      264i64 => self.get_event() // Aeotec Gen6 sensor
                    .and_then(|ev| Some(ev != 0 && ev != 254)), // 254 = "unknown"
      _ => None,
    }
  }

  /// Get whether the alarm is triggered.
  /// Note that this is not true across all sensor types; the Aeotec
  /// Multisensor Gen5 uses this to report alarm triggering, but the
  /// Gen6 sensor does not (instead, it uses "event").
  /// XXX/Note: This may be deprecated in favor of a method that
  /// returns an i64.
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

  /// Get the "event" value.
  pub fn get_event(&self) -> Option<i64> {
    self.json.find_path(&["event", "value"])
      .and_then(|j| j.as_i64())
  }

  /// Get when the "event" value was updated.
  pub fn get_event_updated(&self) -> Option<Timestamp> {
    self.json.find_path(&["event", "updateTime"])
      .and_then(|j| j.as_i64())
  }

  /// Get the "event mask" value
  pub fn get_event_mask(&self) -> Option<i64> {
    self.json.find_path(&["eventMask", "value"])
      .and_then(|j| j.as_i64())
  }

  /// Get the event string
  pub fn get_event_string(&self) -> Option<String> {
    self.json.find_path(&["eventString", "value"])
      .and_then(|j| j.as_string())
      .and_then(|s| Some(s.to_string()))
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
    fn get_activated() {
      let mut json = "{}";

      // Basic fail cases
      assert_eq!(burglar_alarm(json).get_activated(), None);
      assert_eq!(burglar_alarm("{\"foo\": 0}").get_activated(), None);
      assert_eq!(burglar_alarm("{\"eventMask\": 0}").get_activated(), None);

      // Aeotec Multisensor Gen 5
      json = "{\"eventMask\": {\"value\": 128}, \"status\": {\"value\": 0}}";
      assert_eq!(burglar_alarm(json).get_activated(), Some(false));
      json = "{\"eventMask\": {\"value\": 128}, \"status\": {\"value\": 255}}";
      assert_eq!(burglar_alarm(json).get_activated(), Some(true));

      // Aeotec Multisensor Gen 6
      json = "{\"eventMask\": {\"value\": 264}, \"event\": {\"value\": 0}}";
      assert_eq!(burglar_alarm(json).get_activated(), Some(false));
      json = "{\"eventMask\": {\"value\": 264}, \"event\": {\"value\": 254}}";
      assert_eq!(burglar_alarm(json).get_activated(), Some(false));
      json = "{\"eventMask\": {\"value\": 264}, \"event\": {\"value\": 8}}";
      assert_eq!(burglar_alarm(json).get_activated(), Some(true));
    }


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

    #[test]
    fn get_event() {
      // Fail cases
      assert_eq!(burglar_alarm("{}").get_event(), None);
      assert_eq!(burglar_alarm("{\"event\": {\"foo\": 0}}").get_event(), None);
      assert_eq!(burglar_alarm("{\"event\": {\"value\": true}}").get_event(), None);

      // Success cases
      assert_eq!(burglar_alarm("{\"event\": {\"value\": 0}}").get_event(),
                 Some(0i64));
      assert_eq!(burglar_alarm("{\"event\": {\"value\": 255}}").get_event(),
                 Some(255i64));
    }

    #[test]
    fn get_event_updated() {
      // Fail cases
      assert_eq!(burglar_alarm("{}").get_event_updated(), None);
      assert_eq!(burglar_alarm("{\"event\": {\"foo\": 0}}").get_event_updated(),
                 None);
      assert_eq!(burglar_alarm("{\"event\": {\"value\": true}}").get_event_updated(),
                 None);

      // Success cases
      assert_eq!(burglar_alarm("{\"event\": {\"updateTime\": 0}}")
                 .get_event_updated(), Some(0i64));
      assert_eq!(burglar_alarm("{\"event\": {\"updateTime\": 1457816333}}")
                 .get_event_updated(), Some(1457816333i64));
    }

    #[test]
    fn get_event_mask() {
      // Fail cases
      assert_eq!(burglar_alarm("{}").get_event_mask(), None);
      assert_eq!(burglar_alarm("{\"eventMask\": {\"foo\": 0}}").get_event_mask(),
                 None);
      assert_eq!(burglar_alarm("{\"eventMask\": {\"value\": true}}")
                 .get_event_mask(), None);

      // Success cases
      assert_eq!(burglar_alarm("{\"eventMask\": {\"value\": 0}}").get_event_mask(),
                 Some(0i64));
      assert_eq!(burglar_alarm("{\"eventMask\": {\"value\": 255}}").get_event_mask(),
                 Some(255i64));
    }

    #[test]
    fn get_event_string() {
      // Fail cases
      assert_eq!(burglar_alarm("{}").get_event_string(), None);
      assert_eq!(burglar_alarm("{\"eventString\": {\"foo\": 0}}").get_event_string(),
                 None);
      assert_eq!(burglar_alarm("{\"eventString\": {\"value\": true}}")
                 .get_event_string(), None);

      // Success cases
      assert_eq!(burglar_alarm("{\"eventString\": {\"value\": \"\"}}")
                 .get_event_string(), Some("".to_string()));
      assert_eq!(burglar_alarm("{\"eventString\": {\"value\": \"test\"}}")
                 .get_event_string(), Some("test".to_string()));
    }

    // Helper to construct a burglar alarm with the JSON string literal
    fn burglar_alarm(json_string: &str) -> BurglarAlarmData {
      match Json::from_str(json_string) {
        Err(_) => BurglarAlarmData { json: Json::from_str("{}").unwrap() },
        Ok(json) => BurglarAlarmData { json: json.clone() },
      }
    }
  }
}

