# YumChat Architecture

## Overview
YumChat is a terminal UI chat application built with Rust, following a **layered architecture** with clear separation of concerns. The architecture supports a hybrid sync/async model where the UI remains responsive while AI interactions happen asynchronously.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Terminal UI                       │
│              (Synchronous Event Loop)               │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────┐ │
│  │ Status Bar  │  │   History    │  │   Input   │ │
│  │             │  │  (Scrollable)│  │  (Focus)  │ │
│  └─────────────┘  └──────────────┘  └───────────┘ │
└─────────────────────────────────────────────────────┘
                        ↕
┌─────────────────────────────────────────────────────┐
│              Application State (App)                │
│  ┌─────────────────────────────────────────────┐   │
│  │ Messages, Focus, Scroll, Loading, Tokens   │   │
│  └─────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
           ↕                    ↕                  ↕
    ┌─────────┐         ┌─────────────┐    ┌──────────┐
    │ Storage │         │  API Client │    │  Events  │
    │ (Files) │         │  (Async)    │    │ (Channel)│
    └─────────┘         └─────────────┘    └──────────┘
           ↕                    ↕
    ┌─────────┐         ┌─────────────┐
    │ Config  │         │   Ollama    │
    │  Files  │         │     API     │
    └─────────┘         └─────────────┘
```

## Architectural Layers

### 1. Presentation Layer (UI)
**Location**: `src/ui/`
**Responsibilities**:
- Render terminal interface using ratatui
- Convert markdown to styled text
- Handle visual presentation only
- No business logic

**Components**:
- `widgets.rs` - Individual widget rendering functions
- `markdown.rs` - Markdown-to-ASCII converter
- `mod.rs` - Coordinates rendering

**Key Principles**:
- Pure presentation logic
- No state mutation (receives `&App`, not `&mut App`)
- Returns nothing (renders to Frame)
- Stateless between renders

**Example**:
```rust
pub fn render_chat_history(frame: &mut Frame, app: &App, area: Rect) {
    // Convert app state to visual representation
    // No app.foo = bar mutations
}
```

### 2. Application Layer (State Management)
**Location**: `src/app.rs`
**Responsibilities**:
- Hold application state
- Provide state transition methods
- Manage focus and scrolling
- Track token usage

**State Structure**:
```rust
pub struct App {
    // View State
    pub messages: Vec<Message>,
    pub input_buffer: String,
    pub scroll_offset: usize,
    pub focus: Focus,
    pub show_help: bool,
    
    // Model State
    pub current_model: String,
    pub context_window_size: usize,
    
    // UI State
    pub is_loading: bool,
    pub should_quit: bool,
    pub mode: AppMode,
}

pub enum Focus {
    Input,    // User can type
    History,  // User can scroll
}
```

**Key Methods**:
- `scroll_up/down/to_top/to_bottom()` - Navigation
- `toggle_focus()` - Switch between windows
- `calculate_total_lines()` - For scroll clamping
- `total_tokens_used()` - Context tracking
- `context_usage_percentage()` - Display in UI

**Key Principles**:
- Single source of truth for state
- Immutable by default (explicit `mut` methods)
- No I/O operations (delegated to storage/API layers)
- No async code (state is sync)

### 3. API Layer (External Communication)
**Location**: `src/api/`
**Responsibilities**:
- HTTP client for Ollama API
- Request/response serialization
- Streaming response parsing
- Error handling with context

**Architecture Pattern**: **Async Client, Sync Consumer**
```rust
// Async API call
pub async fn generate_stream(
    &self,
    request: GenerateRequest,
) -> Result<Pin<Box<dyn Stream<Item = Result<GenerateResponse>> + Send>>>
```

**Usage Pattern**:
```rust
// Spawn async task
tokio::spawn(async move {
    match client.generate_stream(request).await {
        Ok(mut stream) => {
            while let Some(result) = stream.next().await {
                // Send to sync UI via channel
                tx.send(AppEvent::AiResponseChunk(chunk))
            }
        }
    }
});
```

**Key Components**:
- `OllamaClient` - HTTP client wrapper
- `GenerateRequest/Response` - API types
- `generate_stream()` - Streaming responses
- `list_models()` - Available models
- `show_model()` - Model details

### 4. Storage Layer (Persistence)
**Location**: `src/storage/`
**Responsibilities**:
- File system operations
- Conversation persistence
- Configuration management
- Cross-platform path handling

**File Structure**:
```
~/.config/yumchat/
├── config.toml              # App configuration
├── models.json              # Model definitions
└── chats/
    ├── {uuid}.md           # Conversation content (markdown)
    ├── {uuid}_meta.json    # Metadata (summary, timestamps)
    └── ...
```

**Key Methods**:
- `save_conversation()` - Persist messages as markdown
- `load_conversation()` - Parse markdown to messages
- `save_metadata()` - Store conversation metadata
- `list_conversations()` - Get all conversations

**Key Principles**:
- Markdown for human-readable storage
- JSON for structured metadata
- Cross-platform (uses `dirs` crate)
- Atomic file operations

### 5. Models Layer (Data Structures)
**Location**: `src/models.rs`
**Responsibilities**:
- Define data structures
- Serialization/deserialization
- No business logic

**Key Types**:
```rust
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub tokens: usize,
}

pub enum MessageRole {
    User,
    Assistant,
}

pub struct AppConfig {
    pub ollama_url: String,
}

pub struct ConversationMetadata {
    pub id: Uuid,
    pub summary: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub total_tokens: usize,
}
```

**Key Principles**:
- Pure data (no methods beyond new/constructors)
- Derive Debug, Clone, Serialize, Deserialize as needed
- Public fields for simplicity

### 6. Event System (Async Coordination)
**Location**: `src/events.rs`
**Responsibilities**:
- Define event types for async → sync communication
- Enable decoupling of UI and API layers

**Event Types**:
```rust
pub enum AppEvent {
    AiResponseChunk(String),  // Streaming chunk
    AiResponseDone,           // Completion
    AiError(String),          // Error occurred
}
```

**Communication Flow**:
```
Async Task (API) → Channel → Sync Loop (UI) → App State → Render
```

## Data Flow

### User Input Flow
```
1. User Types → Input Buffer (App State)
2. User Presses Enter → send_message()
3. Create User Message → Add to App.messages
4. Create Empty Assistant Message → Add to App.messages
5. Spawn Async Task → Call API
6. Stream Chunks → Send AppEvent::AiResponseChunk
7. Update Last Message → Append chunk to content
8. Complete → Send AppEvent::AiResponseDone
9. Update UI State → is_loading = false
10. Render → Show updated messages
```

### Rendering Flow
```
1. Main Loop → Draw UI (60 FPS)
2. Call render() → Pass &App to UI layer
3. UI Layer → Convert state to widgets
4. Widgets → Render to terminal
5. No state mutation in UI layer
```

### Scroll Flow
```
1. User Presses Arrow Key (History focused)
2. handle_keyboard_input() → app.scroll_up(1)
3. App State → Update scroll_offset
4. render_chat_history() → Use scroll_offset for display
5. Clamp to valid range → Prevent over-scroll
```

### Focus Flow
```
1. User Presses Tab → app.toggle_focus()
2. Focus Enum → Switch Input ↔ History
3. Border Colors → Update based on focus
4. Keyboard Routing → Route keys to focused window
5. Render → Show visual indicator
```

## Concurrency Model

### Hybrid Sync/Async

**Synchronous**:
- Main event loop (UI rendering)
- State management (App mutations)
- Keyboard input handling
- File system operations (currently)

**Asynchronous**:
- HTTP API calls
- Response streaming
- Future: File I/O (planned)

### Communication Pattern

**Pattern**: Event-Driven with Channels
```rust
// Setup
let (tx, mut rx) = mpsc::unbounded_channel::<AppEvent>();

// Sync loop
loop {
    // Check for async events
    if let Ok(event) = rx.try_recv() {
        handle_app_event(&mut app, event);
    }
    
    // Check for keyboard events
    if event::poll(Duration::from_millis(16))? {
        handle_keyboard(&mut app, key);
    }
    
    // Render
    terminal.draw(|f| ui::render(f, &app))?;
    
    if app.should_quit { break; }
}

// Async task
tokio::spawn(async move {
    // Do async work
    tx.send(AppEvent::AiResponseChunk(chunk)).unwrap();
});
```

### Why This Model?

**Advantages**:
1. UI remains responsive during API calls
2. Simple state management (single-threaded)
3. No locks/mutexes needed
4. Clear separation of sync/async code
5. Easy to reason about

**Trade-offs**:
1. Channel overhead (minimal)
2. Event handling complexity
3. No parallel rendering (not needed for TUI)

## Error Handling Strategy

### Error Flow
```
API Error → Result<T, Error> → anyhow::Result → AppEvent::AiError
   ↓
App State → Add error message
   ↓
UI Layer → Render error in chat history
```

### Error Context Pattern
```rust
// Add context at each layer
api_call()
    .await
    .context("Failed to call API")?
```

### Error Display
- Errors shown in chat history as Assistant messages
- Prefixed with "Error: "
- User can see and scroll to errors
- No modal dialogs or popups

## Testing Architecture

### Layer Testing
1. **Models**: Simple struct tests
2. **Storage**: Uses tempfile for isolation
3. **API**: Uses wiremock for HTTP mocking
4. **App**: Pure unit tests (no I/O)
5. **UI**: Limited (visual validation)

### Test Isolation
- Environment variables saved/restored
- Temporary directories for file tests
- Parallel test execution supported
- No shared state between tests

### Integration Tests
- Marked `#[ignore]` by default
- Require Ollama running
- Test real API interactions
- Run manually: `cargo test -- --ignored`

## Performance Considerations

### Responsive UI
- 16ms event poll (60 FPS)
- Incremental rendering (ratatui handles this)
- Minimal state copies
- Efficient scroll calculations

### Memory Management
- Messages stored in Vec (append-only in memory)
- No memory compaction yet
- Future: LRU cache for old messages

### I/O Performance
- Async API calls (non-blocking)
- Buffered file writes
- Lazy loading planned for conversation list

## Security Architecture

### Current Security
- Local-only API access (localhost)
- No credential storage
- User config directory (standard permissions)
- Input: Basic validation only

### Future Security (Planned)
- API key encryption
- TLS/HTTPS support
- Input sanitization for remote APIs
- Secure credential storage

## Extensibility Points

### Adding New Features

**1. New UI Window**:
- Add widget function in `src/ui/widgets.rs`
- Update layout in `src/ui/mod.rs`
- Add focus state if needed

**2. New API Endpoint**:
- Add request/response types in `src/api/mod.rs`
- Add method to `OllamaClient`
- Follow streaming pattern if needed

**3. New Storage Type**:
- Add methods to `Storage` struct
- Follow markdown/JSON pattern
- Add tests with tempfile

**4. New Event Type**:
- Add variant to `AppEvent` enum
- Add handler in `handle_app_event()`
- Document event flow

## Design Principles

1. **Separation of Concerns**: Each layer has single responsibility
2. **Dependency Direction**: UI → App → API/Storage
3. **Immutability**: Default to immutable, explicit mut
4. **Error Context**: Add context at each layer
5. **Testing**: Write tests alongside implementation
6. **Consistency**: Follow existing patterns
7. **Simplicity**: Prefer simple over clever
8. **Performance**: Optimize after measuring

## Anti-Patterns to Avoid

1. ❌ **State in UI**: UI functions must not mutate app state
2. ❌ **Business Logic in Models**: Keep models as pure data
3. ❌ **Blocking in Async**: Don't use blocking calls in async functions
4. ❌ **Async in UI Loop**: Keep UI loop synchronous
5. ❌ **Shared Mutable State**: Use channels instead
6. ❌ **Error Swallowing**: Always handle or propagate errors
7. ❌ **Copy-Paste**: Extract common patterns into functions

## Future Architecture Evolution

### Phase 7 (Current)
- Conversation persistence
- Markdown file format
- Conversation list UI

### Phase 8
- Settings UI
- Model selection
- Configuration management

### Phase 9+
- Context window management
- Conversation search
- Export capabilities
- Multiple AI providers (Azure, Google)

### Potential Refactorings
- Extract UI into separate crate
- Plugin system for AI providers
- Separate binary for CLI mode
- Background daemon for API calls
