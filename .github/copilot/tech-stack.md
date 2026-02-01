# YumChat Technology Stack

## Overview
YumChat is a terminal-based chat application for AI models, built with Rust and the ratatui TUI framework.

## Language & Runtime
- **Language**: Rust 2021 Edition
- **Toolchain**: Stable (no nightly features)
- **Async Runtime**: Tokio 1.42 (full features)

## Core Dependencies

### UI & Terminal
- **ratatui** `0.29.0` - Terminal UI framework
  - Provides widgets, layouts, and rendering
  - Text styling and color support
  - Widget composition and event handling
  
- **crossterm** `0.28.0` - Cross-platform terminal manipulation
  - Required by ratatui as backend
  - Keyboard event capture
  - Terminal control (raw mode, cursor, etc.)

### Networking & API
- **reqwest** `0.12.x` - HTTP client
  - Features: `json`, `stream`
  - Async HTTP requests
  - Streaming response support for Ollama API
  
- **futures** `0.3.x` - Async utilities
  - Stream combinators
  - Future utilities

### Serialization
- **serde** `1.0.x` - Serialization framework
  - Feature: `derive`
  - Used for all data serialization
  
- **serde_json** `1.0.x` - JSON support
  - API request/response parsing
  - Streaming JSON parsing
  
- **toml** `0.8.x` - TOML parser
  - Configuration file format

### Utilities
- **uuid** `1.11.x` - UUID generation
  - Features: `v4`, `serde`
  - Used for conversation IDs
  
- **chrono** `0.4.x` - Date and time
  - Feature: `serde`
  - Conversation timestamps
  
- **dirs** `5.0.x` - Standard directories
  - Cross-platform config directory resolution
  - Home directory access

### Error Handling
- **anyhow** `1.0.x` - Error handling
  - Used in application code
  - Provides context to errors
  
- **thiserror** `1.0.x` - Error derive macros
  - Used for library error types (if needed)

## Development Dependencies

### Testing
- **mockall** `0.13.x` - Mocking framework
  - Mock traits and structs
  - Used for unit testing API clients
  
- **tempfile** `3.13.x` - Temporary files/directories
  - Test isolation
  - File system testing
  
- **wiremock** `0.6.x` - HTTP mocking
  - Mock HTTP servers for testing
  - API client integration tests
  
- **tokio-test** `0.4.x` - Async test utilities
  - Test async functions
  - Time manipulation in tests

## Build Configuration

### Release Profile
```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit for better optimization
```

### Clippy Lints
```toml
[lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
```

## Architecture Layers

### 1. UI Layer (`src/ui/`)
- **Framework**: ratatui 0.29
- **Backend**: crossterm 0.28
- **Components**:
  - Status bar (model, tokens, loading)
  - Chat history window (scrollable)
  - Input field (expandable)
  - Help overlay
- **Markdown Rendering**: Custom parser (no external deps)

### 2. Application State (`src/app.rs`)
- **Pattern**: Centralized state management
- **State**:
  - Message history
  - Scroll offset
  - Focus management (Input/History)
  - Loading state
  - Context window tracking

### 3. API Client (`src/api/`)
- **HTTP Client**: reqwest with async
- **Target**: Ollama API (localhost:11434)
- **Streaming**: Parses newline-delimited JSON
- **Error Handling**: anyhow with context

### 4. Storage (`src/storage/`)
- **File Format**: Markdown for conversations, JSON for metadata
- **Location**: `~/.config/yumchat/`
- **Cross-platform**: Uses `dirs` crate

### 5. Event System (`src/events.rs`)
- **Pattern**: Event-driven architecture
- **Channel**: tokio unbounded mpsc
- **Events**: AI chunks, completion, errors

## External APIs

### Ollama API
- **Base URL**: `http://localhost:11434`
- **Endpoints Used**:
  - `POST /api/generate` - Generate responses (streaming)
  - `GET /api/tags` - List models
  - `POST /api/show` - Model details
- **Response Format**: Newline-delimited JSON
- **Fields Used**:
  - `response` - Main response text
  - `thinking` - Chain-of-thought (qwen3:4b)
  - `done` - Completion flag
  - `context` - Context array (for continuation)

## Platform Support

### Tested Platforms
- macOS (primary development)
- Linux (supported)
- Windows (supported via crossterm)

### Terminal Requirements
- 256-color support recommended
- Unicode support for box-drawing characters
- Minimum size: 80x24 (typical terminal default)

## Development Tools

### Required
- Rust stable toolchain
- cargo (comes with Rust)

### Recommended
- `cargo-watch` - Auto-rebuild on changes
- `cargo-nextest` - Faster test runner (optional)

### Build Commands
```bash
# Development
cargo run

# Release
cargo build --release
./target/release/yumchat

# Testing
cargo test
cargo test -- --nocapture  # Show output
cargo test --ignored       # Run integration tests

# Linting
cargo clippy --all-targets -- -D warnings
cargo fmt --check

# Clean
cargo clean
```

## Version Compatibility Notes

### Ratatui 0.29
- Compatible with crossterm 0.28
- **Important**: termimad 0.29+ requires crossterm 0.29+ (incompatible)
- Solution: Custom markdown parser (no termimad dependency)

### Tokio 1.42
- Full features enabled
- Hybrid sync/async: UI loop is sync, API calls are async
- Unbounded channels for event passing

### Reqwest 0.12
- Streaming support via `bytes_stream()`
- JSON feature for request/response parsing
- Timeout: 300 seconds for long-running requests

## Future Considerations

### Planned Dependencies (Phase 7+)
- Conversation persistence (using existing deps)
- Conversation list UI (ratatui widgets)
- Settings UI (ratatui widgets)

### Potential Future Additions
- `syntect` - Syntax highlighting (optional)
- `pulldown-cmark` - Markdown parsing alternative (if needed)
- `tui-textarea` - Better text input (if multi-line needed)

## Known Limitations

### Technology Constraints
1. **Markdown Rendering**: Limited to ASCII/Unicode terminal characters
2. **No Images**: Text/markdown only (by design)
3. **Terminal-Bound**: Cannot display rich media
4. **Single Thread UI**: UI loop is synchronous (event polling)

### API Constraints
1. **Ollama Only**: Currently supports only Ollama-compatible APIs
2. **Streaming Required**: Assumes streaming response format
3. **Local Only**: Expects localhost:11434 (configurable)

## Performance Characteristics

### Startup
- Cold start: ~50ms
- Config loading: ~5ms
- UI initialization: ~10ms

### Runtime
- Event loop: 60 FPS (16ms poll interval)
- Rendering: Incremental (only on changes)
- Memory: ~5-10MB base + message history

### Streaming
- Latency: ~16ms per chunk (poll rate)
- Throughput: Depends on Ollama instance
- Buffering: Line-based (newline-delimited JSON)

## Testing Strategy

### Unit Tests (56 tests)
- App state management
- Token counting
- Configuration loading
- Storage operations
- Markdown parsing
- API request/response parsing

### Integration Tests (2, ignored by default)
- Real Ollama API calls
- End-to-end streaming
- Requires: Ollama running with qwen3:4b model

### Test Isolation
- tempfile for file system tests
- Environment variable save/restore
- Parallel execution safe (test-threads)

## Monitoring & Debugging

### Logging
- Currently: Minimal (errors displayed in UI)
- Future: Optional debug logging

### Debugging Tools
- RUST_BACKTRACE=1 for stack traces
- cargo test -- --nocapture for test output
- `dbg!()` macro for quick debugging

## Security Considerations

### Current Measures
- No credential storage yet
- Local-only API access
- File system: User's config directory
- Input validation: Basic (URL validation)

### Future Security
- API key storage (encrypted)
- TLS/HTTPS support for remote APIs
- Input sanitization for remote APIs

## License
- MIT License
- All dependencies: MIT or Apache-2.0 compatible
