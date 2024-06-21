mod request;
mod response;

use std::{
    env,
    fs::File,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    thread,
};

use nom::ToUsize;

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
    let request = request::Request::from_string(&request);

    let content_encoding = if request.accept_encoding == "gzip" {
        "gzip"
    } else {
        ""
    }
    .to_string();

    match request.method.as_str() {
        "POST" => {
            if let Some(dir) = env::args().nth(2) {
                if let Some(file_name) = request.target.strip_prefix("/files/") {
                    let file = File::create(dir.to_owned() + file_name);
                    let content = truncate_content(request.content, request.content_length);

                    let _ = file?.write_all(content.as_bytes());
                    stream.write(format!("HTTP/1.1 201 Created\r\n\r\n").as_bytes())?;
                }
            }
        }

        "GET" => match request.target.as_str() {
            "/" => {
                let res =
                    response::Response::new(200, "".to_string(), content_encoding, "".to_string())
                        .to_string();
                stream.write(res.as_bytes())?;
            }
            "/user-agent" => {
                let res = response::Response::new(
                    200,
                    "text/plain".to_string(),
                    "".to_string(),
                    request.user_agent.to_string(),
                )
                .to_string();

                stream.write(res.as_bytes())?;
            }
            path if path.starts_with("/echo/") => {
                if let Some(text) = path.strip_prefix("/echo/") {
                    let res = response::Response::new(
                        200,
                        "text/plain".to_string(),
                        content_encoding,
                        text.to_string(),
                    )
                    .to_string();
                    stream.write(res.as_bytes())?;
                } else {
                    stream.write(response::Response::new_not_found().to_string().as_bytes())?;
                }
            }
            path if path.starts_with("/files/") => {
                if let Some(file_name) = path.strip_prefix("/files/") {
                    if let Some(dir) = env::args().nth(2) {
                        println!("Directory: {}", &dir);
                        if let Ok(mut file) = File::open(Path::new(&dir).join(file_name)) {
                            let mut buf = Vec::new();
                            file.read_to_end(&mut buf).unwrap();
                            let _ = stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n", buf.len()).as_bytes()).unwrap();
                            let _ = stream.write_all(buf.as_slice()).unwrap();
                        } else {
                            stream.write(
                                response::Response::new_not_found().to_string().as_bytes(),
                            )?;
                        }
                    } else {
                        stream.write(response::Response::new_not_found().to_string().as_bytes())?;
                    }
                } else {
                    println!("noooo file found");
                    stream.write(response::Response::new_not_found().to_string().as_bytes())?;
                }
            }
            _ => {
                stream.write(response::Response::new_not_found().to_string().as_bytes())?;
            }
        },

        _ => {
            stream.write(response::Response::new_not_found().to_string().as_bytes())?;
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
