use std::io::{BufRead, BufReader, BufWriter, Result, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

fn handle_client(stream: TcpStream, publisher: Sender<String>, consumer: Receiver<String>) {
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let mut buffer = String::new();

    // see if consumer has msg
    // don't block on read_line
    while match reader.read_line(&mut buffer) {
        Ok(_) => {
            if buffer.is_empty() {
                println!("Connection dropped: {}", stream.peer_addr().unwrap());
                false
            } else {
                publisher
                    .send(buffer.clone())
                    .expect("Error publishing message.");
                // BufWriter seems like overkill here
                // let _ = writer
                //     .write(buffer.as_bytes())
                //     .expect("Problem writing to stream");
                // writer.flush().expect("Error flushing stream writer");
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

fn start_publisher(publish: Receiver<String>, subscribers: Receiver<Sender<String>>) {
    let mut subs = Vec::new();

    loop {
        if let Ok(new_sub) = subscribers.try_recv() {
            subs.push(new_sub);
        }

        if let Ok(msg) = publish.try_recv() {
            dbg!(&msg);
        }
    }
}

fn start_server(address: String, port: String) -> Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", address, port))?;
    let (pub_tx, publish) = channel();
    let (subscribe, subscribers) = channel();
    thread::spawn(|| start_publisher(publish, subscribers));

    println!("Chat Blast -- listening on {}:{}", address, port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let publisher = pub_tx.clone();
                let (consumer_tx, consumer_rx) = channel();
                let _thread_id =
                    thread::spawn(move || handle_client(stream, publisher, consumer_rx))
                        .thread()
                        .id();
                subscribe
                    .send(consumer_tx)
                    .expect("Error subscribing thread.");
            }
            Err(msg) => panic!("The stream is borked: {}", msg),
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let address = "127.0.0.1".to_string();
    let port = "4888".to_string();

    start_server(address, port)
}
