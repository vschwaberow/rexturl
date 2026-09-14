# DEVELOPMENT.md

This file provides development guidance and conventions for working with this repository.

## Project Overview

rexturl is a command-line tool for parsing and manipulating URLs, written in Rust. It extracts URL components, handles domain/subdomain extraction with multi-part TLD support, and provides flexible output formats (plain, tsv, csv, json, jsonl, custom, sql).

## Development Commands

### Building and Testing
```bash
# Development build
cargo build

# Release build (optimized for size and performance)
cargo build --release

# Run all tests (includes integration tests)
cargo test

# Run integration tests specifically
cargo test --test integration_tests

# Code quality checks
cargo check
cargo clippy --all-targets --all-features
cargo fmt
```

### Running the Tool
```bash
# Run development version
cargo run -- [OPTIONS]

# Examples
cargo run -- --urls "https://example.com:8080" --fields subdomain,port
cargo run -- --urls "https://example.com" --format json --fields domain
echo "https://example.com" | cargo run -- --fields domain
echo "example.com" | cargo run -- --fields scheme,domain
```

## Architecture

### Modular Design
Library modules live under `src/`; `src/main.rs` is a thin CLI orchestrator.

| Module | Role |
|--------|------|
| `url` | Custom URL parser (`Url`, `UrlParseError`) |
| `parser` | Component extraction (`UrlComponents`, schemeless → `https://`) |
| `domain` | Registrable domain / subdomain with `MULTI_PART_TLDS` |
| `formatter` | `UrlRecord`, templates, plain/tsv/csv/json/jsonl/custom/sql |
| `config` | clap `Config` and stdin detection |
| `error` | `AppError` |

### URL Processing Pipeline
Input (`--urls` or stdin) → parallel `to_record` (rayon) → optional sort/unique → format → stdout.

### Key Behaviors
- **Multi-part TLD Support**: `MULTI_PART_TLDS` in `domain.rs` (e.g. `.co.uk`, `.com.au`)
- **Schemeless input**: `example.com` is parsed as `https://example.com`
- **Parallel parsing**: rayon over the `to_record` step for bulk input
- **Legacy CLI flags**: `--json`, `--all`, `--custom` remain but are deprecated; prefer `--format` / `--fields`

## Code Patterns

### Error Handling
- `AppError` with `From` for `io::Error`, `UrlParseError`, and `serde_json::Error`
- Results used for fallible paths; `--strict` exits with code 2 on parse failures

### Performance Optimizations
- Parallel `to_record` with rayon for multiple URLs
- Release profile: `opt-level = "s"`, LTO, symbol stripping

### Domain/Subdomain Logic
- `extract_domain()` identifies the registrable domain
- `extract_subdomain()` isolates subdomain portions
- Special handling for TLDs like `co.uk`, `org.uk`, `com.au`

## Testing Strategy

Integration tests use `assert_cmd` against the CLI:
- Format and field combinations
- JSON / custom template output
- Multi-part TLD edge cases
- Stdin processing

Unit tests cover `parser`, `formatter`, `domain`, and the custom `url` parser.

When adding features, cover new CLI paths in integration tests.

## Task Completion Checklist

Before completing any development task:
1. Run `cargo clippy --all-targets --all-features` and address any lints
2. Run `cargo fmt` to format code
3. Run `cargo test` to ensure all tests pass
4. Run `cargo build --release` to verify release build
5. Test manually with key use cases, especially domain/subdomain extraction with complex TLDs and schemeless input
