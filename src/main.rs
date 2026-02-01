mod api;
mod app;
mod config;
mod events;
mod models;
mod storage;
mod tokens;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{backend::Backend, prelude::*};
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use app::App;
use api::OllamaClient;
use events::AppEvent;

use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state and API client
    let mut app = App::new();
    let client = OllamaClient::with_default_url()?;

    // Fetch model info
    if let Ok(info) = client.show_model(&app.current_model).await {
        app.model_capabilities = info.capabilities;
        app.model_details = info.details;
        
        // Auto-enable thinking visibility if model supports thinking
        if app.model_capabilities.contains(&"thinking".to_string()) {
            app.show_thinking = false; // Keep default hidden, but user can toggle
        }
    }

    // Create channel for async events
    let (tx, mut rx) = mpsc::unbounded_channel::<AppEvent>();

    // Run app
    let res = run_app(&mut terminal, &mut app, &client, &tx, &mut rx);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

fn handle_app_event(app: &mut App, event: AppEvent) {
    match event {
        AppEvent::AiResponseChunk(chunk) => {
            // Ignore chunks if we are no longer loading (e.g. cancelled)
            if !app.is_loading {
                return;
            }

            // Check for thinking tags to toggle status
            if chunk.contains("<thinking>") {
                app.is_thinking = true;
            } else if chunk.contains("</thinking>") {
                app.is_thinking = false;
            }
            
            // Append chunk to the last message (which should be the AI response)
            if let Some(last_msg) = app.messages.last_mut() {
                if last_msg.role == models::MessageRole::Assistant {
                    // Update TPS
                    if app.generation_start_time.is_none() {
                        app.generation_start_time = Some(Instant::now());
                        app.generation_token_count = 0;
                    }
                    
                    // Rough token estimation (chars / 4 is a common approximation)
                    // Or count actual words/subwords if possible. 
                    // Since we get raw text chunks, let's just count chunk length / 4 for now as a rough metric
                    // or better, just count count the chunk count if we assume 1 chunk ~ 1 token (often true for streaming)
                    // But actually chunks can be multiple tokens.
                    // Let's use the actual token counter update logic to track delta
                    let old_tokens = last_msg.tokens;
                    
                    last_msg.content.push_str(&chunk);
                    
                    // Update token count
                    let role_str = match last_msg.role {
                        models::MessageRole::User => "user",
                        models::MessageRole::Assistant => "assistant",
                    };
                    last_msg.tokens = tokens::count_message_tokens(role_str, &last_msg.content);
                    
                    let new_tokens = last_msg.tokens;
                    let delta_tokens = new_tokens.saturating_sub(old_tokens);
                    
                    app.generation_token_count += delta_tokens;
                    
                    if let Some(start) = app.generation_start_time {
                        let elapsed = start.elapsed().as_secs_f64();
                        if elapsed > 0.0 {
                            app.tokens_per_second = app.generation_token_count as f64 / elapsed;
                        }
                    }
                    
                    // Auto-scroll to bottom to show new content
                    app.scroll_to_bottom();
                }
            }
        }
        AppEvent::AiResponseDone => {
            app.is_loading = false;
            app.is_thinking = false;
            app.generation_start_time = None;
            // Ensure we're scrolled to bottom when response completes
            app.scroll_to_bottom();
        }
        AppEvent::AiError(error) => {
            app.is_loading = false;
            app.is_thinking = false;
            // Add error message to chat
            app.messages.push(models::Message::new(
                models::MessageRole::Assistant,
                format!("Error: {error}"),
                0,
            ));
            // Auto-scroll to show error
            app.scroll_to_bottom();
        }
    }
}

const fn handle_help_keys(app: &mut App, key: KeyCode, modifiers: event::KeyModifiers) -> bool {
    if !app.show_help {
        return false;
    }

    match key {
        KeyCode::Char('h') if modifiers.contains(event::KeyModifiers::CONTROL) => {
            app.toggle_help();
        }
        KeyCode::Esc => {
            app.show_help = false;
        }
        _ => {}
    }
    true
}

fn handle_keyboard_input(
    app: &mut App,
    key: KeyCode,
    modifiers: event::KeyModifiers,
    client: &OllamaClient,
    event_tx: &mpsc::UnboundedSender<AppEvent>,
) -> Option<JoinHandle<()>> {
    match key {
        KeyCode::Char('c') if modifiers.contains(event::KeyModifiers::CONTROL) => {
            if app.exit_pending {
                app.quit();
            } else {
                app.exit_pending = true;
            }
        }
        KeyCode::Esc => {
            if app.show_help {
                app.show_help = false;
            } else if app.show_info {
                app.show_info = false;
            } else if app.exit_pending {
                app.exit_pending = false;
            } else if app.is_loading {
                app.abort_generation();
                return None; // Caller will handle task abortion
            }
        }
        _ if app.exit_pending => {
            // Any other key cancels pending exit
            app.exit_pending = false;
        }
        _ => {}
    }

    // If we didn't handle it above (or cancelled exit pending), continue
    if app.exit_pending {
        return None; 
    }

    match key {
        KeyCode::Char('q') if modifiers.contains(event::KeyModifiers::CONTROL) => {
             // Keep Ctrl+Q as instant quit 
            app.quit();
        }
        KeyCode::Char('h') if modifiers.contains(event::KeyModifiers::CONTROL) => {
            app.toggle_help();
        }
        KeyCode::Char('i') if modifiers.contains(event::KeyModifiers::CONTROL) => {
            app.toggle_info();
        }
        KeyCode::Tab => {
            // Toggle visibility of <thinking> blocks
            app.toggle_thinking();
        }
        
        // Navigation keys ALWAYS scroll history
        KeyCode::Up => app.scroll_up(1),
        KeyCode::Down => app.scroll_down(1),
        KeyCode::PageUp => app.scroll_up(10),
        KeyCode::PageDown => app.scroll_down(10),
        KeyCode::Home => app.scroll_to_top(),
        KeyCode::End => app.scroll_to_bottom(),
        
        // Editing keys ALWAYS affect input
        KeyCode::Backspace => {
            app.input_buffer.pop();
        },
        KeyCode::Enter if !app.is_loading => {
            if !app.input_buffer.is_empty() {
                return Some(send_message(app, client, event_tx));
            }
        },
        
        // Typing characters ALWAYS go to input
        KeyCode::Char(c) => {
            app.input_buffer.push(c);
        }
        
        _ => {}
    }
    None
}

fn send_message(
    app: &mut App,
    client: &OllamaClient,
    event_tx: &mpsc::UnboundedSender<AppEvent>,
) -> JoinHandle<()> {
    let user_msg = app.input_buffer.clone();

    // Add user message
    app.messages
        .push(models::Message::new_with_token_count(
            models::MessageRole::User,
            user_msg.clone(),
        ));

    // Add placeholder for AI response
    app.messages.push(models::Message::new(
        models::MessageRole::Assistant,
        String::new(),
        0,
    ));

    app.input_buffer.clear();
    app.is_loading = true;
    app.generation_start_time = None;
    app.tokens_per_second = 0.0;
    
    // Auto-scroll to show user message and prepare for AI response
    app.scroll_to_bottom();

    // Spawn async task to get AI response
    let client_clone = client.clone();
    let model = app.current_model.clone();
    let tx = event_tx.clone();

    tokio::spawn(async move {
        let request = api::GenerateRequest {
            model,
            prompt: user_msg,
            system: None,
            stream: true,
        };

        match client_clone.generate_stream(request).await {
            Ok(mut stream) => {
                let mut received_done = false;
                let mut in_thinking_block = false;
                
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(response) => {
                            // Handle thinking content
                            if !response.thinking.is_empty() {
                                if !in_thinking_block {
                                    let _ = tx.send(AppEvent::AiResponseChunk("<thinking>\n".to_string()));
                                    in_thinking_block = true;
                                }
                                let _ = tx.send(AppEvent::AiResponseChunk(response.thinking));
                            } 
                            
                            // Handle regular response content
                            if !response.response.is_empty() {
                                if in_thinking_block {
                                    let _ = tx.send(AppEvent::AiResponseChunk("\n</thinking>\n".to_string()));
                                    in_thinking_block = false;
                                }
                                let _ = tx.send(AppEvent::AiResponseChunk(response.response));
                            }
                            
                            if response.done {
                                if in_thinking_block {
                                    let _ = tx.send(AppEvent::AiResponseChunk("\n</thinking>\n".to_string()));
                                    in_thinking_block = false; // Not strictly needed but good for correctness
                                }
                                let _ = tx.send(AppEvent::AiResponseDone);
                                received_done = true;
                                break;
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::AiError(e.to_string()));
                            received_done = true;
                            break;
                        }
                    }
                }
                
                // If stream ended without explicit done signal or error, ensure we unblock UI
                if !received_done {
                    if in_thinking_block {
                        let _ = tx.send(AppEvent::AiResponseChunk("\n</thinking>\n".to_string()));
                    }
                    let _ = tx.send(AppEvent::AiResponseDone);
                }
            }
            Err(e) => {
                let _ = tx.send(AppEvent::AiError(e.to_string()));
            }
        }
    })
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    client: &OllamaClient,
    event_tx: &mpsc::UnboundedSender<AppEvent>,
    event_rx: &mut mpsc::UnboundedReceiver<AppEvent>,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        // Check for app events (AI responses) first
        if let Ok(app_event) = event_rx.try_recv() {
            handle_app_event(app, app_event);
        }

        // Check for keyboard input with shorter timeout for better responsiveness
        if event::poll(Duration::from_millis(16))? {  // ~60fps for smooth scrolling
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Handle help window first
                    if handle_help_keys(app, key.code, key.modifiers) {
                        continue;
                    }
                    
                    // Handle info window
                    if app.show_info {
                        if key.code == KeyCode::Esc || 
                           (key.code == KeyCode::Char('i') && key.modifiers.contains(event::KeyModifiers::CONTROL)) {
                            app.show_info = false;
                            continue;
                        }
                    }

                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            if app.exit_pending {
                                app.quit();
                            } else {
                                app.exit_pending = true;
                            }
                            continue;
                        }
                        KeyCode::Esc => {
                            if app.show_help {
                                app.show_help = false;
                                continue;
                            } else if app.show_info {
                                app.show_info = false;
                                continue;
                            } else if app.exit_pending {
                                app.exit_pending = false;
                                continue;
                            }
                        }
                        _ if app.exit_pending => {
                            // Any other key cancels pending exit
                            app.exit_pending = false;
                            // Fall through to process the key normally
                        }
                        _ => {}
                    }

                    // Normal key handling
                    handle_keyboard_input(app, key.code, key.modifiers, client, event_tx);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
