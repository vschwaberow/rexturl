# rexturl

A simple tool to split urls in their protocol, host, port, path and query parts.

## Install

````
cargo install rexturl
````
or clone the source code and run `cargo build --release` to build the binary.

## Usage

````
cat [FILE WITH URLS] | rexturl [OPTIONS]
````

````
rexturl --help

rexturl (c) 2022 by Volker Schwaberow <volker@schwaberow.de>
A simple tool to split urls in their protocol, host, port, path and query parts.

Usage: rexturl [options] [url]
Options:
  -s, --scheme     print the scheme
  -u, --username   print the username
  -H, --host       print the host
  -p, --port       print the port
  -P, --path       print the path
  -q, --query      print the query
  -f, --fragment   print the fragment
  -a, --all        print all parts
  -h, --help       print this help
````

## Contribution 

If you want to contribute to this project, please feel free to do so. I am happy to accept pull requests. Any help is appreciated. If you have any questions, please feel free to contact me.
