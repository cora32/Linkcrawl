extern crate hyper_tls;
extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate regex;
#[macro_use]
extern crate lazy_static;

use std::str;
use std::env;
use std::time::Duration;
use futures::{Future, Stream};
use futures::future::Either;
use futures::future;
use std::io::{self, Write};
use hyper::Client;
use hyper::Uri;
use tokio_core::reactor::Core;
use tokio_core::reactor::Timeout;
use tokio_core::reactor::Handle;
use hyper_tls::HttpsConnector;
use hyper::Body;
use hyper::client::HttpConnector;
use regex::Regex;
use std::borrow::Cow;
use std::error::Error;

struct Connector {
    client: Client<HttpsConnector<HttpConnector>, Body>,
    timeout: Timeout,
    core: Core,
    handle: Handle,
}
lazy_static! {
            static ref RE: Regex = Regex::new(r#"href="(.*?)""#).unwrap();
            }

impl Connector {
    fn get_body<'a>(&'a mut self, address: &String) -> Result<Future, Error> {
        let uri: Uri = address.parse().unwrap();
        println!("Address: {}", uri);

        let request = self.client.get(uri).map(|res| res);
        let timeout = Timeout::new(Duration::from_secs(2), &self.handle).unwrap();
        let work = request.select2(timeout)
            .then(|res| {
            match res {
                Ok(Either::A((got, _timeout))) => {
                    println!("OK:");
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
        }).and_then(|res| {
            if res.status() == hyper::StatusCode::MovedPermanently {
                let new_url = str::from_utf8(res.headers().get_raw("Location")
                    .unwrap().one().unwrap()).unwrap();
                println!("302!");
            }

            res.body().for_each(|chunk| {
                io::stdout()
                    .write_all(&chunk)
                    .map_err(From::from)
            }).map(|x| {
                println!("Done");
                String::from("Hello, world!")
            })
        });

        let res = self.core.run(work);
        res
    }

    fn parse_body<'a>(&'a mut self, body: &String) {
        let link_vector = RE.captures_iter(body).collect::<Vec<_>>();
        let mut res = vec![""];
        for link in link_vector.iter() {
            let string = link.get(1).map_or("", |m| m.as_str());
            println!("Temp: {:?}", string);
            res.push(string);
        }

        res.sort();
        res.dedup();

        for link in res.iter() {
            println!("Result: {}", link);
        }
    }

    fn connect<'a>(&'a mut self, address: &String) {
        let body = self.get_body(address);
        //        match body {
        //            Ok(r) => {
        //                self.parse_body(&r);
        //            }
        //            Err(e) => {
        //                self.connect(&e)
        //            }
        //        }
    }

    pub fn new() -> Connector {
        let core = Core::new().unwrap();
        let handle = core.handle();
        Connector {
            core,
            client: Client::configure()
                .connector(HttpsConnector::new(4, &handle).unwrap())
                .build(&handle),
            timeout: Timeout::new(Duration::from_secs(2), &handle).unwrap(),
            handle,
        }
    }
}

fn main() {
    let raw_address = env::args().nth(1).expect("Missing argument");
    let address = parse_address(raw_address);

    let mut connector = Connector::new();
    connector.connect(&address);
}

fn parse_address(raw_address: String) -> String {
    if raw_address.contains("http://") || raw_address.contains("https://") {
        return raw_address;
    } else {
        return format!("https://{}", raw_address);
    }
}