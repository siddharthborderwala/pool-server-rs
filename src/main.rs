use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook::flag::register;

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
    let port = std::env::var("PORT")
        .expect("PORT env variable not found")
        .parse::<u16>()
        .unwrap();
    // setup the listener
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).expect("Could not bind");
    // announce listening
    println!("🚀 Server listening at {}", listener.local_addr().unwrap());

    // listen for SIGTERM and SIGINT and gracefully stop application
    let running = Arc::new(AtomicBool::new(true));
    register(SIGTERM, Arc::clone(&running)).expect("Failed to register SIGTERM handler");
    register(SIGINT, Arc::clone(&running)).expect("Failed to register SIGINT handler");

    let (sender, recver) = mpsc::channel::<bool>();

    thread::spawn(move || {
        // create a thread pool
        let pool = Arc::new(ThreadPool::new(8));

        for msg in recver {
            if !msg {
                return;
            } else {
                for stream in listener.incoming() {
                    let stream = stream.unwrap();
                    pool.execute(move || handle_connection(stream));
                }
            }
        }
    });

    sender.send(true).unwrap();

    loop {
        if !running.load(Ordering::Relaxed) {
            sender.send(false).unwrap();
        }
    }
}
