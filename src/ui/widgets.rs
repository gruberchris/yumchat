use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap, Clear, List, ListItem},
    Frame,
};

use crate::app::{App, AppMode};

pub fn render_model_selector(frame: &mut Frame, app: &mut App, area: Rect) {
    if app.mode != AppMode::ModelSelector {
        return;
    }

    let popup_width = 60;
    let popup_height = 20;
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x: area.x + x,
        y: area.y + y,
        width: popup_width.min(area.width),
        height: popup_height.min(area.height),
    };
    
    // Clear area behind popup
    frame.render_widget(Clear, popup_area);
    
    let items: Vec<ListItem> = app.available_models
        .iter()
        .map(|m| {
            let content = if m == &app.current_model {
                Line::from(vec![
                    Span::styled(format!("* {m}"), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                ])
            } else {
                Line::from(vec![
                   Span::styled(format!("  {m}"), Style::default().fg(Color::White))
                ])
            };
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Select Model (Enter to confirm, Esc to cancel) ")
            .border_style(Style::default().fg(Color::Yellow))
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(list, popup_area, &mut app.model_list_state);
}

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
        Line::from("  Ctrl+I        - Show/hide model info"),
        Line::from("  Ctrl+M        - Switch Model"),
        Line::from("  Ctrl+Q        - Quit application"),
        Line::from("  Ctrl+C        - Quit application"),
        Line::from(""),
        Line::from(Span::styled("Chat:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Enter         - Send message"),
        Line::from("  Tab           - Toggle thinking"),
        Line::from("  Typing        - Auto-targets input"),
        Line::from(""),
        Line::from(Span::styled("Navigation:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Up/Down       - Scroll history"),
        Line::from("  PgUp/PgDn     - Scroll history"),
        Line::from("  Home/End      - Jump to start/end"),
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
    let popup_height = 25;
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

#[allow(clippy::too_many_lines)]
pub fn render_info_window(frame: &mut Frame, app: &App, area: Rect) {
    let tokens_used = app.total_tokens_used();
    let context_window = app.context_window_size;
    let usage_percentage = app.context_usage_percentage();

    // Center popup
    let popup_width = 50;
    let popup_height = 18;
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x: area.x + x,
        y: area.y + y,
        width: popup_width.min(area.width),
        height: popup_height.min(area.height),
    };

    let mut info_text = vec![
        Line::from(Span::styled(
            "Session Information",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("Model: "),
            Span::styled(&app.current_model, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("Family: "),
            Span::styled(
                app.model_details.as_ref().map_or_else(|| "Unknown".to_string(), |d| d.family.clone()), 
                Style::default().fg(Color::White)
            ),
        ]),
        Line::from(vec![
            Span::raw("Params: "),
            Span::styled(
                app.model_details.as_ref().map_or_else(|| "?".to_string(), |d| d.parameter_size.clone()), 
                Style::default().fg(Color::White)
            ),
        ]),
        Line::from(vec![
            Span::raw("Quantization: "),
            Span::styled(
                app.model_details.as_ref().map_or_else(|| "?".to_string(), |d| d.quantization_level.clone()), 
                Style::default().fg(Color::White)
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled("Capabilities:", Style::default().add_modifier(Modifier::BOLD))),
    ];

    if app.model_capabilities.is_empty() {
         info_text.push(Line::from(Span::styled("  Unknown", Style::default().fg(Color::DarkGray))));
    } else {
        for cap in &app.model_capabilities {
             let (symbol, color) = match cap.as_str() {
                 "thinking" => ("🧠", Color::Magenta),
                 "tools" => ("🛠️", Color::Green),
                 "vision" => ("👁️", Color::Blue),
                 "completion" => ("📝", Color::Cyan),
                 "chat" => ("💬", Color::Yellow),
                 _ => ("•", Color::White),
             };
             info_text.push(Line::from(vec![
                 Span::raw("  "),
                 Span::styled(format!("{symbol} {cap}"), Style::default().fg(color))
             ]));
        }
    }
    
    info_text.extend(vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("Tokens Used: "),
            Span::styled(format!("{tokens_used}"), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::raw("Speed: "),
            Span::styled(format!("{:.1} t/s", app.tokens_per_second), Style::default().fg(Color::Magenta)),
        ]),
        Line::from(vec![
            Span::raw("Context Window: "),
            Span::styled(format!("{context_window}"), Style::default().fg(Color::Blue)),
        ]),
        Line::from(vec![
            Span::raw("Usage: "),
            Span::styled(format!("{usage_percentage:.1}%"), Style::default().fg(
                if usage_percentage > 80.0 { Color::Red }
                else if usage_percentage > 50.0 { Color::Yellow }
                else { Color::Green }
            )),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press Ctrl+I to close",
            Style::default().fg(Color::DarkGray),
        )),
    ]);

    let info_paragraph = Paragraph::new(info_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Model Info ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(Clear, popup_area);
    frame.render_widget(info_paragraph, popup_area);
}

pub fn render_bottom_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (text, style) = if app.exit_pending {
        (
            "Press Ctrl+C again to exit, Esc to cancel",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    } else {
        (
            "Ctrl+C: Quit | Ctrl+I: Model Info | Ctrl+H: Help | Tab: Toggle Thoughts",
            Style::default().fg(Color::DarkGray),
        )
    };

    let bar = Paragraph::new(text)
        .alignment(ratatui::layout::Alignment::Center)
        .style(style);

    frame.render_widget(bar, area);
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

    let loading_indicator = if app.is_loading {
        if app.is_thinking {
            " [Thinking...]"
        } else {
            " [Responding...]"
        }
    } else {
        ""
    };
    
    let status_text = format!(
        "{}{} ({:.1}%)",
        app.current_model, loading_indicator, usage_percentage
    );

    let status = Paragraph::new(status_text)
        .alignment(ratatui::layout::Alignment::Right)
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD));

    frame.render_widget(status, area);
}

#[allow(clippy::too_many_lines)]
pub fn render_chat_history(frame: &mut Frame, app: &mut App, area: Rect) {
    let mut lines = Vec::new();

    if app.messages.is_empty() {
        // Render welcome banner at the bottom of the history area
        let welcome_text = vec![
            Line::from(Span::styled(
                "Welcome to YumChat",
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "Your friendly terminal AI assistant",
                Style::default().fg(Color::Cyan),
            )),
        ];

        let welcome_paragraph = Paragraph::new(welcome_text)
            .alignment(ratatui::layout::Alignment::Center);

        // Position it at the bottom of the history area
        let welcome_height = 2;
        let y_pos = area.y + area.height.saturating_sub(welcome_height);
        
        let welcome_area = Rect {
            x: area.x,
            y: y_pos,
            width: area.width,
            height: welcome_height.min(area.height),
        };

        frame.render_widget(welcome_paragraph, welcome_area);
        return;
    } 
    
    for message in &app.messages {
        lines.push(Line::from(""));

        match message.role {
            crate::models::MessageRole::User => {
                for line in message.content.lines() {
                    lines.push(Line::from(vec![
                        Span::styled("> ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::styled(line, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    ]));
                }
            }
            crate::models::MessageRole::Assistant => {
                // Render content with markdown styling
                if message.content.is_empty() {
                // Show a placeholder for empty AI responses (while streaming)
                lines.push(Line::from(Span::styled("...", Style::default().fg(Color::DarkGray))));
            } else {
                let mut in_code_block = false;
                let mut in_thinking = false;
                let mut thinking_header_shown = false;
                
                for content_line in message.content.lines() {
                    let trimmed = content_line.trim();
                    let has_start = trimmed.contains("<thinking>");
                    let has_end = trimmed.contains("</thinking>");
                    
                    if has_start {
                        in_thinking = true;
                        thinking_header_shown = false;
                        if app.show_thinking {
                             lines.push(Line::from(Span::styled(
                                "  <thinking>", 
                                Style::default().fg(Color::DarkGray)
                            )));
                        }
                    }
                    
                    if in_thinking {
                        // Strip tags to get actual content if any
                        let clean_content = content_line.replace("<thinking>", "").replace("</thinking>", "");
                        let clean_trimmed = clean_content.trim();
                        
                        if !clean_trimmed.is_empty() {
                            if app.show_thinking {
                                lines.push(Line::from(Span::styled(
                                    format!("        {clean_trimmed}"), 
                                    Style::default().fg(Color::DarkGray),
                                )));
                            } else if !thinking_header_shown {
                                if app.is_loading && app.is_thinking {
                                    // Animation based on time
                                    let tick = app.generation_start_time.map_or(0, |start| (start.elapsed().as_millis() / 100) as usize);
                                    
                                    let frames = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
                                    let frame = frames[tick % frames.len()];
                                    let color = match (tick / 8) % 3 {
                                        0 => Color::Magenta,
                                        1 => Color::Cyan,
                                        _ => Color::Blue,
                                    };
                                    
                                    lines.push(Line::from(vec![
                                        Span::styled("    | AI assistant thoughts (Hidden)   ", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
                                        Span::styled(format!("{frame}  "), Style::default().fg(color)),
                                        Span::styled("Thinking", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                                        Span::styled(format!("  {frame}"), Style::default().fg(color)),
                                    ]));
                                } else {
                                    lines.push(Line::from(Span::styled(
                                        "    | AI assistant thoughts (Hidden) - Press Tab to show", 
                                        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                                    )));
                                }
                                thinking_header_shown = true;
                            }
                        }
                    } else {
                        // Regular content processing
                        if trimmed == "[Response stream aborted by user]" {
                            lines.push(Line::from(Span::styled(
                                "[Response stream aborted by user]",
                                Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC),
                            )));
                            continue;
                        }
                        
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
                            if content_line.is_empty() {
                                lines.push(Line::from(""));
                            } else {
                                let rendered_lines = super::markdown::render_markdown_to_lines(content_line);
                                lines.extend(rendered_lines);
                            }
                        }
                    }
                    
                    if has_end {
                        in_thinking = false;
                        if app.show_thinking {
                             lines.push(Line::from(Span::styled(
                                "  </thinking>", 
                                Style::default().fg(Color::DarkGray)
                            )));
                        }
                        // Add blank line after thinking block
                        lines.push(Line::from(""));
                    }
                }
                
                // Add thinking animation if currently thinking at the end of the message (visible mode)
                if app.is_loading && app.is_thinking && in_thinking && app.show_thinking {
                    // Animation based on time
                    let tick = app.generation_start_time.map_or(0, |start| (start.elapsed().as_millis() / 100) as usize);
                    
                    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                    let frame = frames[tick % frames.len()];
                    
                    lines.push(Line::from(Span::styled(
                        format!("        {frame} Thinking..."), 
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
        }
    }
    }
    
    // Calculate scroll position - if scroll_offset is very large, 
    // we want to show the bottom content
    // We must account for line wrapping to calculate the true visual height
    // No borders on history anymore, so use full width
    let available_width = area.width as usize; 
    let mut total_visual_lines = 0;
    
    for line in &lines {
        let line_width = line.width();
        if line_width == 0 {
            total_visual_lines += 1;
        } else {
            // Ceiling division: (width + available - 1) / available
            total_visual_lines += line_width.div_ceil(available_width);
        }
    }

    // No borders, so full height visible
    let visible_height = area.height as usize;
    let max_scroll = total_visual_lines.saturating_sub(visible_height);
    let actual_scroll = app.scroll_offset.min(max_scroll);
    
    // Sync the actual scroll back to the app state
    if app.scroll_offset != actual_scroll {
        app.scroll_offset = actual_scroll;
    }

    let chat_history = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((u16::try_from(actual_scroll).unwrap_or(u16::MAX), 0));

    frame.render_widget(chat_history, area);
}

pub fn render_input_field(frame: &mut Frame, app: &App, area: Rect) {
    let input_text = if app.input_buffer.is_empty() {
        "Type your message..."
    } else {
        &app.input_buffer
    };

    let input_style = if app.input_buffer.is_empty() {
        // Higher contrast for placeholder
        Style::default().fg(Color::Gray)
    } else {
        // Bright/Bold for input text - Match border color (Cyan)
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    };

    // Keep border for input to make it distinct
    let input = Paragraph::new(input_text)
        .style(input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
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
