/*
Copyright 2022 Volker Schwaberow <volker@schwaberow.de>
Permission is hereby granted, free of charge, to any person obtaining a
copy of this software and associated documentation files (the
"Software"), to deal in the Software without restriction, including without
limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the
Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
Author(s): Volker Schwaberow
*/

use atty::Stream;
use clap::{Parser, ValueHint};
use std::cell::RefCell;
use std::io::{self, BufRead, BufWriter, Write};
use std::rc::Rc;
use url::Url;

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "A tool for parsing and manipulating URLs",
    long_about = "This program allows you to parse URLs and extract or manipulate various components. It supports custom output formats and JSON output."
)]
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

    #[arg(
        long,
        help = "Enable custom output mode",
        long_help = "Enable custom output mode. Use with --format to specify the output format."
    )]
    custom: bool,

    #[arg(
        long,
        default_value = "{scheme}://{host}{path}",
        help = "Custom output format",
        long_help = "Specify a custom output format. Available placeholders: \
                     {scheme}, {username}, {host}, {port}, {path}, {query}, {fragment}. \
                     Example: --format \"{scheme}://{host}:{port}{path}?{query}\""
    )]
    format: String,
}

impl Config {
    fn from_args() -> Self {
        Self::parse()
    }
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
        if i > 0 {
            println!(",");
        }
        println!("\"{}\"", url);
    }
    println!("");
    println!("]");
    println!("}}");
}

fn check_for_stdin() {
    if atty::is(Stream::Stdin) && Config::from_args().urls.is_empty() {
        println!("Error: Not in stdin mode - switches ignored.");
        println!();
        std::process::exit(0);
    }
}

fn custom_output(urls: &[String], format: &str) {
    for url_str in urls {
        if let Ok(url) = Url::parse(url_str) {
            let output = format
                .replace("{scheme}", url.scheme())
                .replace("{username}", url.username())
                .replace("{host}", url.host_str().unwrap_or(""))
                .replace(
                    "{port}",
                    &url.port().map_or("".to_string(), |p| p.to_string()),
                )
                .replace("{path}", url.path())
                .replace("{query}", url.query().unwrap_or(""))
                .replace("{fragment}", url.fragment().unwrap_or(""));
            println!("{}", output);
        } else {
            eprintln!("Error parsing URL: {}", url_str);
        }
    }
}

fn main() {
    let config = Config::parse();

    check_for_stdin();

    let urls: Vec<String> = if !config.urls.is_empty() {
        config.urls.clone()
    } else {
        io::stdin()
            .lock()
            .lines()
            .map(|line| line.expect("Failed to read line from stdin"))
            .collect()
    };

    let res_vec: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::with_capacity(urls.len())));

    // Process URLs and populate res_vec
    process_urls(&config, &urls, &res_vec);

    // Apply sorting and deduplication if needed
    if config.sort {
        res_vec.borrow_mut().sort();
    }
    if config.unique {
        res_vec.borrow_mut().dedup();
    }

    match (config.json, config.custom) {
        (true, _) => json_output(&res_vec),
        (_, true) => custom_output(&urls, &config.format),
        _ => stdio_output(&res_vec),
    }
}

fn process_urls(config: &Config, urls: &[String], res_vec: &Rc<RefCell<Vec<String>>>) {
    for url_str in urls {
        if let Ok(url) = Url::parse(url_str) {
            let mut parts = Vec::new();

            if config.all || config.scheme {
                parts.push(url.scheme().to_string());
            }
            if config.all || config.username {
                parts.push(url.username().to_string());
            }
            if config.all || config.host {
                parts.push(url.host_str().unwrap_or("").to_string());
            }
            if config.all || config.port {
                if let Some(port) = url.port() {
                    parts.push(port.to_string());
                }
            }
            if config.all || config.path {
                parts.push(url.path().to_string());
            }
            if config.all || config.query {
                if let Some(query) = url.query() {
                    parts.push(query.to_string());
                }
            }
            if config.all || config.fragment {
                if let Some(fragment) = url.fragment() {
                    parts.push(fragment.to_string());
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
