use crate::protocol::GpsdRequestable;

use super::types::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Devices,
    Watch(Option<Watch>),
    Device(Option<Device>),
    Poll,
    Version,
}

impl GpsdRequestable for Message {
    fn to_command(&self) -> String {
        match self {
            Message::Devices => "?DEVICES;".into(),
            Message::Watch(Some(watch)) => {
                format!("?WATCH={};", serde_json::to_string(watch).unwrap())
            }
            Message::Watch(None) => "?WATCH;".into(),
            Message::Device(Some(device)) => {
                format!("?DEVICE={};", serde_json::to_string(device).unwrap())
            }
            Message::Device(None) => "?DEVICE;".into(),
            Message::Poll => "?POLL;".into(),
            Message::Version => "?VERSION;".into(),
        }
    }
}
