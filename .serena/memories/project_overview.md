# Project Overview

## Purpose
rexturl is a versatile command-line tool for parsing and manipulating URLs written in Rust. It allows users to extract specific URL components, format output, and process multiple URLs from command line or stdin.

## Tech Stack
- **Language**: Rust (edition 2021)
- **CLI Framework**: clap v4.5.4 with derive feature
- **URL Parsing**: url crate v2.3.1
- **Parallel Processing**: rayon v1.10.0
- **JSON Serialization**: serde + serde_json
- **Terminal Detection**: atty v0.2.14

## Key Features
- Extract URL components (scheme, username, host, port, path, query, fragment)
- Domain and subdomain extraction with multi-part TLD support
- Custom output formatting with placeholders
- JSON output support
- Sorting and deduplication
- Parallel processing for multiple URLs
- Stdin input support
- Multi-part TLD detection (co.uk, org.uk, com.au, etc.)

## Project Structure
- `src/main.rs` - Single source file containing all logic
- `tests/integration_tests.rs` - Integration tests using assert_cmd
- `Cargo.toml` - Project configuration with release optimizations
- Simple structure with no modules or subdirectories