use std::net::TcpListener;
use std::io::{Write, Read};
use link_tree::LinkTreeNode;
use std::thread;
use native_tls::{Pkcs12, TlsAcceptor};
use std::sync::Arc;
use std::fs::File;

macro_rules! build_response {
        ($x:expr) => {
            format!("HTTP/1.1 200 OK\r\n\
                     Date: Mon, 27 Jul 2009 12:28:53 GMT\r\n\
                     Server: Apache/2.2.14 (Win32)\r\n\
                     Last-Modified: Wed, 22 Jul 2009 19:15:56 GMT\r\n\
                     Content-Length: {}\r\n\
                     Content-Type: text/html\r\n\
                     Connection: close\r\n\
                     \r\n\
                     {}", $x.len(), $x)
        };
    }

#[derive(Clone)]
pub struct StatStruct {
    pub count: i32,
    pub data_string: String,
    pub link_vector: Vec<String>,
}

pub fn listen(root_node: &LinkTreeNode) {
    let mut file = File::open("pac.pfx").expect("Stat server: PKCS \"pac.pfx\" file not found.");
    let mut pkcs12 = vec![];
    file.read_to_end(&mut pkcs12).unwrap();
    let pkcs12 = Pkcs12::from_der(&pkcs12, "1231231").unwrap();
    let listener = TcpListener::bind("127.0.0.1:8443").unwrap();
    let acceptor = TlsAcceptor::builder(pkcs12).unwrap().build().unwrap();
    let acceptor = Arc::new(acceptor);

    loop {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let acceptor = acceptor.clone();
                    let mut content = "<head>\
                                      <meta charset=\"UTF-8\"> \
                                      </head><pre>".to_owned();
                    content.push_str(&format!("{}\n</pre>", &root_node));
                    thread::spawn(move || {
                        let mut stream = acceptor.accept(stream).unwrap();
                        let response_header = build_response!(content);
                        let _ = stream.write_all(&response_header.as_bytes());
                    });
                }
                Err(e) => { println!("Incomming connection error. {:?}", &e) }
            }
        }
    }
}