extern crate hyper_tls;
extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate regex;
#[macro_use]
extern crate lazy_static;

use std::str;
use std::env;
use std::io;
use std::time::Duration;
use futures::{Future, Stream};
use futures::future::Either;
use hyper::Client;
use hyper::Uri;
use tokio_core::reactor::Core;
use tokio_core::reactor::Timeout;
use tokio_core::reactor::Handle;
use hyper_tls::HttpsConnector;
use hyper::Body;
use hyper::client::HttpConnector;
use regex::Regex;
use std::error::Error;

struct Connector {
    client: Client<HttpsConnector<HttpConnector>, Body>,
    core: Core,
    handle: Handle,
}
lazy_static! {
            static ref RE: Regex = Regex::new(r#"href="(.*?)""#).unwrap();
            }

impl Connector {
    fn get_res(&mut self, address: &String) -> Option<hyper::Response<Body>> {
        let uri: Uri = address.parse().unwrap();
        println!("Connecting to: {}", uri);

        let request = self.client.get(uri).map(|res| res);
        let timeout = Timeout::new(Duration::from_secs(2), &self.handle).unwrap();
        let work = request.select2(timeout)
            .then(|res| {
                match res {
                    Ok(Either::A((got, _timeout))) => {
                        Ok(got)
                    }
                    Ok(Either::B((_timeout_error, _get))) => {
                        Err(hyper::Error::Io(io::Error::new(
                            io::ErrorKind::TimedOut,
                            "Client timed out while connecting",
                        )))
                    }
                    Err(Either::A((get_error, _timeout))) => Err(get_error),
                    Err(Either::B((timeout_error, _get))) => Err(From::from(timeout_error)),
                }
            }).map(|res| res);

        match self.core.run(work) {
            Ok(r) => Some(r),
            Err(e) => {
                println!("Cannot connect to {}: {}", address, e.description());
                None
            }
        }
    }

    fn parse_body<'a>(&'a mut self, body: &String) {
        let link_vector = RE.captures_iter(body).collect::<Vec<_>>();
        let mut res = vec![""];
        for link in link_vector.iter() {
            let string = link.get(1).map_or("", |m| m.as_str());
//            println!("Temp: {:?}", string);
            res.push(string);
        }

        res.sort();
        res.dedup();

        for link in res.iter() {
            println!("Result: {}", link);
            if link.len() == 0 {
                println!("Result: Empty!");
            }
        }
    }

    fn get_redirected_response<'a>(&'a mut self,
                                   _response: hyper::Response<Body>) -> hyper::Response<Body> {
        let mut loop_counter = 0;
        let mut response = _response;
        while response.status() == hyper::StatusCode::MovedPermanently
            || response.status() == hyper::StatusCode::TemporaryRedirect
            || response.status() == hyper::StatusCode::PermanentRedirect {
            if loop_counter == 5 {
                println!("Redirection loop");
                break;
            }

            let new_location = str::from_utf8(response.headers()
                .get_raw("Location")
                .unwrap()
                .one()
                .unwrap())
                .unwrap()
                .to_owned();

            match self.get_res(&new_location) {
                Some(_response) => { response = _response },
                _ => {}
            }
            loop_counter += 1;
        }

        response
    }

    fn run<'a>(&'a mut self, address: &String) {
        let result = self.get_res(address);

        match result {
            Some(response_with_location) => {
                let response = self.get_redirected_response(response_with_location);

                let body_string = response.body().concat2().map(|chunk| {
                    let v = chunk.to_vec();
                    String::from_utf8_lossy(&v).to_string()
                });
                let run = self.core.run(body_string);

                match run {
                    Ok(r) => {
                        self.parse_body(&r);
                    }
                    Err(e) => {
                        println!("Error {:?}", &e);
                    }
                }
            },
            _ => {}
        }
    }

    pub fn new() -> Connector {
        let core = Core::new().unwrap();
        let handle = core.handle();
        Connector {
            core,
            client: Client::configure()
                .connector(HttpsConnector::new(4, &handle).unwrap())
                .build(&handle),
            handle,
        }
    }
}

fn main() {
    let raw_address = env::args().nth(1);
    match raw_address {
        Some(arg) => {
            let address = parse_address(arg);

            let mut connector = Connector::new();
            connector.run(&address);
        },
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