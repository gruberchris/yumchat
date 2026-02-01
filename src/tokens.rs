// Token counting utilities

/// Approximate token count based on character count
/// This is a simple heuristic: ~4 characters per token
/// For production, consider using tiktoken-rs for accurate counts
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn estimate_tokens(text: &str) -> usize {
    // Split on whitespace and punctuation for better estimation
    let words = text.split_whitespace().count();
    // Average: 1.3 tokens per word (accounting for subword tokenization)
    ((words as f64) * 1.3).ceil() as usize
}

/// Calculate tokens for a message including role
pub fn count_message_tokens(_role: &str, content: &str) -> usize {
    // Role overhead: ~4 tokens for role formatting
    let role_tokens = 4;
    let content_tokens = estimate_tokens(content);
    role_tokens + content_tokens
}

/// Calculate total tokens for a conversation
#[allow(dead_code)]
pub fn count_conversation_tokens(messages: &[(String, String)]) -> usize {
    messages
        .iter()
        .map(|(role, content)| count_message_tokens(role, content))
        .sum()
}

/// Calculate remaining tokens in context window
#[allow(dead_code)]
pub const fn remaining_tokens(used_tokens: usize, context_window_size: usize) -> usize {
    context_window_size.saturating_sub(used_tokens)
}

/// Calculate percentage of context window used
#[allow(dead_code, clippy::cast_precision_loss)]
pub fn context_usage_percentage(used_tokens: usize, context_window_size: usize) -> f64 {
    if context_window_size == 0 {
        return 0.0;
    }
    (used_tokens as f64 / context_window_size as f64) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        assert!(estimate_tokens("Hello world") > 0);
        assert!(estimate_tokens("") == 0);

        let short = estimate_tokens("Hi");
        let long = estimate_tokens("This is a much longer sentence with many words");
        assert!(long > short);
    }

    #[test]
    fn test_count_message_tokens() {
        let tokens = count_message_tokens("user", "Hello world");
        assert!(tokens > 4); // Should be more than just role overhead

        let user_tokens = count_message_tokens("user", "Test");
        let assistant_tokens = count_message_tokens("assistant", "Test");
        assert_eq!(user_tokens, assistant_tokens); // Same content, same count
    }

    #[test]
    fn test_count_conversation_tokens() {
        let messages = vec![
            ("user".to_string(), "Hello".to_string()),
            ("assistant".to_string(), "Hi there!".to_string()),
        ];

        let total = count_conversation_tokens(&messages);
        assert!(total > 0);

        let individual_sum =
            count_message_tokens("user", "Hello") + count_message_tokens("assistant", "Hi there!");
        assert_eq!(total, individual_sum);
    }

    #[test]
    fn test_remaining_tokens() {
        assert_eq!(remaining_tokens(100, 1000), 900);
        assert_eq!(remaining_tokens(1000, 1000), 0);
        assert_eq!(remaining_tokens(1500, 1000), 0); // Saturating sub
    }

    #[test]
    fn test_context_usage_percentage() {
        assert!((context_usage_percentage(0, 1000) - 0.0).abs() < f64::EPSILON);
        assert!((context_usage_percentage(500, 1000) - 50.0).abs() < f64::EPSILON);
        assert!((context_usage_percentage(1000, 1000) - 100.0).abs() < f64::EPSILON);
        assert!((context_usage_percentage(100, 0) - 0.0).abs() < f64::EPSILON); // Avoid division by zero
    }

    #[test]
    fn test_token_estimation_consistency() {
        let text = "The quick brown fox jumps over the lazy dog";
        let tokens1 = estimate_tokens(text);
        let tokens2 = estimate_tokens(text);
        assert_eq!(tokens1, tokens2); // Should be deterministic
    }

    #[test]
    fn test_empty_conversation() {
        let messages: Vec<(String, String)> = vec![];
        assert_eq!(count_conversation_tokens(&messages), 0);
    }

    #[test]
    fn test_long_text() {
        let long_text = "word ".repeat(1000);
        let tokens = estimate_tokens(&long_text);
        assert!(tokens > 1000); // Should have meaningful count
        assert!(tokens < 2000); // But not too high
    }
}
