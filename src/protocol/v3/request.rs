use super::types::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Devices,
    Watch(Option<Watch>),
    Device(Option<Device>),
    Poll,
    Version,
}
