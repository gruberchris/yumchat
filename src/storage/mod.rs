// Storage layer for conversations and config

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::models::{ConversationMetadata, Message};

#[allow(dead_code)]
pub struct Storage {
    config_dir: PathBuf,
    chats_dir: PathBuf,
}

#[allow(dead_code)]
impl Storage {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("yumchat");

        let chats_dir = config_dir.join("chats");

        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
        fs::create_dir_all(&chats_dir).context("Failed to create chats directory")?;

        Ok(Self {
            config_dir,
            chats_dir,
        })
    }

    pub fn get_conversation_path(&self, id: &Uuid) -> PathBuf {
        self.chats_dir.join(format!("{id}.md"))
    }

    pub fn get_metadata_path(&self, id: &Uuid) -> PathBuf {
        self.chats_dir.join(format!("{id}_meta.json"))
    }

    pub fn save_conversation(&self, id: &Uuid, messages: &[Message]) -> Result<()> {
        let path = self.get_conversation_path(id);
        let mut content = String::new();

        for message in messages {
            let role = match message.role {
                crate::models::MessageRole::User => "User",
                crate::models::MessageRole::Assistant => "Assistant",
            };
            content.push_str("## ");
            content.push_str(role);
            content.push_str("\n\n");
            content.push_str(&message.content);
            content.push_str("\n\n");
        }

        fs::write(&path, content).context("Failed to write conversation file")?;

        Ok(())
    }

    pub fn load_conversation(&self, id: &Uuid) -> Result<Vec<Message>> {
        let path = self.get_conversation_path(id);

        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path).context("Failed to read conversation file")?;

        let messages = Self::parse_conversation(&content);
        Ok(messages)
    }

    fn parse_conversation(content: &str) -> Vec<Message> {
        let mut messages = Vec::new();
        let sections: Vec<&str> = content.split("## ").collect();

        for section in sections.iter().skip(1) {
            if let Some((role_line, msg_content)) = section.split_once('\n') {
                let role = if role_line.trim() == "User" {
                    crate::models::MessageRole::User
                } else {
                    crate::models::MessageRole::Assistant
                };

                let msg_content = msg_content.trim().to_string();
                // Token count will be calculated properly in token counter
                messages.push(Message::new(role, msg_content, 0));
            }
        }

        messages
    }

    pub fn save_metadata(&self, metadata: &ConversationMetadata) -> Result<()> {
        let path = self.get_metadata_path(&metadata.id);
        let content =
            serde_json::to_string_pretty(metadata).context("Failed to serialize metadata")?;

        fs::write(&path, content).context("Failed to write metadata file")?;

        Ok(())
    }

    pub fn load_metadata(&self, id: &Uuid) -> Result<ConversationMetadata> {
        let path = self.get_metadata_path(id);

        if !path.exists() {
            anyhow::bail!("Metadata file not found");
        }

        let content = fs::read_to_string(&path).context("Failed to read metadata file")?;

        let metadata: ConversationMetadata =
            serde_json::from_str(&content).context("Failed to parse metadata file")?;

        Ok(metadata)
    }

    pub fn list_conversations(&self) -> Result<Vec<ConversationMetadata>> {
        let mut conversations = Vec::new();

        if !self.chats_dir.exists() {
            return Ok(conversations);
        }

        for entry in fs::read_dir(&self.chats_dir).context("Failed to read chats directory")? {
            let entry = entry?;
            let path = entry.path();

            if let Some(filename) = path.file_name() {
                let filename = filename.to_string_lossy();
                if filename.ends_with("_meta.json") {
                    let content = fs::read_to_string(&path)?;
                    if let Ok(metadata) = serde_json::from_str::<ConversationMetadata>(&content) {
                        conversations.push(metadata);
                    }
                }
            }
        }

        // Sort by updated_at, most recent first
        conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(conversations)
    }

    pub fn delete_conversation(&self, id: &Uuid) -> Result<()> {
        let conv_path = self.get_conversation_path(id);
        let meta_path = self.get_metadata_path(id);

        if conv_path.exists() {
            fs::remove_file(conv_path).context("Failed to delete conversation file")?;
        }

        if meta_path.exists() {
            fs::remove_file(meta_path).context("Failed to delete metadata file")?;
        }

        Ok(())
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new().expect("Failed to create storage")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_storage() -> (TempDir, Storage) {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(".config/yumchat");
        let chats_dir = config_dir.join("chats");

        fs::create_dir_all(&chats_dir).unwrap();

        let storage = Storage {
            config_dir,
            chats_dir,
        };

        (temp_dir, storage)
    }

    #[test]
    fn test_storage_creation() {
        // Test storage creation in a controlled temporary environment
        let temp_dir = TempDir::new().unwrap();
        
        // Temporarily override HOME for this test to avoid conflicts
        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", temp_dir.path());
        
        let result = Storage::new();
        
        // Restore original HOME
        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
        
        assert!(result.is_ok(), "Storage creation should succeed in temp directory");
    }

    #[test]
    fn test_save_and_load_conversation() {
        let (_temp, storage) = setup_test_storage();
        let id = Uuid::new_v4();

        let messages = vec![
            Message::new(crate::models::MessageRole::User, "Hello".to_string(), 10),
            Message::new(
                crate::models::MessageRole::Assistant,
                "Hi there!".to_string(),
                15,
            ),
        ];

        let result = storage.save_conversation(&id, &messages);
        assert!(result.is_ok());

        let loaded = storage.load_conversation(&id);
        assert!(loaded.is_ok());
        let loaded_messages = loaded.unwrap();
        assert_eq!(loaded_messages.len(), 2);
        assert_eq!(loaded_messages[0].content, "Hello");
        assert_eq!(loaded_messages[1].content, "Hi there!");
    }

    #[test]
    fn test_save_and_load_metadata() {
        let (_temp, storage) = setup_test_storage();
        let mut metadata = ConversationMetadata::new();
        metadata.set_summary("Test conversation".to_string());
        metadata.update_tokens(100);

        let result = storage.save_metadata(&metadata);
        assert!(result.is_ok());

        let loaded = storage.load_metadata(&metadata.id);
        assert!(loaded.is_ok());
        let loaded_metadata = loaded.unwrap();
        assert_eq!(loaded_metadata.id, metadata.id);
        assert_eq!(
            loaded_metadata.summary,
            Some("Test conversation".to_string())
        );
        assert_eq!(loaded_metadata.total_tokens, 100);
    }

    #[test]
    fn test_list_conversations() {
        let (_temp, storage) = setup_test_storage();

        let mut meta1 = ConversationMetadata::new();
        meta1.set_summary("First".to_string());
        storage.save_metadata(&meta1).unwrap();

        let mut meta2 = ConversationMetadata::new();
        meta2.set_summary("Second".to_string());
        storage.save_metadata(&meta2).unwrap();

        let conversations = storage.list_conversations();
        assert!(conversations.is_ok());
        let conversations = conversations.unwrap();
        assert_eq!(conversations.len(), 2);
    }

    #[test]
    fn test_delete_conversation() {
        let (_temp, storage) = setup_test_storage();
        let id = Uuid::new_v4();

        let messages = vec![Message::new(
            crate::models::MessageRole::User,
            "Test".to_string(),
            5,
        )];

        storage.save_conversation(&id, &messages).unwrap();
        let mut metadata = ConversationMetadata::new();
        metadata.id = id;
        storage.save_metadata(&metadata).unwrap();

        let result = storage.delete_conversation(&id);
        assert!(result.is_ok());

        let loaded = storage.load_conversation(&id);
        assert!(loaded.is_ok());
        assert!(loaded.unwrap().is_empty());
    }

    #[test]
    fn test_parse_conversation() {
        let content = "## User\n\nHello world\n\n## Assistant\n\nHi there!\n\n";

        let messages = Storage::parse_conversation(content);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].content, "Hello world");
        assert_eq!(messages[1].content, "Hi there!");
    }

    #[test]
    fn test_conversation_paths() {
        let (_temp, storage) = setup_test_storage();
        let id = Uuid::new_v4();

        let conv_path = storage.get_conversation_path(&id);
        assert!(conv_path.to_string_lossy().contains(&id.to_string()));
        assert!(conv_path.to_string_lossy().ends_with(".md"));

        let meta_path = storage.get_metadata_path(&id);
        assert!(meta_path.to_string_lossy().contains(&id.to_string()));
        assert!(meta_path.to_string_lossy().ends_with("_meta.json"));
    }
}
