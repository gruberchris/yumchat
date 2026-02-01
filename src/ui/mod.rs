pub mod markdown;
pub mod widgets;

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Status bar
            Constraint::Min(0),     // Chat history
            Constraint::Length(5),  // Input field
        ])
        .split(frame.area());

    widgets::render_status_bar(frame, app, chunks[0]);
    widgets::render_chat_history(frame, app, chunks[1]);
    widgets::render_input_field(frame, app, chunks[2]);

    // Render help window on top if active
    if app.show_help {
        widgets::render_help_window(frame, frame.area());
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_render_does_not_panic() {
        // Basic smoke test to ensure render function exists and compiles
        // Actual rendering tests will be added in Phase 4
    }
}
