# Code Style and Conventions

## General Conventions
- Standard Rust formatting with `cargo fmt`
- Use `cargo clippy` for linting
- Follow Rust naming conventions (snake_case for variables/functions, PascalCase for types)

## Project-Specific Patterns
- Single-file architecture in `src/main.rs`
- Use of clap derive macros for CLI argument parsing
- Error handling with custom `AppError` enum implementing From traits for various error types
- Extensive use of parallel processing with rayon for URL processing
- Buffer writers for performance optimization when writing output

## Key Architectural Patterns
- Configuration struct (`Config`) with clap derive attributes
- URL components extracted into dedicated struct (`UrlComponents`)
- Separate functions for different processing modes (streaming, parallel, custom formatting)
- Multi-part TLD handling with hardcoded list and extraction functions
- JSON output support with serde serialization

## Error Handling
- Custom `AppError` enum with conversions from io::Error, url::ParseError, and serde_json::Error
- Result types used throughout for error propagation
- Proper error context and Display implementation

## Testing Approach
- Integration tests using assert_cmd crate
- Test various combinations of flags and input methods
- Temporary file creation for stdin testing
- JSON output validation
- Multi-part TLD edge case testing

## Dependencies Philosophy
- Minimal external dependencies
- Focus on performance with rayon for parallelization
- Standard library preference where possible
- Well-established crates for specific functionality (clap, url, serde)