use std::io::{BufRead, BufReader, BufWriter, Result, Write};
use std::net::{TcpListener, TcpStream};

fn handle_stream(stream: TcpStream) -> Result<usize> {
    let mut reader = BufReader::new(&stream);
    let mut buffer = String::new();
    let bytes_read = reader.read_line(&mut buffer)?;
    println!("Bytes read: {}", bytes_read);

    let mut writer = BufWriter::new(&stream);
    let _ = writer.write(&buffer.into_bytes())?;
    writer.flush()?;
    Ok(bytes_read)
}

fn main() -> Result<()> {
    let address = "127.0.0.1";
    let port = "4888";

    let listener = TcpListener::bind(format!("{}:{}", address, port))?;
    println!("Chat Blast -- listening on {}:{}", address, port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_stream(stream)?,
            Err(msg) => panic!("The stream is borked: {}", msg),
        };
    }
    Ok(())
}
