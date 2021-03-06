// Copyright (c) 2016-2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

pub use response::DataResponse;
pub use response::GatewayState;
pub use response::PartialGatewayState;
pub use response::ResponseError;
pub use response::Timestamp;
pub use std::time::Duration;
pub use url::ParseError;
pub use url::Url;
use chrono::NaiveDateTime;
use chrono::UTC;
use chrono::datetime::DateTime;
use device::Device;
use device_update::DeviceUpdate;
use error::RazberryError;
use hyper::client::Client;
use hyper::header::ContentType;
use hyper::header::Cookie;
use hyper::header::SetCookie;
use hyper::mime::Attr;
use hyper::mime::Mime;
use hyper::mime::SubLevel;
use hyper::mime::TopLevel;
use hyper::mime::Value;
use hyper::status::StatusCode;
use rustc_serialize::json::Json;
use rustc_serialize::json;
use std::collections::HashMap;
use std::io::Read;

const DEFAULT_PORT : u32 = 8083u32;
const SESSION_COOKIE_NAME : &'static str = "ZWAYSession";

/**
 * Razberry Z-Wave gateway client.
 * Polls the Razberry HTTP endpoint for updates on devices.
 */
pub struct RazberryClient {
  /// Base URL for the Razberry gateway.
  base_url: Url,

  /// Razberry gateway session token for making authenticated requests.
  session_token: Option<String>,

  /// HTTP Client.
  client: Client,

  /// Z-wave devices that have been loaded.
  /// This is a map of device ID to device.
  devices: HashMap<String, Device>,

  /// The last time Z-wave device updates were successfully polled.
  /// Timestamp is that of the Razberry endpoint (not the program's CPU time).
  pub last_update: Option<DateTime<UTC>>, // TODO: Public visibility is temporary
}

#[derive(RustcDecodable, RustcEncodable)]
struct LoginRequest {
  /// Username.
  login: String,
  /// Password.
  password: String,
  /// Misc fields.
  default_ui: u8,
  /// Unknown parameter.
  form: bool,
  /// Unknown parameter.
  keepme: bool,
}

impl RazberryClient {
  /**
   * Construct a client from hostname, using the default port.
   */
  pub fn for_hostname(hostname: &str) -> Result<RazberryClient, ParseError> {
    RazberryClient::new(hostname, DEFAULT_PORT)
  }

  /**
   * Construct a client from hostname and port.
   */
  pub fn new(hostname: &str, port: u32) -> Result<RazberryClient, ParseError> {
    Url::parse(&format!("http://{}:{}", hostname, port)).map(|url| {
      RazberryClient {
        base_url: url,
        session_token: None,
        client: Client::new(),
        devices: HashMap::new(),
        last_update: None,
      }
    })
  }

  /**
   * Set the session for the cookie manually.
   */
  pub fn set_session_token(&mut self, credential: Option<String>) {
    self.session_token = credential;
  }

  /**
   * Get the session token.
   */
  pub fn get_session_token(&self) -> Option<String> {
    // TODO: Cleanup.
    self.session_token.as_ref().map(|s| s.to_string())
  }

  /**
   * Set HTTP client read timeout.
   */
  pub fn set_read_timeout(&mut self, timeout: Option<Duration>) {
    self.client.set_read_timeout(timeout)
  }

  /**
   * Set HTTP client write timeout.
   */
  pub fn set_write_timeout(&mut self, timeout: Option<Duration>) {
    self.client.set_write_timeout(timeout)
  }

  /**
   * Peform a login. If the attempt is successful, store the session token.
   */
  pub fn login(&mut self, username: &str, password: &str)
      -> Result<(), RazberryError> {
    let login_request = try!(json::encode(&LoginRequest {
      login: username.to_string(),
      password: password.to_string(),
      default_ui: 1,
      form: true,
      keepme: false,
    }).map_err(|_| RazberryError::ClientError));

    let url = try!(self.login_url());

    let result = try!(self.client.post(url)
        .body(&login_request)
        .header(ContentType(Mime(TopLevel::Application, SubLevel::Json,
                                     vec![(Attr::Charset, Value::Utf8)])))
        .send()
        .map_err(|_| RazberryError::ClientError));

    match result.status {
      StatusCode::Ok => {}, // Continue
      StatusCode::Unauthorized => { return Err(RazberryError::BadCredentials); },
      _ => { return Err(RazberryError::BadRequest); },
    }

    // Get the session cookie from the response.
    // TODO: Cleanup once 'as_slice' becomes stable.
    let cookies = result.headers.get::<SetCookie>().unwrap().clone();

    for cookie in cookies.iter() {
      match Self::parse_cookie_value(cookie) {
        None => continue,
        Some((name, value)) => {
          if name != SESSION_COOKIE_NAME {
            continue;
          }
          self.session_token = Some(value);
          return Ok(());
        }
      }
    }

    Err(RazberryError::ServerError)
  }

  fn parse_cookie_value(cookie: &str) -> Option<(String, String)> {
    let cookie_parts = cookie.split("; ").collect::<Vec<&str>>();

    if let Some(name_value) = cookie_parts.first().map(|v| v.to_string()) {
      let split = name_value.split("=").collect::<Vec<&str>>();
      let name = split.get(0);
      let value = split.get(1);
      if name.is_some() && value.is_some() {
        return Some((name.unwrap().to_string(), value.unwrap().to_string()));
      }
    }
    None
  }

  // TODO: Test.
  /// Query the initial data payload for devices (the bare /Data endpoint).
  pub fn load_devices(&mut self) -> Result<(), RazberryError> {
    let url = self.data_url(None)?;

    let session_token = self.session_token.as_ref()
        .ok_or(RazberryError::ClientError)?;

    let mut result = self.client.get(url)
        .header(Cookie(vec![
          format!("{}={}", SESSION_COOKIE_NAME, session_token)
        ]))
        .send()
        .map_err(|_| RazberryError::ClientError)?;

    match result.status {
      StatusCode::Ok => {}, // Continue
      StatusCode::Unauthorized => {
        return Err(RazberryError::BadCredentials);
      },
      _ => { return Err(RazberryError::BadRequest); },
    }

    let mut body = String::new();

    result.read_to_string(&mut body)
        .map_err(|_| RazberryError::ServerError)?;

    let json = Json::from_str(&body)?;

    let devices_json = json.find("devices")
        .and_then(|d| d.as_object())
        .ok_or(RazberryError::BadResponse)?;

    let mut devices = HashMap::new();

    for (device_id, device_json) in devices_json {
      let device = Device::initialize_from_json(device_id, &device_json)?;
      devices.insert(device_id.to_string(), device);
    }

    let update_time = Self::parse_update_time(&json)?;

    self.last_update = Some(update_time);
    self.devices = devices; // TODO: Interior mutability.

    Ok(())
  }

  // TODO: Test.
  /// Poll the /Data/{time} endpoint for updates.
  pub fn poll_updates(&mut self) -> Result<(), RazberryError> {
    // Can't poll for updates unless we've loaded devices first.
    let dt = self.last_update.ok_or(RazberryError::ClientError)?;
    let timestamp = dt.timestamp();

    let url = self.data_url(Some(timestamp))?;

    let session_token = self.session_token.as_ref()
        .ok_or(RazberryError::ClientError)?;

    let mut result = self.client.get(url)
        .header(Cookie(vec![
          format!("{}={}", SESSION_COOKIE_NAME, session_token)
        ]))
        .send()
        .map_err(|_| RazberryError::ClientError)?;

    match result.status {
      StatusCode::Ok => {}, // Continue
      StatusCode::Unauthorized => {
        return Err(RazberryError::BadCredentials);
      },
      _ => return Err(RazberryError::BadRequest),
    }

    let mut body = String::new();

    result.read_to_string(&mut body)
        .map_err(|_| RazberryError::ServerError)?;

    let json = Json::from_str(&body)?;

    // TODO: DEVICE UPDATES!
    let updates = DeviceUpdate::parse_updates(&json)?;

    for (device_id, updates) in updates {
      match self.devices.get_mut(&device_id) {
        None => continue, // Perhaps a new device was added. We must ignore.
        Some(ref mut device) => {
          let _r = device.process_updates(updates)?;
        },
      }
    }

    let update_time = Self::parse_update_time(&json)?;

    self.last_update = Some(update_time);
    Ok(())
  }

  // TODO: API is a WIP. Prefer interior mutability.
  // A better approach might even be to return a "DeviceContext" that we use
  // in subsequent polling calls, so users of the library can roll their own
  // atomic guarantees.
  /// Get devices that have been loaded by the client.
  pub fn get_devices(&self) -> Vec<&Device> {
    self.devices.values()
        .map(|d| d)
        .collect()
  }

  // TODO: Unit test this. Make sure Chrono::DateTime.timestamp() equals the original.
  /// Parse the updated time from either JSON endpoint.
  fn parse_update_time(json: &Json) -> Result<DateTime<UTC>, RazberryError> {
    let timestamp = json.find_path(&["updateTime"])
        .and_then(|j| j.as_i64())
        .ok_or(RazberryError::BadResponse)?;

    let dt = NaiveDateTime::from_timestamp(timestamp, 0);
    Ok(DateTime::from_utc(dt, UTC))
  }

  /* ========================= DEPRECATED ========================= */

  /**
   * Get a full data dump of the state of the Razberry gateway and all
   * of its associated devices.
   */
  #[deprecated]
  pub fn fetch_gateway_state(&self) -> Result<GatewayState, RazberryError> {
    let url = try!(self.data_url(None));
    let session_token = self.session_token.as_ref()
        .ok_or(RazberryError::ClientError)?;

    let mut result = try!(self.client.get(url)
        .header(Cookie(vec![
          format!("{}={}", SESSION_COOKIE_NAME, session_token)
        ]))
        .send()
        .map_err(|_| RazberryError::ClientError));

    match result.status {
      StatusCode::Ok => {}, // Continue
      StatusCode::Unauthorized => { return Err(RazberryError::BadCredentials); },
      _ => { return Err(RazberryError::BadRequest); },
    }

    let mut body = String::new();
    try!(result.read_to_string(&mut body)
         .map_err(|_| RazberryError::ServerError));

    GatewayState::build(&body).map_err(|_| RazberryError::ClientError)
  }

  /**
   * Get an updated view of the state of the Razberry gateway. This
   * fetches any state changes since the last fetch or update and
   * patches the delta into the 'GatewayState' object.
   */
  #[deprecated]
  pub fn update_gateway_state(&self, gateway_state: &mut GatewayState) ->
      Result<(), RazberryError> {
    let timestamp = gateway_state.get_end_timestamp();
    let url = try!(self.data_url(Some(timestamp)));

    let session_token = self.session_token.as_ref()
        .ok_or(RazberryError::ClientError)?;

    let mut result = try!(self.client.get(url)
        .header(Cookie(vec![
          format!("{}={}", SESSION_COOKIE_NAME, session_token)
        ]))
        .send()
        .map_err(|_| RazberryError::ClientError));

    match result.status {
      StatusCode::Ok => {}, // Continue
      StatusCode::Unauthorized => { return Err(RazberryError::BadCredentials); },
      _ => { return Err(RazberryError::BadRequest); },
    }

    let mut body = String::new();
    try!(result.read_to_string(&mut body)
         .map_err(|_| RazberryError::ServerError));

    let partial_state = try!(PartialGatewayState::build(&body, timestamp)
        .map_err(|_| RazberryError::ClientError));

    // TODO: Rethink errors.
    gateway_state.merge(&partial_state).map_err(|_| RazberryError::ClientError)
  }

  /// Generate a data URL.
  fn data_url(&self, timestamp: Option<i64>) -> Result<Url, RazberryError> {
    let path = match timestamp {
      None => "/ZWaveAPI/Data".to_string(),
      Some(t) => format!("/ZWaveAPI/Data/{}", t),
    };
    self.base_url.join(&path)
        .map_err(|_| RazberryError::ClientError)
  }

  /// Generate login URL.
  fn login_url(&self) -> Result<Url, RazberryError> {
    self.base_url.join("/ZAutomation/api/v1/login")
        .map_err(|_| RazberryError::ClientError)
  }

  /**
   * XXX: DEPRECATED.
   * Get a full data dump of the state of the Razberry server and all
   * of its associated devices.
   */
  #[deprecated]
  pub fn get_data(&self) -> Result<DataResponse, RazberryError> {
    self.fetch_data(None)
  }

  /**
   * XXX: DEPRECATED.
   * Get a partial data dump of the state changes to the Razberry
   * server and associated devices that occurred after the provided
   * timestamp.
   */
  #[deprecated]
  pub fn get_data_after(&self, timestamp: i64)
      -> Result<DataResponse, RazberryError> {
    self.fetch_data(Some(timestamp))
  }

  /**
   * XXX: DEPRECATED.
   * Fastest way to look up the server timestamp.
   * Calls the data endpoint with an invalid timestamp.
   */
  #[deprecated]
  pub fn get_server_timestamp(&self) -> Result<DataResponse, RazberryError> {
    self.fetch_data(Some(20000000000))
  }

  /// XXX: DEPRECATED.
  /// Do lookup at the data endpoint.
  #[deprecated]
  pub fn fetch_data(&self, timestamp: Option<i64>)
      -> Result<DataResponse, RazberryError> {
    let url = try!(self.data_url(timestamp));
    let session_token = self.session_token.as_ref()
        .ok_or(RazberryError::ClientError)?;

    let mut result = try!(self.client.get(url)
        .header(Cookie(vec![
          format!("{}={}", SESSION_COOKIE_NAME, session_token)
        ]))
        .send()
        .map_err(|_| RazberryError::ClientError));

    match result.status {
      StatusCode::Ok => {}, // Continue
      StatusCode::Unauthorized => { return Err(RazberryError::BadCredentials); },
      _ => { return Err(RazberryError::BadRequest); },
    }

    let mut body = String::new();
    try!(result.read_to_string(&mut body)
         .map_err(|_| RazberryError::ServerError));

    DataResponse::from_str(&body).map_err(|_| RazberryError::ClientError)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // TODO: More testing.

  #[test]
  fn client_with_hostname() {
    assert!(RazberryClient::for_hostname("localhost").is_ok())
  }

  #[test]
  fn client_with_hostname_and_port() {
    assert!(RazberryClient::new("localhost", 1234u32).is_ok())
  }

  #[test]
  fn test_good_cookie_parsing() {
    let cookie = "ZWAYSession=foo-bar-baz; Path=/; HttpOnly";
    let parsed = RazberryClient::parse_cookie_value(cookie);
    assert!(parsed.is_some());

    let pair = parsed.unwrap();
    assert_eq!("ZWAYSession", pair.0);
    assert_eq!("foo-bar-baz", pair.1);
  }

  #[test]
  fn test_bad_cookie_parsing() {
    let cookie = "";
    let parsed = RazberryClient::parse_cookie_value(cookie);
    assert!(parsed.is_none());

    let cookie = "invalid; invalid";
    let parsed = RazberryClient::parse_cookie_value(cookie);
    assert!(parsed.is_none());
  }

  #[test]
  fn test_parse_timestamp() {
    fn make_datetime(ts: i64) -> DateTime<UTC> {
      DateTime::<UTC>::from_utc(NaiveDateTime::from_timestamp(ts, 0), UTC)
    }

    let json = Json::from_str("{\"updateTime\": 0}").unwrap();
    let update_time = RazberryClient::parse_update_time(&json).unwrap();
    let expected = make_datetime(0);

    assert_eq!(update_time, expected);

    let json = Json::from_str("{\"updateTime\": 1492409124}").unwrap();
    let update_time = RazberryClient::parse_update_time(&json).unwrap();
    let expected = make_datetime(1492409124);

    assert_eq!(update_time, expected);
  }
}
