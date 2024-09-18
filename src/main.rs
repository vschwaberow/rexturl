use atty::Stream;
use clap::{Parser, ValueHint};
use std::cell::RefCell;
use std::io::{self, BufRead, BufWriter, Write};
use std::rc::Rc;
use url::Url;

fn extract_domain(host: &str) -> String {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() > 2 {
        parts[parts.len() - 2..].join(".")
    } else {
        host.to_string()
    }
}

fn extract_subdomain(host: &str) -> String {
    let domain = extract_domain(host);
    host.strip_suffix(&domain).unwrap_or(host).trim_end_matches('.').to_string()
}

fn process_urls(config: &Config, urls: &[String], res_vec: &Rc<RefCell<Vec<String>>>) {
    for url_str in urls {
        let url_with_scheme = if !url_str.contains("://") {
            format!("https://{}", url_str)
        } else {
            url_str.to_string()
        };

        if let Ok(url) = Url::parse(&url_with_scheme) {
            let mut parts = Vec::new();

            if config.all || config.scheme { parts.push(url.scheme().to_string()); }
            if config.all || config.username { parts.push(url.username().to_string()); }
            if config.all || config.host { 
                if let Some(host) = url.host_str() {
                    parts.push(extract_subdomain(host));
                }
            }
            if config.all || config.port { if let Some(port) = url.port() { parts.push(port.to_string()); } }
            if config.all || config.path { parts.push(url.path().to_string()); }
            if config.all || config.query { if let Some(query) = url.query() { parts.push(query.to_string()); } }
            if config.all || config.fragment { if let Some(fragment) = url.fragment() { parts.push(fragment.to_string()); } }
            if config.all || config.domain { 
                if let Some(host) = url.host_str() {
                    parts.push(extract_domain(host));
                }
            }

            if !parts.is_empty() {
                res_vec.borrow_mut().push(parts.join("\t"));
            }
        } else {
            eprintln!("Error parsing URL: {}", url_str);
        }
    }
}

fn custom_output(urls: &[String], format: &str) {
    for url_str in urls {
        let url_with_scheme = if !url_str.contains("://") {
            format!("https://{}", url_str)
        } else {
            url_str.to_string()
        };

        if let Ok(url) = Url::parse(&url_with_scheme) {
            let domain = url.host_str().map(extract_domain).unwrap_or_else(|| "N/A".to_string());
            let host = url.host_str().map(extract_subdomain).unwrap_or_else(|| "N/A".to_string());
            let output = format
                .replace("{scheme}", url.scheme())
                .replace("{username}", url.username())
                .replace("{host}", &host)
                .replace("{domain}", &domain)
                .replace("{port}", &url.port().map_or("".to_string(), |p| p.to_string()))
                .replace("{path}", url.path())
                .replace("{query}", url.query().unwrap_or(""))
                .replace("{fragment}", url.fragment().unwrap_or(""));
            println!("{}", output);
        } else {
            eprintln!("Error parsing URL: {}", url_str);
        }
    }
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

fn stdio_output(rvec: &Rc<RefCell<Vec<String>>>) {
    let rv = rvec.borrow();
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    for e in rv.iter() {
        writeln!(writer, "{}", e).unwrap();
    }
}

fn json_output(rvec: &Rc<RefCell<Vec<String>>>) {
    let rv = rvec.borrow();
    println!("{{");
    println!("\"urls\": [");
    for (i, url) in rv.iter().enumerate() {
        if i > 0 { println!(","); }
        println!("\"{}\"", url);
    }
    println!("]");
    println!("}}");
}

fn check_for_stdin() {
    if atty::is(Stream::Stdin) && Config::from_args().urls.is_empty() {
        eprintln!("Error: Not in stdin mode - switches ignored.");
        std::process::exit(1);
    }
}

fn main() {
    let config = Config::parse();
    check_for_stdin();
    let urls: Vec<String> = if !config.urls.is_empty() {
        config.urls.clone()
    } else {
        io::stdin().lock().lines().map(|line| line.expect("Failed to read line from stdin")).collect()
    };
    let res_vec: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::with_capacity(urls.len())));
    process_urls(&config, &urls, &res_vec);
    if config.sort { res_vec.borrow_mut().sort(); }
    if config.unique { res_vec.borrow_mut().dedup(); }
    match (config.json, config.custom) {
        (true, _) => json_output(&res_vec),
        (_, true) => custom_output(&urls, &config.format),
        _ => stdio_output(&res_vec),
    }
}
