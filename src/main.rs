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
            if path == "/" {
                stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
            } else if path.starts_with("/echo/") {
                let echo_content = &path[6..];
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                    echo_content.len(),
                    echo_content
                );
                stream.write_all(response.as_bytes()).unwrap();
            } else if path == "/user-agent" {
                let mut user_agent = String::new();
                for line in buf_reader.lines() {
                    let line = line.unwrap();
                    if line.starts_with("User-Agent: ") {
                        user_agent = line[12..].to_string();
                        break;
                    }
                    if line.is_empty() {
                        break;
                    }
                }
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                    user_agent.len(),
                    user_agent
                );
                stream.write_all(response.as_bytes()).unwrap();
            } else if path.starts_with("/files/") {
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
                stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
            }
        },
        "POST" => {
            if path.starts_with("/files/") {
                let filename = &path[7..];
                let file_path = Path::new(directory).join(filename);

                // Read headers
                let mut content_length = 0;
                for line in buf_reader.by_ref().lines() {
                    let line = line.unwrap();
                    if line.starts_with("Content-Length: ") {
                        content_length = line[16..].parse::<usize>().unwrap_or(0);
                    }
                    if line.is_empty() {
                        break;
                    }
                }

                // Read the request body
                let mut content = vec![0; content_length];
                if buf_reader.read_exact(&mut content).is_err() {
                    stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n").unwrap();
                    return;
                }

                // Write content to file
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
    let mut directory = String::from(".");  // Default to current directory

    for i in 1..args.len() {
        if args[i] == "--directory" && i + 1 < args.len() {
            directory = args[i + 1].clone();
            break;
        }
    }

    println!("Using directory: {}", directory);

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