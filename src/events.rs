// Event types for async communication

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// A chunk of text received from the AI
    AiResponseChunk(String),
    /// AI response completed
    AiResponseDone,
    /// An error occurred during AI generation
    AiError(String),
}
