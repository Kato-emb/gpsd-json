//! GPSD client implementation for communicating with the GPS daemon
//!
//! This module provides the main client interface for connecting to and
//! communicating with a GPSD server. It supports multiple connection types,
//! data streaming formats, and protocol versions.
//!
//! # Example
//!
//! ```no_run
//! use gpsd_json_rs::client::{GpsdClient, StreamOptions};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to GPSD
//! let mut client = GpsdClient::connect_socket("127.0.0.1:2947")?;
//!
//! // Get version info
//! let version = client.version()?;
//! println!("GPSD version: {}", version.release);
//!
//! // Start streaming GPS data
//! let stream = client.stream(StreamOptions::json())?;
//! for msg in stream {
//!     println!("GPS data: {:?}", msg?);
//! }
//! # Ok(())
//! # }
//! ```

use std::{
    io::{BufRead, Read},
    net::{TcpStream, ToSocketAddrs},
};

use crate::{
    Result,
    error::GpsdJsonError,
    protocol::{GpsdJsonDecode, GpsdJsonEncode, GpsdJsonRequest, GpsdJsonResponse, v3},
};

/// Trait defining a GPSD protocol version implementation
///
/// This trait specifies the protocol version and associated message types
/// for a particular version of the GPSD JSON protocol.
pub trait GpsdJsonProtocol {
    /// Major version number of the protocol
    const API_VERSION_MAJOR: i32;
    /// Minor version number of the protocol
    const API_VERSION_MINOR: i32;

    /// Request message type for this protocol version
    type Request: GpsdJsonRequest;
    /// Response message type for this protocol version
    type Response: GpsdJsonResponse;

    /// Ensures the connected GPSD server supports this protocol version
    ///
    /// Reads the version message from GPSD and verifies compatibility.
    /// The client requires the major version to match exactly and the
    /// minor version to be greater than or equal to the expected version.
    fn ensure_version<T: std::io::Read + std::io::Write>(
        reader: &mut std::io::BufReader<T>,
    ) -> Result<()> {
        let mut buf = String::new();
        if let Ok(Some(v3::ResponseMessage::Version(version))) = reader.read_response(&mut buf) {
            if Self::API_VERSION_MAJOR != version.proto_major
                || Self::API_VERSION_MINOR < version.proto_minor
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

/// Base GPSD client implementation
///
/// This struct provides the core functionality for communicating with a GPSD server.
/// It is generic over the transport type (e.g., TCP) and protocol version.
///
/// For most use cases, use the type alias `GpsdClient` instead of this struct directly.
#[derive(Debug)]
pub struct GpsdBaseClient<T: std::io::Read + std::io::Write, Pr: GpsdJsonProtocol> {
    inner: T,
    reader: std::io::BufReader<T>,
    _proto: std::marker::PhantomData<Pr>,
}

/// Type alias for a GPSD client using protocol version 3
///
/// This is the most common client type and should be used for
/// connecting to modern GPSD servers (version 3.x).
#[cfg(feature = "proto-v3")]
pub type GpsdClient<T> = GpsdBaseClient<T, v3::V3>;

impl<T, Pr> GpsdBaseClient<T, Pr>
where
    T: std::io::Read + std::io::Write,
    Pr: GpsdJsonProtocol,
{
    /// Creates a new client from a transport and reader
    ///
    /// This internal method performs version verification during initialization.
    fn open(inner: T, mut reader: std::io::BufReader<T>) -> Result<Self> {
        Pr::ensure_version(reader.by_ref())?;

        Ok(GpsdBaseClient {
            inner,
            reader,
            _proto: std::marker::PhantomData,
        })
    }

    /// Sends a request message to the GPSD server
    fn send(&mut self, msg: &Pr::Request) -> Result<()> {
        self.inner.write_request(msg)
    }

    /// Reads a response message from the GPSD server
    ///
    /// Returns `None` if the connection is closed.
    fn read(&mut self) -> Result<Option<Pr::Response>> {
        let mut buf = String::new();
        match self.reader.read_response(&mut buf)? {
            Some(res) => Ok(Some(res)),
            None => Ok(None),
        }
    }
}

impl<Pr: GpsdJsonProtocol> GpsdBaseClient<TcpStream, Pr> {
    /// Creates a new GPSD client from an existing TCP stream
    ///
    /// # Arguments
    /// * `stream` - An established TCP connection to a GPSD server
    ///
    /// # Returns
    /// A new client instance if the protocol version is compatible
    pub fn try_from_tcp_stream(stream: TcpStream) -> Result<Self> {
        let reader = std::io::BufReader::new(stream.try_clone().map_err(GpsdJsonError::IoError)?);
        GpsdBaseClient::<TcpStream, Pr>::open(stream, reader)
    }

    /// Connects to a GPSD server at the specified address
    ///
    /// # Arguments
    /// * `addr` - Socket address of the GPSD server (typically "host:2947")
    ///
    /// # Returns
    /// A new connected client instance
    ///
    /// # Example
    /// ```no_run
    /// # use gpsd_json_rs::client::GpsdClient;
    /// let client = GpsdClient::connect_socket("127.0.0.1:2947").unwrap();
    /// ```
    pub fn connect_socket<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let stream = TcpStream::connect(addr).map_err(GpsdJsonError::IoError)?;
        Self::try_from_tcp_stream(stream)
    }
}

impl<Pr: GpsdJsonProtocol> TryFrom<TcpStream> for GpsdBaseClient<TcpStream, Pr> {
    type Error = GpsdJsonError;

    fn try_from(stream: TcpStream) -> Result<Self> {
        GpsdBaseClient::<TcpStream, Pr>::try_from_tcp_stream(stream)
    }
}

/// Protocol version 3 specific methods
impl<T> GpsdBaseClient<T, v3::V3>
where
    T: std::io::Read + std::io::Write,
{
    /// Requests version information from the GPSD server
    ///
    /// Returns details about the GPSD server version, protocol version,
    /// and capabilities.
    pub fn version(&mut self) -> Result<v3::response::Version> {
        self.send(&v3::RequestMessage::Version)?;
        match self.read()? {
            Some(v3::ResponseMessage::Version(version)) => Ok(version),
            _ => Err(GpsdJsonError::ProtocolError("Failed to read version")),
        }
    }

    /// Lists all GPS devices known to the GPSD server
    ///
    /// Returns information about each connected GPS receiver including
    /// device paths, driver information, and current status.
    pub fn devices(&mut self) -> Result<v3::response::DeviceList> {
        self.send(&v3::RequestMessage::Devices)?;
        match self.read()? {
            Some(v3::ResponseMessage::Devices(devices)) => Ok(devices),
            _ => Err(GpsdJsonError::ProtocolError("Failed to read devices")),
        }
    }

    /// Gets information about the currently active GPS device
    ///
    /// Returns detailed information about the device currently being
    /// used for GPS data.
    pub fn device(&mut self) -> Result<v3::types::Device> {
        self.send(&v3::RequestMessage::Device(None))?;
        match self.read()? {
            Some(v3::ResponseMessage::Device(device)) => Ok(device),
            _ => Err(GpsdJsonError::ProtocolError("Failed to read device")),
        }
    }

    /// Enables data streaming from GPSD with default settings
    ///
    /// Returns the current watch configuration and list of available devices.
    /// After calling this method, GPS data will be streamed from the server.
    pub fn watch(&mut self) -> Result<(v3::types::Watch, v3::response::DeviceList)> {
        self.send(&v3::RequestMessage::Watch(None))?;
        let devices = match self.read()? {
            Some(v3::ResponseMessage::Devices(devices)) => devices,
            _ => return Err(GpsdJsonError::ProtocolError("Failed to read devices")),
        };
        let watch = match self.read()? {
            Some(v3::ResponseMessage::Watch(watch)) => watch,
            _ => return Err(GpsdJsonError::ProtocolError("Failed to read watch")),
        };

        Ok((watch, devices))
    }

    /// Polls for the current GPS fix data
    ///
    /// Returns the most recent GPS fix information available from
    /// all active devices.
    pub fn poll(&mut self) -> Result<v3::response::Poll> {
        self.send(&v3::RequestMessage::Poll)?;
        match self.read()? {
            Some(v3::ResponseMessage::Poll(poll)) => Ok(poll),
            _ => Err(GpsdJsonError::ProtocolError("Failed to read poll")),
        }
    }

    /// Enables or disables data streaming mode
    ///
    /// # Arguments
    /// * `enable` - true to start streaming, false to stop
    pub fn watch_mode(&mut self, enable: bool) -> Result<()> {
        self.set_watch(v3::types::Watch {
            enable: Some(enable),
            ..Default::default()
        })?;

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
    /// # use gpsd_json_rs::client::{GpsdClient, StreamOptions};
    /// # let client = GpsdClient::connect_socket("127.0.0.1:2947").unwrap();
    /// let stream = client.stream(StreamOptions::json()).unwrap();
    /// ```
    pub fn stream<F: StreamFormat>(
        mut self,
        opts: StreamOptions<F>,
    ) -> Result<GpsdDataStream<T, v3::V3, F>> {
        let (ret, _devices) = self.set_watch(opts.opts)?;
        assert_eq!(ret.enable, Some(true));

        Ok(GpsdDataStream {
            inner: self,
            _format: std::marker::PhantomData,
        })
    }

    /// Sets configuration for a specific GPS device
    ///
    /// This method is currently unused but provided for completeness.
    #[allow(dead_code)]
    fn set_device(&mut self, device: v3::types::Device) -> Result<()> {
        self.send(&v3::RequestMessage::Device(Some(device)))?;
        Ok(())
    }

    /// Configures watch mode settings
    ///
    /// Internal method to set watch parameters and receive confirmation.
    fn set_watch(
        &mut self,
        watch: v3::types::Watch,
    ) -> Result<(v3::types::Watch, v3::response::DeviceList)> {
        self.send(&v3::RequestMessage::Watch(Some(watch)))?;
        let devices = match self.read()? {
            Some(v3::ResponseMessage::Devices(devices)) => devices,
            _ => return Err(GpsdJsonError::ProtocolError("Failed to read devices")),
        };
        let watch = match self.read()? {
            Some(v3::ResponseMessage::Watch(watch)) => watch,
            _ => return Err(GpsdJsonError::ProtocolError("Failed to read watch")),
        };

        Ok((watch, devices))
    }
}

/// Marker trait for data stream output formats
///
/// This trait is used to distinguish between different output formats
/// (JSON, NMEA, Raw) at the type level.
pub trait StreamFormat {}

/// JSON format for structured GPS data
///
/// Provides parsed GPS data as JSON objects including TPV (time/position/velocity),
/// SKY (satellite information), and other message types.
pub struct Json;
impl StreamFormat for Json {}

/// NMEA format for raw GPS sentences
///
/// Provides raw NMEA 0183 sentences from the GPS receiver,
/// such as $GPGGA, $GPRMC, etc.
pub struct Nmea;
impl StreamFormat for Nmea {}

/// Raw format for unprocessed GPS data
///
/// Provides raw binary data from the GPS receiver,
/// optionally with hex dump formatting.
pub struct Raw;
impl StreamFormat for Raw {}

/// Configuration options for GPS data streams
///
/// This struct allows configuring various aspects of the data stream,
/// such as enabling specific data types or formatting options.
///
/// Use the format-specific constructors (`StreamOptions::json()`, etc.)
/// to create instances with appropriate defaults.
#[derive(Debug, Clone)]
pub struct StreamOptions<F: StreamFormat> {
    opts: v3::types::Watch,
    _format: std::marker::PhantomData<F>,
}

impl<F: StreamFormat> StreamOptions<F> {
    /// Enables or disables scaled output
    ///
    /// When enabled, GPSD applies scaling to output values.
    /// This affects units and precision of reported values.
    pub fn scaled(mut self, enable: bool) -> Self {
        self.opts.scaled = Some(enable);
        self
    }

    /// Enables or disables split24 mode
    ///
    /// When enabled, AIS type 24 messages are split into separate
    /// part A and part B messages.
    pub fn split24(mut self, enable: bool) -> Self {
        self.opts.split24 = Some(enable);
        self
    }
}

impl StreamOptions<Json> {
    /// Creates stream options for JSON format output
    ///
    /// Returns a configuration for receiving structured GPS data
    /// as JSON messages.
    pub fn json() -> StreamOptions<Json> {
        let mut opts = v3::types::Watch::default();
        opts.enable = Some(true);
        opts.json = Some(true);

        StreamOptions::<Json> {
            opts,
            _format: std::marker::PhantomData,
        }
    }

    /// Enables or disables PPS (Pulse Per Second) messages
    ///
    /// When enabled, the stream will include PPS timing messages
    /// if the GPS receiver supports precision timing.
    pub fn pps(mut self, enable: bool) -> Self {
        self.opts.pps = Some(enable);
        self
    }

    /// Enables or disables timing information
    ///
    /// When enabled, messages include detailed timing information
    /// about when data was received and processed.
    pub fn timing(mut self, enable: bool) -> Self {
        self.opts.timing = Some(enable);
        self
    }
}

impl StreamOptions<Nmea> {
    /// Creates stream options for NMEA format output
    ///
    /// Returns a configuration for receiving raw NMEA 0183 sentences
    /// from the GPS receiver.
    pub fn nmea() -> StreamOptions<Nmea> {
        let mut opts = v3::types::Watch::default();
        opts.enable = Some(true);
        opts.nmea = Some(true);

        StreamOptions::<Nmea> {
            opts,
            _format: std::marker::PhantomData,
        }
    }

    /// Specifies a particular GPS device to stream from
    ///
    /// # Arguments
    /// * `device` - Path to the GPS device (e.g., "/dev/ttyUSB0")
    pub fn device<S: AsRef<str>>(mut self, device: S) -> Self {
        self.opts.device = Some(device.as_ref().into());
        self
    }
}

impl StreamOptions<Raw> {
    /// Creates stream options for raw format output
    ///
    /// Returns a configuration for receiving raw binary data
    /// from the GPS receiver.
    pub fn raw() -> StreamOptions<Raw> {
        let mut opts = v3::types::Watch::default();
        opts.enable = Some(true);
        opts.raw = Some(1);

        StreamOptions::<Raw> {
            opts,
            _format: std::marker::PhantomData,
        }
    }

    /// Configures hex dump mode for raw data
    ///
    /// # Arguments
    /// * `enable` - true for hex dump format, false for binary
    pub fn hex_dump(mut self, enable: bool) -> Self {
        if enable {
            self.opts.raw = Some(1);
        } else {
            self.opts.raw = Some(2);
        }
        self
    }

    /// Specifies a particular GPS device to stream from
    ///
    /// # Arguments
    /// * `device` - Path to the GPS device (e.g., "/dev/ttyUSB0")
    pub fn device<S: AsRef<str>>(mut self, device: S) -> Self {
        self.opts.device = Some(device.as_ref().into());
        self
    }
}

/// Iterator for streaming GPS data from GPSD
///
/// This struct provides an iterator interface for receiving continuous
/// GPS data from a GPSD server. The format of the data depends on the
/// stream format type parameter.
///
/// The stream continues until explicitly closed or an error occurs.
pub struct GpsdDataStream<T: std::io::Read + std::io::Write, Pr: GpsdJsonProtocol, F: StreamFormat>
{
    inner: GpsdBaseClient<T, Pr>,
    _format: std::marker::PhantomData<F>,
}

impl<T, F> GpsdDataStream<T, v3::V3, F>
where
    T: std::io::Read + std::io::Write,
    F: StreamFormat,
{
    /// Closes the data stream and returns the client
    ///
    /// This method stops the GPS data stream and returns the underlying
    /// client for further operations.
    pub fn close(mut self) -> Result<GpsdBaseClient<T, v3::V3>> {
        let watch = v3::types::Watch::default();

        let (ret, _devices) = self.inner.set_watch(watch)?;
        assert_eq!(ret.enable, Some(false));

        Ok(self.inner)
    }
}

impl<T, Pr> Iterator for GpsdDataStream<T, Pr, Json>
where
    T: std::io::Read + std::io::Write,
    Pr: GpsdJsonProtocol,
{
    type Item = Result<Pr::Response>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.read().transpose()
    }
}

impl<T, Pr> Iterator for GpsdDataStream<T, Pr, Nmea>
where
    T: std::io::Read + std::io::Write,
    Pr: GpsdJsonProtocol,
{
    type Item = Result<String>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = String::new();
        let ret = self
            .inner
            .reader
            .read_line(&mut buf)
            .map_err(GpsdJsonError::IoError);

        match ret {
            Ok(n) if n > 0 => Some(Ok(buf)),
            Ok(_) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

impl<T, Pr> Iterator for GpsdDataStream<T, Pr, Raw>
where
    T: std::io::Read + std::io::Write,
    Pr: GpsdJsonProtocol,
{
    type Item = Result<String>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = String::new();
        let ret = self
            .inner
            .reader
            .read_line(&mut buf)
            .map_err(GpsdJsonError::IoError);

        match ret {
            Ok(n) if n > 0 => Some(Ok(buf)),
            Ok(_) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
