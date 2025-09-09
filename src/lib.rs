//! # gpsd-json
//!
//! A Rust library for interfacing with GPSD (GPS Service Daemon) using its JSON protocol.
//!
//! This library provides a pure Rust implementation for communicating with GPSD,
//! parsing GPS data, and handling various data stream formats without requiring libgps.
//!
//! ## Overview
//!
//! GPSD is a service daemon that monitors one or more GPSes or AIS receivers attached
//! to a host computer through serial or USB ports, making all data on the location/course/velocity
//! of the sensors available to be queried on TCP port 2947 of the host computer.
//!
//! This library implements the JSON-based protocol used by GPSD for client communication,
//! supporting protocol version 3.x as defined in the GPSD project.
//!
//! ## Example
//!
//! ```ignore
//! use gpsd_json::client::{GpsdClient, StreamOptions};
//! use futures::StreamExt;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to GPSD server
//!     let mut client = GpsdClient::connect("127.0.0.1:2947").await?;
//!
//!     // Start streaming GPS data in JSON format
//!     let mut stream = client.stream(StreamOptions::json()).await?;
//!
//!     // Process incoming GPS data
//!     while let Some(result) = stream.next().await {
//!         println!("Received: {:?}", result?);
//!     }
//!     Ok(())
//! }
//! ```

use crate::error::GpsdJsonError;

/// Client module for establishing connections and managing communication with GPSD
pub mod client;

/// Error types used throughout the library
pub mod error;

/// Protocol definitions and message parsing for GPSD JSON protocol
pub mod protocol;

/// Convenience type alias for Results with GpsdJsonError
pub type Result<T> = core::result::Result<T, GpsdJsonError>;
