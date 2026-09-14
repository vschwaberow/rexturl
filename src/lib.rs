pub mod config;
pub mod domain;
pub mod error;
pub mod formatter;
pub mod parser;
pub mod url;

pub use config::{check_for_stdin, Config};
pub use error::AppError;
pub use parser::{extract_url_components, parse_and_extract_components, parse_url, UrlComponents};
pub use url::{Url, UrlParseError};
