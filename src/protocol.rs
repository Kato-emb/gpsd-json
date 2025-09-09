//! GPSD JSON protocol trait definitions and implementations
//!
//! This module provides the core traits and types for implementing
//! the GPSD JSON protocol. It defines how messages are encoded, decoded,
//! and transmitted between client and server.
//!
//! The GPSD protocol uses newline-delimited JSON messages for communication.
//! Each message is a complete JSON object terminated with a newline character.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

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

/// Extension trait for reading GPSD JSON responses from an async buffered reader
///
/// This trait provides functionality to asynchronously read and parse GPSD JSON
/// messages from any type that implements `AsyncBufRead`. Messages are expected
/// to be newline-delimited JSON objects.
///
/// This is the async equivalent of `GpsdJsonDecode`.
pub trait GpsdJsonDecodeAsync: futures_io::AsyncBufRead {
    /// Polls for the next GPSD response message
    ///
    /// This method attempts to read and deserialize a single GPSD response
    /// message from the async stream. It accumulates data in the provided
    /// buffer until a complete message is received (delimited by newline).
    ///
    /// # Arguments
    /// * `self` - Pinned mutable reference to the async reader
    /// * `cx` - The task context for waking
    /// * `buf` - A reusable buffer for accumulating message data
    ///
    /// # Returns
    /// * `Poll::Ready(Ok(Some(response)))` - Successfully parsed response message
    /// * `Poll::Ready(Ok(None))` - End of stream reached
    /// * `Poll::Ready(Err(_))` - I/O or parsing error occurred
    /// * `Poll::Pending` - Not enough data available yet
    ///
    /// # Example
    /// ```no_run
    /// # use std::pin::Pin;
    /// # use std::task::{Context, Poll};
    /// # use futures::AsyncBufReadExt;
    /// # use gpsd_json::protocol::GpsdJsonDecodeAsync;
    /// # use gpsd_json::protocol::v3::ResponseMessage;
    /// # async fn example(reader: &mut (impl AsyncBufReadExt + Unpin)) {
    /// let mut buf = Vec::new();
    /// let fut = futures::future::poll_fn(|cx| {
    ///     Pin::new(&mut *reader).poll_response::<ResponseMessage>(cx, &mut buf)
    /// });
    /// if let Ok(Some(response)) = fut.await {
    ///     // Process the response
    /// }
    /// # }
    /// ```
    fn poll_response<Response>(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut Vec<u8>,
    ) -> Poll<Result<Option<Response>>>
    where
        Response: GpsdJsonResponse,
    {
        loop {
            match self.as_mut().poll_fill_buf(cx) {
                Poll::Ready(Ok(in_buf)) => {
                    if in_buf.is_empty() {
                        return Poll::Ready(Ok(None)); // EOF reached
                    }

                    if let Some(pos) = in_buf.iter().position(|&b| b == b'\n') {
                        // Found a newline, we have a complete message
                        buf.extend_from_slice(&in_buf[..=pos]);
                        self.as_mut().consume(pos + 1); // Consume up to and including the newline

                        return match serde_json::from_slice(buf) {
                            Ok(msg) => {
                                buf.clear();
                                Poll::Ready(Ok(Some(msg)))
                            }
                            Err(e) if e.is_eof() => {
                                // Incomplete JSON, continue reading
                                Poll::Pending
                            }
                            Err(e) => {
                                buf.clear();
                                Poll::Ready(Err(GpsdJsonError::SerdeError(e)))
                            }
                        };
                    } else {
                        // No newline found, append all available data and continue
                        buf.extend_from_slice(in_buf);
                        let len = in_buf.len();
                        self.as_mut().consume(len);
                    }
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(GpsdJsonError::IoError(e))),
                Poll::Pending => return Poll::Pending,
            }
        }
    }

    /// Polls for raw GPSD message data without deserialization
    ///
    /// This method reads raw bytes from the async stream until a complete
    /// message is received (delimited by newline), but returns the raw bytes
    /// instead of deserializing them. This is useful when you need to process
    /// the raw JSON data or handle messages that don't fit standard types.
    ///
    /// # Arguments
    /// * `self` - Pinned mutable reference to the async reader
    /// * `cx` - The task context for waking
    /// * `buf` - A reusable buffer for accumulating message data
    ///
    /// # Returns
    /// * `Poll::Ready(Ok(Some(bytes)))` - Complete raw message including newline
    /// * `Poll::Ready(Ok(None))` - End of stream reached
    /// * `Poll::Ready(Err(_))` - I/O error occurred
    /// * `Poll::Pending` - Not enough data available yet
    ///
    /// # Example
    /// ```no_run
    /// # use std::pin::Pin;
    /// # use std::task::{Context, Poll};
    /// # use futures::AsyncBufReadExt;
    /// # use gpsd_json::protocol::GpsdJsonDecodeAsync;
    /// # async fn example(reader: &mut (impl AsyncBufReadExt + Unpin)) {
    /// let mut buf = Vec::new();
    /// let fut = futures::future::poll_fn(|cx| {
    ///     Pin::new(&mut *reader).poll_raw(cx, &mut buf)
    /// });
    /// if let Ok(Some(raw_msg)) = fut.await {
    ///     // Process raw JSON bytes
    ///     let json_str = String::from_utf8_lossy(&raw_msg);
    ///     println!("Raw message: {}", json_str);
    /// }
    /// # }
    /// ```
    fn poll_raw(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut Vec<u8>,
    ) -> Poll<Result<Option<Vec<u8>>>> {
        loop {
            match self.as_mut().poll_fill_buf(cx) {
                Poll::Ready(Ok(in_buf)) => {
                    if in_buf.is_empty() {
                        return Poll::Ready(Ok(None)); // EOF reached
                    }

                    if let Some(pos) = in_buf.iter().position(|&b| b == b'\n') {
                        // Found a newline, we have a complete message
                        buf.extend_from_slice(&in_buf[..=pos]);
                        self.as_mut().consume(pos + 1); // Consume up to and including the newline

                        let msg = buf.clone();
                        buf.clear();
                        return Poll::Ready(Ok(Some(msg)));
                    } else {
                        // No newline found, append all available data and continue
                        buf.extend_from_slice(in_buf);
                        let len = in_buf.len();
                        self.as_mut().consume(len);
                    }
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(GpsdJsonError::IoError(e))),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl<R: futures_io::AsyncBufRead + Unpin + ?Sized> GpsdJsonDecodeAsync for R {}

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
    /// # use gpsd_json::protocol::GpsdJsonDecode;
    /// # use gpsd_json::protocol::v3::ResponseMessage;
    /// # fn example(reader: &mut BufReader<std::net::TcpStream>) {
    /// let mut buf = Vec::new();
    /// if let Ok(Some(response)) = reader.read_response::<ResponseMessage>(&mut buf) {
    ///     // Process the response
    /// }
    /// # }
    /// ```
    fn read_response<Response>(&mut self, buf: &mut Vec<u8>) -> Result<Option<Response>>
    where
        Response: GpsdJsonResponse,
    {
        let bytes_read = self
            .read_until(b'\n', buf)
            .map_err(GpsdJsonError::IoError)?;
        if bytes_read == 0 {
            return Ok(None); // EOF reached
        }

        match serde_json::from_slice(buf) {
            Ok(msg) => {
                buf.clear();
                Ok(Some(msg))
            }
            Err(e) if e.is_eof() => {
                // Incomplete JSON, continue reading
                Ok(None)
            }
            Err(e) => {
                buf.clear();
                Err(GpsdJsonError::SerdeError(e))
            }
        }
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
    /// # impl gpsd_json::protocol::GpsdJsonRequest for WatchRequest {
    /// fn to_command(&self) -> String {
    ///     "?WATCH={\"enable\":true};".to_string()
    /// }
    /// # }
    /// ```
    fn to_command(&self) -> String;
}

/// Extension trait for writing GPSD JSON requests to an async writer
///
/// This trait provides functionality to asynchronously encode and send GPSD
/// request messages to any type that implements `AsyncWriteExt`. The messages
/// are formatted as GPSD commands and sent to the stream.
///
/// This is the async equivalent of `GpsdJsonEncode`.
pub trait GpsdJsonEncodeAsync: futures_io::AsyncWrite + Unpin {
    /// Writes a request message to the async output stream
    ///
    /// This method converts the request to a GPSD command string and
    /// asynchronously writes it to the underlying stream.
    ///
    /// # Arguments
    /// * `request` - The request message to send
    ///
    /// # Returns
    /// A future that resolves to:
    /// * `Ok(())` - Request successfully written
    /// * `Err(_)` - I/O error occurred during write
    ///
    /// # Example
    /// ```no_run
    /// # use gpsd_json::protocol::GpsdJsonEncodeAsync;
    /// # use gpsd_json::protocol::v3::RequestMessage;
    /// # async fn example(writer: &mut (impl futures::io::AsyncWriteExt + Unpin), request: &RequestMessage) {
    /// writer.write_request(request).await.expect("Failed to send request");
    /// # }
    /// ```
    fn write_request(
        &mut self,
        request: &impl GpsdJsonRequest,
    ) -> impl std::future::Future<Output = Result<()>> {
        let cmd = request.to_command();
        async move {
            use futures_util::io::AsyncWriteExt;
            self.write_all(cmd.as_bytes())
                .await
                .map_err(GpsdJsonError::IoError)
        }
    }
}

impl<W: futures_io::AsyncWrite + Unpin + ?Sized> GpsdJsonEncodeAsync for W {}

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
    /// # use gpsd_json::protocol::GpsdJsonEncode;
    /// # fn example(writer: &mut std::net::TcpStream, request: &impl gpsd_json::protocol::GpsdJsonRequest) {
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
