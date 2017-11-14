extern crate rand;
use rand::distributions::{IndependentSample, Range};
use std::net::TcpStream;
use std::io::{Write, Read};
use std::str;
use std::env;

static UA: &'static [&'static str] = &[
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/40.0.2214.85 Safari/537.36",
    "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/45.0.2454.85 Safari/537.36",
    "Mozilla/5.0 (Windows NT 6.2; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/39.0.2171.95 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.10; rv:34.0) Gecko/20100101 Firefox/34.0",
    "Mozilla/5.0 (Windows NT 6.3; WOW64; rv:34.0) Gecko/20100101 Firefox/34.0",
    "Mozilla/5.0 (Windows NT 6.1; WOW64; rv:30.0) Gecko/20100101 Firefox/30.0",
];

fn main() {
    let address = env::args().nth(1).expect("Missing argument");
    println!("Address: {}\n", address);
    get_links(address)
    //    match get_links(address) {
//        Ok(r) => println!("Ok: {}", r),
//        Err(e) => println!("Err: {}", e)
//    }
}

fn get_links(address: String) {
    let mut sock = TcpStream::connect(format!("{}:{}", address, 80))
        .expect("Couldn't connect to the server...");
    sock.set_read_timeout(None).expect("set_read_timeout call failed");

    let between = Range::new(0, UA.len());
    let mut rng = rand::thread_rng();
    let request: String = format!("GET / HTTP/1.1\r\n\
        Host: {}\r\n\
        Accept: text/css,*/*;q=0.1\r\n\
        Accept-Language: ru-RU,ru;q=0.8,en-US;q=0.5,en;q=0.3\r\n\
        Accept-Encoding: gzip, deflate, br\r\n\
        Referer: https://02ch.net/b/\r\n\
        DNT: 1\r\n\
        Connection: keep-alive\r\n\
        User-Agent: {}\r\n\r\n", address, UA[between.ind_sample(&mut rng)]);
    println!("Sending request: \n===========\n{}\n===========\n", request);
    sock.write_all(request.as_bytes());

//    let mut buf = String::new();
//    sock.read_to_string(&mut buf)?;

//    println!("Reading response...\n {}", buf);
//    let mut buffer = [0; 128];
//    let _ = sock.read(&mut buffer);
//
//    println!("{}", str::from_utf8(&buffer).unwrap().to_owned());

//    let mut tempBuff = [8];
//    while sock.read(&tempBuff) {
//        buffer
//    }

//    Ok("".to_owned())
}

//fn hello<'a>(i : i32) -> std::io::Result<Cow<'a, String>> {
//    let mut sock = TcpStream::connect("127.0.0.1:7834")?;
//    sock.write_all("zaz!".as_bytes())?;
//    println!("Data was sent...");
//
////    let buf = String::new();
////    sock.read_to_string(&mut buf)?;
//
//    let mut buffer = [0; 8];
//    let _ = sock.read(&mut buffer);
//    Ok(Cow::Owned(str::from_utf8(&buffer).unwrap().to_owned())) //Return &str
////    Ok(str::from_utf8(&buffer).unwrap().to_owned()) //Return String
//
////    let (main_tx, main_rx) = channel::<Vec<u8>>();
////    let mut stream = TcpStream::connect("127.0.0.1:7834").unwrap();
//////    let _ = stream.write(&[1]);
//////    let a = stream.read(&mut[0; 128]);
////
////    let mut t = Vec::new();
////    match main_rx.recv() {
////        Ok(data) => {
////            let mut buffer = [0; 1024];
////            let _ = stream.write(&data.into_boxed_slice());
////            let _ = stream.read(&mut buffer);
////
////            for i in buffer.iter() { t.push(*i); }
////            let _ = thread_tx.send(t);
////        }
////
////        Err(_) => return, // This means, that the sender has disconnected
//        // and no further messages can ever be received
////    }
////
////    println!("{:?}", t)
//}
