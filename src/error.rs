use thiserror::Error;

#[derive(Debug, Clone, Copy, Error)]
pub enum Error {
    #[error("KeyAlreadyExists")]
    KeyAlreadyExists,
    #[error("KeyWasNotFound")]
    KeyWasNotFound,
    #[error("UnexpectedError")]
    UnexpectedError,
    #[error("FileAlreadyExists")]
    FileAlreadyExists,
    #[error("FileAlreadyOpened")]
    FileAlreadyOpened,
    #[error("IoError")]
    IoError,
    #[error("FileDoesntExist")]
    FileDoesntExist,
    #[error("DirAlreadyExists")]
    DirAlreadyExists,
    #[error("DirDoesntExist")]
    DirDoesntExist,
    #[error("PathDoesntExist")]
    PathDoesntExist,
    #[error("ErrorDeserializing")]
    ErrorDeserializing,
    #[error("ErrorSerializing")]
    ErrorSerializing,
    #[error("OutOfBounds")]
    OutOfBounds,
    #[error("VertexAlreadyExists")]
    VertexAlreadyExists,
    #[error("EdgeAlreadyExists")]
    EdgeAlreadyExists,
    #[error("NullPointer")]
    NullPointer,
    #[error("MismatchedVicinity")]
    MismatchedVicinity,
    #[error("Error was: {0}")]
    WithMessage(&'static str),
}

impl std::convert::From<std::io::Error> for Error {
    fn from(_e: std::io::Error) -> Error {
        Error::UnexpectedError
    }
}
