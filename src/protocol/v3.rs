//! GPSD JSON Protocol Version 3 implementation
//!
//! This module implements version 3 of the GPSD JSON protocol, which is
//! the current stable protocol used by GPSD 3.x releases.
//!
//! The protocol defines a set of request commands that clients can send
//! to GPSD and response messages that GPSD sends back. All communication
//! uses newline-delimited JSON format.
//!
//! # Protocol Overview
//!
//! - Commands start with '?' and end with ';'
//! - Responses are JSON objects with a "class" field indicating message type
//! - Data can be streamed continuously or polled on demand
//!
//! # References
//!
//! Based on the GPSD project protocol specification:
//! - [GPSD Protocol Documentation](https://gpsd.io/gpsd_json.html)
//! - [Protocol Version History](https://gitlab.com/gpsd/gpsd)

use crate::{
    client::GpsdJsonProtocol,
    protocol::{GpsdJsonRequest, GpsdJsonResponse},
};

/// Request message types and builders
pub mod request;
/// Response message types and parsers
pub mod response;
/// Common data types used in protocol messages
pub mod types;

/// Protocol version 3 major version number
///
/// Reference: [release-3.25](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/SConscript?ref_type=tags#L226)
pub const API_VERSION_MAJOR: i32 = 3;

/// Protocol version 3 minor version number
///
/// This library supports protocol version 3.15 and later
pub const API_VERSION_MINOR: i32 = 15;

/// Protocol version 3 implementation marker
///
/// This struct implements the `GpsdJsonProtocol` trait for version 3
/// of the GPSD JSON protocol.
#[derive(Debug)]
pub struct V3;

impl GpsdJsonProtocol for V3 {
    const API_VERSION_MAJOR: i32 = API_VERSION_MAJOR;
    const API_VERSION_MINOR: i32 = API_VERSION_MINOR;

    type Request = request::Message;
    type Response = response::Message;
}

/// Type alias for version 3 response messages
///
/// This is a convenience alias for `response::Message` that makes it
/// clear we're working with protocol v3 responses.
pub type ResponseMessage = response::Message;
impl GpsdJsonResponse for ResponseMessage {}

/// Type alias for version 3 request messages
///
/// This is a convenience alias for `request::Message` that makes it
/// clear we're working with protocol v3 requests.
pub type RequestMessage = request::Message;

impl GpsdJsonRequest for RequestMessage {
    /// Converts a request message into a GPSD command string
    ///
    /// Each request type is formatted according to the GPSD protocol:
    /// - Simple commands: `?COMMAND;`
    /// - Commands with parameters: `?COMMAND={"json":"params"};`
    fn to_command(&self) -> String {
        match self {
            RequestMessage::Devices => "?DEVICES;".into(),
            RequestMessage::Watch(Some(watch)) => {
                format!("?WATCH={};", serde_json::to_string(watch).unwrap())
            }
            RequestMessage::Watch(None) => "?WATCH;".into(),
            RequestMessage::Device(Some(device)) => {
                format!("?DEVICE={};", serde_json::to_string(device).unwrap())
            }
            RequestMessage::Device(None) => "?DEVICE;".into(),
            RequestMessage::Poll => "?POLL;".into(),
            RequestMessage::Version => "?VERSION;".into(),
        }
    }
}
