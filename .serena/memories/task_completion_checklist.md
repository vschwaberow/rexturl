# Task Completion Checklist

## Standard Development Workflow
When completing any development task, ensure you run:

1. **Code Quality Checks**
   ```bash
   cargo check
   cargo clippy --all-targets --all-features
   cargo fmt
   ```

2. **Testing**
   ```bash
   cargo test
   ```

3. **Build Verification**
   ```bash
   cargo build --release
   ```

## Pre-commit Checklist
- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Code is formatted with `cargo fmt`
- [ ] Clippy lints are addressed
- [ ] Integration tests cover new functionality
- [ ] Release build succeeds

## Release Preparation
- [ ] Version updated in Cargo.toml
- [ ] CHANGELOG updated (if present)
- [ ] All tests pass in release mode
- [ ] Binary tested manually with key use cases
- [ ] Documentation updated if API changes

## Key Areas to Test
- URL parsing with various formats
- Multi-part TLD handling
- JSON output formatting
- Custom format string processing
- Stdin input processing
- Parallel processing with large input sets
- Error handling for malformed URLs