use std::net::IpAddr;

use clap::Parser;

use futures::StreamExt;
use gpsd_json::{
    client::{GpsdClient, StreamOptions},
    protocol::v3::ResponseMessage,
};

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

    let devices = client.devices().await.unwrap();
    for device in devices.devices {
        println!(
            "Device:\n- path: {:?}\n- activated: {:?}",
            device.path.unwrap(),
            device.activated.unwrap()
        );
    }

    let opts = StreamOptions::json().pps(true).timing(true);
    let mut stream = client.stream(opts).await.unwrap();

    loop {
        match stream.next().await {
            Some(Ok(msg)) => {
                match msg {
                    ResponseMessage::Tpv(tpv) => {
                        if let (Some(lat), Some(lon)) = (tpv.lat, tpv.lon) {
                            println!("Current position: lat {lat:6.3}, lon {lon:6.3}");
                        }
                    }
                    ResponseMessage::Sky(sky) => {
                        let used: Vec<_> = sky.satellites.iter().filter(|sat| sat.used).collect();
                        println!(
                            "Satellites in view: {}, used: {}",
                            sky.satellites.len(),
                            used.len()
                        );
                    }
                    _ => { /* ignore other messages */ }
                }
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
