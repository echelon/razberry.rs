// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

pub mod sensor_binary;
pub mod sensor_multilevel;

use command_class::sensor_binary::SensorBinary;
use command_class::sensor_multilevel::SensorMultilevel;
use command_classes::CommandClasses;
use device_update::DeviceUpdate;
use error::RazberryError;
use rustc_serialize::json::Json;
use std::fmt;

/**
 * Polymorphic struct that can contain any command class instance.
 */
#[derive(Debug)]
pub enum CommandClass {
  SensorBinary { inner: SensorBinary },
  SensorMultilevel { inner: SensorMultilevel },
  Unsupported, // FIXME: This bucket is a poor concession since I'm in a hurry
}

impl CommandClass {
  /// Construct a CommandClass from the JSON subset taken from the full device
  /// payload endpoint.
  pub fn initialize_from_json(command_class: CommandClasses, json: &Json)
      -> Result<CommandClass, RazberryError> {

    let result = match command_class {
      CommandClasses::SensorBinary => {
        let sensor = SensorBinary::initialize_from_json(json)?;
        CommandClass::SensorBinary { inner:  sensor }
      },
      _ => CommandClass::Unsupported,
    };

    Ok(result)
  }

  // TODO TEST
  // TODO NON-PUBLIC
  /// Process an update.
  pub fn process_update(&mut self, update: &DeviceUpdate)
      -> Result<(), RazberryError> {
    match self {
      &mut CommandClass::SensorBinary { ref mut inner } => {
        inner.process_update(update)
      },
      _ => Ok(()), // Unsupported
    }
  }
}

impl fmt::Display for CommandClass {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &CommandClass::SensorBinary { ref inner } => inner.fmt(f),
      _ => write!(f, "CommandClass (no fmt::Display impl)"),
    }
  }
}
