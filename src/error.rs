use thiserror::Error;

pub type Result<T> = std::result::Result<T, ZipperError>;

#[derive(Error, Debug)]
pub enum ZipperError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Archive error: {0}")]
    Archive(String),

    #[error("Git repository error: {0}")]
    Git(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

impl ZipperError {
    pub fn archive(msg: impl Into<String>) -> Self {
        Self::Archive(msg.into())
    }

    pub fn git(msg: impl Into<String>) -> Self {
        Self::Git(msg.into())
    }

    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }
}
