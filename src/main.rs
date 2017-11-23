extern crate hyper_tls;
extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate ansi_term;

use std::{env, thread};

pub mod connector;
pub mod statistics_server;

use connector::connector::Connector;
use statistics_server::statistics_server::update as update;
use statistics_server::statistics_server::listen as start_stat_server;

fn main() {
    let raw_address = env::args().nth(1);
    match raw_address {
        Some(arg) => {
            let address = parse_address(arg);

            let thread = thread::spawn(move || {
                let mut connector = Connector::new();
                connector.run(&address, &update);
            });

            let server_thread = thread::spawn(move || {
                start_stat_server();
            });

            let _ = thread.join();
            let _ = server_thread.join();
        }
        None => println!("Missing argument")
    }
}

fn parse_address(raw_address: String) -> String {
    if raw_address.contains("http://") || raw_address.contains("https://") {
        return raw_address;
    } else {
        return format!("https://{}", raw_address);
    }
}