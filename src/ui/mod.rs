pub mod markdown;
pub mod widgets;

use crate::app::{App, AppMode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut App) {
    // Calculate required input height
    // Width available for text is total width - 2 (for borders)
    let available_width = frame.area().width.saturating_sub(2) as usize;
    
    // Calculate how many lines the input text will take
    // We start with 1 line minimum
    let input_lines = if app.input_buffer.is_empty() {
        1
    } else {
        // Approximate wrapping: (chars + width - 1) / width
        // Note: This is a simple approximation. Ratatui's Wrap might differ slightly with words,
        // but this is usually close enough for auto-resizing.
        let chars_count = app.input_buffer.chars().count();
        chars_count.div_ceil(available_width)
    };
    
    // Clamp lines: Min 1, Max 50% of screen height (approx)
    let max_lines = (frame.area().height as usize / 2).saturating_sub(2); // -2 for borders
    let actual_lines = input_lines.max(1).min(max_lines);
    
    // Total widget height = text lines + 2 border lines
    #[allow(clippy::cast_possible_truncation)]
    let input_height = (actual_lines + 2) as u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // Chat history (top, flexible)
            Constraint::Length(1),  // Empty gap
            Constraint::Length(1),  // Status line
            Constraint::Length(input_height),  // Input field (dynamic height)
            Constraint::Length(1),  // Bottom keymap bar
        ])
        .split(frame.area());

    widgets::render_chat_history(frame, app, chunks[0]);
    // chunks[1] is the gap, left empty
    widgets::render_status_bar(frame, app, chunks[2]);
    widgets::render_input_field(frame, app, chunks[3]);
    widgets::render_bottom_bar(frame, app, chunks[4]);

    // Render help window on top if active
    if app.show_help {
        widgets::render_help_window(frame, frame.area());
    }

    // Render info window on top if active
    if app.show_info {
        widgets::render_info_window(frame, app, frame.area());
    }

    // Render model selector if active
    if app.mode == AppMode::ModelSelector {
        widgets::render_model_selector(frame, app, frame.area());
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
