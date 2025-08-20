use std::fmt;
use std::io;

use crate::url::UrlParseError;

#[derive(Debug)]
pub enum AppError {
    IoError(io::Error),
    UrlParseError(UrlParseError),
    JsonError(serde_json::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::IoError(err) => write!(f, "IO error: {err}"),
            AppError::UrlParseError(err) => write!(f, "URL parse error: {err}"),
            AppError::JsonError(err) => write!(f, "JSON error: {err}"),
        }
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl From<UrlParseError> for AppError {
    fn from(err: UrlParseError) -> Self {
        AppError::UrlParseError(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::JsonError(err)
    }
}
