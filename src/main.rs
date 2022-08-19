use std::io::{Read, Result, Write};
use std::net::{TcpListener, TcpStream};

fn handle_stream(mut stream: TcpStream) -> Result<usize> {
    let mut buffer = [0; 10];
    let bytes_read = stream.read(&mut buffer)?;
    println!("Echo: {:?}", std::str::from_utf8(&buffer));
    let _ = stream.write(&buffer)?;
    Ok(bytes_read)
}

fn main() -> Result<()> {
    let address = "127.0.0.1";
    let port = "4888";

    let listener = TcpListener::bind(format!("{}:{}", address, port))?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_stream(stream)?,
            Err(msg) => panic!("The stream is borked: {}", msg),
        };
    }
    Ok(())
}
