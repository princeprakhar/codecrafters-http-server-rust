use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::{env, thread};

fn handle_connection(mut stream: TcpStream, directory: &str) {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut request_line = String::new();

    if buf_reader.read_line(&mut request_line).is_err() {
        return;
    }

    let request_parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if request_parts.len() < 2 {
        stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n").unwrap();
        return;
    }

    let method = request_parts[0];
    let path = request_parts[1];

    match method {
        "GET" => {
            if path.starts_with("/files/") {
                let filename = &path[7..];
                let file_path = Path::new(directory).join(filename);
                if file_path.exists() {
                    match fs::read(&file_path) {
                        Ok(content) => {
                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
                                content.len()
                            );
                            stream.write_all(response.as_bytes()).unwrap();
                            stream.write_all(&content).unwrap();
                        },
                        Err(_) => {
                            stream.write_all(b"HTTP/1.1 500 Internal Server Error\r\n\r\n").unwrap();
                        }
                    }
                } else {
                    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                }
            } else {
                // Handle other GET requests (/, /echo, /user-agent) as before
                // ...
            }
        },
        "POST" => {
            if path.starts_with("/files/") {
                let filename = &path[7..];
                let file_path = format!("{}/{}", directory, filename);

                let mut content = Vec::new();
                buf_reader.read_to_end(&mut content).unwrap();

                match File::create(&file_path) {
                    Ok(mut file) => {
                        if file.write_all(&content).is_ok() {
                            stream.write_all(b"HTTP/1.1 201 Created\r\n\r\n").unwrap();
                        } else {
                            stream.write_all(b"HTTP/1.1 500 Internal Server Error\r\n\r\n").unwrap();
                        }
                    },
                    Err(_) => {
                        stream.write_all(b"HTTP/1.1 500 Internal Server Error\r\n\r\n").unwrap();
                    }
                }
            } else {
                stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
            }
        },
        _ => {
            stream.write_all(b"HTTP/1.1 405 Method Not Allowed\r\n\r\n").unwrap();
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut directory = String::new();

    for i in 1..args.len() {
        if args[i] == "--directory" && i + 1 < args.len() {
            directory = args[i + 1].clone();
            break;
        }
    }

    if directory.is_empty() {
        eprintln!("Error: --directory flag is required");
        std::process::exit(1);
    }

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    println!("Server is running on port 4221...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let directory_clone = directory.clone();
                thread::spawn(move || {
                    handle_connection(stream, &directory_clone);
                });
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}