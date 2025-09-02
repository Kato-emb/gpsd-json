use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use serde_with::skip_serializing_none;

/// * [gps_fix_t.mode](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L181)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize_repr)]
#[repr(i32)]
pub enum FixMode {
    NotSeen = 0,
    NoFix = 1,
    Fix2D = 2,
    Fix3D = 3,
}

/// * [gps_fix_t.status](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L192)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize_repr)]
#[repr(i32)]
pub enum FixStatus {
    /// Unknown status
    Unknown = 0,
    Gps = 1,
    /// with DGPS
    DGps = 2,
    /// with RTK Fixed
    RTKFixed = 3,
    /// with RTK Float
    RTKFloat = 4,
    /// with dead reckoning
    DR = 5,
    /// with GNSS + dead reckoning
    GnssDR = 6,
    /// time only (surveyed in, manual)
    Time = 7,
    /// simulated
    Simulated = 8,
    /// Precise Positioning Service (PPS)
    PpsFix = 9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize_repr)]
#[repr(i32)]
pub enum AntennaStatus {
    Unknown = 0,
    Ok = 1,
    Open = 2,
    Short = 3,
}

/// * [satellite.qualityInd](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L2411)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SatQuality {
    Invalid,
    NoSignal,
    Searching,
    Acquired,
    Unusable,
    CodeLocked,
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
            5 | 6 | 7 => Ok(SatQuality::CodeCarrierLocked),
            _ => Err(serde::de::Error::custom(format!(
                "invalid Satellite QualityInd value: {v}"
            ))),
        }
    }
}

/// * [satellite.gnssid](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L2449)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize_repr)]
#[repr(u8)]
pub enum GnssId {
    Gps = 0,
    Sbas = 1,
    Gal = 2,
    Bd = 3,
    Imes = 4,
    Qzss = 5,
    Glo = 6,
    Irnss = 7,
}

/// * [satellite.health](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L2504)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize_repr)]
#[repr(u8)]
pub enum SatHealth {
    Unknown = 0,
    Ok = 1,
    Bad = 2,
}

bitflags::bitflags! {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Parity {
    No,
    Odd,
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
                "invalid Parity value: {}",
                v
            ))),
        }
    }
}

/// ECEF data, all data in meters, and meters/second, or NaN
/// * [gps_fix_t.ecef](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L245)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Ecef {
    #[serde(rename = "ecefx")]
    pub x: Option<f64>,
    #[serde(rename = "ecefy")]
    pub y: Option<f64>,
    #[serde(rename = "ecefz")]
    pub z: Option<f64>,
    #[serde(rename = "ecefpAcc")]
    pub p_acc: Option<f64>,
    #[serde(rename = "ecefvx")]
    pub vx: Option<f64>,
    #[serde(rename = "ecefvy")]
    pub vy: Option<f64>,
    #[serde(rename = "ecefvz")]
    pub vz: Option<f64>,
    #[serde(rename = "ecefvAcc")]
    pub v_acc: Option<f64>,
}

/// NED data, all data in meters, and meters/second, or NaN
/// * [gps_fix_t.ned](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L252)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Ned {
    #[serde(rename = "relN")]
    pub rel_pos_n: Option<f64>,
    #[serde(rename = "relE")]
    pub rel_pos_e: Option<f64>,
    #[serde(rename = "relD")]
    pub rel_pos_d: Option<f64>,
    #[serde(rename = "relH")]
    pub rel_pos_h: Option<f64>,
    #[serde(rename = "relL")]
    pub rel_pos_l: Option<f64>,
    #[serde(rename = "velN")]
    pub vel_n: Option<f64>,
    #[serde(rename = "velE")]
    pub vel_e: Option<f64>,
    #[serde(rename = "velD")]
    pub vel_d: Option<f64>,
}

/// * [dop_t](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L2557)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Dop {
    #[serde(rename = "xdop")]
    pub x: Option<f64>,
    #[serde(rename = "ydop")]
    pub y: Option<f64>,
    #[serde(rename = "pdop")]
    pub p: Option<f64>,
    #[serde(rename = "hdop")]
    pub h: Option<f64>,
    #[serde(rename = "vdop")]
    pub v: Option<f64>,
    #[serde(rename = "tdop")]
    pub t: Option<f64>,
    #[serde(rename = "gdop")]
    pub g: Option<f64>,
}

/// * [baseline_t](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/include/gps.h?ref_type=tags#L164)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Baseline {
    #[serde(rename = "baseS")]
    pub status: Option<FixStatus>,
    #[serde(rename = "baseE")]
    pub east: Option<f64>,
    #[serde(rename = "baseN")]
    pub north: Option<f64>,
    #[serde(rename = "baseU")]
    pub up: Option<f64>,
    #[serde(rename = "baseL")]
    pub length: Option<f64>,
    #[serde(rename = "baseC")]
    pub course: Option<f64>,
    #[serde(rename = "dgpsRatio")]
    pub ratio: Option<f64>,
}

/// - [json_attrs_meas](https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/libgps_json.c#L226)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Measurement {
    pub gnssid: Option<GnssId>,
    pub svid: Option<u8>,
    pub sigid: Option<u8>,
    pub nsr: Option<u8>,
    pub freqid: Option<u8>,
    pub obs: Option<String>,
    pub lli: Option<u8>,
    pub locktime: Option<u32>,
    pub carrierphase: Option<f64>,
    pub pseudorange: Option<f64>,
    pub doppler: Option<f64>,
    pub c2c: Option<f64>,
    pub l2c: Option<f64>,
}

/// - [json_attrs_satellites](https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/libgps_json.c?ref_type=heads#L295)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Satellite {
    #[serde(rename = "PRN")]
    pub prn: i16,
    #[serde(rename = "az")]
    pub azimuth: Option<f64>,
    #[serde(rename = "el")]
    pub elevation: Option<f64>,
    pub freqid: Option<i8>,
    pub gnssid: Option<GnssId>,
    pub health: Option<SatHealth>,
    pub pr: Option<f64>,
    #[serde(rename = "prRate")]
    pub pr_rate: Option<f64>,
    #[serde(rename = "prRes")]
    pub pr_res: Option<f64>,
    pub ss: Option<f64>,
    pub sigid: Option<u8>,
    pub svid: Option<u8>,
    pub used: bool,
    // #[serde(rename = "qual")]
    // pub quality: Option<SatQuality>,
}

/// # Device Information
/// - [json_device_read](https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/shared_json.c#L28)
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub path: Option<String>,
    pub activated: Option<DateTime<Utc>>,
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

/// # Watch Policy
/// - [json_watch_read](https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/shared_json.c#L95)
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Watch {
    pub device: Option<String>,
    pub enable: Option<bool>,
    pub json: Option<bool>,
    pub nmea: Option<bool>,
    pub pps: Option<bool>,
    pub raw: Option<i32>,
    pub scaled: Option<bool>,
    pub split24: Option<bool>,
    pub timing: Option<bool>,
    pub remote: Option<String>,
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
