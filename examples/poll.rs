// Copyright (c) 2016 Brandon Thomas <bt@brand.io>

extern crate razberry;

use razberry::RazberryClient;
use std::env;
use std::thread::sleep;
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

  println!("Fetching gatway state...");
  let mut gateway_state = client.fetch_gateway_state().unwrap();

  loop {
    // TODO: Don't hardcode the device and instance.
    let alarm = match gateway_state.get_burglar_alarm(4, 0) {
      None => { continue; },
      Some(a) => a,
    };

    let binary = match gateway_state.get_general_purpose_binary(5, 0) {
      None => { continue; },
      Some(a) => a,
    };

    println!("");
    println!("Results as of: {}", gateway_state.get_end_timestamp());
    println!("Alarm status: {}", alarm.get_status().unwrap());
    println!("Alarm status updated: {}", alarm.get_status_updated().unwrap());
    println!("Binary status: {}", binary.get_status().unwrap());
    println!("Binary status updated: {}", binary.get_status_updated().unwrap());

    sleep(Duration::from_secs(1u64));

    print!("Updating gatway state... ");
    match client.update_gateway_state(&mut gateway_state) {
      Ok(_) => { println!("ok"); },
      Err(_) => { println!("error"); },
    };

  }
}

