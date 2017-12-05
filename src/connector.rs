use std::{io, str, thread};
use std::error::Error;
use std::time::Duration;
use tokio_core::reactor::{Core, Timeout, Handle};
use hyper_tls::HttpsConnector;
use hyper::{self, Body, Client, Uri};
use hyper::client::HttpConnector;
use regex::Regex;
use futures::{Future, Stream};
use futures::future::Either;
use ansi_term::Colour::*;
use statistics_server::StatStruct as StatStruct;
use link_tree::LinkTreeNode;
use std::sync::RwLock;

pub struct Connector {
    client: Client<HttpsConnector<HttpConnector>, Body>,
    core: Core,
    handle: Handle,
}

lazy_static! {
            static ref SLEEP_TIME: Duration = Duration::from_millis(1000);
            static ref RE: Regex = Regex::new(r#"href="(.*?)""#).unwrap();
            static ref MUTEX_DUPE_VECTOR: RwLock<Vec<String>> = RwLock::new(vec![]);
            }

impl Connector {
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

    /**
    * Find all href links in body.
    */
    fn parse_body(&mut self, parent_link: &String, body: &String, file_extensions: &Vec<String>) -> Vec<String> {
        let captures_vec = RE.captures_iter(body).collect::<Vec<_>>();
        let mut res: Vec<String> = vec![];
        for link in captures_vec {
            let string: String = link.get(1).map(|m| m.as_str().to_owned()).unwrap();
            if !res.contains(&string) {
                res.push(string);
            }
        }

        let mut link_vector: Vec<String> = vec![];
        'outer: for path in &res {
            //Ignore dupes
            if MUTEX_DUPE_VECTOR.try_read().unwrap().contains(path) {
                continue 'outer;
            }

            //Ignoring external links, anchors and js
            if path.contains("https://")
                || path.contains("http://")
                || path.contains("//")
                || path.contains("javascript:")
                || path.contains("#") {
                continue 'outer;
            }

            //Ignoring files if any
            if file_extensions.len() != 0 {
                for x in file_extensions {
                    if path.contains(x) {
                        continue 'outer;
                    }
                }
            }

            let new_link: String = format!("{}{}", parent_link, path);
            if !link_vector.contains(&new_link) {
//                println!("{}", Fixed(034).bold().paint(format!("Pushing: {}", &path)));
                MUTEX_DUPE_VECTOR.write().unwrap().push(path.clone());
                link_vector.push(new_link);
            }
        }

//            println!("{}", Fixed(034).bold().paint(format!("Links: {:?}", &link_vector)));
        link_vector
    }

    /**
    * Follow redirect location for Moved-responses.
    * If loop detected, None is returned; otherwise Response<Body> is returned.
    */
    fn get_redirected_response(&mut self,
                               link: &String,
                               _response: hyper::Response<Body>) -> (String, hyper::Response<Body>) {
        let mut loop_counter = 0;
        let response = _response;
        let mut new_location: String = link.clone();
        while response.status() == hyper::StatusCode::MovedPermanently
            || response.status() == hyper::StatusCode::TemporaryRedirect
            || response.status() == hyper::StatusCode::PermanentRedirect {
            if loop_counter == 5 {
                println!("Redirection loop");
                break;
            }

            new_location = str::from_utf8(response.headers()
                .get_raw("Location")
                .unwrap()
                .one()
                .unwrap())
                .unwrap()
                .to_owned();


            let uri = new_location.parse();
            match uri {
                Ok(r) => {
                    match self.get_body(r) {
                        Some(_response) => return _response,
                        _ => {}
                    }
                }
                Err(e) => {
                    println!("Link is invalid: {}", e.description());
                }
            }
            loop_counter += 1;
        }

        (new_location, response)
    }

    /**
    * Perform non-blocking http request.
    */
    fn get_body(&mut self, uri: Uri) -> Option<(String, hyper::Response<Body>)> {
        println!("{}", Fixed(032).bold().paint(format!("Connecting to: {}", uri)));

        let link_string = uri.to_string();
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
            Ok(r) => Some(self.get_redirected_response(&link_string, r)),
            Err(e) => {
                println!("Failed to connect: {}", e.description());
                None
            }
        }
    }

    /**
    * Return correctly formatted URI as a string.
    */
    fn get_link(&mut self, address: &String) -> String {
        let parent_uri: Uri = address.parse().unwrap();
        let scheme = parent_uri.scheme().unwrap();
        let authority = parent_uri.authority().unwrap();
        format!("{}://{}", scheme, authority)
    }

    /**
    * Requests @link, follows redirects and finds all href links per html body.
    */
    fn get_link_vector(&mut self, link: &String, f: &Fn(StatStruct), file_extensions: &Vec<String>) -> Option<Vec<String>> {
        let uri = link.parse();
        match uri {
            Ok(r) => {
                let response: Option<(String, hyper::Response<Body>)> = self.get_body(r);
                match response {
                    Some(r) => {
                        let parent_link = r.0;
                        let body_string = r.1.body().concat2().map(|chunk| {
                            let v = chunk.to_vec();
                            String::from_utf8_lossy(&v).to_string()
                        });
                        let run_result = self.core.run(body_string);

                        match run_result {
                            Ok(r) => {
                                let new_link_vector = self.parse_body(&parent_link, &r, file_extensions);
                                let mut counter = 0;
                                new_link_vector.iter()
                                    .for_each(|new_link| {
                                        if !MUTEX_DUPE_VECTOR.try_read().unwrap().contains(&new_link) {
                                            MUTEX_DUPE_VECTOR.write().unwrap().push(new_link.clone());
                                            counter += 1;
                                        }
                                    });
                                let data = format!("Found {} links; added {}; links in vector {}", new_link_vector.len(), counter, MUTEX_DUPE_VECTOR.try_read().unwrap().len());
                                println!("{}", &data);
                                f(StatStruct { count: 123, data_string: data, link_vector: MUTEX_DUPE_VECTOR.try_read().unwrap().clone() });
                                return Some(new_link_vector);
                            }
                            Err(e) => {
                                println!("Error {:?}", &e);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Err(e) => {
                println!("Link \"{}\" is invalid: {}", link, e.description());
            }
        }

        None
    }

    /**
    * Adds all new links to parent node as its children.
    */
    fn add_children(&mut self, node: &mut LinkTreeNode, f: &Fn(StatStruct), file_extensions: &Vec<String>) {
        let link_vector = self.get_link_vector(&node.link(), f, file_extensions);

        match link_vector {
            Some(r) => {
                for x in r {
                    node.add_child(LinkTreeNode::create(&x))
                }
            }
            _ => println!("Empty link vector.")
        }

        thread::sleep(*SLEEP_TIME);
    }

    /**
    * Recursively fills parent nodes with corresponding children.
    */
    fn fill_with_data(&mut self, node: &mut LinkTreeNode, parent_link: &String,
                      f: &Fn(StatStruct), file_extensions: &Vec<String>) {
        self.add_children(node, f, file_extensions);

//        println!("{}", node);

        for mut x in node.node_list() {
            self.fill_with_data(&mut x, parent_link, f, file_extensions);
        }
    }

    /**
    * Starts the site parsing logic.
    */
    pub fn run(&mut self, address: &String, f: &Fn(StatStruct), file_extensions: &Vec<String>) {
        let parent_link = self.get_link(&address);
        let mut root = LinkTreeNode::create(&*address);
        self.fill_with_data(&mut root, &parent_link, f, file_extensions);
    }
}