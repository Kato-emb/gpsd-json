use crate::{
    client::GpsdJsonProtocol,
    protocol::{GpsdJsonRequest, GpsdJsonResponse},
};

pub mod request;
pub mod response;
pub mod types;

/// - [release-3.25](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/SConscript?ref_type=tags#L226)
pub const API_VERSION_MAJOR: i32 = 3;
pub const API_VERSION_MINOR: i32 = 15;

#[derive(Debug)]
pub struct V3;

impl GpsdJsonProtocol for V3 {
    const API_VERSION_MAJOR: i32 = API_VERSION_MAJOR;
    const API_VERSION_MINOR: i32 = API_VERSION_MINOR;

    type Request = request::Message;
    type Response = response::Message;
}

pub type ResponseMessage = response::Message;
impl GpsdJsonResponse for ResponseMessage {}

pub type RequestMessage = request::Message;
impl GpsdJsonRequest for RequestMessage {
    fn to_command(&self) -> String {
        match self {
            RequestMessage::Devices => "?DEVICES;".into(),
            RequestMessage::Watch(Some(watch)) => {
                format!("?WATCH={};", serde_json::to_string(watch).unwrap())
            }
            RequestMessage::Watch(None) => "?WATCH;".into(),
            RequestMessage::Device(Some(device)) => {
                format!("?DEVICE={};", serde_json::to_string(device).unwrap())
            }
            RequestMessage::Device(None) => "?DEVICE;".into(),
            RequestMessage::Poll => "?POLL;".into(),
            RequestMessage::Version => "?VERSION;".into(),
        }
    }
}
