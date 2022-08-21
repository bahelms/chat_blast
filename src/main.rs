use std::io::{BufRead, BufReader, BufWriter, Result, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::ThreadId;

type Message = String;

fn handle_client(
    stream: TcpStream,
    publisher: Sender<(ThreadId, Message)>,
    consumer: Receiver<String>,
) {
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let mut buffer = String::new();

    stream
        .set_nonblocking(true)
        .expect("Set nonblocking stream failed.");

    loop {
        match reader.read_line(&mut buffer) {
            Ok(_) => {
                if buffer.is_empty() {
                    println!("Connection dropped: {}", stream.peer_addr().unwrap());
                    // unsub from publisher
                    break;
                } else {
                    publisher
                        .send((thread::current().id(), buffer.clone()))
                        .expect("Error publishing message.");
                    buffer.clear();
                }
            }

            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // since read_line is not blocking, this empty block just spins
                // the loop as fast as it can, until something enters the stream :/
            }

            Err(e) => {
                println!("Error reading stream: {}", e);
                stream
                    .shutdown(Shutdown::Both)
                    .expect("Error shutting down stream");
                // unsub from publisher
            }
        }

        if let Ok(msg) = consumer.try_recv() {
            // BufWriter seems like overkill here
            let _ = writer
                .write(msg.as_bytes())
                .expect("Problem writing to stream");
            writer.flush().expect("Error flushing stream writer");
        }
    }
}

fn start_publisher(
    publish: Receiver<(ThreadId, Message)>,
    subscribers: Receiver<(ThreadId, Sender<Message>)>,
) {
    let mut subs = Vec::new();

    loop {
        if let Ok(new_sub) = subscribers.try_recv() {
            subs.push(new_sub);
        }

        if let Ok((thread_id, msg)) = publish.try_recv() {
            for (sub_id, sub) in &subs {
                if *sub_id != thread_id {
                    sub.send(msg.clone()).expect("Failed to broadcast message.");
                }
            }
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
                let thread_id =
                    thread::spawn(move || handle_client(stream, publisher, consumer_rx))
                        .thread()
                        .id();
                subscribe
                    .send((thread_id, consumer_tx))
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
