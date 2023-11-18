use clap::Parser;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};

/// Simple HTTP server
#[derive(Parser)]
struct Args {
    /// Port number
    #[arg(short, long, default_value_t = 8080)]
    port_number: u16,
    /// Server root path
    #[arg(short, long, default_value_t = String::from("."))]
    server_path: String,
}

#[derive(Clone, Copy)]
enum HTTPResponse {
    Ok = 200,
    NotFound = 404,
    ServerError = 500,
}
impl HTTPResponse {
    fn status_code(&self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::NotFound => "NOT FOUND",
            Self::ServerError => "SERVER ERROR",
        }
    }
}

struct HTTPResponseBuilder<'a> {
    headers: HashMap<&'a str, &'a str>,
    response: HTTPResponse,
    payload: Vec<u8>,
}

impl<'a> HTTPResponseBuilder<'a> {
    fn new() -> Self {
        Self {
            headers: HashMap::new(),
            response: HTTPResponse::Ok,
            payload: vec![],
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
        result.push(b'\n');
        self.payload.iter().for_each(|data| result.push(*data));
        result
    }
}

fn handle_connection(mut stream: TcpStream, root: String) {
    let mut reader = BufReader::new(&mut stream);
    let mut buffer = String::new();
    reader.read_line(&mut buffer).unwrap();
    let path = &buffer
        .split_whitespace()
        .nth(1)
        .expect("Failed to get the Requested file name from client")[1..];
    let file_path = format!(
        "{root}/{}",
        if path.is_empty() { "index.html" } else { path }
    );
    println!("{file_path}");
    let path = std::path::Path::new(&file_path);
    let extension = &path
        .extension()
        .map(|x| x.to_str().unwrap())
        .unwrap_or("html");
    let content_type = |extension: &str| match extension {
        "wasm" => "application/wasm",
        "html" => "text/html",
        "png" => "image/png",
        "json" => "text/json",
        "jpeg" | "jpg" => "image/jpeg",
        _ => "text/plain",
    };

    let mut http_builder = HTTPResponseBuilder::new();
    match std::fs::read(path) {
        Ok(data) => {
            http_builder.payload = data;
            http_builder
                .headers
                .insert("content-type", content_type(extension));
        }
        Err(e) => {
            http_builder.response = match e.kind() {
                std::io::ErrorKind::NotFound => HTTPResponse::NotFound,
                _ => HTTPResponse::ServerError,
            }
        }
    }
    let mut writer = BufWriter::new(&mut stream);
    writer.write_all(&http_builder.build()).unwrap();
    writer.flush().unwrap();
}

fn main() {
    let args = Args::parse();
    let socket_address = format!("[::]:{}", args.port_number);
    println!("Server started at {socket_address}");
    let listener = TcpListener::bind(socket_address).unwrap();
    for stream in listener.incoming() {
        println!("Incoming connection from: {stream:?}");
        let root = args.server_path.clone();
        std::thread::spawn(move || handle_connection(stream.unwrap(), root));
    }
}
