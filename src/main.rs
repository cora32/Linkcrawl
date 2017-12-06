extern crate hyper_tls;
extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate ansi_term;

use std::{env, thread};
use std::fs::File;
use std::io::Read;

mod connector;
mod statistics_server;
mod link_tree;

use connector::Connector;
use statistics_server::update as update;
use statistics_server::listen as start_stat_server;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut raw_address:Option<String> = None;
    let mut file_extensions: Vec<String> = vec![];
    let mut depth: u32 = 5;
    if env::args().len() > 1  {
        let mut index = 0;
        while index != env::args().len() {
            let x = &args[index];
            if x.eq("-i") {
                file_extensions = get_ignored_file_extensions();

                for x in &file_extensions {
                    println!("Ignoring: {}", x);
                }
            } else if x.eq("-d") {
                index = index + 1;
                let x = &args[index];
                depth = x.parse().unwrap();
            } else {
                raw_address = Some(x.to_owned());
            }
            index = index + 1;
        }
    }

    match raw_address {
        Some(arg) => {
            let address = parse_address(arg);

            let thread = thread::spawn(move || {
                let mut connector = Connector::new();
                connector.run(&address, &update, &file_extensions, &depth);
            });

            let server_thread = thread::spawn(move || {
                start_stat_server();
            });

            let _ = thread.join();
            let _ = server_thread.join();
        },
        _ => println!("Missing argument")
    }
}

fn parse_address(raw_address: String) -> String {
    if raw_address.contains("http://") || raw_address.contains("https://") {
        return raw_address;
    } else {
        return format!("https://{}", raw_address);
    }
}

fn get_ignored_file_extensions() -> Vec<String> {
    let filename = "ignored_extensions.txt";
    let mut f = File::open(filename).expect("\"ignored_extensions.txt\" file not found.");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect("Something went wrong reading the file");
    contents.split("\r\n").map(|s| s.to_owned()).collect()
}