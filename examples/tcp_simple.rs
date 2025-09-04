use std::net::IpAddr;

use clap::Parser;

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

fn main() {
    let args = Args::parse();

    let mut client = GpsdClient::connect_socket(format!("{}:{}", args.addr, args.port)).unwrap();

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

    while let Some(Ok(msg)) = stream.next() {
        match msg {
            ResponseMessage::Tpv(tpv) => {
                println!(
                    "Received TPV: lat {}, lon {}, alt {}",
                    tpv.lat.unwrap_or_default(),
                    tpv.lon.unwrap_or_default(),
                    tpv.alt.unwrap_or_default()
                );
            }
            ResponseMessage::Sky(sky) => {
                println!("Received SKY: satellites {}", sky.satellites.len());
            }
            ResponseMessage::Toff(toff) => {
                println!(
                    "Received TOFF: diff {}",
                    toff.clock.unwrap() - toff.real.unwrap()
                );
            }
            _ => {
                println!("Received other message: {msg:?}");
            }
        }
    }
}
