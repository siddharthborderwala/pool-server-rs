use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

use server::ThreadPool;

fn handle_connection(mut stream: TcpStream) {
    let mut buffer: [u8; 1024] = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "views/hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "views/404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    // get the port
    let port = std::env::var("PORT").unwrap();
    // setup the listener
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    // announce listening
    println!("🚀 Server listening at http://127.0.0.1:{}", port);

    let pool = ThreadPool::new(16);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(move || handle_connection(stream));
    }
}
