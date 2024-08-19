use std::net::SocketAddr;

use azalea::{protocol::ServerAddress, Account, ClientBuilder};
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async_main());
}

async fn async_main() {
    let account = Account::offline(&std::env::var("USERNAME").unwrap_or_else(|_| "bot".to_string()));
        
    let server_ip = std::env::var("SERVER_IP").expect("SERVER_IP environment variable not set");
    let server_port = std::env::var("SERVER_PORT").expect("SERVER_PORT environment variable not set");
    let address: SocketAddr = format!("{}:{}", server_ip, server_port).parse().expect("Failed to parse server address");

    ClientBuilder::new()
        .start(
            account,
            ServerAddress::from(address),
        )
        .await
        .unwrap();
}
