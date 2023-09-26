use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::str::FromStr;
use std::thread;

#[derive(Debug)]
enum HTTPMethod {
    GET,
}

impl HTTPMethod {
    fn from(http_method_str: &str) -> Self {
        match http_method_str {
            "GET" => Self::GET,
            _ => panic!("Invalid or Not implemented HTTP method"),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct HTTPRequest {
    method: HTTPMethod,
    path: String,
    version: String,
    headers: HashMap<String, String>,
}

impl HTTPRequest {
    fn from(mut stream: &TcpStream) -> Self {
        let request_buf_reader = BufReader::new(&mut stream);
        let http_request: Vec<_> = request_buf_reader
            .lines()
            .map(|result| result.expect("Could not get a line"))
            .take_while(|line| !line.is_empty())
            .collect();

        // println!("Request: {:#?}", http_request);

        let start_line = http_request
            .get(0)
            .expect("Could not get the http fist line");

        let mut split = start_line.split(" ");

        let http_method_str = split.next().expect("Could not get http method");
        let http_method = HTTPMethod::from(http_method_str);
        println!("http_method: {:?}", http_method);

        let path = split.next().expect("Could not get path").to_string();
        println!("path: {}", path);

        let http_version = split
            .next()
            .expect("Could not get http version")
            .to_string();
        println!("http_version: {}", http_version);

        let mut request_headers = HashMap::new();

        http_request[1..].iter().for_each(|header_str| {
            let mut split = header_str.split(": ");
            let key = split
                .next()
                .expect("Could not get the http header key")
                .to_string();
            let value = split
                .next()
                .expect("Could not get http header value")
                .to_string();

            request_headers.insert(key, value);
        });

        println!("http request headers: {:?}", request_headers);

        Self {
            method: http_method,
            path,
            version: http_version,
            headers: request_headers,
        }
    }
}

#[derive(Debug)]
enum StatusCode {
    OK,
    NotFound,
}

impl StatusCode {
    fn to_status_string(&self) -> String {
        match self {
            Self::OK => "200 OK".to_string(),
            Self::NotFound => "404 Not Found".to_string(),
        }
    }
}

#[derive(Debug)]
struct HTTPResponse {
    version: String,
    status_code: StatusCode,
    headers: HashMap<String, String>,
    content: Option<String>,
}

impl HTTPResponse {
    fn send(&self, mut stream: &TcpStream) {
        let mut response_text = String::new();

        // set HTTP response's HTTP version and Status Code
        response_text.push_str(
            format!(
                "{} {}\r\n",
                self.version,
                self.status_code.to_status_string()
            )
            .as_str(),
        );

        // set HTTP headers (if any)
        let mut response_headers_str = String::new();
        self.headers.iter().for_each(|(key, value)| {
            response_headers_str.push_str(format!("{}: {}\r\n", key, value).as_str())
        });

        if !response_headers_str.is_empty() {
            response_text.push_str(response_headers_str.as_str());
            response_text.push_str("\r\n");
        }

        // set the content (if any)
        if let Some(c) = &self.content {
            response_text.push_str(c.as_str());
        }

        response_text.push_str("\r\n");

        stream
            .write_all(response_text.as_bytes())
            .expect("Could not send a http response");
    }
}

fn main() {
    let listener =
        TcpListener::bind("127.0.0.1:4221").expect("Could not bind TCP server to the port");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("> accepted new connection");

                thread::spawn(|| handle_connection(stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let request = HTTPRequest::from(&stream);

    let path = request.path.as_str();

    let mut status_code = StatusCode::NotFound;
    let mut response_headers = HashMap::new();
    let mut content: Option<String> = None;
    if path == "/" {
        // setting status code
        status_code = StatusCode::OK;
    } else if path.starts_with("/echo/") {
        // setting status code
        status_code = StatusCode::OK;

        // getting content
        let echo_text = path
            .strip_prefix("/echo/")
            .expect("Could not strip prefix /echo/")
            .to_string();
        println!("echo_text: {}", echo_text);

        let content_length = echo_text.len().to_string();

        // setting response headers
        response_headers.insert("Content-Type".to_string(), "text/plain".to_string());
        response_headers.insert("Content-Length".to_string(), content_length);

        // setting content
        content = Some(echo_text);
    } else if path.starts_with("/user-agent") {
        // setting status code
        status_code = StatusCode::OK;

        // getting content
        let user_agent = request
            .headers
            .get("User-Agent")
            .expect("Could not get User-Agent header")
            .clone();

        let content_length = user_agent.len().to_string();

        // setting headers
        response_headers.insert("Content-Type".to_string(), "text/plain".to_string());
        response_headers.insert("Content-Length".to_string(), content_length);

        // settting content
        content = Some(user_agent);
    } else if path.starts_with("/files/") {
        // get the directory from cmd args
        let args: Vec<String> = env::args().collect();

        // check if --directory arg is provided
        if args.get(1).expect("Could not get the flag") != "--directory" {
            panic!("flag should be `--directory`");
        } else if let Some(directory) = args.get(2) {
            let dir = PathBuf::from(directory);

            if !dir.exists() {
                panic!("The provided directory as cmd arg does not exists");
            }

            let file_name = path
                .strip_prefix("/files/")
                .expect("Could not trim /files/");

            let file_path = dir.join(
                PathBuf::from_str(file_name).expect("Could not convert file_name to PathBuf"),
            );

            if file_path.exists() {
                let file_contents = fs::read_to_string(file_path.clone())
                    .expect(format!("Could not read the file: {:?}", file_path).as_str());

                status_code = StatusCode::OK;

                let content_length = file_contents.len().to_string();

                response_headers.insert(
                    "Content-Type".to_string(),
                    "application/octet-stream".to_string(),
                );
                response_headers.insert("Content-Length".to_string(), content_length);

                content = Some(file_contents);
            }
        }
    }

    let response = HTTPResponse {
        version: request.version,
        status_code,
        headers: response_headers,
        content,
    };

    response.send(&mut stream);
}
