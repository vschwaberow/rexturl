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
use std::env;
use url::Url;
use std::cell::RefCell;
use std::io::{self, BufRead};
use std::rc::Rc;

#[derive(Debug)]
struct Config {
    scheme: bool,
    username: bool,
    host: bool,
    port: bool,
    path: bool,
    query: bool,
    fragment: bool,
    all: bool,
}

fn check_for_stdin() {
    if atty::is(Stream::Stdin) {
        println!("Error: Not in stdin mode - switches ignored.");
        println!();
        print_help();
        std::process::exit(0);
    }
}

fn print_help() {
    print_prg_info();
    println!("Usage: rexturl [options] [url]");
    println!("Options:");
    println!("  -s, --scheme     print the scheme");
    println!("  -u, --username   print the username");
    println!("  -H, --host       print the host");
    println!("  -p, --port       print the port");
    println!("  -P, --path       print the path");
    println!("  -q, --query      print the query");
    println!("  -f, --fragment   print the fragment");
    println!("  -a, --all        print all parts");
    println!("  -h, --help       print this help");
    std::process::exit(0);
}

fn print_prg_info() {
    let prg_info = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let prg_authors = env!("CARGO_PKG_AUTHORS").to_string();
    let prg_description = env!("CARGO_PKG_DESCRIPTION").to_string();
    println!("{} {}", prg_info, prg_authors);
    println!("{}", prg_description);
    println!();
}

fn main() {

    let config = Config {
        scheme: false,
        username: false,
        host: false,
        port: false,
        path: false,
        query: false,
        fragment: false,
        all: false,
    };

    let config_sptr = Rc::new(RefCell::new(config));
    
    check_for_stdin();
    
    let stdin = io::stdin();
    let args: Vec<String> = env::args().collect();
    for arg in args.iter() {
        let mut c = RefCell::borrow_mut(&config_sptr);
        match arg.as_str() {
            "-s" | "--scheme"=> { c.scheme = true;},
            "-u" | "--username" => { c.username = true; },
            "-H" | "--host" => { c.host = true; },
            "-p" | "--port" => { c.port = true; },
            "-P" | "--path" => { c.path = true; },
            "-q" | "--query" => { c.query = true; },
            "-f" | "--fragment" => { c.fragment = true; },
            "-a" | "--all" => { c.all = true; },
            "-h" | "--help" => { print_help(); },
            _ => (),
        }
    }

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let url = Url::parse(&line);
        if url.is_err() {
            println!("Error: {}", url.err().unwrap());
            std::process::exit(1);
        }
        let url = url.unwrap();

        let c = config_sptr.borrow();

        if c.scheme {
            let scheme = url.scheme();
            println!("{}", scheme);
            continue; 
        } else if c.username {
            let username = url.username();
            match username {
                "" => (),
                _ => println!("{}", username),
            }
        } else if c.host {
            let host = url.host();
            match host {
                Some(host) => println!("{}", host),
                None => println!("No hostname"),
            }
        } else if c.port {
            let port = url.port();
            match port {
                Some(port) => println!("{}", port),
                None => println!("No port"),
            }
        } else if c.path {
            let path = url.path();
            println!("{}", path);
        } else if c.query { 
            let query = url.query();
            match query {
                Some(query) => println!("{}", query),
                None => println!("No query"),
            }
        } else if c.fragment {
            let frag = url.fragment();
            match frag {
                Some(frag) => println!("{}", frag),
                None => println!("No fragment"),
            }
        } else if c.all {
            println!("{}", url);
        } else {
            println!("Error: No option selected");
            std::process::exit(1);
            
        }
    }

}
