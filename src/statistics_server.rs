pub mod statistics_server {
    use std::net::{TcpListener, TcpStream};
    use std::io::Read;
    use std::io::Write;

//    struct StatStruct {
//        number: i32,
//        string: &'static str
//    }
//
//    static DATA_STRUCT: StatStruct = StatStruct {
//        number: 10,
//        string: "test",
//    };
    pub static mut MUTABLE_DATA: &str = "qwe";

    pub fn update(new_data: &String) {
        unsafe {
//            MUTABLE_DATA = &new_data.clone()[..];
        }
    }

    pub fn listen() {
//        let content = "<html>\n\
//                            <body>\n\
//                            <h1>heh mda hih</h1>\n\
//                            </body>\n\
//                            </html>";
        let mut response_header: String;

        let listener = TcpListener::bind("127.0.0.1:80").unwrap();
        loop {
            let sock = listener.accept();
            unsafe {
                let content = MUTABLE_DATA.to_string();
                response_header = format!("HTTP/1.1 200 OK\r\n\
                                            Date: Mon, 27 Jul 2009 12:28:53 GMT\r\n\
                                            Server: Apache/2.2.14 (Win32)\r\n\
                                            Last-Modified: Wed, 22 Jul 2009 19:15:56 GMT\r\n\
                                            Content-Length: {}\r\n\
                                            Content-Type: text/html\r\n\
                                            Connection: close\r\n\
                                            \r\n\
                                            {}", content.len(), content);
            }

            let tuple = sock.unwrap();
            let mut sock = tuple.0;
            let address = tuple.1;

            println!("Connection from {}", address);

            sock.write_all(response_header.as_bytes());
        }
    }
}