use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::net::{TcpListener, TcpStream};

#[derive(Clone, Copy)]
enum HTTPResponse {
    Ok = 200,
    ServerError = 500,
    NotFound = 400,
}
impl HTTPResponse {
    fn status_code(&self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::ServerError => "SERVER ERROR",
            Self::NotFound => "NOT FOUND",
        }
    }
}

struct HTTPResponseBuilder<'a> {
    headers: HashMap<&'a str, &'a str>,
    response: HTTPResponse,
    payload: &'a [u8],
}

impl<'a> HTTPResponseBuilder<'a> {
    fn new() -> Self {
        Self {
            headers: HashMap::new(),
            response: HTTPResponse::Ok,
            payload: &[],
        }
    }
    fn build(self) -> Vec<u8> {
        let content_length = self.payload.len();
        let mut result = Vec::with_capacity(content_length);
        format!(
            "HTTP/1.1 {} {}\ncontent-length: {content_length}\n",
            self.response as usize,
            self.response.status_code()
        )
        .as_bytes()
        .iter()
        .for_each(|byte| result.push(*byte));
        for (key, value) in self.headers {
            format!("{key}: {value}\n")
                .as_bytes()
                .iter()
                .for_each(|byte| result.push(*byte));
        }
        result.push('\n' as u8);
        self.payload.iter().for_each(|data| result.push(*data));
        result
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut http_builder = HTTPResponseBuilder::new();
    http_builder.headers.insert("content-type", "text/html");
    http_builder.payload = "<h1>Hello World!</h1>".as_bytes();
    let mut writer = BufWriter::new(&mut stream);
    writer.write_all(&http_builder.build()).unwrap();
    writer.flush().unwrap();
}

fn main() {
    let listener = TcpListener::bind("[::]:8080").unwrap();
    for stream in listener.incoming() {
        println!("Incoming connection from: {stream:?}");
        std::thread::spawn(move || handle_connection(stream.unwrap()));
    }
    println!("Hello, world!");
}
