use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::types::*;

/// # Time-Position-Velocity
/// - [json_tpv_read](https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/libgps_json.c?ref_type=heads#L34)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Tpv {
    pub alt: Option<f64>,
    #[serde(rename = "altHAE")]
    pub alt_hae: Option<f64>,
    #[serde(rename = "altMSL")]
    pub alt_msl: Option<f64>,
    pub ant: Option<AntennaStatus>,
    #[serde(flatten)]
    pub base: Baseline,
    pub climb: Option<f64>,
    pub datum: Option<String>,
    pub device: Option<String>,
    pub depth: Option<f64>,
    #[serde(rename = "dgpsAge")]
    pub dgps_age: Option<f64>,
    #[serde(rename = "dgpsSta")]
    pub dgps_sta: Option<i32>,
    #[serde(flatten)]
    pub ecef: Ecef,
    pub epc: Option<f64>,
    pub epd: Option<f64>,
    pub eph: Option<f64>,
    pub eps: Option<f64>,
    pub ept: Option<f64>,
    pub epx: Option<f64>,
    pub epy: Option<f64>,
    pub epv: Option<f64>,
    #[serde(rename = "geoidSep")]
    pub geoid_sep: Option<f64>,
    pub lat: Option<f64>,
    pub jam: Option<i32>,
    pub leapseconds: Option<i32>,
    pub lon: Option<f64>,
    pub magtrack: Option<f64>,
    pub magvar: Option<f64>,
    pub mode: FixMode,
    #[serde(flatten)]
    pub ned: Option<Ned>,
    pub temp: Option<f64>,
    pub time: Option<DateTime<Utc>>,
    pub track: Option<f64>,
    pub sep: Option<f64>,
    pub speed: Option<f64>,
    pub status: Option<FixStatus>,
    pub wanglem: Option<f64>,
    pub wangler: Option<f64>,
    pub wanglet: Option<f64>,
    pub wspeedr: Option<f64>,
    pub wspeedt: Option<f64>,
    pub wtemp: Option<f64>,
    // policy->timing enable
    #[serde(rename = "rtime")]
    #[serde(default, deserialize_with = "f64_to_datetime")]
    pub rtime: Option<DateTime<Utc>>,
    #[serde(default, deserialize_with = "f64_to_datetime")]
    pub pps: Option<DateTime<Utc>>,
    #[serde(default, deserialize_with = "f64_to_datetime")]
    pub sor: Option<DateTime<Utc>>,
    pub chars: Option<u64>,
    pub sats: Option<i32>,
    pub week: Option<u16>,
    pub tow: Option<f64>,
    pub rollovers: Option<i32>,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

/// # Satellite Sky View
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Sky {
    pub device: Option<String>,
    #[serde(flatten)]
    pub dop: Option<Dop>,
    pub time: Option<DateTime<Utc>>,
    #[serde(rename = "nSat")]
    pub n_sat: Option<i32>,
    #[serde(rename = "uSat")]
    pub u_sat: Option<i32>,
    pub satellites: Option<Vec<Satellite>>,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

/// # Pseudorange Noise Statistics
/// - [json_noise_read](https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/libgps_json.c?ref_type=heads#L175)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Gst {
    pub device: Option<String>,
    pub time: Option<DateTime<Utc>>,
    pub alt: Option<f64>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub major: Option<f64>,
    pub minor: Option<f64>,
    pub orient: Option<f64>,
    pub rms: Option<f64>,
    pub ve: Option<f64>,
    pub vn: Option<f64>,
    pub vu: Option<f64>,
}

/// # Attitude
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Attitude {}

/// # Inertial Measurement Unit
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Imu {}

/// # Time Offset
/// - [json_toff_read](https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/libgps_json.c?ref_type=heads#L667)
#[derive(Debug, Clone, PartialEq)]
pub struct TimeOffset {
    pub device: Option<String>,
    pub real: Option<DateTime<Utc>>,
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

/// # Pulse-Per-Second Timing
#[derive(Debug, Clone, PartialEq)]
pub struct Pps {
    pub device: Option<String>,
    pub real: Option<DateTime<Utc>>,
    pub clock: Option<DateTime<Utc>>,
    pub precision: Option<i32>,
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

/// # Oscillator
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Oscillator {
    pub device: String,
    pub running: bool,
    pub reference: bool,
    pub disciplined: bool,
    // delta:
}

/// # Daemon Version Info
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Version {
    pub release: String,
    pub rev: String,
    pub proto_major: i32,
    pub proto_minor: i32,
    pub remote: Option<String>,
}

/// # Device List
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceList {
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

/// # Poll Snapshot
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Poll {}

/// # Error Notification
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Error {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Rtcm2 {}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Rtcm3 {}

// https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/libgps_json.c#L959
// #[cfg(feature = "ais")]
// #[derive(Debug, Clone, PartialEq, Deserialize)]
// pub struct Aivdm {}

/// Raw GPS data
/// - [json_raw_read](https://gitlab.com/gpsd/gpsd/-/blob/master/libgps/libgps_json.c#L219)
#[derive(Debug, Clone, PartialEq)]
pub struct Raw {
    pub device: Option<String>,
    pub time: Option<DateTime<Utc>>,
    pub rawdata: Option<Vec<Measurement>>,
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
            pub rawdata: Option<Vec<Measurement>>,
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
pub enum Message {
    Tpv(Tpv),
    Gst(Gst),
    Sky(Sky),
    Att(Attitude),
    Imu(Imu),
    Devices(DeviceList),
    Device(Device),
    Watch(Watch),
    Version(Version),
    Rtcm2(Rtcm2),
    Rtcm3(Rtcm3),
    // Ais(Aivdm),
    Error(Error),
    Toff(TimeOffset),
    Pps(Pps),
    Osc(Oscillator),
    Raw(Raw),
    Poll {
        active: Option<i32>,
        time: Option<DateTime<Utc>>,
        tpv: Option<Vec<Tpv>>,
        gst: Option<Vec<Gst>>,
        sky: Option<Vec<Sky>>,
    },
    #[serde(untagged)]
    Other(String),
}

fn f64_to_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<f64>::deserialize(deserializer)?;
    Ok(opt.and_then(|float| {
        DateTime::<Utc>::from_timestamp(float.trunc() as i64, ((float.fract()) * 1e9) as u32)
    }))
}

fn deserialize_to_datetime(sec: Option<i64>, nsec: Option<i64>) -> Option<DateTime<Utc>> {
    match (sec, nsec) {
        (Some(sec), Some(nsec)) => DateTime::<Utc>::from_timestamp(sec, nsec as u32),
        _ => None,
    }
}
