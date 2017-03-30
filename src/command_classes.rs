// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

/**
 * The different ZWave command classes supported by various devices.
 */
pub enum CommandClasses {
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

impl CommandClasses {
  /// Convert a command class identifier into a command class.
  pub fn from_byte(command_class_id: u8) -> Option<CommandClasses> {
    let command_class = match command_class_id {
      0x00 => CommandClasses::NoOperation,
      0x20 => CommandClasses::Basic,
      0x25 => CommandClasses::SwitchBinary,
      0x26 => CommandClasses::SwitchMultilevel,
      0x30 => CommandClasses::SensorBinary,
      0x31 => CommandClasses::SensorMultilevel,
      0x60 => CommandClasses::MultiChannel,
      0x70 => CommandClasses::Configuration,
      0x71 => CommandClasses::Alarm,
      0x73 => CommandClasses::PowerLevel,
      0x80 => CommandClasses::Battery,
      0x81 => CommandClasses::Clock,
      0x84 => CommandClasses::Wakeup,
      0x85 => CommandClasses::Association,
      0x86 => CommandClasses::Version,
      0x9C => CommandClasses::AlarmSensor,
      0x9D => CommandClasses::AlarmSilence,
      0x9E => CommandClasses::SensorConfiguration,
      _ => return None,
    };
    Some(command_class)
  }
}
