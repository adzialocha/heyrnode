#[derive(Debug)]
pub enum ErrorKind {
    SerialPort,
    Io,
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
                description: format!("payload can't be larger than MTU of {}", crate::config::MTU),
            },
            _ => unimplemented!(),
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Io,
            description: error.to_string(),
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
