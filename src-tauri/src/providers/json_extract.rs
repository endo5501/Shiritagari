/// Extract JSON from LLM response text.
/// LLMs often wrap JSON in markdown code blocks like ```json ... ```.
/// This function strips those wrappers and returns the raw JSON string.
pub fn extract_json(text: &str) -> &str {
    // Try to find JSON inside a markdown code block: ```json ... ``` or ``` ... ```
    if let Some(start) = text.find("```") {
        let after_backticks = &text[start + 3..];
        // Skip optional language tag (e.g., "json")
        let content_start = after_backticks
            .find('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let content = &after_backticks[content_start..];
        if let Some(end) = content.find("```") {
            return content[..end].trim();
        }
    }
    text.trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_json_passthrough() {
        let input = r#"{"inference": "testing", "confidence": 0.9}"#;
        assert_eq!(extract_json(input), input);
    }

    #[test]
    fn test_json_code_block() {
        let input = "```json\n{\"inference\": \"testing\", \"confidence\": 0.9}\n```";
        assert_eq!(
            extract_json(input),
            r#"{"inference": "testing", "confidence": 0.9}"#
        );
    }

    #[test]
    fn test_plain_code_block() {
        let input = "```\n{\"inference\": \"testing\"}\n```";
        assert_eq!(extract_json(input), r#"{"inference": "testing"}"#);
    }

    #[test]
    fn test_code_block_with_surrounding_text() {
        let input = "Here is the result:\n```json\n{\"inference\": \"testing\"}\n```\nDone.";
        assert_eq!(extract_json(input), r#"{"inference": "testing"}"#);
    }

    #[test]
    fn test_whitespace_around_json() {
        let input = "  \n{\"inference\": \"testing\"}\n  ";
        assert_eq!(extract_json(input), r#"{"inference": "testing"}"#);
    }

    #[test]
    fn test_code_block_with_indented_json() {
        let input = "```json\n    {\n      \"inference\": \"testing\"\n    }\n```";
        assert_eq!(
            extract_json(input),
            "{\n      \"inference\": \"testing\"\n    }"
        );
    }
}
