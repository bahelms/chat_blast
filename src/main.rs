mod server;

#[tokio::main]
async fn main() {
    let address = "127.0.0.1".to_string();
    let port = "4888".to_string();
    server::start_server(address, port).await;
}
