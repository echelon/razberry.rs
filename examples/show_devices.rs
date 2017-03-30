// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

extern crate razberry;

use razberry::RazberryClient;
use std::env;

pub fn main() {
  let args : Vec<_> = env::args().collect();

  if args.len() != 4 {
    println!("Call with hostname, username, and password as args.");
    return;
  }

  let hostname = args.get(1).unwrap();
  let username = args.get(2).unwrap();
  let password = args.get(3).unwrap();

  let mut client = RazberryClient::for_hostname(&hostname).unwrap();
  let result = client.login(&username, &password);
  let session = client.get_session_token();

  println!("Result: {:?}", result);
  println!("Session: {:?}", session);

  let devices = client.load_devices().unwrap();

  println!("Loaded devices: {}", devices.len());

  for device in devices {
    println!("Device: {}", device);
    println!("\tLast contacted: {}", device.last_contacted);
    for (command_class_id, command_class) in device.command_classes {
      println!("\tCommand class: {}", command_class_id);
    }
  }

  //for device in devices {
  //  println!("Device: {}", device);
  //}

  /*let timestamp = client.get_server_timestamp().unwrap().get_timestamp();
  println!("Server Timestamp: {:?}", timestamp);

  let data = client.get_data().unwrap();
  let alarm = data.get_burglar_alarm(4, 0).unwrap();

  println!("Burglar Alarm Status: {:?}", alarm.get_status());
  println!("Burglar Alarm Status Updated: {:?}", alarm.get_status_updated());

  let binary_sensor = data.get_general_purpose_binary(5, 0).unwrap();

  println!("Binary Sensor (general purpose): {:?}", binary_sensor.get_status());
  println!("Binary Sensor (general purpose) updated: {:?}",
    binary_sensor.get_status_updated());*/
}
