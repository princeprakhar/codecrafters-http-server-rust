use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::{env, thread};

fn supports_gzip(headers: &[String]) -> bool {
    headers.iter().any(|header| {
        header.to_lowercase().starts_with("accept-encoding") && 
        header.to_lowercase().contains("gzip")
    })
}

fn handle_connection(mut stream: TcpStream, directory: &str) {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    let mut headers = Vec::new();

    if buf_reader.read_line(&mut request_line).is_err() {
        return;
    }

    // Read headers
    loop {
        let mut header = String::new();
        if buf_reader.read_line(&mut header).is_err() {
            return;
        }
        if header == "\r\n" {
            break;
        }
        headers.push(header.trim().to_string());
    }

    let request_parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if request_parts.len() < 2 {
        stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n").unwrap();
        return;
    }

    let method = request_parts[0];
    let path = request_parts[1];

    let gzip_supported = supports_gzip(&headers);

    match method {
        "GET" => {
            if path == "/" {
                stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
            } else if path.starts_with("/echo/") {
                let echo_content = &path[6..];
                let response = if gzip_supported {
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\n\r\n{}",
                        echo_content.len(),
                        echo_content
                    )
                } else {
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                        echo_content.len(),
                        echo_content
                    )
                };
                stream.write_all(response.as_bytes()).unwrap();
            } else if path == "/user-agent" {
                let user_agent = headers.iter()
                    .find(|h| h.starts_with("User-Agent: "))
                    .map(|h| &h[12..])
                    .unwrap_or("");
                let response = if gzip_supported {
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\n\r\n{}",
                        user_agent.len(),
                        user_agent
                    )
                } else {
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                        user_agent.len(),
                        user_agent
                    )
                };
                stream.write_all(response.as_bytes()).unwrap();
            } else if path.starts_with("/files/") {
                let filename = &path[7..];
                let file_path = Path::new(directory).join(filename);
                if file_path.exists() {
                    match fs::read(&file_path) {
                        Ok(content) => {
                            let response = if gzip_supported {
                                format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\n\r\n",
                                    content.len()
                                )
                            } else {
                                format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
                                    content.len()
                                )
                            };
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

                // Read content length
                let content_length = headers.iter()
                    .find(|h| h.starts_with("Content-Length: "))
                    .and_then(|h| h[16..].parse::<usize>().ok())
                    .unwrap_or(0);

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