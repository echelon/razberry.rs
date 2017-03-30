// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

pub mod sensor_binary;
pub mod sensor_multilevel;

use command_class::sensor_binary::SensorBinary;
use command_class::sensor_multilevel::SensorMultilevel;

/**
 * Polymorphic struct that can contain any command class instance.
 */
pub enum CommandClass {
  SensorBinary { inner: SensorBinary },
  SensorMultilevel { inner: SensorMultilevel },
}
