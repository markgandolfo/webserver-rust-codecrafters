use std::fmt;

#[derive(Debug)]
pub struct Response {
    status_line: String,
    content_encoding: String,
    content_type: String,
    content: String,
}

impl Response {
    pub fn new_not_found() -> Response {
        Response {
            status_line: "HTTP/1.1 404 Not Found".to_string(),
            content_encoding: "".to_string(),
            content_type: "".to_string(),
            content: "".to_string(),
        }
    }

    pub fn new(
        status_code: u16,
        content_type: String,
        content_encoding: String,
        content: String,
    ) -> Response {
        let status_line: String = match status_code {
            200 => "HTTP/1.1 200 OK".to_string(),
            201 => "HTTP/1.1 201 Created".to_string(),
            _ => "HTTP/1.1 404 Not Found".to_string(),
        };

        Response {
            status_line,
            content_encoding,
            content_type,
            content,
        }
    }

    // pub fn as_bytes(&self) -> &[u8] {
    //     self.to_string().as_bytes()
    // }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        parts.push(format!("{}", self.status_line));

        if !self.content_type.is_empty() {
            parts.push(format!("Content-Type: {}", self.content_type));
        }

        if !self.content_encoding.is_empty() {
            parts.push(format!("Content-Encoding: {}", self.content_encoding));
        }

        if !self.content.is_empty() {
            parts.push(format!("Content-Length: {}", self.content.len()));
        }

        parts.push(format!("\r\n{}", self.content));

        write!(f, "{}", parts.join("\r\n"))
    }
}
