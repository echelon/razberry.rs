// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

use rustc_serialize::json;

/// Represents library errors.
#[derive(Debug)]
pub enum RazberryError {
  // New
  ParseError { cause: json::ParserError },

  // Old:
  ClientError,
  BadRequest,
  BadCredentials,
  ServerError,
}

impl From<json::ParserError> for RazberryError {
  fn from(error: json::ParserError) -> RazberryError {
    RazberryError::ParseError { cause: error }
  }
}