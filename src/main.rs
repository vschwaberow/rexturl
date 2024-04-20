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
use std::io::{self, BufRead};
use std::rc::Rc;
use url::Url;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Config {
    #[arg(long, value_hint = ValueHint::AnyPath)]
    urls: Vec<String>,

    #[arg(long)]
    scheme: bool,
    #[arg(long)]
    username: bool,
    #[arg(long)]
    host: bool,
    #[arg(long)]
    port: bool,
    #[arg(long)]
    path: bool,
    #[arg(long)]
    query: bool,
    #[arg(long)]
    fragment: bool,
    #[arg(long)]
    sort: bool,
    #[arg(long)]
    unique: bool,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    all: bool,
}

impl Config {
    fn from_args() -> Self {
        Self::parse()
    }
}

fn stdio_output(rvec: &Rc<RefCell<Vec<String>>>) {
    let rv = rvec.borrow();
    for e in rv.iter() {
        println!("{}", e);
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

fn main() {
    let config = Config::parse();

    check_for_stdin();

    let res_vec: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));

    let urls = if !config.urls.is_empty() {
        config.urls.clone()
    } else {
        let mut lines = Vec::new();
        for line in io::stdin().lock().lines() {
            lines.push(line.unwrap());
        }
        lines
    };

    for url_str in urls {
        let url = Url::parse(&url_str);
        if url.is_err() {
            println!("Error: {}", url.err().unwrap());
            std::process::exit(1);
        }
        let url = url.unwrap();

        let mut e = RefCell::borrow_mut(&res_vec);

        if config.scheme {
            e.push(url.scheme().to_string());
        } else if config.username {
            let username = url.username();
            if !username.is_empty() {
                e.push(username.to_string());
            }
        } else if config.host {
            if let Some(host) = url.host() {
                e.push(host.to_string());
            }
        } else if config.port {
            if let Some(port) = url.port() {
                e.push(port.to_string());
            }
        } else if config.path {
            e.push(url.path().to_string());
        } else if config.query {
            if let Some(query) = url.query() {
                e.push(query.to_string());
            }
        } else if config.fragment {
            if let Some(frag) = url.fragment() {
                e.push(frag.to_string());
            }
        } else if config.all {
            e.push(url.to_string());
        } else {
            println!("Error: No option selected");
            std::process::exit(1);
        }
    }

    if config.sort {
        res_vec.borrow_mut().sort();
    }

    if config.unique {
        res_vec.borrow_mut().dedup();
    }

    let rv = res_vec;
    if config.json {
        json_output(&rv);
    } else {
        stdio_output(&rv);
    }
}
