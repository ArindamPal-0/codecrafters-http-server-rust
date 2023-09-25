use std::net::TcpListener;
use std::io::Write;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").expect("Could not bind TCP server to the port");
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let response_buf = b"HTTP/1.1 200 OK\r\n\r\n";
                stream.write_all(response_buf).expect("Could not send a http response");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
