# gpsd-json

[![Crates.io](https://img.shields.io/crates/v/gpsd-json.svg)](https://crates.io/crates/gpsd-json)
[![Documentation](https://docs.rs/gpsd-json/badge.svg)](https://docs.rs/gpsd-json)
[![License](https://img.shields.io/crates/l/gpsd-json.svg)](LICENSE)

A Rust library for parsing GPSD JSON protocol messages without dependencies on libgps.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
gpsd-json = "0.1.0"
```

## Quick Start

### Async (tokio)

```rust
use gpsd_json::client::{GpsdClient, StreamOptions};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to GPSD server
    let mut client = GpsdClient::connect("127.0.0.1:2947").await?;
    
    // Start streaming JSON data
    let mut stream = client.stream(StreamOptions::json()).await?;
    
    // Process GPS data
    while let Some(Ok(msg)) = stream.next().await {
        println!("Received: {:?}", msg);
    }
    
    Ok(())
}
```

### Blocking

```rust
use gpsd_json::client::{blocking, StreamOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to GPSD server
    let mut client = blocking::GpsdClient::connect("127.0.0.1:2947")?;
    
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
- **Async and Blocking** - Both async (tokio) and blocking I/O support
- **Streaming API** - Efficient iterator-based data processing
- **Flexible configuration** - Fine-grained control over data streams

## Usage

### JSON Stream (Async)

```rust
use gpsd_json::{
    client::{GpsdClient, StreamOptions},
    protocol::v3::ResponseMessage,
};
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let mut client = GpsdClient::connect("127.0.0.1:2947").await.unwrap();
    
    // Start streaming with JSON format
    let opts = StreamOptions::json()
        .pps(true)      // Enable PPS timing
        .timing(true);  // Enable timing info
    
    let mut stream = client.stream(opts).await.unwrap();
    
    // Receive and process data
    while let Some(Ok(msg)) = stream.next().await {
        match msg {
            ResponseMessage::Tpv(tpv) => {
                if let (Some(lat), Some(lon)) = (tpv.lat, tpv.lon) {
                    println!("Position: lat {}, lon {}", lat, lon);
                }
            }
            ResponseMessage::Sky(sky) => {
                println!("Satellites in view: {}", sky.satellites.len());
            }
            _ => {}
        }
    }
}
```

### Raw Data Stream (Async)

```rust
use gpsd_json::client::{GpsdClient, StreamOptions};
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let mut client = GpsdClient::connect("127.0.0.1:2947").await.unwrap();
    
    // Stream raw data
    let opts = StreamOptions::raw();
    
    let mut stream = client.stream(opts).await.unwrap();
    
    while let Some(Ok(data)) = stream.next().await {
        let msg = String::from_utf8_lossy(&data);
        println!("Raw data: {}", msg.trim_end());
    }
}
```

## Examples

See the [examples](examples/) directory for more usage examples:

- `tcp_simple.rs` - Async TCP connection and JSON streaming with tokio
- `tcp_blocking.rs` - Blocking TCP connection and JSON streaming
- `raw_stream.rs` - Raw data streaming example with async

## Documentation

For detailed API documentation, please visit [docs.rs/gpsd-json](https://docs.rs/gpsd-json).

## Requirements

- Rust 1.85 or later (2024 edition)
- Running GPSD instance (for actual GPS data)

## License

This project is licensed under the BSD-2-Clause License, following GPSD's licensing terms. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Author

Ryohei Kato <r-kato@musen.co.jp>