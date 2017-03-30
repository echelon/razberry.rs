// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

pub struct SensorBinary {
  level: bool,
  //level_update_time: u16,
}

impl SensorBinary {
  pub fn get_level(&self) -> bool {
    self.level
  }
}
