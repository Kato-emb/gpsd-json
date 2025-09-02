pub enum GpsdJsonError {
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
}

impl core::fmt::Debug for GpsdJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpsdJsonError::IoError(err) => write!(f, "IoError: {}", err),
            GpsdJsonError::SerdeError(err) => write!(f, "SerdeError: {}", err),
        }
    }
}

impl core::fmt::Display for GpsdJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpsdJsonError::IoError(err) => write!(f, "IoError: {}", err),
            GpsdJsonError::SerdeError(err) => write!(f, "SerdeError: {}", err),
        }
    }
}

impl core::error::Error for GpsdJsonError {}
