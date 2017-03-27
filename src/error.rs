// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

use rustc_serialize::json;

/// Represents library errors.
#[derive(Debug)]
pub enum RazberryError {
  /// The client was not authorized to poll the gateway.
  /// This is the result of a bad session / invalid credentials.
  BadCredentials,

  /// The client could not parse JSON from the gateway.
  ParseError { /** Original error. */ cause: json::ParserError },

  /// Some kind of error from the Razberry gateway.
  ServerError,

  // Old:
  ClientError,
  BadRequest,
}

impl From<json::ParserError> for RazberryError {
  fn from(error: json::ParserError) -> RazberryError {
    RazberryError::ParseError { cause: error }
  }
}
