pub enum GpsdJsonError {
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
    UnsupportedProtocolVersion((i32, i32)),
    ProtocolError(&'static str),
}

impl core::fmt::Debug for GpsdJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpsdJsonError::IoError(err) => write!(f, "IoError: {}", err),
            GpsdJsonError::SerdeError(err) => write!(f, "SerdeError: {}", err),
            GpsdJsonError::UnsupportedProtocolVersion((major, minor)) => {
                write!(f, "UnsupportedProtocolVersion: {}.{}", major, minor)
            }
            GpsdJsonError::ProtocolError(msg) => write!(f, "ProtocolError: {}", msg),
        }
    }
}

impl core::fmt::Display for GpsdJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpsdJsonError::IoError(err) => write!(f, "IoError: {}", err),
            GpsdJsonError::SerdeError(err) => write!(f, "SerdeError: {}", err),
            GpsdJsonError::UnsupportedProtocolVersion((major, minor)) => {
                write!(f, "UnsupportedProtocolVersion: {}.{}", major, minor)
            }
            GpsdJsonError::ProtocolError(msg) => write!(f, "ProtocolError: {}", msg),
        }
    }
}

impl core::error::Error for GpsdJsonError {}
