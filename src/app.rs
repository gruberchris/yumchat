use crate::models::{ConversationMetadata, Message};

use std::time::Instant;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    Chat,
    ConversationList,
    Settings,
}

#[derive(Debug)]
pub struct App {
    pub mode: AppMode,
    pub should_quit: bool,
    #[allow(dead_code)]
    pub current_conversation: Option<ConversationMetadata>,
    pub messages: Vec<Message>,
    pub input_buffer: String,
    pub scroll_offset: usize,
    pub context_window_size: usize,
    pub show_help: bool,
    pub is_loading: bool,
    pub show_info: bool,
    pub exit_pending: bool,
    pub current_model: String,
    
    // TPS tracking
    pub tokens_per_second: f64,
    pub generation_start_time: Option<Instant>,
    pub generation_token_count: usize,
    
    // UI toggles
    pub show_thinking: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: AppMode::Chat,
            should_quit: false,
            current_conversation: None,
            messages: Vec::new(),
            input_buffer: String::new(),
            scroll_offset: 0,
            context_window_size: 4096,
            show_help: false,
            is_loading: false,
            show_info: false,
            exit_pending: false,
            current_model: "qwen3:4b".to_string(),
            tokens_per_second: 0.0,
            generation_start_time: None,
            generation_token_count: 0,
            show_thinking: false,
        }
    }

    pub const fn quit(&mut self) {
        self.should_quit = true;
    }

    pub const fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub const fn toggle_info(&mut self) {
        self.show_info = !self.show_info;
    }
    
    pub const fn toggle_thinking(&mut self) {
        self.show_thinking = !self.show_thinking;
    }

    pub const fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_add(amount);
    }

    pub const fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub const fn scroll_to_bottom(&mut self) {
        // Set to a very large number to ensure we scroll to the actual bottom
        // The rendering code will clamp this to the maximum possible scroll
        self.scroll_offset = usize::MAX;
    }

    /// Calculate the total number of lines needed to render all messages
    #[allow(dead_code)]
    fn calculate_total_lines(&self) -> usize {
        if self.messages.is_empty() {
            return 1; // Just the "no messages" line
        }
        
        let mut total = 0;
        for message in &self.messages {
            total += 1; // Empty line before
            total += 1; // Role header (## User or ## Assistant)
            total += 1; // Empty line after header
            // Count content lines
            total += message.content.lines().count().max(1); // At least 1 even if empty
        }
        total
    }

    #[allow(dead_code)]
    pub const fn switch_mode(&mut self, mode: AppMode) {
        self.mode = mode;
    }

    pub fn total_tokens_used(&self) -> usize {
        self.messages.iter().map(|m| m.tokens).sum()
    }

    pub fn context_usage_percentage(&self) -> f64 {
        crate::tokens::context_usage_percentage(
            self.total_tokens_used(),
            self.context_window_size,
        )
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MessageRole;

    #[test]
    fn test_app_new() {
        let app = App::new();
        assert_eq!(app.mode, AppMode::Chat);
        assert!(!app.should_quit);
        assert_eq!(app.context_window_size, 4096);
    }

    #[test]
    fn test_app_quit() {
        let mut app = App::new();
        app.quit();
        assert!(app.should_quit);
    }

    #[test]
    fn test_app_switch_mode() {
        let mut app = App::new();
        app.switch_mode(AppMode::Settings);
        assert_eq!(app.mode, AppMode::Settings);
    }

    #[test]
    fn test_total_tokens_used() {
        let mut app = App::new();
        app.messages
            .push(Message::new(MessageRole::User, "Hello".to_string(), 10));
        app.messages
            .push(Message::new(MessageRole::Assistant, "Hi".to_string(), 5));
        assert_eq!(app.total_tokens_used(), 15);
    }

    #[test]
    fn test_context_usage_percentage() {
        let mut app = App::new();
        app.context_window_size = 100;
        app.messages
            .push(Message::new(MessageRole::User, "Test".to_string(), 50));
        assert!((app.context_usage_percentage() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_toggle_help() {
        let mut app = App::new();
        assert!(!app.show_help);
        app.toggle_help();
        assert!(app.show_help);
        app.toggle_help();
        assert!(!app.show_help);
    }

    #[test]
    fn test_scroll_up() {
        let mut app = App::new();
        app.scroll_offset = 10;
        app.scroll_up(3);
        assert_eq!(app.scroll_offset, 7);
        app.scroll_up(10);
        assert_eq!(app.scroll_offset, 0); // saturating_sub
    }

    #[test]
    fn test_scroll_down() {
        let mut app = App::new();
        for i in 0..10 {
            app.messages.push(Message::new(
                MessageRole::User,
                format!("msg {i}"),
                10,
            ));
        }
        app.scroll_down(3);
        assert_eq!(app.scroll_offset, 3);
        
        // Test that we can scroll past the calculated total lines (because of potential wrapping)
        // The clamping happens in the UI layer now
        app.scroll_down(100);
        assert_eq!(app.scroll_offset, 103);
    }

    #[test]
    fn test_scroll_to_top() {
        let mut app = App::new();
        app.scroll_offset = 10;
        app.scroll_to_top();
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_to_bottom() {
        let mut app = App::new();
        for i in 0..10 {
            app.messages.push(Message::new(
                MessageRole::User,
                format!("msg {i}"),
                10,
            ));
        }
        app.scroll_to_bottom();
        // Should scroll to show bottom content
        assert!(app.scroll_offset > 0);
    }

    #[test]
    fn test_calculate_total_lines() {
        let mut app = App::new();
        
        // Empty should be 1
        assert_eq!(app.calculate_total_lines(), 1);
        
        // Single line message
        app.messages.push(Message::new(
            MessageRole::User,
            "Hello".to_string(),
            10,
        ));
        // 1 (empty) + 1 (## User) + 1 (empty) + 1 (content) = 4
        assert_eq!(app.calculate_total_lines(), 4);
        
        // Multi-line message
        app.messages.push(Message::new(
            MessageRole::Assistant,
            "Line 1\nLine 2\nLine 3".to_string(),
            10,
        ));
        // Previous 4 + 1 (empty) + 1 (## Assistant) + 1 (empty) + 3 (content) = 10
        assert_eq!(app.calculate_total_lines(), 10);
    }
}
