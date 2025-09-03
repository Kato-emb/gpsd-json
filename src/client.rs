use std::{
    io::{BufRead, Read},
    net::{TcpStream, ToSocketAddrs},
};

use crate::{
    Result,
    error::GpsdJsonError,
    protocol::{GpsdJsonDecode, GpsdJsonEncode, GpsdJsonRequest, GpsdJsonResponse, v3},
};

pub trait GpsdJsonProtocol {
    const API_VERSION_MAJOR: i32;
    const API_VERSION_MINOR: i32;

    type Request: GpsdJsonRequest;
    type Response: GpsdJsonResponse;

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

#[derive(Debug)]
pub struct GpsdBaseClient<T: std::io::Read + std::io::Write, Pr: GpsdJsonProtocol> {
    inner: T,
    reader: std::io::BufReader<T>,
    _proto: std::marker::PhantomData<Pr>,
}

#[cfg(feature = "proto-v3")]
pub type GpsdClient<T> = GpsdBaseClient<T, v3::V3>;

impl<T, Pr> GpsdBaseClient<T, Pr>
where
    T: std::io::Read + std::io::Write,
    Pr: GpsdJsonProtocol,
{
    fn open(inner: T, mut reader: std::io::BufReader<T>) -> Result<Self> {
        Pr::ensure_version(reader.by_ref())?;

        Ok(GpsdBaseClient {
            inner,
            reader,
            _proto: std::marker::PhantomData,
        })
    }

    fn send(&mut self, msg: &Pr::Request) -> Result<()> {
        self.inner.write_request(msg)
    }

    fn read(&mut self) -> Result<Option<Pr::Response>> {
        let mut buf = String::new();
        match self.reader.read_response(&mut buf)? {
            Some(res) => Ok(Some(res)),
            None => Ok(None),
        }
    }
}

impl<Pr: GpsdJsonProtocol> GpsdBaseClient<TcpStream, Pr> {
    pub fn try_from_tcp_stream(stream: TcpStream) -> Result<Self> {
        let reader = std::io::BufReader::new(stream.try_clone().map_err(GpsdJsonError::IoError)?);
        GpsdBaseClient::<TcpStream, Pr>::open(stream, reader)
    }

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

impl<T> GpsdBaseClient<T, v3::V3>
where
    T: std::io::Read + std::io::Write,
{
    pub fn version(&mut self) -> Result<v3::response::Version> {
        self.send(&v3::RequestMessage::Version)?;
        match self.read()? {
            Some(v3::ResponseMessage::Version(version)) => Ok(version),
            _ => Err(GpsdJsonError::ProtocolError("Failed to read version")),
        }
    }

    pub fn devices(&mut self) -> Result<v3::response::DeviceList> {
        self.send(&v3::RequestMessage::Devices)?;
        match self.read()? {
            Some(v3::ResponseMessage::Devices(devices)) => Ok(devices),
            _ => Err(GpsdJsonError::ProtocolError("Failed to read devices")),
        }
    }

    pub fn device(&mut self) -> Result<v3::types::Device> {
        self.send(&v3::RequestMessage::Device(None))?;
        match self.read()? {
            Some(v3::ResponseMessage::Device(device)) => Ok(device),
            _ => Err(GpsdJsonError::ProtocolError("Failed to read device")),
        }
    }

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

    pub fn poll(&mut self) -> Result<v3::response::Poll> {
        self.send(&v3::RequestMessage::Poll)?;
        match self.read()? {
            Some(v3::ResponseMessage::Poll(poll)) => Ok(poll),
            _ => Err(GpsdJsonError::ProtocolError("Failed to read poll")),
        }
    }

    pub fn watch_mode(&mut self, enable: bool) -> Result<()> {
        self.set_watch(v3::types::Watch {
            enable: Some(enable),
            ..Default::default()
        })?;

        Ok(())
    }

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

    #[allow(dead_code)]
    fn set_device(&mut self, device: v3::types::Device) -> Result<()> {
        self.send(&v3::RequestMessage::Device(Some(device)))?;
        Ok(())
    }

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

pub trait StreamFormat {}

pub struct Json;
impl StreamFormat for Json {}
pub struct Nmea;
impl StreamFormat for Nmea {}
pub struct Raw;
impl StreamFormat for Raw {}

#[derive(Debug, Clone)]
pub struct StreamOptions<F: StreamFormat> {
    opts: v3::types::Watch,
    _format: std::marker::PhantomData<F>,
}

impl<F: StreamFormat> StreamOptions<F> {
    pub fn scaled(mut self, enable: bool) -> Self {
        self.opts.scaled = Some(enable);
        self
    }

    pub fn split24(mut self, enable: bool) -> Self {
        self.opts.split24 = Some(enable);
        self
    }
}

impl StreamOptions<Json> {
    pub fn json() -> StreamOptions<Json> {
        let mut opts = v3::types::Watch::default();
        opts.enable = Some(true);
        opts.json = Some(true);

        StreamOptions::<Json> {
            opts,
            _format: std::marker::PhantomData,
        }
    }

    pub fn pps(mut self, enable: bool) -> Self {
        self.opts.pps = Some(enable);
        self
    }

    pub fn timing(mut self, enable: bool) -> Self {
        self.opts.timing = Some(enable);
        self
    }
}

impl StreamOptions<Nmea> {
    pub fn nmea() -> StreamOptions<Nmea> {
        let mut opts = v3::types::Watch::default();
        opts.enable = Some(true);
        opts.nmea = Some(true);

        StreamOptions::<Nmea> {
            opts,
            _format: std::marker::PhantomData,
        }
    }

    pub fn device<S: AsRef<str>>(mut self, device: S) -> Self {
        self.opts.device = Some(device.as_ref().into());
        self
    }
}

impl StreamOptions<Raw> {
    pub fn raw() -> StreamOptions<Raw> {
        let mut opts = v3::types::Watch::default();
        opts.enable = Some(true);
        opts.raw = Some(1);

        StreamOptions::<Raw> {
            opts,
            _format: std::marker::PhantomData,
        }
    }

    pub fn hex_dump(mut self, enable: bool) -> Self {
        if enable {
            self.opts.raw = Some(1);
        } else {
            self.opts.raw = Some(2);
        }
        self
    }

    pub fn device<S: AsRef<str>>(mut self, device: S) -> Self {
        self.opts.device = Some(device.as_ref().into());
        self
    }
}

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
