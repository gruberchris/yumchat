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
use std::time::Duration;
use tokio::sync::mpsc;

use app::App;
use api::OllamaClient;
use events::AppEvent;

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
            // Append chunk to the last message (which should be the AI response)
            if let Some(last_msg) = app.messages.last_mut() {
                if last_msg.role == models::MessageRole::Assistant {
                    last_msg.content.push_str(&chunk);
                    // Update token count
                    let role_str = match last_msg.role {
                        models::MessageRole::User => "user",
                        models::MessageRole::Assistant => "assistant",
                    };
                    last_msg.tokens = tokens::count_message_tokens(role_str, &last_msg.content);
                    
                    // Auto-scroll to bottom to show new content
                    app.scroll_to_bottom();
                }
            }
        }
        AppEvent::AiResponseDone => {
            app.is_loading = false;
            // Ensure we're scrolled to bottom when response completes
            app.scroll_to_bottom();
        }
        AppEvent::AiError(error) => {
            app.is_loading = false;
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
) {
    match key {
        KeyCode::Char('c' | 'q') if modifiers.contains(event::KeyModifiers::CONTROL) => {
            app.quit();
        }
        KeyCode::Char('h') if modifiers.contains(event::KeyModifiers::CONTROL) => {
            app.toggle_help();
        }
        KeyCode::Tab => {
            app.toggle_focus();
        }
        // Scrolling only works when History window has focus
        KeyCode::Up if matches!(app.focus, app::Focus::History) => {
            app.scroll_up(1);
        }
        KeyCode::Down if matches!(app.focus, app::Focus::History) => {
            app.scroll_down(1);
        }
        KeyCode::PageUp if matches!(app.focus, app::Focus::History) => {
            app.scroll_up(10);  // Use reasonable default page size
        }
        KeyCode::PageDown if matches!(app.focus, app::Focus::History) => {
            app.scroll_down(10);  // Use reasonable default page size
        }
        KeyCode::Home if matches!(app.focus, app::Focus::History) => {
            app.scroll_to_top();
        }
        KeyCode::End if matches!(app.focus, app::Focus::History) => {
            app.scroll_to_bottom();
        }
        // Typing only works when Input window has focus
        KeyCode::Char(c) if matches!(app.focus, app::Focus::Input) => {
            app.input_buffer.push(c);
        }
        KeyCode::Backspace if matches!(app.focus, app::Focus::Input) => {
            app.input_buffer.pop();
        }
        KeyCode::Enter if matches!(app.focus, app::Focus::Input) && !app.is_loading => {
            if !app.input_buffer.is_empty() {
                send_message(app, client, event_tx);
            }
        }
        _ => {}
    }
}

fn send_message(
    app: &mut App,
    client: &OllamaClient,
    event_tx: &mpsc::UnboundedSender<AppEvent>,
) {
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
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(response) => {
                            let text = response.get_text();
                            if !text.is_empty() {
                                let _ = tx.send(AppEvent::AiResponseChunk(text.to_string()));
                            }
                            if response.done {
                                let _ = tx.send(AppEvent::AiResponseDone);
                                break;
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::AiError(e.to_string()));
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                let _ = tx.send(AppEvent::AiError(e.to_string()));
            }
        }
    });
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
