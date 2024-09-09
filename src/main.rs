// use core::str;
#[allow(unused_imports)]
use std::io::{Read, Write, BufRead}; // Import BufRead trait
use std::{io::BufReader, net::{TcpListener, TcpStream}};

fn handle_the_connection(mut stream: TcpStream) {
    // Do something with the stream
    let buf_reader = BufReader::new(&mut stream);
    let response = "HTTP/1.1 200 OK\r\n\r\n";
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    let response = match request_line.as_str() {
        "GET / HTTP/1.1" => "HTTP/1.1 200 OK\r\n\r\n",
        _ => "HTTP/1.1 404 Not Found\r\n\r\n",
    };
    stream.write_all(response.as_bytes()).unwrap();
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
     
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                handle_the_connection(_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
