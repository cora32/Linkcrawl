use std::net::TcpListener;
use std::io::{Read, Write};
use link_tree::LinkTreeNode;
use std::thread;
use native_tls::{Pkcs12, TlsAcceptor};
use std::sync::Arc;
use std::fs::File;
use serde_json;

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
                                       </head><pre>"
                        .to_owned();
                    //                    content.push_str(&format!("{}\n</pre>", &root_node));
                    content.push_str(&format!(
                        "{}\n</pre>",
                        &serde_json::to_string(&root_node).unwrap()
                    ));
                    content.push_str(&format!("{}\n", &get_canvas(&root_node)));
                    content.push_str(
                        "<canvas id=\"myCanvas\" width=\"578\" height=\"200\"></canvas>
                                            <script>
                                              var canvas = document.getElementById('myCanvas');
                                              canvas.width = document.body.clientWidth;
                                              canvas.height = document.body.clientHeight;
                                              var context = canvas.getContext('2d');

                                              context.beginPath();
                                              context.rect(188, 50, 200, 100);
                                              context.fillStyle = 'yellow';
                                              context.fill();
                                              context.lineWidth = 1;
                                              context.strokeStyle = 'black';

                                              context.moveTo(288, 150);
                                              context.lineTo(150, 200);

                                              context.moveTo(288, 150);
                                              context.lineTo(288 + (288 - 150), 200);

                                              context.stroke();
                                            </script>",
                    );
                    thread::spawn(move || {
                        let mut stream = acceptor.accept(stream).unwrap();
                        let response_header = build_response!(content);
                        let _ = stream.write_all(&response_header.as_bytes());
                    });
                }
                Err(e) => println!("Incoming connection error. {:?}", &e),
            }
        }
    }
}

fn get_canvas(root_node: &LinkTreeNode) -> String {
    let node_height = 50;
    //    let node_width = 100;
    //    let x = canvas_width/2 - node_width/2;
    let header = format!(
        "<canvas id=\"myCanvas\"></canvas>\n
    <script>
        var canvas = document.getElementById('myCanvas');
        canvas.width = document.body.clientWidth;
        canvas.height = document.body.clientHeight;
        var context = canvas.getContext('2d');
        var nodeWidth = 100;
        var nodeHeight = {};

        context.beginPath();",
        node_height
    );
    let footer = "context.stroke()
        </script>";
    let data = draw_tree(root_node, 0);

    format!("{}{}{}", &header, &data, &footer)
}

fn draw_tree(node: &LinkTreeNode, y: u32) -> String {
    let mut data: String = "".to_owned();

    //Create rect
    data.push_str(&format!(
        "context.rect({}, {}, nodeWidth, nodeHeight);
        context.fillStyle = 'yellow';
        context.fill();
        context.lineWidth = 1;
        context.strokeStyle = 'black';\n",
        100, y
    ));

    //    let children = node.node_list_immutable();
    //    let link = node.link();
    //    let size = children.len();

    //    for x in children {
    //        //Create lines
    //        data.push_str(&format!("
    //        context.moveTo(288, {});
    //        context.lineTo(150, 200);\n", y));
    //    }

    data
}
