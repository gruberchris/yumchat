// Simple markdown rendering for terminal display

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// Convert markdown text to ratatui Lines with styling
pub fn render_markdown_to_lines(markdown: &str) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    
    for line in markdown.lines() {
        lines.push(render_markdown_line(line));
    }
    
    lines
}

/// Check if a line is a markdown table row
pub fn is_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.matches('|').count() >= 2
}

/// Check if a line is a table separator (|---|---|)
pub fn is_table_separator(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return false;
    }
    
    // Check if it's mostly dashes and pipes
    trimmed.chars().all(|c| c == '|' || c == '-' || c == ' ' || c == ':')
}

/// Render a markdown table row - simplified for better readability
fn render_table_row(line: &str) -> Line<'static> {
    let trimmed = line.trim();
    // Remove leading and trailing pipes
    let content = trimmed.trim_start_matches('|').trim_end_matches('|');
    
    // Split by pipe and format cells
    let cells: Vec<&str> = content.split('|').map(str::trim).collect();
    
    // For better readability in terminal, just display cells with clear spacing
    // Instead of trying to align columns (which is hard without knowing column widths),
    // display as: Cell1  |  Cell2  |  Cell3
    let formatted = cells.join(" | ");
    
    Line::from(Span::styled(
        format!("  {formatted}"),
        Style::default().fg(Color::Cyan),
    ))
}

/// Render a single line of markdown with basic styling
#[allow(clippy::too_many_lines)]
fn render_markdown_line(line: &str) -> Line<'static> {
    // Check for table rows first
    if is_table_separator(line) {
        // Skip separator lines - they're just visual noise in terminals
        return Line::from("");
    }
    
    if is_table_row(line) {
        return render_table_row(line);
    }
    
    let mut spans = Vec::new();
    let mut current_text = String::new();
    let mut chars = line.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            // Bold: **text**
            '*' if chars.peek() == Some(&'*') => {
                if !current_text.is_empty() {
                    spans.push(Span::raw(current_text.clone()));
                    current_text.clear();
                }
                chars.next(); // consume second *
                
                // Find closing **
                let mut bold_text = String::new();
                let mut found_close = false;
                while let Some(ch) = chars.next() {
                    if ch == '*' && chars.peek() == Some(&'*') {
                        chars.next(); // consume second *
                        found_close = true;
                        break;
                    }
                    bold_text.push(ch);
                }
                
                if found_close {
                    spans.push(Span::styled(
                        bold_text,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    // No closing **, treat as literal
                    current_text.push_str("**");
                    current_text.push_str(&bold_text);
                }
            }
            // Inline code: `code`
            '`' => {
                if !current_text.is_empty() {
                    spans.push(Span::raw(current_text.clone()));
                    current_text.clear();
                }
                
                // Find closing `
                let mut code_text = String::new();
                let mut found_close = false;
                for ch in chars.by_ref() {
                    if ch == '`' {
                        found_close = true;
                        break;
                    }
                    code_text.push(ch);
                }
                
                if found_close {
                    spans.push(Span::styled(
                        code_text,
                        Style::default().fg(Color::Magenta),
                    ));
                } else {
                    // No closing `, treat as literal
                    current_text.push('`');
                    current_text.push_str(&code_text);
                }
            }
            // Headers: # ## ###
            '#' if current_text.is_empty() => {
                let mut level = 1;
                while chars.peek() == Some(&'#') {
                    level += 1;
                    chars.next();
                }
                
                // Skip space after #
                if chars.peek() == Some(&' ') {
                    chars.next();
                }
                
                // Rest of line is header
                let header_text: String = chars.collect();
                let color = match level {
                    1 => Color::Yellow,
                    2 => Color::Cyan,
                    _ => Color::Blue,
                };
                
                return Line::from(Span::styled(
                    header_text.trim().to_string(),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ));
            }
            // List items: - item or * item
            '-' | '*' if current_text.is_empty() && chars.peek() == Some(&' ') => {
                chars.next(); // consume space
                let rest: String = chars.collect();
                spans.push(Span::styled("• ", Style::default().fg(Color::Cyan)));
                spans.push(Span::raw(rest.trim().to_string()));
                break;
            }
            _ => {
                current_text.push(ch);
            }
        }
    }
    
    if !current_text.is_empty() {
        spans.push(Span::raw(current_text));
    }
    
    if spans.is_empty() {
        Line::from("")
    } else {
        Line::from(spans)
    }
}

/// Detect if a line is a code block fence
pub fn is_code_fence(line: &str) -> bool {
    line.trim().starts_with("```")
}

/// Extract language from code fence
pub fn extract_code_language(line: &str) -> Option<String> {
    line.trim()
        .strip_prefix("```")
        .map(str::trim)
        .filter(|lang| !lang.is_empty())
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_plain_text() {
        let lines = render_markdown_to_lines("Hello world");
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_render_bold_text() {
        let lines = render_markdown_to_lines("This is **bold** text");
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_render_inline_code() {
        let lines = render_markdown_to_lines("Use `println!` macro");
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_render_header() {
        let lines = render_markdown_to_lines("## Header");
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_render_list() {
        let lines = render_markdown_to_lines("- List item");
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_is_code_fence() {
        assert!(is_code_fence("```"));
        assert!(is_code_fence("```python"));
        assert!(is_code_fence("```rust"));
        assert!(!is_code_fence("code"));
    }

    #[test]
    fn test_extract_code_language() {
        assert_eq!(extract_code_language("```python"), Some("python".to_string()));
        assert_eq!(extract_code_language("```rust"), Some("rust".to_string()));
        assert_eq!(extract_code_language("```"), None);
    }

    #[test]
    fn test_is_table_row() {
        assert!(is_table_row("| Col1 | Col2 |"));
        assert!(is_table_row("|A|B|C|"));
        assert!(!is_table_row("Not a table"));
        assert!(!is_table_row("| Only one pipe"));
    }

    #[test]
    fn test_is_table_separator() {
        assert!(is_table_separator("|---|---|"));
        assert!(is_table_separator("| --- | --- |"));
        assert!(is_table_separator("|:---|---:|"));
        assert!(!is_table_separator("| Col1 | Col2 |"));
    }
}
