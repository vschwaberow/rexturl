use atty::Stream;
use clap::{Parser, ValueHint};
use rayon::prelude::*;
use serde::Serialize;
use std::fmt;
use std::io::{self, BufRead, BufWriter, Write};
use url::Url;

#[derive(Debug)]
enum AppError {
    IoError(io::Error),
    UrlParseError(url::ParseError),
    JsonError(serde_json::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::IoError(err) => write!(f, "IO error: {}", err),
            AppError::UrlParseError(err) => write!(f, "URL parse error: {}", err),
            AppError::JsonError(err) => write!(f, "JSON error: {}", err),
        }
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl From<url::ParseError> for AppError {
    fn from(err: url::ParseError) -> Self {
        AppError::UrlParseError(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::JsonError(err)
    }
}

#[derive(Serialize)]
struct UrlsOutput {
    urls: Vec<String>,
}

struct UrlComponents {
    scheme: String,
    username: String,
    subdomain: String,
    domain: String,
    hostname: String,
    port: String,
    path: String,
    query: String,
    fragment: String,
}

const MULTI_PART_TLDS: &[&str] = &[
    "co.uk", "org.uk", "ac.uk", "gov.uk", "me.uk", "net.uk", 
    "sch.uk", "com.au", "net.au", "org.au", "edu.au", "gov.au", 
    "co.nz", "net.nz", "org.nz", "govt.nz", "co.za", "org.za", 
    "com.br", "net.br", "org.br", "co.jp", "com.mx", "com.ar", 
    "com.sg", "com.my", "co.id", "com.hk", "co.th", "in.th"
];

fn is_multi_part_tld(domain: &str) -> bool {
    MULTI_PART_TLDS.iter().any(|tld| domain.ends_with(&format!(".{}", tld)))
}

fn extract_domain(host: &str) -> String {
    let parts: Vec<&str> = host.split('.').collect();
    let parts_len = parts.len();
    
    if parts_len <= 2 {
        return host.to_string();
    }
    
    let potential_domain = parts[(parts_len - 3)..].join(".");
    if parts_len >= 3 && is_multi_part_tld(&potential_domain) {
        return potential_domain;
    }
    
    parts[(parts_len - 2)..].join(".")
}

fn extract_subdomain(host: &str) -> String {
    let domain = extract_domain(host);
    
    if host == domain {
        return "".to_string();
    }
    
    host.strip_suffix(&format!(".{}", domain))
        .or_else(|| host.strip_suffix(&domain))
        .unwrap_or(host)
        .trim_end_matches('.')
        .to_string()
}

fn parse_url(url_str: &str) -> Result<Url, url::ParseError> {
    let url_with_scheme = if !url_str.contains("://") {
        format!("https://{}", url_str)
    } else {
        url_str.to_string()
    };
    
    Url::parse(&url_with_scheme)
}

fn extract_url_components(url: &Url) -> UrlComponents {
    let host_str = url.host_str().unwrap_or("");
    let domain = if !host_str.is_empty() { extract_domain(host_str) } else { String::new() };
    let subdomain = if !host_str.is_empty() { extract_subdomain(host_str) } else { String::new() };
    
    UrlComponents {
        scheme: url.scheme().to_string(),
        username: url.username().to_string(),
        subdomain,
        domain,
        hostname: host_str.to_string(),
        port: url.port().map_or(String::new(), |p| p.to_string()),
        path: url.path().to_string(),
        query: url.query().unwrap_or("").to_string(),
        fragment: url.fragment().unwrap_or("").to_string(),
    }
}

fn process_url(config: &Config, url_str: &str) -> Option<String> {
    match parse_url(url_str) {
        Ok(url) => {
            let components = extract_url_components(&url);
            let mut parts = Vec::new();

            if config.all || config.scheme { parts.push(components.scheme); }
            if config.all || config.username { parts.push(components.username); }
            if config.all || config.host { parts.push(components.subdomain); }
            if config.all || config.port { if !components.port.is_empty() { parts.push(components.port); } }
            if config.all || config.path { parts.push(components.path); }
            if config.all || config.query { if !components.query.is_empty() { parts.push(components.query); } }
            if config.all || config.fragment { if !components.fragment.is_empty() { parts.push(components.fragment); } }
            if config.all || config.domain { parts.push(components.domain); }

            if !parts.is_empty() {
                return Some(parts.join("\t"));
            }
        },
        Err(err) => {
            eprintln!("Error parsing URL '{}': {}", url_str, err);
        }
    }
    None
}

fn custom_format_url(url_str: &str, format: &str) -> Result<String, AppError> {
    match parse_url(url_str) {
        Ok(url) => {
            let components = extract_url_components(&url);
            
            let output = format
                .replace("{scheme}", &components.scheme)
                .replace("{username}", &components.username)
                .replace("{subdomain}", &components.subdomain)
                .replace("{host}", &components.hostname)
                .replace("{hostname}", &components.hostname)
                .replace("{domain}", &components.domain)
                .replace("{port}", &components.port)
                .replace("{path}", &components.path)
                .replace("{query}", &components.query)
                .replace("{fragment}", &components.fragment);
            Ok(output)
        },
        Err(err) => {
            eprintln!("Error parsing URL '{}': {}", url_str, err);
            Ok(String::new())
        }
    }
}

fn process_urls_parallel(config: &Config, urls: &[String]) -> Vec<String> {
    urls.par_iter()
        .filter_map(|url_str| process_url(config, url_str))
        .collect()
}

fn output_json(results: &[String]) -> Result<(), AppError> {
    let output = UrlsOutput { 
        urls: results.to_vec() 
    };
    
    let json_string = serde_json::to_string_pretty(&output)?;
    println!("{}", json_string);
    Ok(())
}

fn process_urls_streaming<R: BufRead>(config: &Config, reader: R) -> Result<(), AppError> {
    let mut results = Vec::new();
    
    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }
                
                if config.custom && !config.json {
                    if let Ok(output) = custom_format_url(&line, &config.format) {
                        if !output.is_empty() {
                            println!("{}", output);
                        }
                    }
                } else if config.custom && config.json {
                    if let Ok(output) = custom_format_url(&line, &config.format) {
                        if !output.is_empty() {
                            results.push(output);
                        }
                    }
                } else {
                    if let Some(result) = process_url(config, &line) {
                        results.push(result);
                    }
                }
            },
            Err(e) => {
                eprintln!("Error reading line: {}", e);
            }
        }
    }
    
    if !results.is_empty() || config.json {
        if config.sort { results.sort(); }
        if config.unique { results.dedup(); }
        
        if config.json {
            output_json(&results)?;
        } else {
            let stdout = io::stdout();
            let mut writer = BufWriter::new(stdout.lock());
            for line in results {
                if let Err(e) = writeln!(writer, "{}", line) {
                    eprintln!("Error writing output: {}", e);
                }
            }
        }
    }
    
    Ok(())
}

#[derive(Debug, Parser, Clone)]
#[command(author, version, about = "A tool for parsing and manipulating URLs", long_about = None)]
struct Config {
    #[arg(long, value_hint = ValueHint::AnyPath, help = "Input URLs to process")]
    urls: Vec<String>,
    #[arg(long, help = "Extract and display the URL scheme")]
    scheme: bool,
    #[arg(long, help = "Extract and display the username from the URL")]
    username: bool,
    #[arg(long, help = "Extract and display the hostname")]
    host: bool,
    #[arg(long, help = "Extract and display the port number")]
    port: bool,
    #[arg(long, help = "Extract and display the URL path")]
    path: bool,
    #[arg(long, help = "Extract and display the query string")]
    query: bool,
    #[arg(long, help = "Extract and display the URL fragment")]
    fragment: bool,
    #[arg(long, help = "Sort the output")]
    sort: bool,
    #[arg(long, help = "Remove duplicate entries from the output")]
    unique: bool,
    #[arg(long, help = "Output results in JSON format")]
    json: bool,
    #[arg(long, help = "Display all URL components")]
    all: bool,
    #[arg(long, help = "Enable custom output mode")]
    custom: bool,
    #[arg(
        long, 
        num_args = 0..=1,
        default_missing_value = "{scheme}://{host}{path}",
        default_value = "{scheme}://{host}{path}",
        help = "Custom output format. Available placeholders: {scheme}, {username}, {subdomain}, {host}, {hostname}, {domain}, {port}, {path}, {query}, {fragment}"
    )]
    format: String,
    #[arg(long, help = "Extract and display the domain")]
    domain: bool,
}

impl Config {
    fn from_args() -> Self { Self::parse() }
}

fn check_for_stdin() -> Result<(), AppError> {
    if atty::is(Stream::Stdin) && Config::from_args().urls.is_empty() {
        eprintln!("Error: No input URLs provided. Use --urls or pipe input from stdin.");
        std::process::exit(1);
    }
    Ok(())
}

fn main() -> Result<(), AppError> {
    let mut config = Config::parse();
    
    if config.urls.is_empty() {
        check_for_stdin()?;
    }
    
    if !config.format.is_empty() || config.custom {
        config.custom = true;
    }
    
    if !config.urls.is_empty() {
        if config.custom && !config.json {
            config.urls.iter().for_each(|url_str| {
                if let Ok(output) = custom_format_url(url_str, &config.format) {
                    if !output.is_empty() {
                        println!("{}", output);
                    }
                }
            });
        } else {
            let mut results = if config.custom {
                config.urls.par_iter()
                    .filter_map(|url_str| {
                        if let Ok(output) = custom_format_url(url_str, &config.format) {
                            if !output.is_empty() {
                                return Some(output);
                            }
                        }
                        None
                    })
                    .collect()
            } else {
                process_urls_parallel(&config, &config.urls)
            };
            
            if config.sort { results.sort(); }
            if config.unique { results.dedup(); }
            
            if config.json {
                output_json(&results)?;
            } else {
                let stdout = io::stdout();
                let mut writer = BufWriter::new(stdout.lock());
                for line in results {
                    writeln!(writer, "{}", line)?;
                }
            }
        }
    } else {
        let stdin = io::stdin();
        process_urls_streaming(&config, stdin.lock())?;
    }
    
    Ok(())
}
