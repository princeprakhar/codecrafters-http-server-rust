use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::{env, thread};
use mime_guess;

fn handle_connection(mut stream: TcpStream, directory: &str) {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut request_line = String::new();

    // Reading the request line
    if let Err(e) = buf_reader.read_line(&mut request_line) {
        eprintln!("Failed to read request line: {}", e);
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
        // Handle GET requests
        "GET" => {
            if path == "/" {
                let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 13\r\n\r\nHello, world!";
                stream.write_all(response.as_bytes()).unwrap();
            } else if path.starts_with("/files/") {
                let filename = &path[7..];
                let file_path = Path::new(directory).join(filename);

                if file_path.exists() {
                    match fs::read(&file_path) {
                        Ok(content) => {
                            let mime_type = mime_guess::from_path(&file_path).first_or_octet_stream();
                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                                mime_type,
                                content.len()
                            );
                            stream.write_all(response.as_bytes()).unwrap();
                            stream.write_all(&content).unwrap();
                        },
                        Err(e) => {
                            eprintln!("Failed to read file {}: {}", file_path.display(), e);
                            stream.write_all(b"HTTP/1.1 500 Internal Server Error\r\n\r\n").unwrap();
                        }
                    }
                } else {
                    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                }
            } else if path.starts_with("/user-agent") {
                // Echo the User-Agent header
                let mut headers = String::new();
                if buf_reader.read_line(&mut headers).is_ok() {
                    if headers.starts_with("User-Agent:") {
                        let user_agent = headers.trim_start_matches("User-Agent: ").trim();
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                            user_agent.len(),
                            user_agent
                        );
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                } else {
                    stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n").unwrap();
                }
            } else {
                stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
            }
        },

        // Handle POST requests
        "POST" => {
            if path.starts_with("/files/") {
                let filename = &path[7..];
                let file_path = format!("{}/{}", directory, filename);

                let mut content = Vec::new();
                if let Err(e) = buf_reader.read_to_end(&mut content) {
                    eprintln!("Failed to read POST body: {}", e);
                    stream.write_all(b"HTTP/1.1 500 Internal Server Error\r\n\r\n").unwrap();
                    return;
                }

                match File::create(&file_path) {
                    Ok(mut file) => {
                        if file.write_all(&content).is_ok() {
                            stream.write_all(b"HTTP/1.1 201 Created\r\n\r\n").unwrap();
                        } else {
                            stream.write_all(b"HTTP/1.1 500 Internal Server Error\r\n\r\n").unwrap();
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to create file {}: {}", file_path, e);
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

    // Parsing the --directory flag
    for i in 1..args.len() {
        if args[i] == "--directory" && i + 1 < args.len() {
            directory = args[i + 1].clone();
            break;
        }
    }

    // Provide a default directory if --directory flag is missing
    if directory.is_empty() {
        directory = "/tmp".to_string();  // Default to /tmp directory
        println!("Warning: --directory flag not provided, defaulting to /tmp");
    }

    // Ensure directory exists
    if let Err(e) = fs::create_dir_all(&directory) {
        eprintln!("Failed to create directory {}: {}", directory, e);
        return;
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
