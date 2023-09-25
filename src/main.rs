use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::vec;

fn main() {
    let listener =
        TcpListener::bind("127.0.0.1:4221").expect("Could not bind TCP server to the port");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("> accepted new connection");

                handle_connection(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
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

    let start_line_words: Vec<_> = start_line.split(" ").collect();

    let http_method = *start_line_words.get(0).expect("Could not get http method");
    println!("http_method: {}", http_method);

    let path = *start_line_words.get(1).expect("Could not get path");
    println!("path: {}", path);

    let http_version = *start_line_words.get(2).expect("Could not get http version");
    println!("http_version: {}", http_version);

    let mut response_text;

    if path == "/" {
        response_text = "HTTP/1.1 200 OK\r\n\r\n".to_string();
    } else if path.starts_with("/echo/") {
        let echo_text = path
            .strip_prefix("/echo/")
            .expect("Could not strip prefix /echo/");
        println!("echo_text: {}", echo_text);

        response_text = "HTTP/1.1 200 OK\r\n".to_string();
        let response_headers = vec![
            "Content-Type: text/plain".to_string(),
            format!("Content-Length: {}", echo_text.len()),
        ];

        let response_headers_str = response_headers.join("\r\n");

        response_text.push_str(response_headers_str.as_str());
        response_text.push_str("\r\n\r\n");
        response_text.push_str(echo_text);
        response_text.push_str("\r\n");
    } else {
        response_text = "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
    }

    stream
        .write_all(response_text.as_bytes())
        .expect("Could not send a http response");
}
