use std::sync::mpsc;

use crate::config::MTU;

#[derive(Debug)]
pub enum ErrorKind {
    SerialPort,
    Internal,
    PayloadTooLarge,
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub description: String,
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.description
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.description)
    }
}

impl Error {
    pub(crate) fn from_kind(kind: ErrorKind) -> Self {
        match kind {
            ErrorKind::PayloadTooLarge => Self {
                kind: ErrorKind::PayloadTooLarge,
                description: format!("payload can't be larger than MTU of {}", MTU),
            },
            _ => unimplemented!(),
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl From<mpsc::SendError<Vec<u8>>> for Error {
    fn from(_error: mpsc::SendError<Vec<u8>>) -> Self {
        Self {
            kind: ErrorKind::Internal,
            description: "internal tx thread shut down".to_string(),
        }
    }
}

impl From<serialport::Error> for Error {
    fn from(error: serialport::Error) -> Self {
        Self {
            kind: ErrorKind::SerialPort,
            description: error.description,
        }
    }
}
