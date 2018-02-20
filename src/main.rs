extern crate ansi_term;
extern crate crossbeam;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate native_tls;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

use std::{env, thread};
use std::fs::File;
use std::io::Read;

mod connector;
mod statistics_server;
mod link_tree;

use connector::Connector;

fn main() {
    let mut raw_address: Option<String> = None;
    let mut file_extensions: Vec<String> = vec![];
    let mut depth: u32 = 5;

    if env::args().len() > 1 {
        let mut arg_iter = env::args().skip(1);
        while let Some(x) = arg_iter.next() {
            if x.eq("-i") {
                file_extensions = get_ignored_file_extensions();

                for x in &file_extensions {
                    println!("Ignoring: {}", x);
                }
            } else if x.eq("-d") {
                match arg_iter.next() {
                    Some(x) => {
                        depth = x.parse().unwrap();
                    }
                    None => println!("-d: No argument specified."),
                }
            } else {
                raw_address = Some(x.to_owned());
            }
        }

        match raw_address {
            Some(arg) => {
                let address = parse_address(arg);

                let thread = thread::spawn(move || {
                    Connector::new().run(&address, &file_extensions, &depth);
                });

                let _ = thread.join();
            }
            _ => println!("Missing argument"),
        }
    } else {
        println!("Usage: bla-bla")
    }
}

fn parse_address(raw_address: String) -> String {
    if !(raw_address.contains("http://") || raw_address.contains("https://")) {
        return format!("https://{}", raw_address);
    }
    raw_address
}

fn get_ignored_file_extensions() -> Vec<String> {
    let filename = "ignored_extensions.txt";
    let mut f = File::open(filename).expect("\"ignored_extensions.txt\" file not found.");
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("Something went wrong reading the file");
    contents.split("\r\n").map(|s| s.to_owned()).collect()
}
