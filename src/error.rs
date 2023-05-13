use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZipParseError {
    #[error("file too big. was {0} bytes")]
    FileTooLarge(u64),
    #[error("io error {0}")]
    IoError(#[from] std::io::Error),
}
