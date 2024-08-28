#[allow(unused_imports)]
use std::net::TcpListener;
use std::io::Write;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
     
    for stream in listener.incoming() {
        match stream {
            Ok( _stream) => {
                let mut mut_stream = _stream;
                println!("accepted new connection");
                mut_stream.write(b"HTTP/1.1 200 OK\r\n\r\n").expect("404 OK\r\n\r\n");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
