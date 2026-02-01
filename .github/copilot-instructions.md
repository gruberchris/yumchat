# GitHub Copilot Instructions for YumChat

## Priority Guidelines

When generating code for this repository:

1. **Version Compatibility**: Always respect the exact versions of Rust, crates, and dependencies specified in Cargo.toml
2. **Architecture Consistency**: Maintain the layered architecture (UI, State, API, Storage, Models)
3. **Code Quality**: Prioritize maintainability, testability, and zero-warning builds
4. **Test-Driven**: Write unit tests for all new behaviors added to source code; maintain 100% test pass rate
5. **Pattern Following**: Match existing patterns for error handling, async operations, and UI rendering

## Technology Stack (Exact Versions)

### Language
- **Rust Edition**: 2021
- Language features: Limited to Rust 2021 edition stable features
- Toolchain: Use features available in stable Rust (no nightly features)

### Core Dependencies
```toml
ratatui = "0.29"          # TUI framework
crossterm = "0.28"        # Terminal backend
tokio = "1.42"            # Async runtime (full features)
reqwest = "0.12"          # HTTP client (json, stream features)
serde = "1.0"             # Serialization
serde_json = "1.0"        # JSON handling
toml = "0.8"              # Config parsing
uuid = "1.11"             # UUIDs (v4, serde features)
chrono = "0.4"            # Timestamps (serde feature)
dirs = "5.0"              # Cross-platform directories
anyhow = "1.0"            # Error handling
thiserror = "1.0"         # Error types
futures = "0.3"           # Async utilities
```

### Development Dependencies
```toml
mockall = "0.13"          # Mocking
tempfile = "3.13"         # Temporary files for tests
wiremock = "0.6"          # HTTP mocking
tokio-test = "0.4"        # Async test utilities
```

### Build Configuration
- **Release Profile**: Optimized (opt-level 3, LTO enabled, single codegen unit)
- **Lints**: All clippy warnings enabled (all, pedantic, nursery)
- **Build Standard**: Zero warnings required (`-D warnings`)

## Architecture

### Module Organization
```
src/
├── main.rs           # Entry point, event loop, async coordination
├── app.rs            # Application state management
├── events.rs         # Event types for async communication
├── models.rs         # Data structures (Message, AppConfig, etc.)
├── tokens.rs         # Token counting utilities
├── config.rs         # Configuration management
├── api/
│   └── mod.rs        # Ollama API client with streaming
├── storage/
│   └── mod.rs        # File system operations
└── ui/
    ├── mod.rs        # UI rendering coordinator
    ├── widgets.rs    # Widget rendering functions
    └── markdown.rs   # Markdown-to-ASCII converter
```

### Architecture Layers

**1. UI Layer** (`src/ui/`)
- Renders terminal UI using ratatui
- Converts markdown to styled text
- Handles visual presentation only
- No business logic in UI code

**2. State Management** (`src/app.rs`)
- Holds application state (messages, scroll, focus)
- Provides methods for state transitions
- Implements Focus enum pattern
- Manages scroll offset and window focus

**3. API Client** (`src/api/`)
- Async HTTP client for Ollama API
- Streaming response parsing
- Error handling with context
- Returns `Pin<Box<dyn Stream>>` for streaming

**4. Storage Layer** (`src/storage/`)
- File system operations (conversations, config)
- Cross-platform path handling
- Markdown file format for conversations
- JSON for metadata

**5. Models** (`src/models.rs`)
- Pure data structures with derive macros
- Serde serialization/deserialization
- No business logic in models

## Code Quality Standards

### Maintainability

**Naming Conventions:**
- `snake_case` for functions, variables, modules
- `PascalCase` for types, traits, enums
- `SCREAMING_SNAKE_CASE` for constants
- Descriptive names: `calculate_total_lines()` not `calc_lines()`
- Boolean functions: `is_*`, `has_*`, `should_*`

**Function Design:**
- Single responsibility principle
- Keep functions under 50 lines (use `#[allow(clippy::too_many_lines)]` if necessary)
- Extract complex logic into separate functions
- Use const fn when possible

**Code Organization:**
- Group related functions together
- Public API at top of module
- Private helpers below
- Tests in `#[cfg(test)]` module at bottom

### Error Handling

**Pattern: Context-Rich Errors**
```rust
use anyhow::{Context, Result};

pub fn load_config() -> Result<AppConfig> {
    let path = get_config_path()?;
    
    let contents = fs::read_to_string(&path)
        .context("Failed to read config file")?;
    
    toml::from_str(&contents)
        .context("Failed to parse config file")
}
```

**Error Types:**
- Use `anyhow::Result` for application code
- Use `thiserror` for library error types
- Always add context with `.context()` or `.with_context()`
- Never use `.unwrap()` in production code
- Use `.expect()` only with clear explanation

### Async Patterns

**Hybrid Async/Sync Architecture:**
- Main UI loop is **synchronous** (blocking event poll)
- AI API calls spawn **async** tokio tasks
- Communication via `mpsc::unbounded_channel`

**Event-Driven Pattern:**
```rust
// Define events
pub enum AppEvent {
    AiResponseChunk(String),
    AiResponseDone,
    AiError(String),
}

// Spawn async task
tokio::spawn(async move {
    match api_call().await {
        Ok(response) => tx.send(AppEvent::AiResponseChunk(response)),
        Err(e) => tx.send(AppEvent::AiError(e.to_string())),
    }
});

// Handle in sync loop
if let Ok(event) = rx.try_recv() {
    handle_app_event(app, event);
}
```

**Streaming Pattern:**
```rust
pub async fn generate_stream(
    &self,
    request: GenerateRequest,
) -> Result<Pin<Box<dyn Stream<Item = Result<GenerateResponse>> + Send>>> {
    let response = self.client.post(&url).json(&request).send().await?;
    let stream = response.bytes_stream().map(|result| {
        // Parse each chunk
    });
    Ok(Box::pin(stream))
}
```

### Testing Standards

**Unit Test Structure:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_descriptive_name() {
        // Arrange
        let mut app = App::new();
        
        // Act
        app.scroll_down(5);
        
        // Assert
        assert_eq!(app.scroll_offset, 5);
    }
}
```

**Test Naming:**
- Format: `test_<function>_<scenario>_<expected>`
- Example: `test_scroll_down_clamps_to_max`
- Be descriptive: `test_load_config_creates_default_when_missing`

**Test Isolation:**
- Use `tempfile::TempDir` for file system tests
- Save/restore environment variables
- Clean up in same test that modifies state
- Run tests in parallel by default

**Float Comparisons:**
```rust
// Don't use assert_eq! for floats
assert!((result - expected).abs() < f64::EPSILON);
```

**Async Tests:**
```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

**Ignored Tests:**
```rust
#[test]
#[ignore = "Requires Ollama running"]
fn test_real_api() {
    // Integration test
}
```

### Documentation

**Public API Documentation:**
```rust
/// Calculate the total number of lines needed to render all messages
///
/// This includes:
/// - Empty lines between messages
/// - Role headers (## User, ## Assistant)
/// - Content lines (at least 1 even if empty)
///
/// # Returns
/// Total line count, minimum 1
fn calculate_total_lines(&self) -> usize {
    // Implementation
}
```

**Comment Style:**
- Use `///` for public API docs
- Use `//` for inline comments
- Explain *why*, not *what* (code shows what)
- Document non-obvious behavior
- Include examples for complex functions

### Code Style

**Formatting:**
- Run `cargo fmt` before committing
- Line length: 100 characters max (enforced by rustfmt)
- Imports organized: std, external crates, local modules

**Clippy Compliance:**
- Zero clippy warnings required
- Run `cargo clippy --all-targets -- -D warnings`
- Fix warnings, don't suppress unless necessary
- Use `#[allow(clippy::...)]` with explanation if needed

**Match Expressions:**
```rust
// Prefer early returns for simple cases
match result {
    Ok(value) => value,
    Err(e) => return Err(e),
}

// Use if let for single pattern
if let Some(last_msg) = app.messages.last_mut() {
    last_msg.content.push_str(&chunk);
}
```

## UI/UX Patterns

### Focus Management

**Focus Enum:**
```rust
pub enum Focus {
    Input,   // User can type, cannot scroll
    History, // User can scroll, cannot type
}
```

**Visual Indicators:**
- **Cyan border**: Input focused (can type)
- **Yellow border**: History focused (can scroll)
- **DarkGray border**: Window not focused
- No "[Focused]" text in titles

**Focus-Aware Controls:**
- Tab switches focus between Input/History
- Arrow keys only work when History focused
- Typing only works when Input focused

### Scrolling Behavior

**Auto-Scroll Pattern:**
```rust
// After sending message
app.scroll_to_bottom();

// After receiving AI chunk
app.scroll_to_bottom();

// After response completes
app.scroll_to_bottom();
```

**Manual Scroll (History focus only):**
- Up/Down: Scroll 1 line
- PageUp/PageDown: Scroll 10 lines
- Home: Jump to top
- End: Jump to bottom

**Scroll Implementation:**
```rust
pub const fn scroll_to_bottom(&mut self) {
    // Set to max value, renderer will clamp
    self.scroll_offset = usize::MAX;
}
```

### Markdown Rendering

**Storage Format:** Pure markdown (`.md` files)
**Display Format:** Styled ASCII using custom parser

**Supported Elements:**
- **Bold**: `**text**` → Yellow + Bold modifier
- **Inline Code**: `` `code` `` → Magenta color
- **Headers**: `# ## ###` → Cyan/Blue, bold
- **Lists**: `- item` or `* item` → Cyan bullet (•)
- **Code Blocks**: ` ```lang ` → Boxed with borders
- **Tables**: `| cell |` → Simplified with `|` separators
- **Thinking Tags**: `<thinking>` → DarkGray, indented 4 spaces

**Code Block Rendering:**
```
┌─ python ──────────────────
  def hello():
    print("Hello")
└───────────────────────────
```

### Event Loop Pattern

**Timing:**
```rust
// Poll every 16ms (~60fps) for responsive input
if event::poll(Duration::from_millis(16))? {
    if let Event::Key(key) = event::read()? {
        // Handle keyboard input
    }
}
```

**Event Handling Order:**
1. Check for async events (AI responses)
2. Check for keyboard input
3. Render UI
4. Repeat

## Technology-Specific Guidelines

### Rust-Specific Patterns

**Const Functions:**
```rust
pub const fn quit(&mut self) {
    self.should_quit = true;
}
```

**Error Propagation:**
```rust
// Use ? operator
let config = load_config()?;

// Not this
let config = match load_config() {
    Ok(c) => c,
    Err(e) => return Err(e),
};
```

**Iterators Over Loops:**
```rust
// Prefer
let total: usize = self.messages.iter().map(|m| m.tokens).sum();

// Over
let mut total = 0;
for message in &self.messages {
    total += message.tokens;
}
```

**String Handling:**
```rust
// Use format! for complex strings
format!("Error: {error}")

// Use .to_string() for String conversion
"hello".to_string()

// Use &str in function parameters when possible
fn process(text: &str) { }
```

### Ratatui Patterns

**Widget Rendering:**
```rust
pub fn render_widget(frame: &mut Frame, app: &App, area: Rect) {
    let lines = vec![/* styled lines */];
    
    let widget = Paragraph::new(lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Title")
            .border_style(Style::default().fg(color)))
        .wrap(Wrap { trim: false })
        .scroll((scroll_y, scroll_x));
    
    frame.render_widget(widget, area);
}
```

**Color Usage:**
- `Color::Cyan` - Input focus, user messages
- `Color::Yellow` - History focus, headers
- `Color::Green` - AI messages, code
- `Color::DarkGray` - Unfocused, thinking, placeholders
- `Color::Magenta` - Inline code
- `Color::Red` - Errors (when usage >= 80%)

### Tokio Patterns

**Runtime Setup:**
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Hybrid sync UI + async API
}
```

**Channel Pattern:**
```rust
let (tx, mut rx) = mpsc::unbounded_channel::<AppEvent>();

// Sender clones
let tx_clone = tx.clone();

// Receiver
if let Ok(event) = rx.try_recv() {
    handle_event(event);
}
```

**Stream Usage:**
```rust
use futures::StreamExt;

while let Some(result) = stream.next().await {
    match result {
        Ok(chunk) => process(chunk),
        Err(e) => handle_error(e),
    }
}
```

## File and Path Handling

**Cross-Platform Paths:**
```rust
use dirs;
use std::path::PathBuf;

pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .join("yumchat");
    
    fs::create_dir_all(&config_dir)?;
    Ok(config_dir)
}
```

**File Operations:**
```rust
// Use PathBuf for path manipulation
let path = config_dir.join("config.toml");

// Use fs::read_to_string for text files
let contents = fs::read_to_string(&path)
    .context("Failed to read file")?;

// Use fs::write for atomic writes
fs::write(&path, contents)
    .context("Failed to write file")?;
```

## API Client Patterns

**Request/Response Types:**
```rust
#[derive(Debug, Serialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub stream: bool,
}

#[derive(Debug, Deserialize)]
pub struct GenerateResponse {
    #[serde(default)]
    pub response: String,
    #[serde(default)]
    pub thinking: String,
    pub done: bool,
    #[serde(default)]
    pub context: Vec<i32>,
}
```

**Helper Methods:**
```rust
impl GenerateResponse {
    /// Get text content (prioritize response over thinking)
    pub fn get_text(&self) -> &str {
        if self.response.is_empty() {
            &self.thinking
        } else {
            &self.response
        }
    }
}
```

## Project-Specific Standards

### Build Verification
Before submitting changes, run:
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

All must pass with:
- Zero clippy warnings
- 100% test pass rate
- Successful release build

### Commit Standards
- Fix all warnings before committing
- Ensure tests pass
- Update documentation if behavior changes
- Maintain zero-warning standard

### Version Management
- Use Semantic Versioning (0.1.0)
- Update version in Cargo.toml
- Document changes in CHANGELOG (when created)

## Common Patterns

### State Updates
```rust
// Immutable by default
let config = load_config()?;

// Explicit mut when needed
let mut app = App::new();
app.scroll_down(5);
```

### Option Handling
```rust
// Use if let for single pattern
if let Some(last_msg) = app.messages.last_mut() {
    last_msg.content.push_str(&chunk);
}

// Use unwrap_or for defaults
let text = response.thinking.unwrap_or_default();

// Use ok_or for Result conversion
let home = std::env::var("HOME").ok();
```

### Collection Operations
```rust
// Check emptiness
if app.messages.is_empty() { }

// Count items
let count = app.messages.len();

// Filter and collect
let errors: Vec<_> = results.iter()
    .filter(|r| r.is_err())
    .collect();
```

## Testing Requirements

### Coverage Expectations
- All public functions must have tests
- Test both happy path and error cases
- Test edge cases (empty input, max values)
- Mock external dependencies (file system, HTTP)

### Test Organization
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn setup_test_env() -> TempDir {
        TempDir::new().unwrap()
    }
    
    #[test]
    fn test_feature() {
        let temp = setup_test_env();
        // Test implementation
    }
}
```

### Mock Usage
```rust
#[cfg(test)]
use mockall::automock;

#[automock]
trait ApiClient {
    fn generate(&self, request: Request) -> Result<Response>;
}
```

## General Best Practices

1. **Follow Existing Patterns**: When unsure, match the style of similar code in the codebase
2. **Prioritize Consistency**: Consistency with existing code > external best practices
3. **Zero Warnings**: Fix all warnings; never commit code with warnings
4. **Test Coverage**: Write tests alongside implementation
5. **Error Context**: Always add context to errors for debugging
6. **Documentation**: Document public APIs and non-obvious behavior
7. **Performance**: Use appropriate data structures; profile before optimizing
8. **Safety**: Prefer safe Rust; document any unsafe usage thoroughly

## When In Doubt

1. Check similar files for patterns
2. Refer to this document
3. Prioritize code consistency
4. Ask for clarification rather than guessing
5. Write tests to validate behavior
