use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZipParseError {
    #[error("file too big. was {0} bytes")]
    FileTooLarge(u64),
    #[error("io error {0}")]
    IoError(#[from] std::io::Error),
    #[error("found {found:?}, expected {expected:?}")]
    MalformedSignature { found: [u8; 4], expected: [u8; 4] },
    #[error("generic error: {0}")]
    Generic(&'static str),
    #[error("expected file to be longer")]
    UnexpectedEof,
    #[error("unable to locate central directory signature")]
    MissingCentralDirectory,
}
