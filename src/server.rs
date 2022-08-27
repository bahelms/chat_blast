use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{self, Receiver, Sender};

type Message = String;

#[derive(Debug, Clone)]
struct Event(SocketAddr, Message);

pub async fn start(address: String, port: String) {
    let location = format!("{}:{}", address, port);
    let listener = TcpListener::bind(&location)
        .await
        .expect("Failed to bind to addr");

    println!("Chat Blast -- listening on {}", location);

    let (tx, _) = broadcast::channel(32);
    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        println!("Connection accepted: {}", addr);
        let publisher = tx.clone();
        let consumer = tx.subscribe();
        tokio::spawn(async move {
            handle_stream(stream, publisher, consumer, addr).await;
        });
    }
}

async fn handle_stream(
    stream: TcpStream,
    publisher: Sender<Event>,
    mut consumer: Receiver<Event>,
    addr: std::net::SocketAddr,
) {
    let mut reader = BufReader::new(stream);
    let mut buffer = String::new();
    loop {
        tokio::select! {
            result = reader.read_line(&mut buffer) => {
                if buffer.is_empty() {
                    println!("Connection dropped: {}", addr);
                    break;
                }
                handle_incoming_message(buffer.clone(), result, &publisher, addr);
                buffer.clear();
            }

            event = consumer.recv() => {
                let Event(id, msg) = event.expect("Parsing event failed");
                if id != addr {
                    let formatted_msg = format!("[{}]: {}", id, msg);
                    let _ = reader.write(formatted_msg.as_bytes()).await.expect("Broadcast write failed");
                }
            }
        }
    }
}

fn handle_incoming_message(
    message: String,
    read_result: Result<usize, std::io::Error>,
    publisher: &Sender<Event>,
    addr: SocketAddr,
) {
    match read_result {
        Ok(_) => {
            publisher
                .send(Event(addr, message))
                .expect("Error publishing message.");
        }
        Err(e) => {
            println!("Error reading stream: {}", e);
        }
    }
}
