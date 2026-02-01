use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap, Clear},
    Frame,
};

use crate::app::App;

pub fn render_help_window(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled(
            "YumChat - Keyboard Shortcuts",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled("General:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+H        - Show/hide this help"),
        Line::from("  Ctrl+Q        - Quit application"),
        Line::from("  Ctrl+C        - Quit application"),
        Line::from(""),
        Line::from(Span::styled("Chat:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Enter         - Send message (when input focused)"),
        Line::from("  Backspace     - Delete character"),
        Line::from("  Tab           - Switch focus between input/history"),
        Line::from(""),
        Line::from(Span::styled("Focus Indicators:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Cyan border   - Input has focus (can type)"),
        Line::from("  Yellow border - History has focus (can scroll)"),
        Line::from("  Gray border   - Window not focused"),
        Line::from(""),
        Line::from(Span::styled("Navigation (History focus only):", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Up Arrow      - Scroll up one line"),
        Line::from("  Down Arrow    - Scroll down one line"),
        Line::from("  Page Up       - Scroll up one page"),
        Line::from("  Page Down     - Scroll down one page"),
        Line::from("  Home          - Jump to start of conversation"),
        Line::from("  End           - Jump to end of conversation"),
        Line::from(""),
        Line::from(Span::styled("Coming Soon:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+N        - New conversation"),
        Line::from("  Ctrl+L        - List conversations"),
        Line::from("  Ctrl+S        - Settings"),
        Line::from(""),
        Line::from(Span::styled(
            "Press Ctrl+H or Esc to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    // Calculate centered position
    let popup_width = 60;
    let popup_height = 35;  // Increased from 28 to accommodate new help lines
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x: area.x + x,
        y: area.y + y,
        width: popup_width.min(area.width),
        height: popup_height.min(area.height),
    };

    frame.render_widget(Clear, popup_area);
    frame.render_widget(help_paragraph, popup_area);
}

pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let usage_percentage = app.context_usage_percentage();
    
    let color = if usage_percentage > 80.0 {
        Color::Red
    } else if usage_percentage > 50.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let tokens_used = app.total_tokens_used();
    let context_window = app.context_window_size;
    
    let loading_indicator = if app.is_loading { " [Thinking...]" } else { "" };
    
    let status_text = format!(
        " Model: {} | Tokens: {}/{} ({:.1}%){}",
        app.current_model, tokens_used, context_window, usage_percentage, loading_indicator
    );

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("YumChat"));

    frame.render_widget(status, area);
}

pub fn render_chat_history(frame: &mut Frame, app: &mut App, area: Rect) {
    let mut lines = Vec::new();

    if app.messages.is_empty() {
        lines.push(Line::from(Span::styled(
            "No messages yet. Start typing below to begin a conversation.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for message in &app.messages {
            let (role, color) = match message.role {
                crate::models::MessageRole::User => ("User", Color::Cyan),
                crate::models::MessageRole::Assistant => ("Assistant", Color::Green),
            };

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("## {role}"),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            // Render content with markdown styling
            if message.content.is_empty() {
                // Show a placeholder for empty AI responses (while streaming)
                lines.push(Line::from(Span::styled("...", Style::default().fg(Color::DarkGray))));
            } else {
                // Check if we're in a code block or thinking block
                let mut in_code_block = false;
                let mut in_thinking = false;
                
                for content_line in message.content.lines() {
                    // Check for thinking tags first
                    if content_line.trim() == "<thinking>" {
                        in_thinking = true;
                        lines.push(Line::from(Span::styled(
                            "  [Thinking...]",
                            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                        )));
                        continue;
                    }
                    
                    if content_line.trim() == "</thinking>" {
                        in_thinking = false;
                        continue;
                    }
                    
                    // If in thinking block, render with subdued style
                    if in_thinking {
                        lines.push(Line::from(Span::styled(
                            format!("    {content_line}"),
                            Style::default().fg(Color::DarkGray),
                        )));
                        continue;
                    }
                    
                    // Check for code fence
                    if super::markdown::is_code_fence(content_line) {
                        if in_code_block {
                            // Closing fence
                            lines.push(Line::from(Span::styled(
                                "└──────────────────────────────────────────────",
                                Style::default().fg(Color::DarkGray),
                            )));
                            in_code_block = false;
                        } else {
                            // Opening fence
                            in_code_block = true;
                            let code_lang = super::markdown::extract_code_language(content_line);
                            let lang_display = code_lang.as_deref().unwrap_or("code");
                            lines.push(Line::from(Span::styled(
                                format!("┌─ {lang_display} ───────────────────────────────────────────"),
                                Style::default().fg(Color::DarkGray),
                            )));
                        }
                    } else if in_code_block {
                        // Inside code block - render with simple prefix
                        lines.push(Line::from(Span::styled(
                            format!("  {content_line}"),
                            Style::default().fg(Color::Green),
                        )));
                    } else {
                        // Regular markdown line
                        let rendered_lines = super::markdown::render_markdown_to_lines(content_line);
                        lines.extend(rendered_lines);
                    }
                }
            }
        }
    }

    let has_focus = matches!(app.focus, crate::app::Focus::History);
    let border_color = if has_focus {
        Color::Yellow  // Bright yellow when focused
    } else {
        Color::DarkGray  // Dim gray when unfocused
    };

    let title = " Conversation ";  // No [Focused] text

    // Calculate scroll position - if scroll_offset is very large, 
    // we want to show the bottom content
    // We must account for line wrapping to calculate the true visual height
    let available_width = area.width.saturating_sub(2) as usize; // Subtract borders
    let mut total_visual_lines = 0;
    
    for line in &lines {
        let line_width = line.width();
        if line_width == 0 {
            total_visual_lines += 1;
        } else {
            // Ceiling division: (width + available - 1) / available
            total_visual_lines += (line_width + available_width - 1) / available_width;
        }
    }

    let visible_height = area.height.saturating_sub(2) as usize; // Subtract borders
    let max_scroll = total_visual_lines.saturating_sub(visible_height);
    let actual_scroll = app.scroll_offset.min(max_scroll);
    
    // Sync the actual scroll back to the app state
    // This ensures that when the user tries to scroll up/down, they are starting
    // from the visual position, not the logical position (which might be usize::MAX)
    if app.scroll_offset != actual_scroll {
        app.scroll_offset = actual_scroll;
    }

    let chat_history = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(border_color)),
        )
        .wrap(Wrap { trim: false })
        .scroll((u16::try_from(actual_scroll).unwrap_or(u16::MAX), 0));

    frame.render_widget(chat_history, area);
}

pub fn render_input_field(frame: &mut Frame, app: &App, area: Rect) {
    let input_text = if app.input_buffer.is_empty() {
        "Type your message... (Press Enter to send, Tab to switch focus)"
    } else {
        &app.input_buffer
    };

    let input_style = if app.input_buffer.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let has_focus = matches!(app.focus, crate::app::Focus::Input);
    let border_color = if has_focus {
        Color::Cyan  // Bright cyan when focused
    } else {
        Color::DarkGray  // Dim gray when unfocused
    };

    let title = " Input ";  // No [Focused] text

    let input = Paragraph::new(input_text)
        .style(input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(border_color)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(input, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_color_logic() {
        let mut app = App::new();
        app.context_window_size = 100;
        
        // Test green (< 50%)
        app.messages.clear();
        let pct = app.context_usage_percentage();
        assert!(pct < 50.0);
        
        // Test yellow (50-80%)
        app.messages.push(crate::models::Message::new(
            crate::models::MessageRole::User,
            "test".to_string(),
            60,
        ));
        let pct = app.context_usage_percentage();
        assert!(pct > 50.0 && pct < 80.0);
        
        // Test red (> 80%)
        app.messages.push(crate::models::Message::new(
            crate::models::MessageRole::Assistant,
            "test".to_string(),
            30,
        ));
        let pct = app.context_usage_percentage();
        assert!(pct > 80.0);
    }
}
