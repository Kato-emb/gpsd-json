use crate::error::GpsdJsonError;

pub mod client;
pub mod error;
pub mod protocol;

pub type Result<T> = core::result::Result<T, GpsdJsonError>;
