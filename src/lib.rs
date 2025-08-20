pub mod config;
pub mod domain;
pub mod error;
pub mod formatter;
pub mod output;
pub mod parser;
pub mod processor;
pub mod url;
pub mod url_parser;

pub use config::{check_for_stdin, Config};
pub use error::AppError;
pub use output::{custom_format_url, output_json};
pub use parser::{extract_url_components, parse_and_extract_components, parse_url, UrlComponents};
pub use processor::{process_url, process_urls_parallel, process_urls_streaming};
pub use url::{Url, UrlParseError};
