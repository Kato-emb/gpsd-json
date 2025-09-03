//! GPSD JSON protocol trait definitions and implementations
//!
//! This module provides the core traits and types for implementing
//! the GPSD JSON protocol. It defines how messages are encoded, decoded,
//! and transmitted between client and server.
//!
//! The GPSD protocol uses newline-delimited JSON messages for communication.
//! Each message is a complete JSON object terminated with a newline character.

use crate::{Result, error::GpsdJsonError};

/// Protocol version 3 implementation
///
/// This is the current stable version of the GPSD JSON protocol,
/// supporting all standard GPS data types and control messages.
pub mod v3;

/// Trait for types that can be deserialized as GPSD response messages
///
/// All GPSD response message types must implement this trait,
/// which ensures they can be properly deserialized from JSON.
pub trait GpsdJsonResponse: serde::de::DeserializeOwned {}

/// Extension trait for reading GPSD JSON responses from a buffered reader
///
/// This trait provides functionality to read and parse GPSD JSON messages
/// from any type that implements `BufRead`. Messages are expected to be
/// newline-delimited JSON objects.
pub trait GpsdJsonDecode: std::io::BufRead {
    /// Reads and deserializes a single GPSD response message
    ///
    /// # Arguments
    /// * `buf` - A reusable string buffer for reading data
    ///
    /// # Returns
    /// * `Ok(Some(response))` - Successfully parsed response message
    /// * `Ok(None)` - End of stream reached
    /// * `Err(_)` - I/O or parsing error occurred
    ///
    /// # Example
    /// ```no_run
    /// # use std::io::BufReader;
    /// # use gpsd_json_rs::protocol::GpsdJsonDecode;
    /// # use gpsd_json_rs::protocol::v3::ResponseMessage;
    /// # fn example(reader: &mut BufReader<std::net::TcpStream>) {
    /// let mut buf = String::new();
    /// if let Ok(Some(response)) = reader.read_response::<ResponseMessage>(&mut buf) {
    ///     // Process the response
    /// }
    /// # }
    /// ```
    fn read_response<Res>(&mut self, buf: &mut String) -> Result<Option<Res>>
    where
        Res: GpsdJsonResponse,
    {
        buf.clear();
        let bytes_read = self.read_line(buf).map_err(GpsdJsonError::IoError)?;
        if bytes_read == 0 {
            return Ok(None); // EOF reached
        }

        let response = serde_json::from_str(buf).map_err(GpsdJsonError::SerdeError)?;
        Ok(Some(response))
    }
}

impl<R: std::io::BufRead + ?Sized> GpsdJsonDecode for R {}

/// Trait for types that can be serialized as GPSD request messages
///
/// Request messages in GPSD follow a specific command format,
/// typically starting with '?' and ending with ';'.
pub trait GpsdJsonRequest {
    /// Converts the request into a GPSD command string
    ///
    /// The returned string should be a valid GPSD command that can be
    /// sent directly to the server. Commands typically follow the format:
    /// `?COMMAND[=JSON_PARAMS];`
    ///
    /// # Example
    /// ```
    /// # struct WatchRequest;
    /// # impl gpsd_json_rs::protocol::GpsdJsonRequest for WatchRequest {
    /// fn to_command(&self) -> String {
    ///     "?WATCH={\"enable\":true};".to_string()
    /// }
    /// # }
    /// ```
    fn to_command(&self) -> String;
}

/// Extension trait for writing GPSD JSON requests to a writer
///
/// This trait provides functionality to encode and send GPSD request
/// messages to any type that implements `Write`.
pub trait GpsdJsonEncode: std::io::Write {
    /// Writes a request message to the output stream
    ///
    /// # Arguments
    /// * `request` - The request message to send
    ///
    /// # Returns
    /// * `Ok(())` - Request successfully written
    /// * `Err(_)` - I/O error occurred during write
    ///
    /// # Example
    /// ```no_run
    /// # use gpsd_json_rs::protocol::GpsdJsonEncode;
    /// # fn example(writer: &mut std::net::TcpStream, request: &impl gpsd_json_rs::protocol::GpsdJsonRequest) {
    /// writer.write_request(request).expect("Failed to send request");
    /// # }
    /// ```
    fn write_request(&mut self, request: &impl GpsdJsonRequest) -> Result<()> {
        let cmd = request.to_command();
        self.write_all(cmd.as_bytes())
            .map_err(GpsdJsonError::IoError)
    }
}

impl<W: std::io::Write + ?Sized> GpsdJsonEncode for W {}
