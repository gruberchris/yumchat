// Ollama API client

use anyhow::{Context, Result};
use futures::stream::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct OllamaClient {
    base_url: String,
    client: Client,
}

#[derive(Debug, Serialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub stream: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GenerateResponse {
    #[serde(default)]
    pub response: String,
    #[serde(default)]
    pub thinking: String,
    pub done: bool,
    #[serde(default)]
    pub context: Vec<i32>,
}

impl GenerateResponse {
    /// Get the text content (prioritize response over thinking)
    #[allow(dead_code)]
    pub fn get_text(&self) -> &str {
        if self.response.is_empty() {
            &self.thinking
        } else {
            &self.response
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct TagsResponse {
    pub models: Vec<ModelInfo>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct ShowResponse {
    #[serde(default)]
    pub modelfile: String,
    #[serde(default)]
    pub parameters: String,
    #[serde(default)]
    pub template: String,
    #[serde(default)]
    pub details: Option<ModelDetails>,
    #[serde(default)]
    pub model_info: std::collections::HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModelDetails {
    #[serde(default)]
    pub parent_model: String,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub family: String,
    #[serde(default)]
    pub families: Vec<String>,
    #[serde(default)]
    pub parameter_size: String,
    #[serde(default)]
    pub quantization_level: String,
}

#[allow(dead_code)]
impl OllamaClient {
    pub fn new(base_url: String, request_timeout: u64) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(request_timeout))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { base_url, client })
    }

    pub fn with_default_url() -> Result<Self> {
        Self::new("http://localhost:11434".to_string(), 600)
    }

    #[allow(dead_code)]
    pub async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse> {
        let url = format!("{}/api/generate", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send generate request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {status}: {text}");
        }

        let result = response
            .json::<GenerateResponse>()
            .await
            .context("Failed to parse generate response")?;

        Ok(result)
    }

    /// Stream the generate response line by line
    pub async fn generate_stream(
        &self,
        request: GenerateRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<GenerateResponse>> + Send>>> {
        let url = format!("{}/api/generate", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send generate request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {status}: {text}");
        }

        // Use a stateful stream that buffers incomplete lines
        let stream = futures::stream::unfold(
            (response.bytes_stream(), Vec::new()),
            |(mut byte_stream, mut buffer)| async move {
                loop {
                    // Try to find a newline in the buffer
                    if let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                        // Extract the line including the newline
                        let mut line = buffer.split_off(pos + 1);
                        // Swap buffer and line so buffer has the rest and line has the line
                        std::mem::swap(&mut buffer, &mut line);
                        // Now 'line' has the bytes up to newline, 'buffer' has the rest

                        let text = String::from_utf8_lossy(&line);
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            let result = serde_json::from_str::<GenerateResponse>(trimmed)
                                .with_context(|| "Failed to parse streaming response");
                            return Some((result, (byte_stream, buffer)));
                        }
                        // If empty line, loop again to get next line or more bytes
                        continue;
                    }

                    // Try to parse the entire buffer as a complete JSON object
                    // This handles cases where the last chunk doesn't end with a newline
                    // e.g. {"done":true}
                    if !buffer.is_empty() {
                         let text = String::from_utf8_lossy(&buffer);
                         let trimmed = text.trim();
                         if !trimmed.is_empty() {
                             if let Ok(result) = serde_json::from_str::<GenerateResponse>(trimmed) {
                                 // Success! We parsed the whole buffer
                                 buffer.clear();
                                 return Some((Ok(result), (byte_stream, buffer)));
                             }
                         }
                    }

                    // No newline found and not a complete object, need more bytes
                    match byte_stream.next().await {
                        Some(Ok(bytes)) => {
                            buffer.extend_from_slice(&bytes);
                            // Loop back to check for newline
                        }
                        Some(Err(e)) => {
                            return Some((Err(anyhow::anyhow!("Stream error: {e}")), (byte_stream, buffer)));
                        }
                        None => {
                            // End of stream
                            if !buffer.is_empty() {
                                // Process remaining buffer
                                let text = String::from_utf8_lossy(&buffer);
                                let trimmed = text.trim();
                                if !trimmed.is_empty() {
                                    let result = serde_json::from_str::<GenerateResponse>(trimmed)
                                        .with_context(|| "Failed to parse final streaming response");
                                    // Clear buffer to end loop next time
                                    buffer.clear();
                                    return Some((result, (byte_stream, buffer)));
                                }
                            }
                            return None;
                        }
                    }
                }
            },
        );

        Ok(Box::pin(stream))
    }

    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let url = format!("{}/api/tags", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send tags request")?;

        if !response.status().is_success() {
            let status = response.status();
            anyhow::bail!("Failed to list models: {status}");
        }

        let result = response
            .json::<TagsResponse>()
            .await
            .context("Failed to parse tags response")?;

        Ok(result.models)
    }

    #[allow(dead_code)]
    pub async fn show_model(&self, model_name: &str) -> Result<ShowResponse> {
        let url = format!("{}/api/show", self.base_url);

        let request = serde_json::json!({
            "name": model_name
        });

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send show request")?;

        if !response.status().is_success() {
            let status = response.status();
            anyhow::bail!("Failed to show model: {status}");
        }

        let result = response
            .json::<ShowResponse>()
            .await
            .context("Failed to parse show response")?;

        Ok(result)
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.base_url);

        Ok(self
            .client
            .get(&url)
            .send()
            .await
            .is_ok_and(|response| response.status().is_success()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = OllamaClient::new("http://localhost:11434".to_string(), 300);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_with_default_url() {
        let client = OllamaClient::with_default_url();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let client = OllamaClient::with_default_url().unwrap();
        // This will pass if Ollama is running, fail otherwise
        let is_healthy = client.health_check().await.unwrap_or(false);
        // We just check it doesn't panic
        println!("Ollama health check: {is_healthy}");
    }

    #[tokio::test]
    async fn test_list_models() {
        let client = OllamaClient::with_default_url().unwrap();
        if client.health_check().await.unwrap_or(false) {
            let models = client.list_models().await;
            if let Ok(models) = models {
                println!("Found {} models", models.len());
                assert!(!models.is_empty());
            }
        }
    }

    #[tokio::test]
    async fn test_generate_request_serialization() {
        let request = GenerateRequest {
            model: "test".to_string(),
            prompt: "Hello".to_string(),
            system: None,
            stream: false,
        };

        let json = serde_json::to_string(&request);
        assert!(json.is_ok());
        assert!(json.unwrap().contains("test"));
    }

    #[tokio::test]
    async fn test_generate_response_deserialization() {
        let json = r#"{"response":"Hello","done":true,"context":[]}"#;
        let response: Result<GenerateResponse, _> = serde_json::from_str(json);
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.response, "Hello");
        assert!(response.done);
    }

    #[tokio::test]
    #[ignore = "Only run with --ignored flag when Ollama is running"]
    async fn test_generate_with_real_model() {
        let client = OllamaClient::with_default_url().unwrap();

        if !client.health_check().await.unwrap_or(false) {
            println!("Skipping: Ollama not running");
            return;
        }

        let request = GenerateRequest {
            model: "qwen3:4b".to_string(),
            prompt: "Say 'test successful' and nothing else".to_string(),
            system: None,
            stream: false,
        };

        let response = client.generate(request).await;
        assert!(
            response.is_ok(),
            "Generate request failed: {:?}",
            response.err()
        );

        let response = response.unwrap();
        assert!(response.done);
        assert!(!response.response.is_empty());
        println!("Model response: {}", response.response);
    }

    #[tokio::test]
    #[ignore = "Only run with --ignored flag when Ollama is running"]
    async fn test_show_model_with_real_instance() {
        let client = OllamaClient::with_default_url().unwrap();

        if !client.health_check().await.unwrap_or(false) {
            println!("Skipping: Ollama not running");
            return;
        }

        let result = client.show_model("qwen3:4b").await;
        assert!(result.is_ok(), "Show model failed: {:?}", result.err());

        let info = result.unwrap();
        println!("Model info retrieved successfully");
        println!("Template length: {}", info.template.len());
    }
}
