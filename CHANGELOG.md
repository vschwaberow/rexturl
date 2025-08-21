# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.1] - 2025-08-21

### Security
- Replaced deprecated `atty` dependency with `std::io::IsTerminal` to address CVE security vulnerability
  - Fixes potential unaligned read issue on Windows platforms
  - Eliminates dependency on unmaintained crate (last release 3 years ago)
  - Uses Rust standard library alternative available since Rust 1.70.0

### Changed
- Terminal detection now uses `std::io::stdin().is_terminal()` instead of `atty::is(Stream::Stdin)`
- Reduced dependency tree by removing external `atty` crate

## [0.4.0] - 2025-08-20

### Changed
- **BREAKING**: Major UX overhaul with new structured formatting system
  - Single `--format` flag controls output format (plain, tsv, csv, json, jsonl, custom, sql)
  - `--fields` parameter for explicit field selection and ordering
  - Consistent behavior across all output formats
- Refactored `main.rs` into modular architecture
- Replaced standard URL parsing with custom parser implementation
- Enhanced README.md with comprehensive examples and new CLI documentation
- Updated dependencies to latest compatible versions

### Added
- **New Formatting System**:
  - Seven output formats: plain, tsv, csv, json, jsonl, custom, sql
  - `--fields` for precise field selection (e.g., `domain,path,url`)
  - `--header` flag for tabular formats (tsv, csv)
  - `--pretty` flag for pretty-printed JSON
  - `--null-empty` for custom null values in tabular formats
  - `--strict` mode with proper error codes
  - `--no-newline` for shell pipeline compatibility
- **Custom Format Templates**: Flexible template system with placeholder syntax
  - Template syntax: `{field}`, `{field:default}`, `{field?present}`, `{field!missing}`
  - `--template` flag for custom output formats
  - `--escape` parameter with modes: none, shell, csv, json, sql
- **SQL Output Format**: Generate SQL INSERT statements from URL data
  - `--format sql` for SQL output
  - `--sql-table` for custom table names
  - `--sql-create-table` flag to include CREATE TABLE statements
  - `--sql-dialect` supporting postgres, mysql, sqlite, generic
  - Proper SQL escaping and type mapping per dialect
- **Performance Optimizations**:
  - Parallel processing for bulk URL operations
  - Memory-efficient streaming support
- Modular code structure with dedicated modules:
  - `formatter.rs` - Unified formatting system with UrlRecord data model
  - `url_parser.rs` - URL component parsing and extraction logic
  - `domain.rs` - Domain and subdomain extraction with multi-part TLD support
- Comprehensive test suite including new formatting tests

### Deprecated
- `--json` flag (use `--format json`)
- `--all` flag (use `--fields` with specific field names)

### Removed
- Legacy URL parsing dependencies in favor of custom implementation

## [0.3.3] - 2025-03-23

### Added
- Enhanced error handling for URL processing and output functions
- Multi-part TLD support for complex domains (co.uk, com.au, etc.)
- JSON output support with `--json` flag
- Parallel processing with Rayon for improved performance
- Custom format support with placeholder substitution
- Comprehensive integration test suite
- Domain and subdomain extraction with `--domain` flag
- Support for extracting specific URL components

### Changed
- Refactored URL processing to use structured component extraction
- Improved README with detailed examples for domain/subdomain extraction
- Enhanced dependency management with updated crates
- Performance improvements through async processing and parallelism

### Fixed
- URL parsing edge cases with missing schemes
- Error handling for malformed URLs

## [0.3.2] - 2024

### Changed
- Various improvements and bug fixes

## [0.3.1] - 2024

### Changed
- General improvements and updates

## [0.3.0] - 2024

### Added
- GitHub Actions CI/CD pipeline with `build.yml`
- Migration from structopt to clap for CLI argument parsing

### Changed
- Updated CLI argument parsing framework
- Code cleanup and comment removal

### Fixed
- Various code improvements and fixes

## [0.2.2] - 2023

### Added
- Sorted output functionality with `--sort` flag
- Unique output functionality with `--unique` flag for deduplication

### Fixed
- Missing help output for sort and unique functions

## [0.2.1] - 2023

### Fixed
- Various code fixes and improvements

## [0.2.0] - 2023

### Added
- Enhanced URL processing capabilities
- Improved error handling

### Fixed
- Multiple bug fixes and stability improvements

## [0.1.2] - 2022

### Fixed
- Typo corrections in main.rs
- Code quality improvements

## [0.1.1] - 2022

### Added
- MIT license
- README documentation

### Fixed
- Initial bug fixes and improvements

## [0.1.0] - 2022

### Added
- Initial release of rexturl
- Basic URL parsing and component extraction
- Command-line interface for URL manipulation
- Support for extracting scheme, host, port, path, and query components
- Stdin input support for processing multiple URLs

[0.4.1]: https://github.com/vschwaberow/rexturl/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/vschwaberow/rexturl/compare/v0.3.3...v0.4.0
[0.3.3]: https://github.com/vschwaberow/rexturl/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/vschwaberow/rexturl/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/vschwaberow/rexturl/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/vschwaberow/rexturl/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/vschwaberow/rexturl/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/vschwaberow/rexturl/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/vschwaberow/rexturl/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/vschwaberow/rexturl/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/vschwaberow/rexturl/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/vschwaberow/rexturl/releases/tag/v0.1.0