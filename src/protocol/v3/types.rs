//! Common data types used in GPSD protocol v3 messages
//!
//! This module contains the shared data structures used across different
//! GPSD message types. These include enumerations for fix modes, status codes,
//! and complex types for representing GPS data.
//!
//! Most types correspond directly to structures defined in the GPSD C implementation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use serde_with::skip_serializing_none;

/// GPS fix mode indicating the quality/dimension of the position fix
///
/// Reference: [gps_fix_t.mode](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L181)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize_repr)]
#[repr(i32)]
pub enum FixMode {
    /// No GPS data has been seen yet
    NotSeen = 0,
    /// GPS is online but no fix acquired
    NoFix = 1,
    /// 2D fix (latitude/longitude only, no altitude)
    Fix2D = 2,
    /// 3D fix (full position including altitude)
    Fix3D = 3,
}

/// GPS fix status indicating the positioning method and augmentation used
///
/// Reference: [gps_fix_t.status](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L192)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize_repr)]
#[repr(i32)]
pub enum FixStatus {
    /// Unknown or no status information
    Unknown = 0,
    /// Standard GPS fix
    Gps = 1,
    /// Differential GPS (enhanced accuracy)
    DGps = 2,
    /// Real-Time Kinematic with fixed integers (centimeter accuracy)
    RTKFixed = 3,
    /// Real-Time Kinematic with float solution (decimeter accuracy)
    RTKFloat = 4,
    /// Dead reckoning (position estimated from sensors)
    DR = 5,
    /// GNSS combined with dead reckoning
    GnssDR = 6,
    /// Time-only fix (surveyed position, no navigation)
    Time = 7,
    /// Simulated/test data
    Simulated = 8,
    /// Precise Positioning Service
    PpsFix = 9,
}

/// GPS antenna status
///
/// Indicates the electrical status of the GPS antenna connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize_repr)]
#[repr(i32)]
pub enum AntennaStatus {
    /// Status unknown or not reported
    Unknown = 0,
    /// Antenna connected and functioning normally
    Ok = 1,
    /// Open circuit detected (antenna disconnected)
    Open = 2,
    /// Short circuit detected in antenna connection
    Short = 3,
}

/// Satellite signal quality indicator
///
/// Indicates the tracking status and signal quality for a satellite.
///
/// Reference: [satellite.qualityInd](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L2411)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SatQuality {
    /// Invalid or no data
    Invalid,
    /// No signal detected from satellite
    NoSignal,
    /// Searching for satellite signal
    Searching,
    /// Signal acquired but not yet usable
    Acquired,
    /// Signal acquired but unusable (e.g., too weak)
    Unusable,
    /// Code lock achieved (coarse position)
    CodeLocked,
    /// Code and carrier lock (precise position)
    CodeCarrierLocked,
}

impl<'de> Deserialize<'de> for SatQuality {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = i8::deserialize(deserializer)?;
        match v {
            -1 => Ok(SatQuality::Invalid),
            0 => Ok(SatQuality::NoSignal),
            1 => Ok(SatQuality::Searching),
            2 => Ok(SatQuality::Acquired),
            3 => Ok(SatQuality::Unusable),
            4 => Ok(SatQuality::CodeLocked),
            5..=7 => Ok(SatQuality::CodeCarrierLocked),
            _ => Err(serde::de::Error::custom(format!(
                "invalid Satellite QualityInd value: {v}"
            ))),
        }
    }
}

/// Global Navigation Satellite System identifier
///
/// Identifies which satellite constellation a satellite belongs to.
///
/// Reference: [satellite.gnssid](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L2449)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize_repr)]
#[repr(u8)]
pub enum GnssId {
    /// GPS (USA)
    Gps = 0,
    /// Satellite-Based Augmentation System
    Sbas = 1,
    /// Galileo (European Union)
    Gal = 2,
    /// BeiDou (China)
    Bd = 3,
    /// IMES (Indoor Messaging System, Japan)
    Imes = 4,
    /// QZSS (Quasi-Zenith Satellite System, Japan)
    Qzss = 5,
    /// GLONASS (Russia)
    Glo = 6,
    /// IRNSS/NavIC (India)
    Irnss = 7,
}

/// Satellite health status
///
/// Indicates whether a satellite's signals are reliable for navigation.
///
/// Reference: [satellite.health](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L2504)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize_repr)]
#[repr(u8)]
pub enum SatHealth {
    /// Health status unknown
    Unknown = 0,
    /// Satellite is healthy and usable
    Ok = 1,
    /// Satellite is unhealthy, do not use
    Bad = 2,
}

bitflags::bitflags! {
    /// Device property flags
    ///
    /// Indicates what types of data have been seen from a GPS device.
    /// These flags are set when GPSD detects specific data types.
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct PropertyFlags: u32 {
        /// GPS data has been seen on this device
        const SEEN_GPS = 0x01;
        /// RTCM2 data has been seen on this device
        const SEEN_RTCM2 = 0x02;
        /// RTCM3 data has been seen on this device
        const SEEN_RTCM3 = 0x04;
        /// AIS data has been seen on this device
        const SEEN_AIS = 0x08;
    }
}

impl Serialize for PropertyFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}

impl<'de> Deserialize<'de> for PropertyFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = u32::deserialize(deserializer)?;
        Ok(PropertyFlags::from_bits_truncate(bits))
    }
}

/// Serial port parity setting
///
/// Defines the parity bit configuration for serial communication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Parity {
    /// No parity bit
    No,
    /// Odd parity
    Odd,
    /// Even parity
    Even,
}

impl Serialize for Parity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            Parity::No => "N",
            Parity::Odd => "O",
            Parity::Even => "E",
        };
        serializer.serialize_str(s)
    }
}

impl<'de> Deserialize<'de> for Parity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = String::deserialize(deserializer)?;
        match v.as_str() {
            "N" => Ok(Parity::No),
            "O" => Ok(Parity::Odd),
            "E" => Ok(Parity::Even),
            _ => Err(serde::de::Error::custom(format!(
                "invalid Parity value: {v}",
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusCode {
    /// magnetometer calibration alarm
    Calibration,
    /// low alarm
    Low,
    /// low warning
    LowWarning,
    /// normal
    Normal,
    /// high warning
    HighWarning,
    /// high alarm
    High,
    /// magnetometer voltage level alarm
    VoltageLevel,
}

impl<'de> Deserialize<'de> for StatusCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = String::deserialize(deserializer)?;
        match v.as_str() {
            "C" => Ok(StatusCode::Calibration),
            "L" => Ok(StatusCode::Low),
            "M" => Ok(StatusCode::LowWarning),
            "N" => Ok(StatusCode::Normal),
            "O" => Ok(StatusCode::HighWarning),
            "P" => Ok(StatusCode::High),
            "V" => Ok(StatusCode::VoltageLevel),
            _ => Err(serde::de::Error::custom(format!(
                "invalid StatusCode value: {v}",
            ))),
        }
    }
}

/// Earth-Centered, Earth-Fixed (ECEF) coordinates
///
/// Represents position and velocity in the ECEF coordinate system,
/// where the origin is at Earth's center of mass.
///
/// Reference: [gps_fix_t.ecef](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L245)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Ecef {
    /// X coordinate in meters
    #[serde(rename = "ecefx")]
    pub x: Option<f64>,
    /// Y coordinate in meters
    #[serde(rename = "ecefy")]
    pub y: Option<f64>,
    /// Z coordinate in meters
    #[serde(rename = "ecefz")]
    pub z: Option<f64>,
    /// Position accuracy in meters
    #[serde(rename = "ecefpAcc")]
    pub p_acc: Option<f64>,
    /// X velocity in meters/second
    #[serde(rename = "ecefvx")]
    pub vx: Option<f64>,
    /// Y velocity in meters/second
    #[serde(rename = "ecefvy")]
    pub vy: Option<f64>,
    /// Z velocity in meters/second
    #[serde(rename = "ecefvz")]
    pub vz: Option<f64>,
    /// Velocity accuracy in meters/second
    #[serde(rename = "ecefvAcc")]
    pub v_acc: Option<f64>,
}

/// North-East-Down (NED) coordinate system data
///
/// Represents position and velocity in the local tangent plane coordinate system.
/// NED is a local coordinate system with origin at the receiver position.
///
/// Reference: [gps_fix_t.ned](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L252)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Ned {
    /// Relative position North in meters
    #[serde(rename = "relN")]
    pub rel_pos_n: Option<f64>,
    /// Relative position East in meters
    #[serde(rename = "relE")]
    pub rel_pos_e: Option<f64>,
    /// Relative position Down in meters
    #[serde(rename = "relD")]
    pub rel_pos_d: Option<f64>,
    /// Relative horizontal position in meters
    #[serde(rename = "relH")]
    pub rel_pos_h: Option<f64>,
    /// Relative position length in meters
    #[serde(rename = "relL")]
    pub rel_pos_l: Option<f64>,
    /// Velocity North in meters/second
    #[serde(rename = "velN")]
    pub vel_n: Option<f64>,
    /// Velocity East in meters/second
    #[serde(rename = "velE")]
    pub vel_e: Option<f64>,
    /// Velocity Down in meters/second
    #[serde(rename = "velD")]
    pub vel_d: Option<f64>,
}

/// Dilution of Precision (DOP) values
///
/// DOP values indicate the quality of satellite geometry.
/// Lower values indicate better precision.
///
/// Reference: [dop_t](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L2557)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Dop {
    /// Longitude dilution of precision
    #[serde(rename = "xdop")]
    pub x: Option<f64>,
    /// Latitude dilution of precision
    #[serde(rename = "ydop")]
    pub y: Option<f64>,
    /// Position (3D) dilution of precision
    #[serde(rename = "pdop")]
    pub p: Option<f64>,
    /// Horizontal dilution of precision
    #[serde(rename = "hdop")]
    pub h: Option<f64>,
    /// Vertical dilution of precision
    #[serde(rename = "vdop")]
    pub v: Option<f64>,
    /// Time dilution of precision
    #[serde(rename = "tdop")]
    pub t: Option<f64>,
    /// Geometric dilution of precision
    #[serde(rename = "gdop")]
    pub g: Option<f64>,
}

/// RTK baseline information
///
/// Contains data about the RTK base station and baseline vector.
/// Used for high-precision positioning with RTK corrections.
///
/// Reference: [baseline_t](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L164)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Baseline {
    /// RTK solution status
    #[serde(rename = "baseS")]
    pub status: Option<FixStatus>,
    /// Baseline East component in meters
    #[serde(rename = "baseE")]
    pub east: Option<f64>,
    /// Baseline North component in meters
    #[serde(rename = "baseN")]
    pub north: Option<f64>,
    /// Baseline Up component in meters
    #[serde(rename = "baseU")]
    pub up: Option<f64>,
    /// Baseline length in meters
    #[serde(rename = "baseL")]
    pub length: Option<f64>,
    /// Baseline course in degrees
    #[serde(rename = "baseC")]
    pub course: Option<f64>,
    /// DGPS solution quality ratio
    #[serde(rename = "dgpsRatio")]
    pub ratio: Option<f64>,
}

/// Raw satellite measurement data
///
/// Contains raw observables from GPS receivers including
/// pseudoranges, carrier phases, and signal quality metrics.
///
/// Reference: [json_attrs_meas](https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/libgps_json.c#L226)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Measurement {
    /// GNSS system identifier
    pub gnssid: Option<GnssId>,
    /// Satellite ID within the GNSS
    pub svid: Option<u8>,
    /// Signal ID
    pub sigid: Option<u8>,
    /// Noise-to-signal ratio
    pub nsr: Option<u8>,
    /// Frequency channel ID (for GLONASS)
    pub freqid: Option<u8>,
    /// Observation code
    pub obs: Option<String>,
    /// Loss of Lock Indicator
    pub lli: Option<u8>,
    /// Carrier lock time in milliseconds
    pub locktime: Option<u32>,
    /// Carrier phase measurement in cycles
    pub carrierphase: Option<f64>,
    /// Pseudorange measurement in meters
    pub pseudorange: Option<f64>,
    /// Doppler frequency in Hz
    pub doppler: Option<f64>,
    /// Carrier-to-noise density ratio (dB-Hz)
    pub c2c: Option<f64>,
    /// L2C signal strength (dB-Hz)
    pub l2c: Option<f64>,
}

/// Information about a single satellite
///
/// Contains tracking status, signal strength, and position data
/// for an individual satellite.
///
/// Reference: [json_attrs_satellites](https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/libgps_json.c?ref_type=heads#L295)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Satellite {
    /// Pseudo-Random Noise code (satellite identifier)
    #[serde(rename = "PRN")]
    pub prn: i16,
    /// Azimuth angle in degrees (0-360)
    #[serde(rename = "az")]
    pub azimuth: Option<f64>,
    /// Elevation angle in degrees (0-90)
    #[serde(rename = "el")]
    pub elevation: Option<f64>,
    /// Frequency ID (for GLONASS)
    pub freqid: Option<i8>,
    /// GNSS system identifier
    pub gnssid: Option<GnssId>,
    /// Satellite health status
    pub health: Option<SatHealth>,
    /// Pseudorange in meters
    pub pr: Option<f64>,
    /// Pseudorange rate in meters/second
    #[serde(rename = "prRate")]
    pub pr_rate: Option<f64>,
    /// Pseudorange residual in meters
    #[serde(rename = "prRes")]
    pub pr_res: Option<f64>,
    /// Signal strength in dB-Hz
    pub ss: Option<f64>,
    /// Signal ID
    pub sigid: Option<u8>,
    /// Space vehicle ID
    pub svid: Option<u8>,
    /// Whether satellite is used in navigation solution
    pub used: bool,
    // Quality indicator (commented out in original)
    // #[serde(rename = "qual")]
    // pub quality: Option<SatQuality>,
}

/// GPS device configuration and status
///
/// Represents a GPS receiver device connected to GPSD,
/// including its configuration parameters and current status.
///
/// Reference: [json_device_read](https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/shared_json.c#L28)
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Device {
    /// Device path (e.g., "/dev/ttyUSB0")
    pub path: Option<String>,
    /// Timestamp when device was activated
    pub activated: Option<DateTime<Utc>>,
    /// Device capability flags
    pub flags: Option<PropertyFlags>,
    /// Driver name
    pub driver: Option<String>,
    /// Hex string for vendor/product or subtype
    pub hexdata: Option<String>,
    /// Serial number
    pub sernum: Option<String>,
    /// Device subtype
    pub subtype: Option<String>,
    /// Secondary device subtype
    pub subtype1: Option<String>,
    /// Native mode (0=native, 1=binary)
    pub native: Option<i32>,
    /// Serial port speed in bits per second
    pub bps: Option<i32>,
    /// Serial port parity
    pub parity: Option<Parity>,
    /// Number of stop bits
    pub stopbits: Option<u32>,
    /// Device cycle time in seconds
    pub cycle: Option<f64>,
    /// Minimum cycle time in seconds
    pub mincycle: Option<f64>,
}

/// Watch mode configuration
///
/// Controls what data GPSD streams to the client and in what format.
/// Used to enable/disable data streaming and configure output options.
///
/// Reference: [json_watch_read](https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/shared_json.c#L95)
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Watch {
    /// Specific device to watch (or all if None)
    pub device: Option<String>,
    /// Enable/disable streaming
    pub enable: Option<bool>,
    /// Enable JSON output
    pub json: Option<bool>,
    /// Enable NMEA output
    pub nmea: Option<bool>,
    /// Enable PPS timing output
    pub pps: Option<bool>,
    /// Raw mode (0=off, 1=hex, 2=binary)
    pub raw: Option<i32>,
    /// Enable scaled output
    pub scaled: Option<bool>,
    /// Split AIS type 24 messages
    pub split24: Option<bool>,
    /// Enable timing information
    pub timing: Option<bool>,
    /// Remote server URL
    pub remote: Option<String>,
}

impl Default for Watch {
    fn default() -> Self {
        Watch {
            device: None,
            enable: Some(false),
            json: Some(false),
            nmea: Some(false),
            pps: Some(false),
            raw: Some(0),
            scaled: Some(false),
            split24: Some(false),
            timing: Some(false),
            remote: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proto_v3_types_flags() {
        let flags = PropertyFlags::SEEN_GPS | PropertyFlags::SEEN_AIS;
        let serialized = serde_json::to_string(&flags).unwrap();
        assert_eq!(serialized, "9");

        let deserialized: PropertyFlags = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, flags);
    }
}
