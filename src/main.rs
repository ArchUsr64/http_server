use std::io::{BufWriter, Write};
use std::net::{TcpListener, TcpStream};

fn handle_connection(mut stream: TcpStream) {
    let mut writer = BufWriter::new(&mut stream);
    writer.write_all("HTTP/1.1 200 OK\n".as_bytes()).unwrap();
    writer
        .write_all("Content-Type: text/html\n".as_bytes())
        .unwrap();
    writer.write_all("\nHello World\n".as_bytes()).unwrap();

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
