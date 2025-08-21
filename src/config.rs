use clap::{Parser, ValueEnum, ValueHint};
use std::io::IsTerminal;

use crate::error::AppError;
use crate::formatter::{EscapeMode, Format, SqlDialect};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ColorMode {
    Auto,
    Never,
    Always,
}

impl Default for ColorMode {
    fn default() -> Self {
        ColorMode::Auto
    }
}

#[derive(Debug, Parser, Clone)]
#[command(author, version, about = "A tool for parsing and manipulating URLs", long_about = None)]
pub struct Config {
    #[arg(long, value_hint = ValueHint::AnyPath, num_args = 1.., help = "Input URLs to process")]
    pub urls: Vec<String>,

    #[arg(long, help = "Extract and display the URL scheme")]
    pub scheme: bool,
    #[arg(long, help = "Extract and display the username from the URL")]
    pub username: bool,
    #[arg(long, help = "Extract and display the hostname")]
    pub host: bool,
    #[arg(long, help = "Extract and display the port number")]
    pub port: bool,
    #[arg(long, help = "Extract and display the URL path")]
    pub path: bool,
    #[arg(long, help = "Extract and display the query string")]
    pub query: bool,
    #[arg(long, help = "Extract and display the URL fragment")]
    pub fragment: bool,
    #[arg(long, help = "Extract and display the domain")]
    pub domain: bool,

    #[arg(long, value_enum, default_value = "plain", help = "Output format")]
    pub format: Format,
    #[arg(
        long,
        help = "Comma-separated list of fields to output (e.g., domain,path,url)"
    )]
    pub fields: Option<String>,
    #[arg(long, help = "Include header row for tabular formats")]
    pub header: bool,
    #[arg(long, help = "Pretty-print JSON output")]
    pub pretty: bool,
    #[arg(
        long,
        value_enum,
        default_value = "auto",
        help = "When to use colored output (plain format only)"
    )]
    pub color: ColorMode,
    #[arg(
        long,
        default_value = "\\N",
        help = "Value to print for missing fields in tabular formats"
    )]
    pub null_empty: String,
    #[arg(long, help = "Exit with non-zero code if any URL fails to parse")]
    pub strict: bool,
    #[arg(long, help = "Suppress trailing newline")]
    pub no_newline: bool,

    #[arg(long, help = "Sort the output")]
    pub sort: bool,
    #[arg(long, help = "Remove duplicate entries from the output")]
    pub unique: bool,

    #[arg(
        long,
        help = "Custom format template (e.g., '{scheme}://{domain}{path}')"
    )]
    pub template: Option<String>,
    #[arg(
        long,
        value_enum,
        default_value = "none",
        help = "Escaping mode for custom format"
    )]
    pub escape: EscapeMode,

    #[arg(long, default_value = "urls", help = "Table name for SQL output")]
    pub sql_table: String,
    #[arg(long, help = "Include CREATE TABLE statement in SQL output")]
    pub sql_create_table: bool,
    #[arg(long, value_enum, default_value = "postgres", help = "SQL dialect")]
    pub sql_dialect: SqlDialect,

    #[arg(
        long,
        help = "Output results in JSON format (deprecated, use --format json)"
    )]
    pub json: bool,
    #[arg(long, help = "Display all URL components (deprecated, use --fields)")]
    pub all: bool,
    #[arg(long, help = "Enable custom output mode (deprecated)")]
    pub custom: bool,
    #[arg(
        long,
        help = "Custom output format (deprecated, use --format and --fields)"
    )]
    pub legacy_format: Option<String>,
}

impl Config {
    pub fn from_args() -> Self {
        Self::parse()
    }
}

pub fn check_for_stdin() -> Result<(), AppError> {
    if std::io::stdin().is_terminal() && Config::from_args().urls.is_empty() {
        eprintln!("Error: No input URLs provided. Use --urls or pipe input from stdin.");
        std::process::exit(1);
    }
    Ok(())
}
