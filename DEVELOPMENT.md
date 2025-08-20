# DEVELOPMENT.md

This file provides development guidance and conventions for working with this repository.

## Project Overview

rexturl is a command-line tool for parsing and manipulating URLs, written in Rust. It extracts URL components, handles domain/subdomain extraction with multi-part TLD support, and provides flexible output formatting including JSON.

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
cargo run -- [OPTIONS] [URLS...]

# Examples
cargo run -- --host --port https://example.com:8080
cargo run -- --json --all https://example.com
echo "https://example.com" | cargo run -- --domain
```

## Architecture

### Single-File Design
The entire application logic is contained in `src/main.rs` with integration tests in `tests/integration_tests.rs`. This architectural choice keeps the codebase focused and simple.

### Key Components
- **Config struct**: Uses clap derive macros for CLI argument parsing
- **URL Processing Pipeline**: Input → Parsing → Component Extraction → Formatting → Output
- **Multi-part TLD Support**: Hardcoded list in `MULTI_PART_TLDS` constant handles complex domains like .co.uk, .com.au
- **Parallel Processing**: Uses rayon for concurrent URL processing when handling multiple URLs
- **Custom Formatting**: Template-based output with placeholder substitution

### Processing Modes
- **Parallel Mode**: For multiple URLs from command line
- **Streaming Mode**: Line-by-line processing for stdin input  
- **Custom Format Mode**: Template-based output with placeholders like `{scheme}://{host}{path}`
- **JSON Mode**: Structured output with serde serialization

## Code Patterns

### Error Handling
- Custom `AppError` enum with `From` trait implementations for io::Error, url::ParseError, and serde_json::Error
- Result types used throughout for error propagation

### Performance Optimizations
- `BufWriter` for efficient output buffering
- Parallel processing with rayon for multiple URLs
- Release profile configured for size optimization (`opt-level = "s"`, LTO enabled)

### Domain/Subdomain Logic
The domain extraction logic handles complex multi-part TLDs:
- `extract_domain()` function identifies the registrable domain
- `extract_subdomain()` isolates subdomain portions  
- Special handling for TLDs like co.uk, org.uk, com.au, etc.

## Testing Strategy

Integration tests use `assert_cmd` crate to test the CLI interface:
- Various flag combinations and input methods
- JSON output validation
- Multi-part TLD edge cases
- Stdin processing with temporary files

When adding features, ensure integration tests cover new functionality and edge cases.

## Task Completion Checklist

Before completing any development task:
1. Run `cargo clippy --all-targets --all-features` and address any lints
2. Run `cargo fmt` to format code
3. Run `cargo test` to ensure all tests pass
4. Run `cargo build --release` to verify release build
5. Test manually with key use cases, especially domain/subdomain extraction with complex TLDs