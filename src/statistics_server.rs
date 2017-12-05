use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::{Write, Error};
use std::sync::RwLock;

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

lazy_static! {
        pub static ref MUTEX_STAT_STRUCT:RwLock<Option<StatStruct>>= RwLock::new(None);
    }

#[derive(Clone)]
pub struct StatStruct {
    pub count: i32,
    pub data_string: String,
    pub link_vector: Vec<String>,
}

pub fn update(new_data: StatStruct) {
    *MUTEX_STAT_STRUCT.write().unwrap() = Some(new_data);
}

pub fn listen() {
    let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    loop {
        let sock = listener.accept();

        let temp_live_prolonger = &MUTEX_STAT_STRUCT.try_read().unwrap();
        let option: Option<&StatStruct> = temp_live_prolonger.as_ref();

        match option {
            Some(r) => {
                let mut content = "<head>\
                                      <meta charset=\"UTF-8\"> \
                                      </head><pre>".to_owned();
                content.push_str(&r.data_string);

                let mut vec_string: String = "".to_owned();
                for item in &r.link_vector {
                    vec_string.push_str(item);
                    vec_string.push_str("\n");
                }

                content.push_str("\n");
                content.push_str(&vec_string);
                content.push_str("</pre>");

                let response_header = build_response!(content);
                send_data_to_client(&sock, &response_header.as_bytes());
            }
            _ => {
                let content = "No data";
                let response_header = build_response!(content);
                send_data_to_client(&sock, &response_header.as_bytes());
            }
        }
    }
}

fn send_data_to_client(sock: &Result<(TcpStream, SocketAddr), Error>, data: &[u8]) {
    let tuple = sock.as_ref().unwrap();
    let mut stream = &tuple.0;
    let address = tuple.1;

    println!("Connection from {}", address);

    let _ = stream.write_all(data);
}