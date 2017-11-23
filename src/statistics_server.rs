pub mod statistics_server {
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
        pub static ref MUTEX_VEC:RwLock<Option<String>>= RwLock::new(None);
    }

    pub fn update(new_data: &String) {
        *MUTEX_VEC.write().unwrap() = Some(new_data.clone());
    }

    pub fn listen() {
        let listener = TcpListener::bind("127.0.0.1:80").unwrap();
        loop {
            let sock = listener.accept();

            let temp_live_prolonger = &MUTEX_VEC.try_read().unwrap();
            let option: Option<&String> = temp_live_prolonger.as_ref();

            match option {
                Some(content) => {
                    let response_header = build_response!(content);
                    send_data_to_client(&sock, &response_header.as_bytes());
                },
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
}