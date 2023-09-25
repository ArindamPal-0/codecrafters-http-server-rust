use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

fn main() {
    let listener =
        TcpListener::bind("127.0.0.1:4221").expect("Could not bind TCP server to the port");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("> accepted new connection");

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

                let response_buf = if path == "/" {
                    "HTTP/1.1 200 OK\r\n\r\n"
                } else {
                    "HTTP/1.1 404 Not Found\r\n\r\n"
                };

                stream
                    .write_all(response_buf.as_bytes())
                    .expect("Could not send a http response");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
