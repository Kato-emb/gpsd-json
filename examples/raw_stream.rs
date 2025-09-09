use std::net::IpAddr;

use clap::Parser;

use futures::StreamExt;
use gpsd_json::client::{GpsdClient, StreamOptions};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "0.0.0.0")]
    addr: IpAddr,
    #[arg(short, long, default_value = "2947")]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut client = GpsdClient::connect(format!("{}:{}", args.addr, args.port))
        .await
        .unwrap();

    let version = client.version().await.unwrap();
    println!("GPSD Version: {}", version.release);

    let opts = StreamOptions::raw();
    let mut stream = client.stream(opts).await.unwrap();

    loop {
        match stream.next().await {
            Some(Ok(msg)) => {
                let msg = String::from_utf8_lossy(&msg).to_string();
                println!("{}", msg.trim_end());
            }
            Some(Err(e)) => {
                eprintln!("Error receiving message: {e}");
                return;
            }
            None => {
                eprintln!("Stream ended unexpectedly");
                return;
            }
        }
    }
}
