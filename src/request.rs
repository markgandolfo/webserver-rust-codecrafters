use std::collections::HashMap;

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub target: String,
    pub version: String,
    pub user_agent: String,
    pub accept: String,
    pub content_type: String,
    pub content_length: u32,
    pub accept_encoding: String,
    pub content: String,
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

impl Request {
    pub fn from_string(request: &str) -> Request {
        let c = request.split_once("\r\n\r\n").unwrap().1;
        let (first_line, rest_lines) = request.split_once("\r\n").unwrap();
        let (method, target, version) = parse_first_line(first_line);
        let headers = parse_headers(rest_lines);

        let content_length: u32 = get_from_map("Content-Length", &headers)
            .parse()
            .unwrap_or(0);

        let _headers_vec = rest_lines.split("\r\n");

        Request {
            method: method.to_string(),
            target: target.to_string(),
            version: version.to_string(),
            user_agent: get_from_map("User-Agent", &headers),
            accept: get_from_map("Accept", &headers),
            content_type: get_from_map("Content-Type", &headers),
            accept_encoding: get_from_map("Accept-Encoding", &headers),
            content_length,
            content: c.to_string(),
        }
    }
}
