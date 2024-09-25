use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;

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
            let response = match path {
                "/" => "HTTP/1.1 200 OK\r\n\r\n".to_string(),
                path if path.starts_with("/echo/") => {
                    let message = &path[6..];
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                        message.len(),
                        message
                    )
                },
                "/user-agent" => {
                    let user_agent = request_parts.iter()
                        .find(|&&header| header.starts_with("User-Agent:"))
                        .map(|&header| &header[12..])
                        .unwrap_or("");
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                        user_agent.len(),
                        user_agent
                    )
                },
                path if path.starts_with("/files/") => {
                    let filename = &path[7..];
                    let file_path = format!("{}/{}", directory, filename);
                    if Path::new(&file_path).exists() {
                        match fs::read(&file_path) {
                            Ok(content) => format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
                                content.len()
                            ) + &String::from_utf8_lossy(&content),
                            Err(_) => "HTTP/1.1 500 Internal Server Error\r\n\r\n".to_string(),
                        }
                    } else {
                        "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
                    }
                },
                _ => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
            };
            stream.write_all(response.as_bytes()).unwrap();
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
    let directory = "your_directory_path"; // Replace with your actual directory

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    println!("Server is running on port 4221...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let directory_clone = directory.to_string();
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