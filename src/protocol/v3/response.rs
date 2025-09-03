//! GPSD Protocol v3 response message types
//!
//! This module defines all the response messages that GPSD can send to clients.
//! Each message type corresponds to a specific class of GPS data or status information.
//!
//! Response messages are identified by their "class" field in the JSON structure.
//! Common message types include:
//! - TPV (Time-Position-Velocity): Core GPS fix data
//! - SKY: Satellite visibility and signal strength
//! - GST: GPS pseudorange error statistics
//! - ATT: Attitude/orientation data
//! - DEVICE/DEVICES: GPS receiver information
//! - VERSION: GPSD daemon version information
//!
//! All timestamps use the ISO 8601 format and are represented as `DateTime<Utc>`.

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::types::*;

/// Time-Position-Velocity (TPV) report
///
/// The TPV message is the core GPS fix report, containing time, position, and velocity data.
/// This is the primary message type for navigation applications.
///
/// Reference: [json_tpv_read](https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/libgps_json.c?ref_type=heads#L34)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Tpv {
    /// Altitude in meters (deprecated, use altMSL or altHAE)
    pub alt: Option<f64>,
    /// Altitude, height above ellipsoid, in meters
    #[serde(rename = "altHAE")]
    pub alt_hae: Option<f64>,
    /// Altitude, MSL (mean sea level) in meters
    #[serde(rename = "altMSL")]
    pub alt_msl: Option<f64>,
    /// Antenna status (OK, OPEN, SHORT)
    pub ant: Option<AntennaStatus>,
    /// RTK baseline information (flattened)
    #[serde(flatten)]
    pub base: Baseline,
    /// Climb/sink rate in meters per second
    pub climb: Option<f64>,
    /// Geodetic datum (usually WGS84)
    pub datum: Option<String>,
    /// Device path that provided this data
    pub device: Option<String>,
    /// Depth below mean sea level in meters
    pub depth: Option<f64>,
    /// Age of DGPS corrections in seconds
    #[serde(rename = "dgpsAge")]
    pub dgps_age: Option<f64>,
    /// DGPS station ID
    #[serde(rename = "dgpsSta")]
    pub dgps_sta: Option<i32>,
    /// ECEF coordinates and velocities (flattened)
    #[serde(flatten)]
    pub ecef: Ecef,
    /// Estimated climb error in meters/second
    pub epc: Option<f64>,
    /// Estimated track error in degrees
    pub epd: Option<f64>,
    /// Estimated horizontal position error in meters
    pub eph: Option<f64>,
    /// Estimated speed error in meters/second
    pub eps: Option<f64>,
    /// Estimated time error in seconds
    pub ept: Option<f64>,
    /// Longitude error estimate in meters
    pub epx: Option<f64>,
    /// Latitude error estimate in meters
    pub epy: Option<f64>,
    /// Estimated vertical error in meters
    pub epv: Option<f64>,
    /// Geoid separation (height of geoid above WGS84 ellipsoid) in meters
    #[serde(rename = "geoidSep")]
    pub geoid_sep: Option<f64>,
    /// Latitude in degrees (positive = North)
    pub lat: Option<f64>,
    /// Jamming indicator
    pub jam: Option<i32>,
    /// Current leap seconds (GPS-UTC offset)
    pub leapseconds: Option<i32>,
    /// Longitude in degrees (positive = East)
    pub lon: Option<f64>,
    /// Magnetic track (course over ground relative to magnetic north)
    pub magtrack: Option<f64>,
    /// Magnetic variation in degrees
    pub magvar: Option<f64>,
    /// GPS fix mode (NoFix, 2D, 3D)
    pub mode: FixMode,
    /// NED velocity components (flattened)
    #[serde(flatten)]
    pub ned: Option<Ned>,
    /// Temperature in degrees Celsius
    pub temp: Option<f64>,
    /// GPS time of fix
    pub time: Option<DateTime<Utc>>,
    /// True track (course over ground) in degrees
    pub track: Option<f64>,
    /// Spherical error probability in meters
    pub sep: Option<f64>,
    /// Speed over ground in meters/second
    pub speed: Option<f64>,
    /// GPS fix status (standard, DGPS, RTK, etc.)
    pub status: Option<FixStatus>,
    /// Wind angle magnetic in degrees
    pub wanglem: Option<f64>,
    /// Wind angle relative in degrees
    pub wangler: Option<f64>,
    /// Wind angle true in degrees
    pub wanglet: Option<f64>,
    /// Wind speed relative in meters/second
    pub wspeedr: Option<f64>,
    /// Wind speed true in meters/second
    pub wspeedt: Option<f64>,
    /// Water temperature in degrees Celsius
    pub wtemp: Option<f64>,
    /// Reception time (when enabled by timing policy)
    #[serde(rename = "rtime")]
    #[serde(default, deserialize_with = "f64_to_datetime")]
    pub rtime: Option<DateTime<Utc>>,
    /// PPS edge time (when enabled by timing policy)
    #[serde(default, deserialize_with = "f64_to_datetime")]
    pub pps: Option<DateTime<Utc>>,
    /// Start of response time (when enabled by timing policy)
    #[serde(default, deserialize_with = "f64_to_datetime")]
    pub sor: Option<DateTime<Utc>>,
    /// Character count in the sentence
    pub chars: Option<u64>,
    /// Number of satellites used in solution
    pub sats: Option<i32>,
    /// GPS week number
    pub week: Option<u16>,
    /// GPS time of week in seconds
    pub tow: Option<f64>,
    /// GPS week rollover count
    pub rollovers: Option<i32>,
    #[cfg(feature = "extra-fields")]
    /// Additional fields not explicitly defined
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

/// Satellite Sky View (SKY) report
///
/// The SKY message reports the satellites visible to the GPS receiver,
/// including signal strength, elevation, azimuth, and usage status.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Sky {
    /// Device path that provided this data
    pub device: Option<String>,
    /// Dilution of precision values (flattened)
    #[serde(flatten)]
    pub dop: Option<Dop>,
    /// GPS time of this sky view
    pub time: Option<DateTime<Utc>>,
    /// Number of satellites visible
    #[serde(rename = "nSat")]
    pub n_sat: Option<i32>,
    /// Number of satellites used in navigation solution
    #[serde(rename = "uSat")]
    pub u_sat: Option<i32>,
    /// List of visible satellites with their properties
    pub satellites: Vec<Satellite>,
    #[cfg(feature = "extra-fields")]
    /// Additional fields not explicitly defined
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

/// GPS Pseudorange Error Statistics (GST)
///
/// The GST message provides GPS pseudorange noise statistics,
/// including RMS values of standard deviation ranges.
///
/// Reference: [json_noise_read](https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/libgps_json.c?ref_type=heads#L175)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Gst {
    /// Device path that provided this data
    pub device: Option<String>,
    /// GPS time of these statistics
    pub time: Option<DateTime<Utc>>,
    /// Altitude error in meters (1-sigma)
    pub alt: Option<f64>,
    /// Latitude error in meters (1-sigma)
    pub lat: Option<f64>,
    /// Longitude error in meters (1-sigma)
    pub lon: Option<f64>,
    /// Semi-major axis of error ellipse in meters
    pub major: Option<f64>,
    /// Semi-minor axis of error ellipse in meters
    pub minor: Option<f64>,
    /// Orientation of error ellipse in degrees from true north
    pub orient: Option<f64>,
    /// RMS value of standard deviation ranges
    pub rms: Option<f64>,
    /// East velocity error in meters/second (1-sigma)
    pub ve: Option<f64>,
    /// North velocity error in meters/second (1-sigma)
    pub vn: Option<f64>,
    /// Up velocity error in meters/second (1-sigma)
    pub vu: Option<f64>,
}

/// Attitude/orientation data
///
/// Reports the orientation of the device in 3D space.
/// Currently a placeholder for future implementation.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Attitude {}

/// Inertial Measurement Unit data
///
/// Reports accelerometer and gyroscope readings.
/// Currently a placeholder for future implementation.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Imu {}

/// Time Offset report
///
/// Reports the offset between system clock and GPS time.
///
/// Reference: [json_toff_read](https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/libgps_json.c?ref_type=heads#L667)
#[derive(Debug, Clone, PartialEq)]
pub struct TimeOffset {
    /// Device path that provided this data
    pub device: Option<String>,
    /// GPS time
    pub real: Option<DateTime<Utc>>,
    /// System clock time
    pub clock: Option<DateTime<Utc>>,
}

impl<'de> Deserialize<'de> for TimeOffset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        struct RawTimeOffset {
            pub device: Option<String>,
            pub real_sec: Option<i64>,
            pub real_nsec: Option<i64>,
            pub clock_sec: Option<i64>,
            pub clock_nsec: Option<i64>,
        }

        let raw = RawTimeOffset::deserialize(deserializer)?;

        Ok(TimeOffset {
            device: raw.device,
            real: deserialize_to_datetime(raw.real_sec, raw.real_nsec),
            clock: deserialize_to_datetime(raw.clock_sec, raw.clock_nsec),
        })
    }
}

/// Pulse-Per-Second (PPS) timing report
///
/// Reports precise timing information from PPS-capable GPS receivers.
#[derive(Debug, Clone, PartialEq)]
pub struct Pps {
    /// Device path that provided this data
    pub device: Option<String>,
    /// GPS time of PPS edge
    pub real: Option<DateTime<Utc>>,
    /// System clock time of PPS edge
    pub clock: Option<DateTime<Utc>>,
    /// Clock precision in nanoseconds
    pub precision: Option<i32>,
    /// Quantization error of PPS signal
    pub q_err: Option<i32>,
}

impl<'de> Deserialize<'de> for Pps {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawPps {
            pub device: Option<String>,
            pub real_sec: Option<i64>,
            pub real_nsec: Option<i64>,
            pub clock_sec: Option<i64>,
            pub clock_nsec: Option<i64>,
            pub precision: Option<i32>,
            #[serde(rename = "qErr")]
            pub q_err: Option<i32>,
        }

        let raw = RawPps::deserialize(deserializer)?;
        Ok(Pps {
            device: raw.device,
            real: deserialize_to_datetime(raw.real_sec, raw.real_nsec),
            clock: deserialize_to_datetime(raw.clock_sec, raw.clock_nsec),
            precision: raw.precision,
            q_err: raw.q_err,
        })
    }
}

/// Oscillator/clock discipline status
///
/// Reports the status of the system's precision time reference.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Oscillator {
    /// Device path of the oscillator
    pub device: String,
    /// Whether the oscillator is running
    pub running: bool,
    /// Whether this is the reference clock
    pub reference: bool,
    /// Whether the clock is disciplined (synchronized)
    pub disciplined: bool,
    // delta: field commented out in original
}

/// GPSD daemon version information
///
/// Reports version and protocol information about the GPSD server.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Version {
    /// GPSD release version string
    pub release: String,
    /// Git revision hash
    pub rev: String,
    /// Protocol major version number
    pub proto_major: i32,
    /// Protocol minor version number
    pub proto_minor: i32,
    /// Remote server URL (if applicable)
    pub remote: Option<String>,
}

/// List of GPS devices known to GPSD
///
/// Contains information about all GPS receivers connected to GPSD.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceList {
    /// List of available GPS devices
    pub devices: Vec<Device>,
}

impl<'de> Deserialize<'de> for DeviceList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawSubDevice {
            pub path: Option<String>,
            pub activated: Option<serde_json::Value>,
            pub flags: Option<PropertyFlags>,
            pub driver: Option<String>,
            pub hexdata: Option<String>,
            pub sernum: Option<String>,
            pub subtype: Option<String>,
            pub subtype1: Option<String>,
            pub native: Option<i32>,
            pub bps: Option<i32>,
            pub parity: Option<Parity>,
            pub stopbits: Option<u32>,
            pub cycle: Option<f64>,
            pub mincycle: Option<f64>,
        }

        #[derive(Deserialize)]
        struct RawDeviceList {
            pub devices: Vec<RawSubDevice>,
        }

        let raw = RawDeviceList::deserialize(deserializer)?;

        let mut devices = Vec::with_capacity(raw.devices.len());
        for device in raw.devices.into_iter() {
            let activated = device.activated;
            let mut device = Device {
                path: device.path,
                activated: None,
                flags: device.flags,
                driver: device.driver,
                hexdata: device.hexdata,
                sernum: device.sernum,
                subtype: device.subtype,
                subtype1: device.subtype1,
                native: device.native,
                bps: device.bps,
                parity: device.parity,
                stopbits: device.stopbits,
                cycle: device.cycle,
                mincycle: device.mincycle,
            };

            match activated {
                Some(serde_json::Value::String(iso_time)) => {
                    device.activated = DateTime::parse_from_rfc3339(&iso_time)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc));
                }
                Some(serde_json::Value::Number(unix_time)) => {
                    if let Some(secs) = unix_time.as_f64() {
                        device.activated = DateTime::<Utc>::from_timestamp(
                            secs.trunc() as i64,
                            ((secs.fract()) * 1e9) as u32,
                        )
                    }
                }
                Some(_) => {
                    return Err(serde::de::Error::custom(
                        "Invalid type for 'activated' field",
                    ));
                }
                None => {}
            }

            devices.push(device);
        }

        Ok(DeviceList { devices })
    }
}

/// Poll response with current GPS state
///
/// Returns a snapshot of the current GPS fix data from all active devices.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Poll {
    /// Number of active devices
    active: Option<i32>,
    /// Timestamp of this poll
    time: Option<DateTime<Utc>>,
    /// TPV data from active devices
    tpv: Vec<Tpv>,
    /// GST data from active devices
    gst: Vec<Gst>,
    /// Sky view from active devices
    sky: Vec<Sky>,
}

/// Error notification from GPSD
///
/// Reports errors that occur during GPSD operation.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Error {
    /// Error message text
    pub message: String,
}

/// RTCM2 differential correction data
///
/// Real Time Correction Messages version 2.
/// Currently a placeholder for future implementation.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Rtcm2 {}

/// RTCM3 differential correction data
///
/// Real Time Correction Messages version 3.
/// Currently a placeholder for future implementation.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Rtcm3 {}

// https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/libgps_json.c#L959
// #[cfg(feature = "ais")]
// #[derive(Debug, Clone, PartialEq, Deserialize)]
// pub struct Aivdm {}

/// Raw GPS receiver data
///
/// Contains raw measurement data from the GPS receiver,
/// including pseudoranges, carrier phases, and signal strengths.
///
/// Reference: [json_raw_read](https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/libgps_json.c#L219)
#[derive(Debug, Clone, PartialEq)]
pub struct Raw {
    /// Device path that provided this data
    pub device: Option<String>,
    /// GPS time of these measurements
    pub time: Option<DateTime<Utc>>,
    /// Raw measurement data for each satellite
    pub rawdata: Vec<Measurement>,
}

impl<'de> Deserialize<'de> for Raw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawRaw {
            pub device: Option<String>,
            pub time: Option<f64>,
            pub nsec: Option<f64>,
            pub rawdata: Vec<Measurement>,
        }

        let raw = RawRaw::deserialize(deserializer)?;
        let time = match (raw.time, raw.nsec) {
            (Some(sec), Some(nsec)) => Some(DateTime::<Utc>::from_timestamp(
                sec.trunc() as i64,
                ((sec.fract()) * 1e9) as u32 + nsec as u32,
            )),
            (Some(sec), None) => Some(DateTime::<Utc>::from_timestamp(
                sec.trunc() as i64,
                ((sec.fract()) * 1e9) as u32,
            )),
            _ => None,
        }
        .flatten();

        Ok(Raw {
            device: raw.device,
            time,
            rawdata: raw.rawdata,
        })
    }
}

/// - [libgps_json_unpack](https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/libgps_json.c#L792)
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "class", rename_all = "UPPERCASE")]
/// GPSD response message types
///
/// This enum represents all possible response messages from GPSD.
/// Each variant corresponds to a specific "class" value in the JSON response.
pub enum Message {
    /// Time-Position-Velocity report
    Tpv(Tpv),
    /// GPS pseudorange error statistics
    Gst(Gst),
    /// Satellite sky view report
    Sky(Sky),
    /// Attitude/orientation data
    Att(Attitude),
    /// Inertial measurement unit data
    Imu(Imu),
    /// List of available GPS devices
    Devices(DeviceList),
    /// Single GPS device information
    Device(Device),
    /// Current watch settings
    Watch(Watch),
    /// GPSD version information
    Version(Version),
    /// RTCM2 differential correction data
    Rtcm2(Rtcm2),
    /// RTCM3 differential correction data
    Rtcm3(Rtcm3),
    // AIS vessel data (commented out)
    // Ais(Aivdm),
    /// Error message from GPSD
    Error(Error),
    /// Time offset report
    Toff(TimeOffset),
    /// Pulse-per-second timing report
    Pps(Pps),
    /// Oscillator/clock discipline status
    Osc(Oscillator),
    /// Raw GPS receiver data
    Raw(Raw),
    /// Poll response with current fixes
    Poll(Poll),
    /// Unknown/unsupported message type
    #[serde(untagged)]
    Other(String),
}

/// Helper function to deserialize floating-point Unix timestamps to DateTime
///
/// Converts a floating-point number representing seconds since Unix epoch
/// to a DateTime<Utc> object, preserving sub-second precision.
fn f64_to_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<f64>::deserialize(deserializer)?;
    Ok(opt.and_then(|float| {
        DateTime::<Utc>::from_timestamp(float.trunc() as i64, ((float.fract()) * 1e9) as u32)
    }))
}

/// Helper function to convert separate seconds and nanoseconds to DateTime
///
/// Combines Unix timestamp seconds and nanoseconds into a DateTime<Utc> object.
fn deserialize_to_datetime(sec: Option<i64>, nsec: Option<i64>) -> Option<DateTime<Utc>> {
    match (sec, nsec) {
        (Some(sec), Some(nsec)) => DateTime::<Utc>::from_timestamp(sec, nsec as u32),
        _ => None,
    }
}
