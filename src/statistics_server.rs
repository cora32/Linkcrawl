use std::net::TcpListener;
use std::io::{Read, Write};
use link_tree::LinkTreeNode;
use std::thread;
use native_tls::{Pkcs12, TlsAcceptor};
use std::sync::Arc;
use std::fs::File;
use serde_json;
use std::collections::HashMap;
use imageproc::drawing::draw_text_mut;
use imageproc::drawing::draw_line_segment_mut;
use rusttype::{FontCollection, Scale};

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

pub fn listen(
    root_node: &LinkTreeNode,
    width: &u32,
    height: &u32,
    depth_map: &mut HashMap<u32, u32>,
) {
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

                    draw_tree_png(root_node, width, height, depth_map);
                    println!("height! {} width! {}", height, width)
                }
                Err(e) => println!("Incoming connection error. {:?}", &e),
            }
        }
    }
}

use std::collections::VecDeque;
fn draw_tree_png(
    root_node: &LinkTreeNode,
    width: &u32,
    height: &u32,
    depth_map: &HashMap<u32, u32>,
) {
    use image::{Rgb, RgbImage};
    use imageproc::drawing::draw_filled_circle_mut;

    let node_height = 40;
    let node_width = 40;
    let mut radius = node_height / 2;
    let img_width = width * node_width;
    let img_height = height * node_height * 4;
    let half_height = img_height / 2;
    let half_width = img_width / 2;
    let mut img = RgbImage::new(img_width, img_height);

    let mut stack = VecDeque::new();
    stack.push_back(root_node);

    let text_height = 14.0;
    let text_half_height = text_height / 2.0;
    let scale = Scale {
        x: text_height * 1.0,
        y: text_height,
    };
    let font = Vec::from(include_bytes!("DejaVuSans.ttf") as &[u8]);
    let font = FontCollection::from_bytes(font).into_font().unwrap();
    let mut previous_depth_value = 0;
    let mut x_step = img_width / 2;
    let mut center_x = x_step;
    let mut center_y = radius;
    let mut center_x_to = center_x;
    let mut center_y_to = center_y;
    while let Some(node) = stack.pop_front() {
        if *node.depth() != previous_depth_value {
            previous_depth_value = *node.depth();
            match depth_map.get(&previous_depth_value) {
                Some(r) => {
                    center_x_to = center_x - x_step;
                    x_step = img_width / (r + 1);
                    center_x = x_step;
                    center_y_to = center_y;
                    center_y += 4 * radius;
                    // radius = x_step / 2;
                    println!(
                        "center_x {} center_y {}; {} items on level {}; id {}",
                        center_x,
                        center_y,
                        r,
                        &previous_depth_value,
                        *node.get_id()
                    );
                }
                None => {
                    println!(" -- FAIL WTF {}", previous_depth_value);
                }
            }
        } else {
            match depth_map.get(&previous_depth_value) {
                Some(r) => {
                    println!(
                        "(inner) center_x {} center_y {}; {} items on level {}; id {}",
                        center_x,
                        center_y,
                        r,
                        &previous_depth_value,
                        *node.get_id()
                    );
                }
                None => {
                    println!(
                        "(inner) center_x {} center_y {}; {} items on level {}; id {}",
                        center_x,
                        center_y,
                        1,
                        &previous_depth_value,
                        *node.get_id()
                    );
                }
            }
        }

        draw_line_segment_mut(
            &mut img,
            (center_x as f32, center_y as f32),
            (center_x_to as f32, center_y_to as f32),
            Rgb([255u8, 103u8, 33u8]),
        );

        draw_filled_circle_mut(
            &mut img,
            (center_x as i32, center_y as i32),
            radius as i32,
            Rgb([69u8, 203u8, 133u8]),
        );

        draw_text_mut(
            &mut img,
            Rgb([99u8, 103u8, 233u8]),
            center_x - text_half_height as u32,
            center_y - text_half_height as u32,
            scale,
            &font,
            &node.get_id().to_string().to_owned(),
        );

        center_x += x_step;

        for r in node.node_list_immutable() {
            stack.push_back(&r);
        }
    }

    img.save("fractal.png").unwrap();
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
