use std::io::{BufRead, BufReader, BufWriter, Result, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

fn handle_stream(stream: TcpStream) {
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let mut buffer = String::new();

    while match reader.read_line(&mut buffer) {
        Ok(_) => {
            if buffer.is_empty() {
                println!("Connection dropped: {}", stream.peer_addr().unwrap());
                false
            } else {
                // BufWriter seems like overkill here
                let _ = writer
                    .write(buffer.as_bytes())
                    .expect("Problem writing to stream");
                writer.flush().expect("Error flushing stream writer");
                buffer.clear();
                true
            }
        }
        Err(e) => {
            println!("Error reading stream: {}", e);
            stream
                .shutdown(Shutdown::Both)
                .expect("Error shutting down stream");
            false
        }
    } {}
}

fn main() -> Result<()> {
    let address = "127.0.0.1";
    let port = "4888";

    let listener = TcpListener::bind(format!("{}:{}", address, port))?;
    println!("Chat Blast -- listening on {}:{}", address, port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move || handle_stream(stream));
            }
            Err(msg) => panic!("The stream is borked: {}", msg),
        }
    }
    Ok(())
}
