use std::{
    any::type_name,
    collections::HashMap,
    env,
    fs::File,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    thread,
};

use nom::ToUsize;

#[derive(Debug)]
struct Request {
    method: String,
    target: String,
    version: String,
    user_agent: String,
    accept: String,
    content_type: String,
    content_length: u32,
    content: String,
}

fn parse_first_line(request_line: &str) -> (&str, &str, &str) {
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    let method = parts[0];
    let target = parts[1];
    let version = parts[2];
    (method, target, version)
}

fn parse_headers(headers: &str) -> HashMap<String, String> {
    let headers = headers.split("\r\n");
    let mut header_map = HashMap::new();

    for header in headers {
        if let Some((key, value)) = header.split_once(": ") {
            header_map.insert(key.trim().to_string(), value.trim().to_string());
        } else {
            // Handle cases where there might be no value after the colon
            header_map.insert(header.trim().to_string(), "".to_string());
        }
    }

    header_map
}

fn get_from_map(key: &str, map: &HashMap<String, String>) -> String {
    map.get(key).map(|s| s.as_str()).unwrap_or("").to_string()
}

fn print_type_of<T>(_: &T) {
    println!("type: {}", type_name::<T>());
}

impl Request {
    fn from_string(request: &str) -> Request {
        let c = request.split_once("\r\n\r\n").unwrap().1;
        let (first_line, rest_lines) = request.split_once("\r\n").unwrap();
        let (method, target, version) = parse_first_line(first_line);
        let headers = parse_headers(rest_lines);

        let content_length: u32 = get_from_map("Content-Length", &headers)
            .parse()
            .unwrap_or(0);

        let headers_vec = rest_lines.split("\r\n");
        print_type_of(&headers_vec);

        Request {
            method: method.to_string(),
            target: target.to_string(),
            version: version.to_string(),
            user_agent: get_from_map("User-Agent", &headers),
            accept: get_from_map("Accept", &headers),
            content_type: get_from_map("Content-Type", &headers),
            content_length: content_length,
            content: c.to_string(),
        }
    }
}

fn truncate_content(content: String, length: u32) -> String {
    let length = length.to_usize();

    content[..content
        .char_indices()
        .nth(length)
        .map_or(content.len(), |(i, _)| i)]
        .to_string()
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buf = [0; 1024];
    stream.read(&mut buf)?;

    let request = String::from_utf8_lossy(&buf);
    let request = Request::from_string(&request);

    match request.method.as_str() {
        "POST" => {
            if let Some(dir) = env::args().nth(2) {
                if let Some(file_name) = request.target.strip_prefix("/files/") {
                    let file = File::create(dir.to_owned() + file_name);
                    let content = truncate_content(request.content, request.content_length);

                    file?.write_all(content.as_bytes());
                    stream.write(format!("HTTP/1.1 201 Created\r\n\r\n").as_bytes())?;
                }
            }
        }

        "GET" => match request.target.as_str() {
            "/" => {
                stream.write(b"HTTP/1.1 200 OK\r\n\r\n")?;
            }
            "/user-agent" => {
                stream.write(
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                            request.user_agent.len(),
                            request.user_agent
                        )
                        .as_bytes(),
                    )?;
            }
            path if path.starts_with("/echo/") => {
                if let Some(text) = path.strip_prefix("/echo/") {
                    stream.write(
                            format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                                text.len(),
                                text
                            )
                            .as_bytes()
                        )?;
                } else {
                    stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n")?;
                }
            }
            path if path.starts_with("/files/") => {
                if let Some(file_name) = path.strip_prefix("/files/") {
                    if let Some(dir) = env::args().nth(2) {
                        println!("Directory: {}", &dir);
                        if let Ok(mut file) = File::open(Path::new(&dir).join(file_name)) {
                            let mut buf = Vec::new();
                            file.read_to_end(&mut buf).unwrap();
                            stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n", buf.len()).as_bytes()).unwrap();
                            stream.write_all(buf.as_slice()).unwrap();
                        } else {
                            stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                        }
                    }
                } else {
                    eprint!("the file prefix couldn't be stripped");
                    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                }
            }
            _ => {
                stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n")?;
            }
        },

        _ => {
            stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n")?;
        }
    }

    Ok(())
}

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                thread::spawn(move || handle_client(_stream));
            }
            Err(e) => {
                eprintln!("failed to accept client: {}", e);
            }
        }
    }
}
