use std::net::IpAddr;

use clap::Parser;

use gpsd_json::client::{GpsdClient, StreamOptions};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "0.0.0.0")]
    addr: IpAddr,
    #[arg(short, long, default_value = "2947")]
    port: u16,
}

fn main() {
    let args = Args::parse();

    let mut client = GpsdClient::connect_socket(format!("{}:{}", args.addr, args.port)).unwrap();

    let version = client.version().unwrap();
    println!("GPSD Version: {}", version.release);

    let devices = client.devices().unwrap();
    for device in devices.devices {
        println!(
            "Device:\n- path: {:?}\n- activated: {:?}\n- Seen: {:?}",
            device.path.unwrap(),
            device.activated.unwrap(),
            device.flags.unwrap()
        );
    }

    let opts = StreamOptions::raw();
    let mut stream = client.stream(opts).unwrap();

    while let Some(Ok(msg)) = stream.next() {
        println!("{msg}");
    }
}
