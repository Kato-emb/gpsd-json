//! Blocking (synchronous) GPSD client implementation
//!
//! This module provides a synchronous version of the GPSD client for
//! applications that don't require asynchronous I/O. It offers the same
//! functionality as the async client but with blocking operations.

use std::io::BufRead;
use std::net::{TcpStream, ToSocketAddrs};

use crate::client::{Json, Nmea, Raw, StreamFormat, StreamOptions};
use crate::error::GpsdJsonError;
use crate::protocol::{GpsdJsonDecode, GpsdJsonEncode, v3};
use crate::{Result, client::GpsdJsonProtocol};

/// Core implementation of a blocking GPSD client
///
/// This struct provides the fundamental functionality for synchronous
/// communication with a GPSD server. It handles protocol negotiation,
/// message serialization/deserialization, and maintains the connection state.
///
/// # Type Parameters
/// * `Stream` - The underlying I/O stream type (e.g., TcpStream)
/// * `Proto` - The GPSD protocol version implementation
#[derive(Debug)]
pub struct GpsdClientCore<Stream, Proto> {
    reader: std::io::BufReader<Stream>,
    buf: Vec<u8>,
    _proto: std::marker::PhantomData<Proto>,
}

impl<Stream, Proto> GpsdClientCore<Stream, Proto>
where
    Proto: GpsdJsonProtocol,
{
    /// Opens a new GPSD client connection using the provided stream
    ///
    /// This method initializes the client with the given I/O stream and
    /// performs protocol version negotiation with the GPSD server.
    ///
    /// # Arguments
    /// * `stream` - The I/O stream for communication with GPSD
    ///
    /// # Returns
    /// * `Ok(client)` - Successfully connected and negotiated protocol
    /// * `Err(_)` - Connection or protocol negotiation failed
    pub fn open(stream: Stream) -> Result<Self>
    where
        Stream: std::io::Read + std::io::Write,
    {
        let reader = std::io::BufReader::new(stream);
        let mut client = GpsdClientCore {
            reader,
            buf: Vec::new(),
            _proto: std::marker::PhantomData,
        };

        client.ensure_version()?;
        Ok(client)
    }

    /// Sends a request message to the GPSD server
    fn send(&mut self, msg: &Proto::Request) -> Result<()>
    where
        Stream: std::io::Write,
    {
        self.reader.get_mut().write_request(msg)
    }

    /// Receives a response message from the GPSD server
    ///
    /// Returns `None` if the connection is closed.
    fn recv(&mut self) -> Result<Option<Proto::Response>>
    where
        Stream: std::io::Read,
    {
        self.buf.clear();
        match self.reader.read_response(&mut self.buf)? {
            Some(resp) => Ok(Some(resp)),
            None => Ok(None), // EOF reached
        }
    }

    /// Ensures the connected GPSD server supports this protocol version
    ///
    /// Reads the version message from GPSD and verifies compatibility.
    /// The client requires the major version to match exactly and the
    /// minor version to be greater than or equal to the expected version.
    fn ensure_version(&mut self) -> Result<()>
    where
        Stream: std::io::Read,
    {
        self.buf.clear();
        if let Ok(Some(v3::ResponseMessage::Version(version))) =
            self.reader.read_response(&mut self.buf)
        {
            if Proto::API_VERSION_MAJOR != version.proto_major
                || Proto::API_VERSION_MINOR < version.proto_minor
            {
                Err(GpsdJsonError::UnsupportedProtocolVersion((
                    version.proto_major,
                    version.proto_minor,
                )))
            } else {
                Ok(())
            }
        } else {
            Err(GpsdJsonError::ProtocolError(
                "Failed to read version message from GPSD",
            ))
        }
    }
}

impl<Proto> GpsdClientCore<TcpStream, Proto>
where
    Proto: GpsdJsonProtocol,
{
    /// Connects to a GPSD server over TCP
    ///
    /// Creates a TCP connection to the specified address and initializes
    /// a GPSD client with protocol negotiation.
    ///
    /// # Arguments
    /// * `addr` - Socket address of the GPSD server (e.g., "127.0.0.1:2947")
    ///
    /// # Returns
    /// * `Ok(client)` - Successfully connected to GPSD
    /// * `Err(_)` - Connection failed or protocol negotiation failed
    ///
    /// # Example
    /// ```no_run
    /// # use gpsd_json::client::blocking::GpsdClient;
    /// let client = GpsdClient::connect("127.0.0.1:2947").unwrap();
    /// ```
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let stream = TcpStream::connect(addr).map_err(GpsdJsonError::IoError)?;
        Self::open(stream)
    }
}

impl<Proto> TryFrom<TcpStream> for GpsdClientCore<TcpStream, Proto>
where
    Proto: GpsdJsonProtocol,
{
    type Error = GpsdJsonError;

    fn try_from(stream: TcpStream) -> Result<Self> {
        Self::open(stream)
    }
}

/// Type alias for a GPSD client using protocol version 3
///
/// This is the most common client type and should be used for
/// connecting to modern GPSD servers (version 3.x).
#[cfg(feature = "proto-v3")]
pub type GpsdClient<Stream> = GpsdClientCore<Stream, v3::V3>;

impl<Stream> GpsdClient<Stream>
where
    Stream: std::io::Read + std::io::Write,
{
    /// Requests version information from the GPSD server
    ///
    /// Returns details about the GPSD server version, protocol version,
    /// and capabilities.
    pub fn version(&mut self) -> Result<v3::response::Version> {
        self.send(&v3::RequestMessage::Version)?;
        if let Some(v3::ResponseMessage::Version(version)) = self.recv()? {
            Ok(version)
        } else {
            Err(GpsdJsonError::ProtocolError(
                "Expected version response from GPSD",
            ))
        }
    }

    /// Lists all GPS devices known to the GPSD server
    ///
    /// Returns information about each connected GPS receiver including
    /// device paths, driver information, and current status.
    pub fn devices(&mut self) -> Result<v3::response::DeviceList> {
        self.send(&v3::RequestMessage::Devices)?;
        if let Some(v3::ResponseMessage::Devices(devices)) = self.recv()? {
            Ok(devices)
        } else {
            Err(GpsdJsonError::ProtocolError(
                "Expected devices response from GPSD",
            ))
        }
    }

    /// Gets information about the currently active GPS device
    ///
    /// Returns detailed information about the device currently being
    /// used for GPS data.
    pub fn device(&mut self) -> Result<v3::types::Device> {
        self.send(&v3::RequestMessage::Device(None))?;
        if let Some(v3::ResponseMessage::Device(device)) = self.recv()? {
            Ok(device)
        } else {
            Err(GpsdJsonError::ProtocolError(
                "Expected device response from GPSD",
            ))
        }
    }

    /// Enables data streaming from GPSD with default settings
    ///
    /// Returns the current watch configuration and list of available devices.
    /// After calling this method, GPS data will be streamed from the server.
    pub fn watch(&mut self) -> Result<(v3::types::Watch, v3::response::DeviceList)> {
        self.send(&v3::RequestMessage::Watch(None))?;
        let Some(v3::ResponseMessage::Devices(devices)) = self.recv()? else {
            return Err(GpsdJsonError::ProtocolError(
                "Expected devices response from GPSD",
            ));
        };
        let Some(v3::ResponseMessage::Watch(watch)) = self.recv()? else {
            return Err(GpsdJsonError::ProtocolError(
                "Expected watch response from GPSD",
            ));
        };

        Ok((watch, devices))
    }

    /// Polls for the current GPS fix data
    ///
    /// Returns the most recent GPS fix information available from
    /// all active devices.
    pub fn poll(&mut self) -> Result<v3::response::Poll> {
        self.send(&v3::RequestMessage::Poll)?;
        if let Some(v3::ResponseMessage::Poll(poll)) = self.recv()? {
            Ok(poll)
        } else {
            Err(GpsdJsonError::ProtocolError(
                "Expected poll response from GPSD",
            ))
        }
    }

    /// Enables or disables data streaming mode
    ///
    /// # Arguments
    /// * `enable` - true to start streaming, false to stop
    pub fn watch_mode(&mut self, enable: bool) -> Result<()> {
        let (watch, _devices) = self.set_watch(v3::types::Watch {
            enable: Some(enable),
            ..Default::default()
        })?;

        assert_eq!(watch.enable, Some(enable));
        Ok(())
    }

    /// Starts a data stream with the specified format and options
    ///
    /// This method consumes the client and returns a stream iterator
    /// that yields GPS data in the requested format.
    ///
    /// # Arguments
    /// * `opts` - Stream configuration options
    ///
    /// # Example
    /// ```no_run
    /// # use gpsd_json::client::blocking::GpsdClient;
    /// # use gpsd_json::client::StreamOptions;
    /// # let client = GpsdClient::connect("127.0.0.1:2947").unwrap();
    /// let stream = client.stream(StreamOptions::json()).unwrap();
    /// ```
    pub fn stream<Format: StreamFormat>(
        mut self,
        opts: StreamOptions<Format>,
    ) -> Result<GpsdDataStream<Stream, v3::V3, Format>> {
        let (watch, _devices) = self.set_watch(opts.inner)?;
        assert_eq!(watch.enable, Some(true));

        Ok(GpsdDataStream {
            inner: self,
            _format: std::marker::PhantomData,
        })
    }

    /// Configures watch mode settings
    ///
    /// Internal method to set watch parameters and receive confirmation.
    fn set_watch(
        &mut self,
        watch: v3::types::Watch,
    ) -> Result<(v3::types::Watch, v3::response::DeviceList)> {
        self.send(&v3::RequestMessage::Watch(Some(watch)))?;
        let Some(v3::ResponseMessage::Devices(devices)) = self.recv()? else {
            return Err(GpsdJsonError::ProtocolError(
                "Expected devices response from GPSD",
            ));
        };
        let Some(v3::ResponseMessage::Watch(watch)) = self.recv()? else {
            return Err(GpsdJsonError::ProtocolError(
                "Expected watch response from GPSD",
            ));
        };

        Ok((watch, devices))
    }
}

/// Iterator for streaming GPS data from GPSD
///
/// This struct provides an iterator interface for receiving continuous
/// GPS data from a GPSD server. The format of the data depends on the
/// stream format type parameter.
///
/// The stream continues until explicitly closed or an error occurs.
pub struct GpsdDataStream<Stream, Proto, Format>
where
    Proto: GpsdJsonProtocol,
    Format: StreamFormat,
{
    inner: GpsdClientCore<Stream, Proto>,
    _format: std::marker::PhantomData<Format>,
}

impl<Stream, Format> GpsdDataStream<Stream, v3::V3, Format>
where
    Stream: std::io::Read + std::io::Write,
    Format: StreamFormat,
{
    /// Closes the data stream and returns the client
    ///
    /// This method stops the GPS data stream and returns the underlying
    /// client for further operations.
    pub fn close(mut self) -> Result<GpsdClient<Stream>> {
        let watch = v3::types::Watch::default();

        let (watch, _devices) = self.inner.set_watch(watch)?;
        assert_eq!(watch.enable, Some(false));

        Ok(self.inner)
    }
}

impl<Stream, Proto> Iterator for GpsdDataStream<Stream, Proto, Json>
where
    Stream: std::io::Read,
    Proto: GpsdJsonProtocol,
{
    type Item = Result<Proto::Response>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.recv().transpose()
    }
}

impl<Stream, Proto> Iterator for GpsdDataStream<Stream, Proto, Nmea>
where
    Stream: std::io::Read,
    Proto: GpsdJsonProtocol,
{
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.buf.clear();

        match self.inner.reader.read_until(b'\n', &mut self.inner.buf) {
            Ok(0) => None, // EOF reached
            Ok(_) => Some(Ok(String::from_utf8_lossy(&self.inner.buf)
                .trim_end()
                .to_string())),
            Err(e) => Some(Err(GpsdJsonError::IoError(e))),
        }
    }
}

impl<Stream, Proto> Iterator for GpsdDataStream<Stream, Proto, Raw>
where
    Stream: std::io::Read,
    Proto: GpsdJsonProtocol,
{
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.buf.clear();

        match self.inner.reader.read_until(b'\n', &mut self.inner.buf) {
            Ok(0) => None, // EOF reached
            Ok(_) => Some(Ok(String::from_utf8_lossy(&self.inner.buf)
                .trim_end()
                .to_string())),
            Err(e) => Some(Err(GpsdJsonError::IoError(e))),
        }
    }
}
