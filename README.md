# rexturl

A versatile command-line tool for parsing and manipulating URLs.

## Features

- Extract specific URL components (scheme, username, host, port, path, query, fragment)
- Extract domain and subdomain information
- Custom output formatting
- JSON output support
- Sorting and deduplication of results
- Process multiple URLs from command line or stdin
- Automatic handling of URLs without schemes (defaults to https)

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

## Usage

```bash
rexturl [OPTIONS] [URLS...]
```


If no URLs are provided, rexturl will read from stdin.

## Options

`--urls <URLS>` Input URLs to process
`--scheme` Extract and display the URL scheme
`--username` Extract and display the username from the URL
`--host` Extract and display the hostname
`--port` Extract and display the port number
`--path` Extract and display the URL path
`--query` Extract and display the query string
`--fragment` Extract and display the URL fragment
`--sort` Sort the output
`--unique` Remove duplicate entries from the output
`--json` Output results in JSON format
`--all` Display all URL components
`--custom` Enable custom output mode
`--domain` Extract the domain
`--format <FORMAT>` Custom output format (default: "{scheme}://{host}{path}")
`-h`, `--help` Print help information
`-V`, `--version` Print version information


## Examples

1. Extract all components from a single URL:
   ```bash
   rexturl --all https://user:pass@example.com:8080/path?query=value#fragment
   ```

2. Extract host and port from multiple URLs:
   ```bash
   rexturl --host --port https://example.com https://api.example.com:8443
   ```

3. Process URLs from a file, extracting paths and sorting results:
   ```bash
   cat urls.txt | rexturl --path --sort
   ```

4. Use custom output format:
   ```bash
   rexturl --custom --format "{scheme}://{host}:{port}{path}" https://example.com:8080/api
   ```

5. Output results in JSON format:
   ```bash
   rexturl --json --all https://example.com https://api.example.com
   ```

6. Sort and deduplicate results:
   ```bash
   echo -e "https://example.com\nhttps://example.com\nhttps://api.example.com" | rexturl --host --sort --unique
   ```

## Domain and Subdomain Extraction

`rexturl` includes special handling for domains and subdomains:

- The `--domain` flag extracts the domain name from the URL, with proper handling of multi-part TLDs
- When using `--host` alone (without other component flags), it extracts the subdomain by default
- Multi-part TLDs like co.uk, org.uk, com.au, etc. are automatically detected

Examples:

```bash
# Extract domain from a URL with multi-part TLD
echo "https://blog.example.co.uk/posts" | rexturl --domain
# Output: example.co.uk

# Extract subdomain from a URL
echo "https://blog.example.co.uk/posts" | rexturl --host
# Output: blog

# Extract subdomain and domain separately using custom format
echo "https://blog.example.co.uk/posts" | rexturl --custom --format "Subdomain: {subdomain}, Domain: {domain}"
# Output: Subdomain: blog, Domain: example.co.uk
```

## Custom Output Format

When using `--custom` and `--format`, you can use the following placeholders:

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

Example:

```bash
rexturl --custom --format "Host: {host}, Path: {path}" https://example.com/api
```

```bash
rexturl --custom --format "Subdomain: {subdomain}, Domain: {domain}" https://blog.example.co.uk/posts
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
