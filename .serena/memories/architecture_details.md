# Architecture Details

## Core Architecture
rexturl follows a single-file architecture pattern with all logic contained in `src/main.rs`. This design choice keeps the codebase simple and focused.

## Key Components

### Configuration System
- `Config` struct using clap derive macros for CLI parsing
- Comprehensive flag system for URL component extraction
- Custom format string support with placeholder substitution
- JSON output mode support

### URL Processing Pipeline
1. **Input Handling**: URLs from command line args or stdin
2. **Parsing**: url crate for RFC-compliant URL parsing
3. **Component Extraction**: Custom functions for each URL part
4. **Processing**: Parallel or streaming based on input source
5. **Output**: Formatted text or JSON with optional sorting/deduplication

### Domain/Subdomain Logic
- Hardcoded list of multi-part TLDs (MULTI_PART_TLDS constant)
- `extract_domain()` function with smart TLD detection
- `extract_subdomain()` function for subdomain isolation
- Special handling for complex TLD patterns (co.uk, com.au, etc.)

### Processing Modes
- **Parallel Mode**: Uses rayon for concurrent URL processing
- **Streaming Mode**: Line-by-line processing for stdin input
- **Custom Format Mode**: Template-based output formatting
- **JSON Mode**: Structured output with serde serialization

### Error Handling Strategy
- Custom `AppError` enum unifying different error types
- From trait implementations for seamless error conversion
- Result propagation throughout the codebase
- Graceful handling of malformed URLs

### Performance Optimizations
- Parallel processing with rayon for multiple URLs
- BufWriter for efficient output buffering
- Release profile optimizations (LTO, size optimization)
- Minimal allocations in hot paths

## Data Flow
1. CLI args parsed into Config struct
2. Input validation and stdin detection
3. URL processing based on selected mode
4. Component extraction and formatting
5. Optional sorting and deduplication
6. Output via buffered writers or JSON serialization