use std::{
    env,
    fs::File,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    thread,
};

#[derive(Debug)]
struct Request {
    method: String,
    target: String,
    version: String,
    user_agent: String,
}

impl Request {
    fn from_string(request: &str) -> Request {
        let header_lines: Vec<_> = request.split("\n").collect();
        let user_agent: Vec<_> = header_lines[2].split(":").collect();
        let mut user_agent_string = String::from("");
        if user_agent.len() == 2 {
            user_agent_string = user_agent[1].trim().to_string();
        }

        let mut parts = header_lines[0].split_whitespace();
        Request {
            method: parts.next().unwrap().to_string(),
            target: parts.next().unwrap().to_string(),
            version: parts.next().unwrap().to_string(),
            user_agent: user_agent_string,
        }
    }
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buf = [0; 1024];
    stream.read(&mut buf)?;

    let request = String::from_utf8_lossy(&buf);
    let request = Request::from_string(&request);

    match request.target.as_str() {
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
