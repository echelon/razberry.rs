// Copyright (c) 2017 Brandon Thomas <bt@brand.io, echelon@gmail.com>

extern crate razberry;

use razberry::RazberryClient;
use std::env;
use std::thread;
use std::time::Duration;

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

  let _r = client.load_devices().unwrap();
  let devices = client.get_devices();

  println!("Loaded devices: {}", devices.len());

  for device in devices {
    println!("Device: {}", device);
    println!("  Last contacted: {}", device.last_contacted);
    for (command_class_id, command_class) in &device.command_classes {
      println!("  Command class: {}", command_class);
    }
  }

  println!("\nUpdate loop...\n");

  loop {
    thread::sleep(Duration::from_millis(1000));
  }
}
