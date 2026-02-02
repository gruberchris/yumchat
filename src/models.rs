use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationMetadata {
    pub id: Uuid,
    pub summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub total_tokens: usize,
}

#[allow(dead_code)]
impl ConversationMetadata {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            summary: None,
            created_at: now,
            updated_at: now,
            total_tokens: 0,
        }
    }

    pub fn update_tokens(&mut self, tokens: usize) {
        self.total_tokens += tokens;
        self.updated_at = Utc::now();
    }

    pub fn set_summary(&mut self, summary: String) {
        self.summary = Some(summary);
        self.updated_at = Utc::now();
    }
}

impl Default for ConversationMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub tokens: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

#[allow(dead_code)]
impl Message {
    pub const fn new(role: MessageRole, content: String, tokens: usize) -> Self {
        Self {
            role,
            content,
            tokens,
        }
    }

    pub fn new_with_token_count(role: MessageRole, content: String) -> Self {
        let role_str = match role {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        };
        let tokens = crate::tokens::count_message_tokens(role_str, &content);
        Self {
            role,
            content,
            tokens,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ollama_url: String,
    pub default_model: String,
    #[serde(default = "default_timeout")]
    pub request_timeout: u64,
    pub theme: ThemeConfig,
}

const fn default_timeout() -> u64 {
    600
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ollama_url: "http://localhost:11434".to_string(),
            default_model: "qwen3:4b".to_string(),
            request_timeout: default_timeout(),
            theme: ThemeConfig::default(),
        }
    }
}

#[allow(dead_code, clippy::struct_field_names)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub user_message_color: String,
    pub assistant_message_color: String,
    pub border_color: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            user_message_color: "blue".to_string(),
            assistant_message_color: "green".to_string(),
            border_color: "cyan".to_string(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub context_window_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_metadata_new() {
        let meta = ConversationMetadata::new();
        assert!(meta.summary.is_none());
        assert_eq!(meta.total_tokens, 0);
    }

    #[test]
    fn test_conversation_metadata_update_tokens() {
        let mut meta = ConversationMetadata::new();
        meta.update_tokens(100);
        assert_eq!(meta.total_tokens, 100);
        meta.update_tokens(50);
        assert_eq!(meta.total_tokens, 150);
    }

    #[test]
    fn test_conversation_metadata_set_summary() {
        let mut meta = ConversationMetadata::new();
        meta.set_summary("Test summary".to_string());
        assert_eq!(meta.summary, Some("Test summary".to_string()));
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::new(MessageRole::User, "Hello".to_string(), 10);
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello");
        assert_eq!(msg.tokens, 10);
    }

    #[test]
    fn test_message_with_token_count() {
        let msg = Message::new_with_token_count(MessageRole::User, "Hello world".to_string());
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello world");
        assert!(msg.tokens > 0);
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.ollama_url, "http://localhost:11434");
        assert_eq!(config.default_model, "qwen3:4b");
    }
}
