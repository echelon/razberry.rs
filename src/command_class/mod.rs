// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

pub mod sensor_binary;
pub mod sensor_multilevel;

use command_class::sensor_binary::SensorBinary;
use command_class::sensor_multilevel::SensorMultilevel;
use command_classes::CommandClasses;
use error::RazberryError;
use rustc_serialize::json::Json;

/**
 * Polymorphic struct that can contain any command class instance.
 */
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
}
