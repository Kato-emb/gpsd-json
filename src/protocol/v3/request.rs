//! GPSD Protocol v3 request message types
//!
//! This module defines the request messages that clients can send to GPSD.
//! Each request type corresponds to a specific GPSD command.

use super::types::*;

/// Request message types for GPSD protocol v3
///
/// These are the commands that a client can send to GPSD to query
/// information or control data streaming.
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    /// Request a list of all GPS devices known to GPSD
    ///
    /// Command: `?DEVICES;`
    Devices,

    /// Control data streaming from GPSD
    ///
    /// - `None`: Query current watch settings (`?WATCH;`)
    /// - `Some(watch)`: Set watch parameters (`?WATCH={...};`)
    Watch(Option<Watch>),

    /// Control or query a specific GPS device
    ///
    /// - `None`: Query current device (`?DEVICE;`)
    /// - `Some(device)`: Configure device (`?DEVICE={...};`)
    Device(Option<Device>),

    /// Poll for current GPS data from all devices
    ///
    /// Command: `?POLL;`
    Poll,

    /// Request GPSD version information
    ///
    /// Command: `?VERSION;`
    Version,
}
