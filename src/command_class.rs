// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

/**
 * The different ZWave command classes supported by various devices.
 */
pub enum CommandClass {
  Alarm,
  AlarmSensor,
  AlarmSilence,
  Association,
  Basic,
  Battery,
  Clock,
  Configuration,
  MultiChannel,
  NoOperation,
  PowerLevel,
  SensorBinary,
  SensorConfiguration,
  SensorMultilevel,
  SwitchBinary,
  SwitchMultilevel,
  Version,
  Wakeup,
}

impl CommandClass {
  /// Convert a command class identifier into a command class.
  pub fn from_byte(command_class_id: u8) -> Option<CommandClass> {
    let command_class = match command_class_id {
      0x00 => CommandClass::NoOperation,
      0x20 => CommandClass::Basic,
      0x25 => CommandClass::SwitchBinary,
      0x26 => CommandClass::SwitchMultilevel,
      0x30 => CommandClass::SensorBinary,
      0x31 => CommandClass::SensorMultilevel,
      0x60 => CommandClass::MultiChannel,
      0x70 => CommandClass::Configuration,
      0x71 => CommandClass::Alarm,
      0x73 => CommandClass::PowerLevel,
      0x80 => CommandClass::Battery,
      0x81 => CommandClass::Clock,
      0x84 => CommandClass::Wakeup,
      0x85 => CommandClass::Association,
      0x86 => CommandClass::Version,
      0x9C => CommandClass::AlarmSensor,
      0x9D => CommandClass::AlarmSilence,
      0x9E => CommandClass::SensorConfiguration,
      _ => return None,
    };
    Some(command_class)
  }
}