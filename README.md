# YumChat

A terminal UI chat application for AI models powered by Ollama.

## Status

🚀 **Phase 6 Complete** - Real AI chat is now working!

### Completed
- ✅ Phase 1: Project Setup & Core Infrastructure
- ✅ Phase 2: Configuration & Storage  
- ✅ Phase 3: Ollama API Client
- ✅ Phase 4: Basic UI Layout
- ✅ Phase 5: Keyboard Navigation & Focus Management
- ✅ Phase 6: Streaming AI Chat Integration

### Current Features
- **Real AI Chat** - Streaming responses from Ollama models (qwen3:4b)
- **Beautiful TUI** - Status bar, chat history, and input field
- **Arrow Key Navigation** - Fine-grained line-by-line scrolling with Up/Down arrows
- **Token Tracking** - Real-time token counting with color-coded context window indicator
- **Loading Indicator** - Visual feedback while waiting for AI responses
- **Auto-Scroll** - Smart auto-scrolling to show latest content
- **Help System** - Built-in keyboard shortcuts (Ctrl+H)
- **Cross-platform** - Works on macOS, Linux, Windows

### Try It Now!
```bash
# Make sure Ollama is running with qwen3:4b model
ollama pull qwen3:4b

# Run YumChat
cargo run
```

**Controls:**
- **Type & Enter** - Send message to AI
- **Tab** - Toggle hidden thinking blocks
- **Up/Down Arrow** - Scroll chat history
- **PageUp/PageDown** - Scroll one page
- **Home/End** - Jump to start/end
- **Ctrl+H** - Show/hide help window
- **Ctrl+I** - Show/hide model info
- **Ctrl+Q** or **Ctrl+C** - Quit
- **Esc** - Close help window

### Coming Next
- 🔨 Phase 7: Conversation Management (save/load conversations)
- 📋 Phase 8: Settings UI (model selection)
- 🤖 Phase 9: Auto-generated conversation summaries

## Development

### Prerequisites
- Rust 1.75+ (with 2021 edition)
- Ollama (for AI features - optional for UI testing)

### Build
```bash
cargo build
```

### Test
```bash
# Run all tests
cargo test

# Run with real Ollama integration tests
cargo test -- --ignored
```

### Run
```bash
cargo run
```

### Quality Checks
```bash
# Format code
cargo fmt

# Check for warnings  
cargo clippy -- -D warnings

# Build release
cargo build --release
```

## Project Structure
```
src/
├── main.rs        # Entry point and async event loop
├── app.rs         # Application state management  
├── events.rs      # Async event types
├── models.rs      # Data structures
├── config.rs      # Configuration management
├── tokens.rs      # Token counting utilities
├── ui/            # UI rendering
│   ├── mod.rs     # Main UI layout
│   └── widgets.rs # UI components (status, chat, input, help)
├── api/           # Ollama API client with streaming
│   └── mod.rs
└── storage/       # File system operations
    └── mod.rs
```

## Configuration

YumChat stores its configuration in `~/.config/yumchat/`:
- `config.toml` - App settings (Ollama URL, model, theme)
- `models.json` - Model definitions (context window sizes)
- `chats/` - Conversation files (coming in Phase 7)

## Requirements

- Rust 1.75+ (2021 edition)
- Ollama running locally with at least one model installed
- Terminal with true color support (recommended)

## Build Quality

- **Zero warnings** enforced via clippy
- **All tests passing** (46 unit tests + 2 integration tests)
- **Strict linting** enabled (clippy::all, pedantic, nursery)

## License
MIT
