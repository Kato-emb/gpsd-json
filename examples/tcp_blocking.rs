use std::net::IpAddr;

use clap::Parser;
use gpsd_json::{
    client::{StreamOptions, blocking},
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

fn main() {
    let args = Args::parse();

    let mut client = blocking::GpsdClient::connect(format!("{}:{}", args.addr, args.port)).unwrap();

    let version = client.version().unwrap();
    println!("GPSD Version: {}", version.release);

    let devices = client.devices().unwrap();
    for device in devices.devices {
        println!(
            "Device:\n- path: {:?}\n- activated: {:?}",
            device.path.unwrap(),
            device.activated.unwrap()
        );
    }

    let opts = StreamOptions::json().pps(true).timing(true);
    let mut stream = client.stream(opts).unwrap();

    loop {
        match stream.next() {
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
