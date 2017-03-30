// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

use std::fmt;

/**
 * The different ZWave command classes supported by various devices.
 */
#[derive(Debug)]
pub enum CommandClasses {
  Alarm,
  AlarmSensor,
  AlarmSilence,
  Association,
  Basic,
  Battery,
  Clock,
  Configuration,
  FirmwareUpdate,
  MultiChannel,
  MultiChannelAssociation,
  NoOperation,
  NodeNaming,
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
      0x7A => CommandClasses::FirmwareUpdate,
      0x70 => CommandClasses::Configuration,
      0x71 => CommandClasses::Alarm,
      0x73 => CommandClasses::PowerLevel,
      0x77 => CommandClasses::NodeNaming,
      0x80 => CommandClasses::Battery,
      0x81 => CommandClasses::Clock,
      0x84 => CommandClasses::Wakeup,
      0x85 => CommandClasses::Association,
      0x86 => CommandClasses::Version,
      0x8E => CommandClasses::MultiChannelAssociation,
      0x9C => CommandClasses::AlarmSensor,
      0x9D => CommandClasses::AlarmSilence,
      0x9E => CommandClasses::SensorConfiguration,
      _ => return None,
    };
    Some(command_class)
  }

  // TODO(MERGE-BLOCKER): TEST.
  /// Convert a command class string identifier into a command class.
  pub fn from_str(command_class_id: &str) -> Option<CommandClasses> {
    command_class_id.parse::<u8>()
        .ok() // Discard parse errors.
        .and_then(|cc| Self::from_byte(cc))
  }
}

impl fmt::Display for CommandClasses {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let s = match *self {
      CommandClasses::Alarm => "Alarm",
      CommandClasses::AlarmSensor => "AlarmSensor",
      CommandClasses::AlarmSilence => "AlarmSilence",
      CommandClasses::Association => "Association",
      CommandClasses::Basic => "Basic",
      CommandClasses::Battery => "Battery",
      CommandClasses::Clock => "Clock",
      CommandClasses::Configuration => "Configuration",
      CommandClasses::FirmwareUpdate => "FirmwareUpdate",
      CommandClasses::MultiChannel => "MultiChannel",
      CommandClasses::MultiChannelAssociation => "MultiChannelAssociation",
      CommandClasses::NoOperation => "NoOperation",
      CommandClasses::NodeNaming => "NodeNaming",
      CommandClasses::PowerLevel => "PowerLevel",
      CommandClasses::SensorBinary => "SensorBinary",
      CommandClasses::SensorConfiguration => "SensorConfiguration",
      CommandClasses::SensorMultilevel => "SensorMultilevel",
      CommandClasses::SwitchBinary => "SwitchBinary",
      CommandClasses::SwitchMultilevel => "SwitchMultilevel",
      CommandClasses::Version => "Version",
      CommandClasses::Wakeup => "Wakeup",
    };
    write!(f, "<CommandClasses::{}>", s)
  }
}
