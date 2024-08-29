use std::net::TcpListener;
use std::{io::Write, net::TcpListener};
fn main() {
    println!("Logs from your program will appear here!");
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let response = String::from("HTTP/1.1 200 OK\r\n\r\n");
                _stream
                    .write_all(response.as_bytes())
                    .expect("could not send response");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}