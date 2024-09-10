// use core::str;
#[allow(unused_imports)]
use std::io::{Read, Write, BufRead}; // Import BufRead trait
use std::{io::BufReader, net::{TcpListener, TcpStream}};

fn handle_the_connection(mut stream: TcpStream) {
    // Do something with the stream
    let buf_reader = BufReader::new(&mut stream);
    let response = "HTTP/1.1 200 OK\r\n\r\n";
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    for line in request_line.split("\r\n") {
        let header: Vec<&str> = line.split(" ").collect();
        // println!("TEST: {}", &header[1][6..]);
        if header[0] == "GET" {
            if header[1] == "/" {
                stream
                    .write(response.as_bytes())
                    .expect("200");
            } else if &header[1][..6] == "/echo/" {
                let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-length: {}\r\n\r\n{}", header[1][6..].len(), &header[1][6..]);
                stream.write(response.as_bytes()).expect("200");
            } else {
                stream
                    .write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes())
                    .expect("404");
            }
        }
    }
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
