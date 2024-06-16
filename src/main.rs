use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

#[derive(Debug)]
struct Request {
    method: String,
    target: String,
    version: String,
}

impl Request {
    fn from_string(request: &str) -> Request {
        let mut parts = request.split_whitespace();
        Request {
            method: parts.next().unwrap().to_string(),
            target: parts.next().unwrap().to_string(),
            version: parts.next().unwrap().to_string(),
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
            stream.write(b"Hello, World!")?;
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
                if let Err(e) = handle_client(_stream) {
                    eprintln!("failed to handle client: {}", e);
                }
            }
            Err(e) => {
                eprintln!("failed to accept client: {}", e);
            }
        }
    }
}
