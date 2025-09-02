use std::{
    io::BufReader,
    net::{IpAddr, TcpStream},
};

use clap::Parser;
use gpsd_json_rs::protocol::{
    GpsdJsonDecode, GpsdJsonEncode,
    v3::{RequestMessage, ResponseMessage, types::Watch},
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

    let mut stream = TcpStream::connect(format!("{}:{}", args.addr, args.port)).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut buf = String::new();

    if let Ok(Some(res)) = reader.read_response(&mut buf) {
        if let ResponseMessage::Version(version) = res {
            println!(
                "Connected to GPSD\nversion: {}\nproto_version: {}.{}",
                version.release, version.proto_major, version.proto_minor
            );
        }
    }

    stream
        .write_request(&RequestMessage::Watch(Some(Watch {
            device: None,
            enable: Some(true),
            json: Some(true),
            nmea: Some(false),
            pps: Some(true),
            raw: None,
            scaled: Some(true),
            split24: Some(true),
            timing: Some(true),
            remote: None,
        })))
        .unwrap();

    // stream.write_request(&RequestMessage::Poll).unwrap();

    loop {
        match reader.read_response(&mut buf) {
            Ok(Some(res)) => {
                println!("{:?}", res);
            }
            Ok(None) => {
                println!("Connection closed by server");
                break;
            }
            Err(_) => {
                println!("{}", buf);
            }
        }
    }
}
