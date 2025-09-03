use crate::{Result, error::GpsdJsonError};

pub mod v3;

pub trait GpsdJsonResponse: serde::de::DeserializeOwned {}

pub trait GpsdJsonDecode: std::io::BufRead {
    fn read_response<Res>(&mut self, buf: &mut String) -> Result<Option<Res>>
    where
        Res: GpsdJsonResponse,
    {
        buf.clear();
        let bytes_read = self.read_line(buf).map_err(GpsdJsonError::IoError)?;
        if bytes_read == 0 {
            return Ok(None); // EOF reached
        }

        let response = serde_json::from_str(buf).map_err(GpsdJsonError::SerdeError)?;
        Ok(Some(response))
    }
}

impl<R: std::io::BufRead + ?Sized> GpsdJsonDecode for R {}

pub trait GpsdJsonRequest {
    fn to_command(&self) -> String;
}

pub trait GpsdJsonEncode: std::io::Write {
    fn write_request(&mut self, request: &impl GpsdJsonRequest) -> Result<()> {
        let cmd = request.to_command();
        self.write_all(cmd.as_bytes())
            .map_err(GpsdJsonError::IoError)
    }
}

impl<W: std::io::Write + ?Sized> GpsdJsonEncode for W {}
