
use command_classes::sensor_binary::SensorBinary;
use command_classes::sensor_multilevel::SensorMultilevel;

pub enum DeviceCommandClass {
  SensorBinary { inner: SensorBinary },
  SensorMultilevel { inner: SensorMultilevel },
}
