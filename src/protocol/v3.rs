pub mod request;
pub mod response;
pub mod types;

/// - [release-3.25](https://gitlab.com/gpsd/gpsd/-/blob/release-3.25/SConscript?ref_type=tags#L226)
pub const API_VERSION_MINOR: i32 = 15;

pub type ResponseMessage = response::Message;
pub type RequestMessage = request::Message;
