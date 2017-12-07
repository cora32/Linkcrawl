use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::{Write, Error};
use link_tree::LinkTreeNode;
use std::thread;
use std::time::Duration;

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

pub fn listen(tt: &LinkTreeNode) {
    let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    let wait_time = Duration::from_millis(10);
    loop {
        let sock = listener.accept();
        let mut content = "<head>\
                                      <meta charset=\"UTF-8\"> \
                                      </head><pre>".to_owned();
        content.push_str(&format!("{}\n</pre>", &tt));

        let response_header = build_response!(content);
        send_data_to_client(&sock, &response_header.as_bytes());
        // Sometimes the connection is closed before browser expects it.
        // To prevent the "Connection was closed" error, a little sleep is used.
        thread::sleep(wait_time);
    }
}

fn send_data_to_client(sock: &Result<(TcpStream, SocketAddr), Error>, data: &[u8]) {
    let tuple = sock.as_ref().unwrap();
    let mut stream = &tuple.0;
    let address = tuple.1;

    println!("Connection from {}", address);

    let _ = stream.write_all(data);
}