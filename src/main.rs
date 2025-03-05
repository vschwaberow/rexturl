use atty::Stream;
use clap::{Parser, ValueHint};
use std::cell::RefCell;
use std::fmt;
use std::io::{self, BufRead, BufWriter, Write};
use std::rc::Rc;
use url::Url;

#[derive(Debug)]
enum AppError {
    IoError(io::Error),
    UrlParseError(url::ParseError),
}

struct UrlComponents {
    scheme: String,
    username: String,
    subdomain: String,
    domain: String,
    port: String,
    path: String,
    query: String,
    fragment: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::IoError(err) => write!(f, "IO error: {}", err),
            AppError::UrlParseError(err) => write!(f, "URL parse error: {}", err),
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
    let subdomain = if !host_str.is_empty() { extract_subdomain(host_str) } else { String::new() };
    let domain = if !host_str.is_empty() { extract_domain(host_str) } else { String::new() };
    
    UrlComponents {
        scheme: url.scheme().to_string(),
        username: url.username().to_string(),
        subdomain,
        domain,
        port: url.port().map_or(String::new(), |p| p.to_string()),
        path: url.path().to_string(),
        query: url.query().unwrap_or("").to_string(),
        fragment: url.fragment().unwrap_or("").to_string(),
    }
}

fn process_urls(config: &Config, urls: &[String], res_vec: &Rc<RefCell<Vec<String>>>) -> Result<(), AppError> {
    for url_str in urls {
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
                    res_vec.borrow_mut().push(parts.join("\t"));
                }
            },
            Err(err) => {
                eprintln!("Error parsing URL '{}': {}", url_str, err);
            }
        }
    }
    Ok(())
}

fn custom_output(urls: &[String], format: &str) -> Result<(), AppError> {
    for url_str in urls {
        match parse_url(url_str) {
            Ok(url) => {
                let components = extract_url_components(&url);
                
                let output = format
                    .replace("{scheme}", &components.scheme)
                    .replace("{username}", &components.username)
                    .replace("{host}", &components.subdomain)
                    .replace("{domain}", &components.domain)
                    .replace("{port}", &components.port)
                    .replace("{path}", &components.path)
                    .replace("{query}", &components.query)
                    .replace("{fragment}", &components.fragment);
                println!("{}", output);
            },
            Err(err) => {
                eprintln!("Error parsing URL '{}': {}", url_str, err);
            }
        }
    }
    Ok(())
}

#[derive(Debug, Parser)]
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
    #[arg(long, default_value = "{scheme}://{host}{path}", help = "Custom output format")]
    format: String,
    #[arg(long, help = "Extract and display the domain")]
    domain: bool,
}

impl Config {
    fn from_args() -> Self { Self::parse() }
}

fn stdio_output(rvec: &Rc<RefCell<Vec<String>>>) -> Result<(), AppError> {
    let rv = rvec.borrow();
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    for e in rv.iter() {
        writeln!(writer, "{}", e)?;
    }
    Ok(())
}

fn json_output(rvec: &Rc<RefCell<Vec<String>>>) -> Result<(), AppError> {
    let rv = rvec.borrow();
    println!("{{");
    println!("\"urls\": [");
    for (i, url) in rv.iter().enumerate() {
        if i > 0 { println!(","); }
        println!("\"{}\"", url);
    }
    println!("]");
    println!("}}");
    Ok(())
}

fn check_for_stdin() -> Result<(), AppError> {
    if atty::is(Stream::Stdin) && Config::from_args().urls.is_empty() {
        eprintln!("Error: Not in stdin mode - switches ignored.");
        std::process::exit(1);
    }
    Ok(())
}

fn main() -> Result<(), AppError> {
    let config = Config::parse();
    check_for_stdin()?;
    
    let urls: Vec<String> = if !config.urls.is_empty() {
        config.urls.clone()
    } else {
        io::stdin()
            .lock()
            .lines()
            .map(|line| line.map_err(AppError::from))
            .collect::<Result<Vec<String>, AppError>>()?
    };
    
    let res_vec: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::with_capacity(urls.len())));
    process_urls(&config, &urls, &res_vec)?;
    if config.sort { res_vec.borrow_mut().sort(); }
    if config.unique { res_vec.borrow_mut().dedup(); }
    
    match (config.json, config.custom) {
        (true, _) => json_output(&res_vec)?,
        (_, true) => custom_output(&urls, &config.format)?,
        _ => stdio_output(&res_vec)?,
    }
    
    Ok(())
}
