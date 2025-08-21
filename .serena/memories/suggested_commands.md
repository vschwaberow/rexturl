# Suggested Commands

## Build Commands
```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Install locally
cargo install --path .
```

## Testing Commands
```bash
# Run all tests
cargo test

# Run integration tests specifically
cargo test --test integration_tests

# Run with verbose output
cargo test -- --nocapture
```

## Development Commands
```bash
# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy for linting
cargo clippy

# Run clippy with all targets
cargo clippy --all-targets --all-features

# Clean build artifacts
cargo clean
```

## Running the Tool
```bash
# Run development version
cargo run -- [OPTIONS] [URLS...]

# Run installed version
rexturl [OPTIONS] [URLS...]

# Examples
cargo run -- --host --port https://example.com:8080
cargo run -- --json --all https://example.com
echo "https://example.com" | cargo run -- --domain
```

## Release Commands
```bash
# Build optimized release
cargo build --release

# The binary will be in target/release/rexturl
./target/release/rexturl --help
```