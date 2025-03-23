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

            if config.host && !config.all && !config.domain && !config.scheme && 
               !config.username && !config.port && !config.path && 
               !config.query && !config.fragment {
                if !components.subdomain.is_empty() {
                    return Some(components.subdomain);
                }
                return None;
            }

            if config.all || config.scheme { parts.push(components.scheme); }
            
            if (config.all || config.username) && !components.username.is_empty() { 
                parts.push(components.username); 
            }
            
            if config.all || config.host { 
                if config.host && !config.all {
                    parts.push(components.subdomain); 
                } else if config.all {
                    parts.push(components.hostname);
                }
            }
            
            if (config.all || config.port) && !components.port.is_empty() { 
                parts.push(components.port); 
            }
            
            if config.all || config.path { parts.push(components.path); }
            
            if (config.all || config.query) && !components.query.is_empty() { parts.push(components.query); }
            if (config.all || config.fragment) && !components.fragment.is_empty() { parts.push(components.fragment); }
            
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
    let output = if results.iter().all(|s| !s.contains('\t')) && results.len() == 1 {
        UrlsOutput { 
            urls: results.to_vec() 
        }
    } else {
        UrlsOutput { 
            urls: results.to_vec() 
        }
    };
    
    let json_string = serde_json::to_string_pretty(&output)?;
    println!("{}", json_string);
    Ok(())
}

fn process_urls_streaming<R: BufRead>(config: &Config, reader: R) -> Result<(), AppError> {
    let mut results = Vec::new();
    
    for line_result in reader.lines() {
        if let Ok(line) = line_result {
            let line = line.trim();
            if !line.is_empty() {
                if config.host && !config.all && !config.custom && !config.json && 
                   !config.domain && !config.scheme && !config.username && !config.port && 
                   !config.path && !config.query && !config.fragment {
                    if let Ok(url) = parse_url(line) {
                        if let Some(host_str) = url.host_str() {
                            let subdomain = extract_subdomain(host_str);
                            if !subdomain.is_empty() {
                                results.push(subdomain);
                            } else {
                                results.push(host_str.to_string());
                            }
                        }
                    }
                } else if config.custom && !config.json {
                    if let Ok(output) = custom_format_url(line, &config.format) {
                        if !output.is_empty() {
                            results.push(output);
                        }
                    }
                } else {
                    if let Some(result) = process_url(config, line) {
                        results.push(result);
                    }
                }
            }
        }
    }
    
    if config.sort { results.sort(); }
    if config.unique { results.dedup(); }
    
    if config.json {
        output_json(&results)?;
    } else {
        let stdout = io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        for result in results {
            writeln!(writer, "{}", result)?;
        }
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
    
    if config.host && !config.all && !config.custom && !config.urls.is_empty() && 
       !config.domain && !config.scheme && !config.username && !config.port && 
       !config.path && !config.query && !config.fragment {
        let mut results = Vec::new();
        
        for url_str in &config.urls {
            let url_str = url_str.trim();
            if let Ok(url) = parse_url(url_str) {
                if let Some(host) = url.host_str() {
                    let subdomain = extract_subdomain(host);
                    if !subdomain.is_empty() {
                        results.push(subdomain);
                    }
                }
            }
        }
        
        if config.sort { results.sort(); }
        if config.unique { results.dedup(); }
        
        let stdout = io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        for result in results {
            writeln!(writer, "{}", result)?;
        }
        
        return Ok(());
    }
    
    if config.all && !config.urls.is_empty() {
        for url_str in &config.urls {
            let url_str = url_str.trim();
            if let Ok(url) = parse_url(url_str) {
                let components = extract_url_components(&url);
                
                let mut parts = Vec::new();
                parts.push(components.scheme);
                
                if !components.username.is_empty() {
                    parts.push(components.username);
                }
                
                parts.push(components.hostname);
                
                if !components.port.is_empty() {
                    parts.push(components.port);
                }
                
                parts.push(components.path);
                
                if !components.query.is_empty() {
                    parts.push(components.query);
                }
                
                if !components.fragment.is_empty() {
                    parts.push(components.fragment);
                }
                
                parts.push(components.domain);
                
                println!("{}", parts.join("\t"));
            }
        }
        return Ok(());
    }
    
    if !config.urls.is_empty() {
        if config.custom && !config.json {
            config.urls.iter().for_each(|url_str| {
                let url_str = url_str.trim();
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
                        let url_str = url_str.trim();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain_simple() {
        assert_eq!(extract_domain("example.com"), "example.com");
        assert_eq!(extract_domain("www.example.com"), "example.com");
        assert_eq!(extract_domain("blog.example.com"), "example.com");
    }

    #[test]
    fn test_extract_domain_multipart_tld() {
        assert_eq!(extract_domain("example.co.uk"), "example.co.uk");
        assert_eq!(extract_domain("www.example.co.uk"), "example.co.uk");
        assert_eq!(extract_domain("blog.example.co.uk"), "example.co.uk");
    }

    #[test]
    fn test_extract_subdomain() {
        assert_eq!(extract_subdomain("example.com"), "");
        assert_eq!(extract_subdomain("www.example.com"), "www");
        assert_eq!(extract_subdomain("blog.example.com"), "blog");
        assert_eq!(extract_subdomain("blog.dev.example.com"), "blog.dev");
    }

    #[test]
    fn test_extract_subdomain_multipart_tld() {
        assert_eq!(extract_subdomain("example.co.uk"), "");
        assert_eq!(extract_subdomain("www.example.co.uk"), "www");
        assert_eq!(extract_subdomain("blog.example.co.uk"), "blog");
    }

    #[test]
    fn test_parse_url() {
        assert!(parse_url("example.com").is_ok());
        assert!(parse_url("https://example.com").is_ok());
        assert!(parse_url("http://example.com").is_ok());
        
        let url = parse_url("example.com").unwrap();
        assert_eq!(url.scheme(), "https");
    }

    #[test]
    fn test_extract_url_components() {
        let url = parse_url("https://user@www.example.co.uk:8080/path?query=value#fragment").unwrap();
        let components = extract_url_components(&url);
        
        assert_eq!(components.scheme, "https");
        assert_eq!(components.username, "user");
        assert_eq!(components.hostname, "www.example.co.uk");
        assert_eq!(components.subdomain, "www");
        assert_eq!(components.domain, "example.co.uk");
        assert_eq!(components.port, "8080");
        assert_eq!(components.path, "/path");
        assert_eq!(components.query, "query=value");
        assert_eq!(components.fragment, "fragment");
    }

    #[test]
    fn test_process_url() {
        let mut config = Config::parse_from([""]);
        config.host = true;
        
        let result = process_url(&config, "https://www.example.com");
        assert_eq!(result, Some("www".to_string()));
        
        config.host = false;
        config.domain = true;
        let result = process_url(&config, "https://www.example.com");
        assert_eq!(result, Some("example.com".to_string()));
        
        config.domain = false;
        config.all = true;
        let result = process_url(&config, "https://www.example.com");
        assert!(result.is_some());
    }

    #[test]
    fn test_custom_format_url() {
        let format = "{scheme}://{host}{path}";
        let result = custom_format_url("https://www.example.com/path", format).unwrap();
        assert_eq!(result, "https://www.example.com/path");
        
        let format = "{scheme}://{subdomain}.{domain}{path}";
        let result = custom_format_url("https://www.example.com/path", format).unwrap();
        assert_eq!(result, "https://www.example.com/path");
        
        let format = "{scheme}://{hostname}{path}?{query}#{fragment}";
        let result = custom_format_url("https://www.example.com/path?q=1#f", format).unwrap();
        assert_eq!(result, "https://www.example.com/path?q=1#f");
        
        let format = "{scheme}://{username}@{subdomain}.{domain}:{port}{path}?{query}#{fragment}";
        let result = custom_format_url("https://user@blog.example.com:8080/path?q=1#f", format).unwrap();
        assert_eq!(result, "https://user@blog.example.com:8080/path?q=1#f");
    }
}
