// Configuration management

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::models::{AppConfig, ModelInfo};

pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .join("yumchat");

    fs::create_dir_all(&config_dir).context("Failed to create config directory")?;

    Ok(config_dir)
}

#[allow(dead_code)]
pub fn get_config_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("config.toml"))
}

#[allow(dead_code)]
pub fn get_models_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("models.json"))
}

#[allow(dead_code)]
pub fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        let default_config = AppConfig::default();
        save_config(&default_config)?;
        return Ok(default_config);
    }

    let contents = fs::read_to_string(&config_path).context("Failed to read config file")?;

    let config: AppConfig = toml::from_str(&contents).context("Failed to parse config file")?;

    Ok(config)
}

#[allow(dead_code)]
pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = get_config_path()?;

    let contents = toml::to_string_pretty(config).context("Failed to serialize config")?;

    fs::write(&config_path, contents).context("Failed to write config file")?;

    Ok(())
}

#[allow(dead_code)]
pub fn load_models() -> Result<Vec<ModelInfo>> {
    let models_path = get_models_path()?;

    if !models_path.exists() {
        // Return default models
        let default_models = vec![
            ModelInfo {
                name: "llama2".to_string(),
                context_window_size: 4096,
            },
            ModelInfo {
                name: "mistral".to_string(),
                context_window_size: 8192,
            },
        ];
        save_models(&default_models)?;
        return Ok(default_models);
    }

    let contents = fs::read_to_string(&models_path).context("Failed to read models file")?;

    let models: Vec<ModelInfo> =
        serde_json::from_str(&contents).context("Failed to parse models file")?;

    Ok(models)
}

#[allow(dead_code)]
pub fn save_models(models: &[ModelInfo]) -> Result<()> {
    let models_path = get_models_path()?;

    let contents = serde_json::to_string_pretty(models).context("Failed to serialize models")?;

    fs::write(&models_path, contents).context("Failed to write models file")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_test_env() -> TempDir {
        TempDir::new().unwrap()
    }

    #[test]
    fn test_load_config_creates_default() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let temp_dir = setup_test_env();
        
        // Save and restore HOME for test isolation
        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", temp_dir.path());
        
        let config = load_config();
        
        // Restore HOME immediately after config is loaded
        if let Some(home) = &original_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
        
        // Check result and provide helpful error message
        if let Some(home) = &original_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
        
        // Check result and provide helpful error message
        assert!(
            config.is_ok(),
            "Config loading failed: {:?}. HOME was set to: {:?}",
            config.as_ref().err(),
            temp_dir.path()
        );
        let config = config.unwrap();
        assert_eq!(config.ollama_url, "http://localhost:11434");
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = setup_test_env();
        let config_dir = temp_dir.path().join(".config/yumchat");
        fs::create_dir_all(&config_dir).unwrap();

        let config = AppConfig {
            ollama_url: "http://custom:8080".to_string(),
            ..Default::default()
        };

        let config_path = config_dir.join("config.toml");
        let contents = toml::to_string(&config).unwrap();
        fs::write(&config_path, contents).unwrap();

        let loaded_contents = fs::read_to_string(&config_path).unwrap();
        let loaded_config: AppConfig = toml::from_str(&loaded_contents).unwrap();

        assert_eq!(loaded_config.ollama_url, "http://custom:8080");
    }

    #[test]
    fn test_load_models_creates_default() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let temp_dir = setup_test_env();
        
        // Save and restore HOME for test isolation
        let original_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", temp_dir.path());

        let models = load_models();
        
        // Restore HOME immediately after models are loaded
        if let Some(home) = &original_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
        
        // Check result with helpful error message
        assert!(
            models.is_ok(),
            "Models loading failed: {:?}. HOME was set to: {:?}",
            models.as_ref().err(),
            temp_dir.path()
        );
        let models = models.unwrap();
        assert!(!models.is_empty());
    }

    #[test]
    fn test_save_and_load_models() {
        let temp_dir = setup_test_env();
        let config_dir = temp_dir.path().join(".config/yumchat");
        fs::create_dir_all(&config_dir).unwrap();

        let models = vec![ModelInfo {
            name: "test-model".to_string(),
            context_window_size: 16384,
        }];

        let models_path = config_dir.join("models.json");
        let contents = serde_json::to_string(&models).unwrap();
        fs::write(&models_path, contents).unwrap();

        let loaded_contents = fs::read_to_string(&models_path).unwrap();
        let loaded_models: Vec<ModelInfo> = serde_json::from_str(&loaded_contents).unwrap();

        assert_eq!(loaded_models.len(), 1);
        assert_eq!(loaded_models[0].name, "test-model");
        assert_eq!(loaded_models[0].context_window_size, 16384);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let serialized = toml::to_string(&config);
        assert!(serialized.is_ok());

        let deserialized: Result<AppConfig, _> = toml::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_models_serialization() {
        let models = vec![ModelInfo {
            name: "model1".to_string(),
            context_window_size: 2048,
        }];

        let serialized = serde_json::to_string(&models);
        assert!(serialized.is_ok());

        let deserialized: Result<Vec<ModelInfo>, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
    }
}
