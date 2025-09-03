# gpsd-json-rs

[![Crates.io](https://img.shields.io/crates/v/gpsd-json-rs.svg)](https://crates.io/crates/gpsd-json-rs)
[![Documentation](https://docs.rs/gpsd-json-rs/badge.svg)](https://docs.rs/gpsd-json-rs)
[![License](https://img.shields.io/crates/l/gpsd-json-rs.svg)](LICENSE)

A Rust library for parsing GPSD JSON protocol messages without dependencies on libgps.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
gpsd-json-rs = "0.1.0"
```

## Quick Start

```rust
use gpsd_json_rs::client::{GpsdClient, StreamOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to GPSD server
    let client = GpsdClient::connect_socket("127.0.0.1:2947")?;
    
    // Start streaming JSON data
    let mut stream = client.stream(StreamOptions::json())?;
    
    // Process GPS data
    while let Some(Ok(msg)) = stream.next() {
        println!("Received: {:?}", msg);
    }
    
    Ok(())
}
```

## Features

- **Pure Rust** - No C dependencies or libgps required
- **Type-safe** - Leverage Rust's type system for safe GPS data handling
- **Multiple protocols** - Support for JSON, NMEA, and raw data streams
- **Streaming API** - Efficient iterator-based data processing
- **Flexible configuration** - Fine-grained control over data streams

## Usage

### JSON Stream

```rust
use gpsd_json_rs::{
    client::{GpsdClient, StreamOptions},
    protocol::v3::ResponseMessage,
};

fn main() {
    let mut client = GpsdClient::connect_socket("127.0.0.1:2947").unwrap();
    
    // Start streaming with JSON format
    let opts = StreamOptions::json()
        .pps(true)      // Enable PPS timing
        .timing(true);  // Enable timing info
    
    let mut stream = client.stream(opts).unwrap();
    
    // Receive and process data
    while let Some(Ok(msg)) = stream.next() {
        match msg {
            ResponseMessage::Tpv(tpv) => {
                println!("Position: lat {}, lon {}, alt {}",
                    tpv.lat.unwrap_or_default(),
                    tpv.lon.unwrap_or_default(),
                    tpv.alt.unwrap_or_default()
                );
            }
            ResponseMessage::Sky(sky) => {
                println!("Satellites in view: {}", sky.satellites.len());
            }
            _ => {}
        }
    }
}
```

### Raw Data Stream

```rust
use gpsd_json_rs::client::{GpsdClient, StreamOptions};

fn main() {
    let mut client = GpsdClient::connect_socket("127.0.0.1:2947").unwrap();
    
    // Stream raw data
    let opts = StreamOptions::raw()
        .hex_dump(true);  // Enable hex dump
    
    let mut stream = client.stream(opts).unwrap();
    
    while let Some(Ok(data)) = stream.next() {
        println!("Raw data: {}", data);
    }
}
```

## Examples

See the [examples](examples/) directory for more usage examples:

- `tcp_simple.rs` - Basic TCP connection and JSON streaming
- `raw_stream.rs` - Raw data streaming example

## Documentation

For detailed API documentation, please visit [docs.rs/gpsd-json-rs](https://docs.rs/gpsd-json-rs).

## Requirements

- Rust 1.70 or later
- Running GPSD instance (for actual GPS data)

## License

This project is licensed under the BSD-2-Clause License, following GPSD's licensing terms. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Author

Ryohei Kato <r-kato@musen.co.jp>