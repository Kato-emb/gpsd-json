//! Asynchronous GPSD client implementation for communicating with the GPS daemon
//!
//! This module provides the async client interface for connecting to and
//! communicating with a GPSD server. It supports multiple connection types,
//! data streaming formats, and protocol versions using async/await syntax.
//!
//! For synchronous/blocking operations, see the `blocking` submodule.
//!
//! # Example
//!
//! ```no_run
//! use gpsd_json::client::{GpsdClient, StreamOptions};
//! use futures::StreamExt;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to GPSD asynchronously
//! let mut client = GpsdClient::connect("127.0.0.1:2947").await?;
//!
//! // Get version info
//! let version = client.version().await?;
//! println!("GPSD version: {}", version.release);
//!
//! // Start streaming GPS data
//! let mut stream = client.stream(StreamOptions::json()).await?;
//! while let Some(msg) = stream.next().await {
//!     println!("GPS data: {:?}", msg?);
//! }
//! # Ok(())
//! # }
//! ```

use crate::{
    Result,
    error::GpsdJsonError,
    protocol::{GpsdJsonDecodeAsync, GpsdJsonEncodeAsync, GpsdJsonRequest, GpsdJsonResponse, v3},
};

/// Blocking (synchronous) client implementation
pub mod blocking;

/// Trait defining a GPSD protocol version implementation
///
/// This trait specifies the protocol version and associated message types
/// for a particular version of the GPSD JSON protocol.
pub trait GpsdJsonProtocol: Send + Sync {
    /// Major version number of the protocol
    const API_VERSION_MAJOR: i32;
    /// Minor version number of the protocol
    const API_VERSION_MINOR: i32;

    /// Request message type for this protocol version
    type Request: GpsdJsonRequest + Send + Sync;
    /// Response message type for this protocol version
    type Response: GpsdJsonResponse + Send + Sync;
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
    inner: v3::types::Watch,
    _format: std::marker::PhantomData<F>,
}

impl<F: StreamFormat> StreamOptions<F> {
    /// Enables or disables scaled output
    ///
    /// When enabled, GPSD applies scaling to output values.
    /// This affects units and precision of reported values.
    pub fn scaled(mut self, enable: bool) -> Self {
        self.inner.scaled = Some(enable);
        self
    }

    /// Enables or disables split24 mode
    ///
    /// When enabled, AIS type 24 messages are split into separate
    /// part A and part B messages.
    pub fn split24(mut self, enable: bool) -> Self {
        self.inner.split24 = Some(enable);
        self
    }
}

impl StreamOptions<Json> {
    /// Creates stream options for JSON format output
    ///
    /// Returns a configuration for receiving structured GPS data
    /// as JSON messages.
    pub fn json() -> StreamOptions<Json> {
        let opts = v3::types::Watch {
            enable: Some(true),
            json: Some(true),
            ..Default::default()
        };

        StreamOptions::<Json> {
            inner: opts,
            _format: std::marker::PhantomData,
        }
    }

    /// Enables or disables PPS (Pulse Per Second) messages
    ///
    /// When enabled, the stream will include PPS timing messages
    /// if the GPS receiver supports precision timing.
    pub fn pps(mut self, enable: bool) -> Self {
        self.inner.pps = Some(enable);
        self
    }

    /// Enables or disables timing information
    ///
    /// When enabled, messages include detailed timing information
    /// about when data was received and processed.
    pub fn timing(mut self, enable: bool) -> Self {
        self.inner.timing = Some(enable);
        self
    }
}

impl StreamOptions<Nmea> {
    /// Creates stream options for NMEA format output
    ///
    /// Returns a configuration for receiving raw NMEA 0183 sentences
    /// from the GPS receiver.
    pub fn nmea() -> StreamOptions<Nmea> {
        let opts = v3::types::Watch {
            enable: Some(true),
            nmea: Some(true),
            ..Default::default()
        };

        StreamOptions::<Nmea> {
            inner: opts,
            _format: std::marker::PhantomData,
        }
    }

    /// Specifies a particular GPS device to stream from
    ///
    /// # Arguments
    /// * `device` - Path to the GPS device (e.g., "/dev/ttyUSB0")
    pub fn device<S: AsRef<str>>(mut self, device: S) -> Self {
        self.inner.device = Some(device.as_ref().into());
        self
    }
}

impl StreamOptions<Raw> {
    /// Creates stream options for raw format output
    ///
    /// Returns a configuration for receiving raw binary data
    /// from the GPS receiver.
    pub fn raw() -> StreamOptions<Raw> {
        let opts = v3::types::Watch {
            enable: Some(true),
            raw: Some(1),
            ..Default::default()
        };

        StreamOptions::<Raw> {
            inner: opts,
            _format: std::marker::PhantomData,
        }
    }

    /// Configures hex dump mode for raw data
    ///
    /// # Arguments
    /// * `enable` - true for hex dump format, false for binary
    pub fn hex_dump(mut self, enable: bool) -> Self {
        if enable {
            self.inner.raw = Some(1);
        } else {
            self.inner.raw = Some(2);
        }
        self
    }

    /// Specifies a particular GPS device to stream from
    ///
    /// # Arguments
    /// * `device` - Path to the GPS device (e.g., "/dev/ttyUSB0")
    pub fn device<S: AsRef<str>>(mut self, device: S) -> Self {
        self.inner.device = Some(device.as_ref().into());
        self
    }
}

/// Core implementation of an asynchronous GPSD client
///
/// This struct provides the fundamental functionality for asynchronous
/// communication with a GPSD server. It handles protocol negotiation,
/// message serialization/deserialization, and maintains the connection state.
///
/// # Type Parameters
/// * `Stream` - The underlying async I/O stream type (e.g., TcpStream)
/// * `Proto` - The GPSD protocol version implementation
#[derive(Debug)]
pub struct GpsdClientCore<Stream, Proto> {
    reader: futures_util::io::BufReader<Stream>,
    buf: Vec<u8>,
    _proto: std::marker::PhantomData<Proto>,
}

impl<Stream, Proto> GpsdClientCore<Stream, Proto>
where
    Proto: GpsdJsonProtocol,
{
    /// Opens a new GPSD client connection using the provided async stream
    ///
    /// This method initializes the client with the given async I/O stream and
    /// performs protocol version negotiation with the GPSD server.
    ///
    /// # Arguments
    /// * `stream` - The async I/O stream for communication with GPSD
    ///
    /// # Returns
    /// * `Ok(client)` - Successfully connected and negotiated protocol
    /// * `Err(_)` - Connection or protocol negotiation failed
    pub fn open(stream: Stream) -> impl std::future::Future<Output = Result<Self>>
    where
        Stream: futures_io::AsyncRead + futures_io::AsyncWrite + Unpin,
    {
        async move {
            let reader = futures_util::io::BufReader::new(stream);
            let mut client = GpsdClientCore {
                reader,
                buf: Vec::new(),
                _proto: std::marker::PhantomData,
            };

            client.ensure_version().await?;
            Ok(client)
        }
    }

    /// Sends a request message to the GPSD server asynchronously
    fn send(&mut self, msg: &Proto::Request) -> impl std::future::Future<Output = Result<()>>
    where
        Stream: futures_io::AsyncWrite + Unpin,
    {
        async move { self.reader.write_request(msg).await }
    }

    /// Receives a response message from the GPSD server asynchronously
    ///
    /// Returns `None` if the connection is closed.
    fn recv(&mut self) -> impl std::future::Future<Output = Result<Option<Proto::Response>>>
    where
        Stream: futures_io::AsyncRead + Unpin,
    {
        futures_util::future::poll_fn(|cx| {
            std::pin::Pin::new(&mut self.reader).poll_response::<Proto::Response>(cx, &mut self.buf)
        })
    }

    /// Ensures the connected GPSD server supports this protocol version
    ///
    /// Reads the version message from GPSD and verifies compatibility.
    /// The client requires the major version to match exactly and the
    /// minor version to be greater than or equal to the expected version.
    fn ensure_version(&mut self) -> impl std::future::Future<Output = Result<()>>
    where
        Stream: futures_io::AsyncRead + Unpin,
    {
        async move {
            use futures_util::AsyncBufReadExt;
            self.buf.clear();
            let bytes_read = self
                .reader
                .read_until(b'\n', &mut self.buf)
                .await
                .map_err(GpsdJsonError::IoError)?;

            if bytes_read == 0 {
                return Err(GpsdJsonError::ProtocolError(
                    "Connection closed by GPSD before version message",
                ));
            }

            let ret = if let Ok(Some(v3::ResponseMessage::Version(version))) =
                serde_json::from_slice(&self.buf)
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
            };

            self.buf.clear();
            ret
        }
    }
}

#[cfg(feature = "tokio")]
impl<Proto> GpsdClientCore<tokio_util::compat::Compat<tokio::net::TcpStream>, Proto>
where
    Proto: GpsdJsonProtocol,
{
    /// Connects to a GPSD server over TCP asynchronously
    ///
    /// Creates an async TCP connection to the specified address and initializes
    /// a GPSD client with protocol negotiation.
    ///
    /// # Arguments
    /// * `addr` - Socket address of the GPSD server (e.g., "127.0.0.1:2947")
    ///
    /// # Example
    /// ```no_run
    /// # use gpsd_json::client::GpsdClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = GpsdClient::connect("127.0.0.1:2947").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect<A: tokio::net::ToSocketAddrs>(addr: A) -> Result<Self> {
        use tokio_util::compat::TokioAsyncReadCompatExt;

        let stream = tokio::net::TcpStream::connect(addr)
            .await
            .map_err(GpsdJsonError::IoError)?;
        let client = GpsdClientCore::open(stream.compat()).await?;
        Ok(client)
    }
}

/// Type alias for an async GPSD client using protocol version 3
///
/// This is the most common async client type and should be used for
/// connecting to modern GPSD servers (version 3.x) with async/await.
#[cfg(feature = "proto-v3")]
pub type GpsdClient<Stream> = GpsdClientCore<Stream, v3::V3>;

impl<Stream> GpsdClient<Stream>
where
    Stream: futures_io::AsyncRead + futures_io::AsyncWrite + Unpin,
{
    /// Requests version information from the GPSD server
    ///
    /// Returns details about the GPSD server version, protocol version,
    /// and capabilities.
    pub async fn version(&mut self) -> Result<v3::response::Version> {
        self.send(&v3::RequestMessage::Version).await?;
        if let Some(v3::ResponseMessage::Version(version)) = self.recv().await? {
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
    pub async fn devices(&mut self) -> Result<v3::response::DeviceList> {
        self.send(&v3::RequestMessage::Devices).await?;
        if let Some(v3::ResponseMessage::Devices(devices)) = self.recv().await? {
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
    pub async fn device(&mut self) -> Result<v3::types::Device> {
        self.send(&v3::RequestMessage::Device(None)).await?;
        if let Some(v3::ResponseMessage::Device(device)) = self.recv().await? {
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
    pub async fn watch(&mut self) -> Result<(v3::types::Watch, v3::response::DeviceList)> {
        self.send(&v3::RequestMessage::Watch(None)).await?;
        let Some(v3::ResponseMessage::Devices(devices)) = self.recv().await? else {
            return Err(GpsdJsonError::ProtocolError(
                "Expected devices response from GPSD",
            ));
        };
        let Some(v3::ResponseMessage::Watch(watch)) = self.recv().await? else {
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
    pub async fn poll(&mut self) -> Result<v3::response::Poll> {
        self.send(&v3::RequestMessage::Poll).await?;
        if let Some(v3::ResponseMessage::Poll(poll)) = self.recv().await? {
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
    pub async fn watch_mode(&mut self, enable: bool) -> Result<()> {
        let (watch, _devices) = self
            .set_watch(v3::types::Watch {
                enable: Some(enable),
                ..Default::default()
            })
            .await?;

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
    /// # use gpsd_json::client::{GpsdClient, StreamOptions};
    /// # use futures::StreamExt;
    /// # async fn example(client: GpsdClient<impl futures_io::AsyncRead + futures_io::AsyncWrite + Unpin>) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut stream = client.stream(StreamOptions::json()).await?;
    /// while let Some(msg) = stream.next().await {
    ///     println!("GPS data: {:?}", msg?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn stream<Format: StreamFormat>(
        mut self,
        opts: StreamOptions<Format>,
    ) -> Result<GpsdDataStream<Stream, v3::V3, Format>> {
        let (watch, _devices) = self.set_watch(opts.inner).await?;
        assert_eq!(watch.enable, Some(true));

        Ok(GpsdDataStream {
            inner: self,
            _format: std::marker::PhantomData,
        })
    }

    /// Configures watch mode settings
    ///
    /// Internal method to set watch parameters and receive confirmation.
    async fn set_watch(
        &mut self,
        watch: v3::types::Watch,
    ) -> Result<(v3::types::Watch, v3::response::DeviceList)> {
        self.send(&v3::RequestMessage::Watch(Some(watch))).await?;
        let Some(v3::ResponseMessage::Devices(devices)) = self.recv().await? else {
            return Err(GpsdJsonError::ProtocolError(
                "Expected devices response from GPSD",
            ));
        };
        let Some(v3::ResponseMessage::Watch(watch)) = self.recv().await? else {
            return Err(GpsdJsonError::ProtocolError(
                "Expected watch response from GPSD",
            ));
        };

        Ok((watch, devices))
    }
}

/// Async stream for receiving GPS data from GPSD
///
/// This struct provides an async stream interface (implements `futures::Stream`)
/// for receiving continuous GPS data from a GPSD server. The format of the data
/// depends on the stream format type parameter.
///
/// The stream continues until explicitly closed or an error occurs.
///
/// # Example
/// ```no_run
/// # use gpsd_json::client::{GpsdClient, StreamOptions};
/// # use futures::StreamExt;
/// # async fn example(client: GpsdClient<impl futures_io::AsyncRead + futures_io::AsyncWrite + Unpin>) -> Result<(), Box<dyn std::error::Error>> {
/// let mut stream = client.stream(StreamOptions::json()).await?;
/// while let Some(result) = stream.next().await {
///     match result {
///         Ok(data) => println!("Received: {:?}", data),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// # Ok(())
/// # }
/// ```
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
    Stream: futures_io::AsyncRead + futures_io::AsyncWrite + Unpin,
    Format: StreamFormat,
{
    /// Closes the data stream and returns the client
    ///
    /// This method stops the GPS data stream and returns the underlying
    /// client for further operations.
    pub async fn close(mut self) -> Result<GpsdClient<Stream>> {
        let watch = v3::types::Watch::default();
        self.inner
            .send(&v3::RequestMessage::Watch(Some(watch)))
            .await?;

        loop {
            match self.inner.recv().await {
                Ok(Some(v3::ResponseMessage::Watch(watch))) => {
                    assert_eq!(watch.enable, Some(false));
                    break;
                }
                Ok(Some(_)) | Err(GpsdJsonError::SerdeError(_)) => continue,
                Err(e) => return Err(e),
                Ok(None) => {
                    return Err(GpsdJsonError::ProtocolError(
                        "Stream ended unexpectedly while closing",
                    ));
                }
            }
        }

        self.inner.buf.clear();
        Ok(self.inner)
    }
}

impl<Stream, Proto> futures_util::Stream for GpsdDataStream<Stream, Proto, Json>
where
    Stream: futures_io::AsyncRead + Unpin,
    Proto: GpsdJsonProtocol + Unpin,
{
    type Item = Result<Proto::Response>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let reader = std::pin::Pin::new(&mut this.inner.reader);

        match reader.poll_response::<Proto::Response>(cx, &mut this.inner.buf) {
            std::task::Poll::Ready(Ok(Some(msg))) => std::task::Poll::Ready(Some(Ok(msg))),
            std::task::Poll::Ready(Ok(None)) => std::task::Poll::Ready(None),
            std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Some(Err(e))),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl<Stream, Proto> futures_util::Stream for GpsdDataStream<Stream, Proto, Nmea>
where
    Stream: futures_io::AsyncRead + Unpin,
    Proto: GpsdJsonProtocol + Unpin,
{
    type Item = Result<String>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let reader = std::pin::Pin::new(&mut this.inner.reader);

        match reader.poll_raw(cx, &mut this.inner.buf) {
            std::task::Poll::Ready(Ok(Some(line))) => {
                let line_str = String::from_utf8_lossy(&line).trim_end().to_string();
                std::task::Poll::Ready(Some(Ok(line_str)))
            }
            std::task::Poll::Ready(Ok(None)) => std::task::Poll::Ready(None),
            std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Some(Err(e))),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl<Stream, Proto> futures_util::Stream for GpsdDataStream<Stream, Proto, Raw>
where
    Stream: futures_io::AsyncRead + Unpin,
    Proto: GpsdJsonProtocol + Unpin,
{
    type Item = Result<Vec<u8>>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let reader = std::pin::Pin::new(&mut this.inner.reader);

        match reader.poll_raw(cx, &mut this.inner.buf) {
            std::task::Poll::Ready(Ok(Some(line))) => std::task::Poll::Ready(Some(Ok(line))),
            std::task::Poll::Ready(Ok(None)) => std::task::Poll::Ready(None),
            std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Some(Err(e))),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}
