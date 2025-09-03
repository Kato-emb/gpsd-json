//! Error types for GPSD JSON protocol operations
//!
//! This module defines the error types that can occur when communicating
//! with GPSD or parsing its JSON protocol messages.

/// Main error type for GPSD JSON protocol operations
///
/// This enum represents all possible errors that can occur during
/// communication with GPSD or while parsing protocol messages.
#[derive(Debug)]
pub enum GpsdJsonError {
    /// I/O error occurred during network communication
    ///
    /// This typically happens when the connection to GPSD is lost,
    /// the server is unreachable, or there are network-related issues.
    IoError(std::io::Error),
    
    /// JSON serialization/deserialization error
    ///
    /// Occurs when GPSD sends malformed JSON or when the response
    /// doesn't match the expected message structure.
    SerdeError(serde_json::Error),
    
    /// GPSD protocol version is not supported
    ///
    /// The tuple contains (major, minor) version numbers.
    /// This library requires protocol version 3.x compatibility.
    UnsupportedProtocolVersion((i32, i32)),
    
    /// Protocol-level error
    ///
    /// Indicates an error in the GPSD protocol communication,
    /// such as unexpected message sequences or missing required responses.
    ProtocolError(&'static str),
}


impl core::fmt::Display for GpsdJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpsdJsonError::IoError(err) => write!(f, "IoError: {}", err),
            GpsdJsonError::SerdeError(err) => write!(f, "SerdeError: {}", err),
            GpsdJsonError::UnsupportedProtocolVersion((major, minor)) => {
                write!(f, "UnsupportedProtocolVersion: {}.{}", major, minor)
            }
            GpsdJsonError::ProtocolError(msg) => write!(f, "ProtocolError: {}", msg),
        }
    }
}

impl core::error::Error for GpsdJsonError {}
