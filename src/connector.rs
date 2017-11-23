pub mod connector {
    use std::{io, str, thread};
    use std::error::Error;
    use std::time::{self, Duration};
    use tokio_core::reactor::{Core, Timeout, Handle};
    use hyper_tls::HttpsConnector;
    use hyper::{self, Body, Client, Uri};
    use hyper::client::HttpConnector;
    use regex::Regex;
    use futures::{Future, Stream};
    use futures::future::Either;
    use ansi_term::Colour::*;

    pub struct Connector {
        client: Client<HttpsConnector<HttpConnector>, Body>,
        core: Core,
        handle: Handle,
    }
    lazy_static! {
            static ref RE: Regex = Regex::new(r#"href="(.*?)""#).unwrap();
            }

    impl Connector {
        fn get_res(&mut self, uri: Uri) -> Option<hyper::Response<Body>> {
            println!("{}", Fixed(032).bold().paint(format!("Connecting to: {}", uri)));

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
                    println!("Failed to connect: {}", e.description());
                    None
                }
            }
        }

        fn parse_body<'a>(&'a mut self, parent_link: &String, body: &String) -> Vec<String> {
            let link_vector = RE.captures_iter(body).collect::<Vec<_>>();
            let mut res = vec![""];
            for link in link_vector.iter() {
                let string = link.get(1).map_or("", |m| m.as_str());
                //            println!("Temp: {:?}", string);
                if !res.contains(&string) {
                    res.push(string);
                }
            }

            //            res.sort();
            //            res.dedup();
            let mut link_vector: Vec<String> = vec![];
            res.iter()
                .for_each(|x| {
                    let path = String::from(*x);
                    if !(path.contains("https://")
                        || path.contains("http://")
                        || path.contains("//")) {
                        let new_link: String = format!("{}{}", parent_link, path);
                        if !link_vector.contains(&new_link) {
                            link_vector.push(new_link);
                        }
                    }
                });

            println!("{}", Fixed(034).bold().paint(format!("Links: {:?}", &link_vector)));
            link_vector
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


                let uri = new_location.parse();
                match uri {
                    Ok(r) => {
                        match self.get_res(r) {
                            Some(_response) => { response = _response }
                            _ => {}
                        }
                    },
                    Err(e) => {
                        println!("Link is invalid: {}", e.description());
                    }
                }
                loop_counter += 1;
            }

            response
        }

        pub fn run<'a>(&'a mut self, address: &String, f: &Fn(&String)) {
            let parent_uri: Uri = address.parse().unwrap();
            let scheme = parent_uri.scheme().unwrap();
            let authority = parent_uri.authority().unwrap();
            let parent_link = format!("{}://{}", scheme, authority);

            let sleep_time = time::Duration::from_millis(10000);
            let mut link_vector = vec![address.clone()];
            let mut index = 0;

            while !link_vector.is_empty() && index < link_vector.len() {
                let mut result = None;
                {
                    let link = &link_vector[index];
                    index += 1;
                    if link.is_empty() {
                        continue;
                    }

                    let uri = link.parse();
                    match uri {
                        Ok(r) => result = self.get_res(r),
                        Err(e) => {
                            println!("Link \"{}\" is invalid: {}", address, e.description());
                        }
                    }
                }

                let new_link_vector: Vec<String>;
                match result {
                    Some(response_with_location) => {
                        let response = self.get_redirected_response(response_with_location);

                        let body_string = response.body().concat2().map(|chunk| {
                            let v = chunk.to_vec();
                            String::from_utf8_lossy(&v).to_string()
                        });
                        let run_result = self.core.run(body_string);

                        match run_result {
                            Ok(r) => {
                                new_link_vector = self.parse_body(&parent_link,&r);
                                let mut counter = 0;
                                new_link_vector.iter()
                                    .for_each(|new_link| {
                                        if !link_vector.contains(&new_link) {
                                            link_vector.push(new_link.clone());
                                            counter += 1;
                                        }
                                    });
                                let data = format!("Found {} links; added {}; links in vector {}; current index {}", new_link_vector.len(), counter, link_vector.len(), index);
                                println!("{}", &data);
                                f(&data);
                            }
                            Err(e) => {
                                println!("Error {:?}", &e);
                            }
                        }
                    }
                    _ => {}
                }

                thread::sleep(sleep_time);
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
}