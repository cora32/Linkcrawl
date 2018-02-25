extern crate ansi_term;
extern crate crossbeam;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate image;
extern crate imageproc;
extern crate native_tls;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
extern crate rusttype;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

use std::{env, thread};
use std::fs::File;
use std::io::Read;

mod connector;
mod statistics_server;
mod link_tree;

use connector::Connector;
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_line_segment_mut;
use imageproc::drawing::draw_antialiased_line_segment;
use imageproc::drawing::draw_cross_mut;
use imageproc::drawing::draw_cubic_bezier_curve_mut;
use imageproc::drawing::draw_text_mut;
use rusttype::{FontCollection, Scale};

fn test() {
    use imageproc::pixelops::interpolate;
    let mut img = RgbImage::new(100, 100);
    // let red = Rgb([255, 0, 0]);
    // let green = Rgb([0, 255, 0]);

    // We'll create an 800 pixel wide gradient image.
    // let left_weight = |x| x as f32 / 800.0;

    // let naive_blend = |x| interpolate(red, green, left_weight(x));

    // let mut naive_image = ImageBuffer::new(800, 400);
    // for y in 0..naive_image.height() {
    //     for x in 0..naive_image.width() {
    //         naive_image.put_pixel(x, y, naive_blend(x));
    //     }
    // }
    // naive_image.save("naive_blend.png").unwrap();

    // let gamma = 2.2f32;
    // let gamma_inv = 1.0 / gamma;

    // let gamma_blend_channel = |l, r, w| {
    //     let l = (l as f32).powf(gamma);
    //     let r = (r as f32).powf(gamma);
    //     let s: f32 = l * w + r * (1.0 - w);
    //     s.powf(gamma_inv) as u8
    // };

    draw_line_segment_mut(
        &mut img,
        (5f32, 5f32),              // start point
        (95f32, 95f32),            // end point
        Rgb([69u8, 203u8, 133u8]), // RGB colors
    );

    // let color = image::Luma([2u8]);
    draw_antialiased_line_segment(
        &mut img,
        (5i32, 8i32),   // start point
        (5i32, 100i32), // end point
        Rgb([69u8, 203u8, 133u8]),
        interpolate,
    );
    draw_cross_mut(&mut img, Rgb([69u8, 203u8, 133u8]), 34, 65);
    draw_cubic_bezier_curve_mut(
        &mut img,
        (5f32, 8f32),    // start point
        (91f32, 100f32), // end point
        (55f32, 1f32),   // start point
        (25f32, 100f32), // end point
        Rgb([69u8, 203u8, 133u8]),
    );
    let height = 12.4;
    let scale = Scale {
        x: height * 1.0,
        y: height,
    };
    let font = Vec::from(include_bytes!("DejaVuSans.ttf") as &[u8]);
let font = FontCollection::from_bytes(font).into_font().unwrap();
    draw_text_mut(
        &mut img,
        Rgb([69u8, 203u8, 133u8]),
        25u32,
        55u32,
        scale,
        &font,
        "Hello, world!",
    );
    img.save("fractal.png").unwrap();

    // let width = 800;
    // let height = 800;
    // let mut imgbuf = image::ImageBuffer::new(width, height);

    // let imgx = 800;
    // let imgy = 800;
    // let max_iterations = 256u16;

    // let scalex = 4.0 / imgx as f32;
    // let scaley = 4.0 / imgy as f32;

    // // Create a new ImgBuf with width: imgx and height: imgy
    // let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    // // Iterate over the coordinates and pixels of the image
    // // for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
    // //     let cy = y as f32 * scaley - 2.0;
    // //     let cx = x as f32 * scalex - 2.0;

    // //     let mut z = Complex::new(cx, cy);
    // //     let c = Complex::new(-0.4, 0.6);

    // //     let mut i = 0;

    // //     for t in 0..max_iterations {
    // //         if z.norm() > 3.0 {
    // //             break
    // //         }
    // //         z = z * z + c;
    // //         i = t;
    // //     }

    // //     // Create an 8bit pixel of type Luma and value i
    // //     // and assign in to the pixel at position (x, y)
    // //     *pixel = image::Luma([i as u8]);

    // // }

    // // Save the image as “fractal.png”
    // let ref mut fout = File::create("fractal.png").unwrap();

    // // We must indicate the image's color type and what format to save as
    // image::ImageLuma8(imgbuf).save(fout, image::PNG).unwrap();
}

fn main() {
    // test();
    let mut raw_address: Option<String> = None;
    let mut file_extensions: Vec<String> = vec![];
    let mut depth: u32 = 5;

    if env::args().len() > 1 {
        let mut arg_iter = env::args().skip(1);
        while let Some(x) = arg_iter.next() {
            if x.eq("-i") {
                file_extensions = get_ignored_file_extensions();

                for x in &file_extensions {
                    println!("Ignoring: {}", x);
                }
            } else if x.eq("-d") {
                match arg_iter.next() {
                    Some(x) => {
                        depth = x.parse().unwrap();
                        depth -= 1;
                    }
                    None => println!("-d: No argument specified."),
                }
            } else {
                raw_address = Some(x.to_owned());
            }
        }

        match raw_address {
            Some(arg) => {
                let address = parse_address(arg);

                let thread = thread::spawn(move || {
                    Connector::new().run(&address, &file_extensions, &depth);
                });

                let _ = thread.join();
            }
            _ => println!("Missing argument"),
        }
    } else {
        println!("Usage: bla-bla")
    }
}

fn parse_address(raw_address: String) -> String {
    if !(raw_address.contains("http://") || raw_address.contains("https://")) {
        return format!("https://{}", raw_address);
    }
    raw_address
}

fn get_ignored_file_extensions() -> Vec<String> {
    let filename = "ignored_extensions.txt";
    let mut f = File::open(filename).expect("\"ignored_extensions.txt\" file not found.");
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("Something went wrong reading the file");
    contents.split("\r\n").map(|s| s.to_owned()).collect()
}
