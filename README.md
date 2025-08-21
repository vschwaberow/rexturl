# rexturl

[![Version](https://img.shields.io/badge/version-0.4.1-blue.svg)](https://github.com/vschwaberow/rexturl/releases/tag/v0.4.1)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)

A command-line tool for parsing and manipulating URLs with predictable output formats.

## Key Features

### Clean UX Design
- One flag controls format: `--format {plain,tsv,csv,json,jsonl,custom,sql}`
- Precise field selection: `--fields domain,path,url` 
- Custom templates: `--template '{scheme}://{domain}{path}'`
- SQL generation: Multi-dialect INSERT statements with proper escaping
- Consistent output: Same field order across all formats
- Machine-friendly: Proper headers, null handling, exit codes

### Technical Implementation
- Custom URL parser with optimized component extraction
- Zero-copy parsing with minimal allocations
- Parallel processing for bulk operations
- Multi-part TLD support (co.uk, com.au, etc.)
- Template engine with conditional logic and escaping modes
- SQL generation with dialect-specific type mapping

### Processing Features
- Field extraction: scheme, username, host, domain, subdomain, port, path, query, fragment
- Data processing: Sort, deduplicate, filter
- Input flexibility: Command line args or stdin

## Installation

```bash
cargo install rexturl
```
or clone the repository and build from source:

```bash
git clone https://github.com/vschwaberow/rexturl.git
cd rexturl
cargo build --release
```

## Quick Start

**Extract domain from URL:**
```bash
rexturl --urls "https://www.example.com/path" --fields domain
# Output: example.com
```

**TSV format with headers:**
```bash
echo "https://blog.example.co.uk/posts" | rexturl --fields subdomain,domain,path --format tsv --header
# Output:
# subdomain    domain          path
# blog         example.co.uk   /posts
```

**JSON output for APIs:**
```bash
curl -s api.com/urls | rexturl --fields domain --format json
# Output: {"urls":[{"domain":"api.com"}]}
```

## Usage

```bash
rexturl [OPTIONS]
```

### Input Methods
- `--urls <URLS>` - Specify URLs as command-line arguments
- **stdin** - Pipe URLs from other commands (default if no --urls)
- Supports single or multiple URLs

## Options

### Core Options

| Option | Values | Description |
|--------|--------|-------------|
| `--format` | `plain`, `tsv`, `csv`, `json`, `jsonl`, `custom`, `sql` | Output format (default: `plain`) |
| `--fields` | `domain,path,url` | Comma-separated fields to extract |
| `--urls` | URL strings | Input URLs to process |
| `--header` | - | Include header row for tabular formats |
| `--sort` | - | Sort output by first field |
| `--unique` | - | Remove duplicate entries |

### Available Fields

| Field | Description | Example |
|-------|-------------|---------|
| `url` | Original URL string | `https://www.example.com/path` |
| `scheme` | Protocol | `https` |
| `username` | Username portion | `user` |
| `host`/`hostname` | Full hostname | `www.example.com` |
| `subdomain` | Subdomain only | `www` |
| `domain` | Registrable domain | `example.com` |
| `port` | Port number | `8080` |
| `path` | URL path | `/path` |
| `query` | Query parameters | `q=search` |
| `fragment` | Fragment identifier | `section` |

### Advanced Options

| Option | Values | Description |
|--------|--------|-------------|
| `--pretty` | - | Pretty-print JSON output |
| `--strict` | - | Exit code 2 if any URL fails to parse |
| `--no-newline` | - | Suppress trailing newline |
| `--null-empty` | Custom string | Value for missing fields (default: `\N`) |
| `--color` | `auto`, `never`, `always` | Colored output for plain format |

### Custom Format Options

| Option | Values | Description |
|--------|--------|-------------|
| `--template` | Template string | Custom format template (e.g., `'{scheme}://{domain}{path}'`) |
| `--escape` | `none`, `shell`, `csv`, `json`, `sql` | Escaping mode for custom format |

### SQL Output Options

| Option | Values | Description |
|--------|--------|-------------|
| `--sql-table` | Table name | SQL table name (default: `urls`) |
| `--sql-create-table` | - | Include CREATE TABLE statement |
| `--sql-dialect` | `postgres`, `mysql`, `sqlite`, `generic` | SQL dialect for type mapping |

### Legacy Field Flags (Still Supported)

These flags automatically add fields - use `--fields` for explicit control:

| Flag | Equivalent | Description |
|------|------------|-------------|
| `--domain` | `--fields domain` | Extract domain |
| `--host` | `--fields subdomain` | Extract subdomain |
| `--scheme` | `--fields scheme` | Extract scheme |
| `--path` | `--fields path` | Extract path |

### Deprecated Options

| Option | Use Instead | Description |
|--------|-------------|-------------|
| `--json` | `--format json` | JSON output (deprecated) |
| `--all` | `--fields` with specific names | All fields (deprecated) |
| `--custom` | `--format` and `--fields` | Custom format (deprecated) |


## Examples

### Most Common Use Cases

**1. Extract domains for analysis:**
```bash
cat urls.txt | rexturl --fields domain --sort --unique
# Clean list of unique domains
```

**2. Create a spreadsheet-ready CSV:**
```bash
rexturl --urls "https://api.example.com/v1/users" --fields subdomain,domain,path --format csv --header
# subdomain,domain,path
# api,example.com,/v1/users
```

**3. JSON for APIs and scripts:**
```bash
curl -s api.com/endpoints | rexturl --fields domain,path --format json
# {"urls":[{"domain":"api.com","path":"/endpoints"}]}
```

### All Format Examples

**Plain (default):**
```bash
rexturl --urls "https://blog.example.com/posts" --fields subdomain,domain,path
# blog example.com /posts
```

**TSV with header:**
```bash
echo "https://api.example.com/v1" | rexturl --fields subdomain,domain,path --format tsv --header
# subdomain    domain        path
# api          example.com   /v1
```

**CSV for spreadsheets:**
```bash
rexturl --fields url,domain --format csv --header < urls.txt
# url,domain
# https://www.example.com,example.com
```

**JSON for APIs:**
```bash
echo "https://api.example.com" | rexturl --fields domain,path --format json --pretty
# {
#   "urls": [
#     {
#       "domain": "example.com", 
#       "path": "/"
#     }
#   ]
# }
```

**JSONL for streaming:**
```bash
cat large-urls.txt | rexturl --fields domain --format jsonl | head -3
# {"domain":"example.com"}
# {"domain":"api.com"}  
# {"domain":"blog.net"}
```

**Custom format with templates:**
```bash
rexturl --urls "https://api.example.com/v1/users" --format custom --template "{scheme}://{domain}{path}"
# https://example.com/v1/users
```

**SQL INSERT statements:**
```bash
rexturl --urls "https://www.example.com/path" --format sql --fields domain,path
# INSERT INTO urls (domain, path) VALUES ('example.com', '/path');
```

### Advanced Examples

**Multi-part TLD handling:**
```bash
rexturl --urls "https://blog.example.co.uk/posts" --fields subdomain,domain,path --format tsv
# blog    example.co.uk    /posts
```

**Handle missing values:**
```bash
echo "https://example.com" | rexturl --fields domain,port --format tsv --null-empty "N/A"
# example.com    N/A
```

**Error handling with strict mode:**
```bash
rexturl --urls "not-a-url" --strict --fields domain
# Error: Failed to parse URL: not-a-url
# Exit code: 2
```

**Legacy syntax (still works):**
```bash
rexturl --urls "https://www.example.com" --domain --path
# example.com /
```

## Domain and Subdomain Extraction

`rexturl` includes intelligent handling for domains and subdomains:

- **Multi-part TLD Support**: Automatically detects complex TLDs like `co.uk`, `org.uk`, `com.au`, etc.
- **Domain Extraction**: The `--domain` flag extracts the registrable domain name
- **Subdomain Extraction**: When using `--host` alone, it extracts the subdomain portion
- **Smart Detection**: Handles edge cases with nested subdomains and international domains

Supported multi-part TLDs include:
`co.uk`, `org.uk`, `ac.uk`, `gov.uk`, `me.uk`, `net.uk`, `sch.uk`, `com.au`, `net.au`, `org.au`, `edu.au`, `gov.au`, `co.nz`, `net.nz`, `org.nz`, `govt.nz`, `co.za`, `org.za`, `com.br`, `net.br`, `org.br`, `co.jp`, `com.mx`, `com.ar`, `com.sg`, `com.my`, `co.id`, `com.hk`, `co.th`, `in.th`

Examples:

```bash
# Using custom format for specific extraction
echo "https://blog.example.co.uk/posts" | rexturl --format custom --template "Subdomain: {subdomain}, Domain: {domain}"
# Output: Subdomain: blog, Domain: example.co.uk

# Extract all components (tab-separated format)
rexturl --urls "https://user@blog.example.co.uk:8080/posts?q=test#frag" --fields scheme,username,hostname,port,path,query,fragment,domain --format tsv
# Output: https	user	blog.example.co.uk	8080	/posts	q=test	frag	example.co.uk

# Extract components with URLs flag
rexturl --urls "https://blog.example.co.uk/posts" --fields domain
# Output: example.co.uk
```

## Custom Templates

### Template Syntax

Use `--format custom --template` for flexible output formatting:

**Basic fields:**
- `{field}` - Insert field value or empty string if missing
- `{field:default}` - Insert field value or default if missing
- `{field?text}` - Insert text only if field has a value
- `{field!text}` - Insert text only if field is missing

**Available fields:**
- `{scheme}` - URL scheme (http, https, etc.)
- `{username}` - Username portion of the URL 
- `{host}` - Full hostname
- `{hostname}` - Alias for host
- `{subdomain}` - Subdomain portion (e.g., "www" in www.example.com)
- `{domain}` - Domain name (e.g., "example.com")
- `{port}` - Port number
- `{path}` - URL path
- `{query}` - Query string (without the leading ?)
- `{fragment}` - Fragment identifier (without the leading #)

**Escaping modes:**
- `--escape none` - No escaping (default)
- `--escape shell` - Shell-safe quoting
- `--escape csv` - CSV-compatible escaping
- `--escape json` - JSON string escaping
- `--escape sql` - SQL value escaping

### Template Examples

```bash
# Basic template
rexturl --urls "https://example.com/api" --format custom --template "Host: {host}, Path: {path}"
# Output: Host: example.com, Path: /api

# With defaults
rexturl --urls "https://example.com" --format custom --template "{scheme}://{domain}:{port:80}"
# Output: https://example.com:80

# Conditional text
rexturl --urls "https://example.com/path?q=test" --format custom --template "{domain}{query?&found}"
# Output: example.com&found

# Shell escaping
rexturl --urls "https://example.com/path with spaces" --format custom --template "{url}" --escape shell
# Output: 'https://example.com/path with spaces'
```

## SQL Output

Generate SQL INSERT statements from URL data:

```bash
# Basic SQL output
rexturl --urls "https://www.example.com/path" --format sql --fields domain,path
# INSERT INTO urls (domain, path) VALUES ('example.com', '/path');

# With CREATE TABLE
rexturl --urls "https://example.com" --format sql --fields domain --sql-create-table
# CREATE TABLE IF NOT EXISTS urls (
#     id SERIAL PRIMARY KEY,
#     domain VARCHAR(253),
#     created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
# );
# INSERT INTO urls (domain) VALUES ('example.com');

# Custom table and dialect
rexturl --urls "https://example.com:3306" --format sql --fields domain,port --sql-table my_urls --sql-dialect mysql
# INSERT INTO my_urls (domain, port) VALUES ('example.com', '3306');
```

## Performance & Architecture

### URL Parser Implementation
- Custom URL parser with optimized component extraction
- Zero-copy parsing with minimal memory allocations
- Parallel processing using Rayon for bulk operations

### Architecture
- Unified data model: Single `UrlRecord` struct for all formats
- Template engine: Flexible custom formatting with conditional logic
- SQL generation: Multi-dialect support with proper type mapping
- Predictable output: Same field order across all formats
- Proper error handling: Exit codes and stderr for failures
- Streaming support: Memory-efficient for large datasets

### Benchmarks
```bash
cargo bench
# fast_url_parsing        time:   [823.79 ns 827.53 ns 831.87 ns]
# fast_url_component_access time: [69.100 ns 69.309 ns 69.527 ns]
```

### Technical Details
- Modular design: Separate parsing, formatting, and domain intelligence
- Multi-part TLD support: Handles complex domains like `example.co.uk`
- Memory efficient: <1KB overhead per URL

## Changelog

For a detailed list of changes and version history, see [CHANGELOG.md](CHANGELOG.md).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with proper tests
4. Ensure all tests pass (`cargo test`)
5. Run formatting and linting (`cargo fmt && cargo clippy`)
6. Commit your changes (`git commit -m 'Add some amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
