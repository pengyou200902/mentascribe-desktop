//! Text processing module for post-transcription transformations

/// Process transcribed text with various transformations
pub fn process_text(text: &str, auto_capitalize: bool) -> String {
    if !auto_capitalize {
        return text.to_string();
    }

    capitalize_sentences(text)
}

/// Capitalize the first letter of the text and after sentence-ending punctuation
fn capitalize_sentences(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut capitalize_next = true;

    for c in text.chars() {
        if capitalize_next && c.is_alphabetic() {
            result.push(c.to_uppercase().next().unwrap_or(c));
            capitalize_next = false;
        } else {
            result.push(c);
        }

        // Capitalize after sentence-ending punctuation followed by space
        if c == '.' || c == '!' || c == '?' {
            capitalize_next = true;
        }

        // Don't capitalize immediately after punctuation without space
        if c.is_alphanumeric() && capitalize_next && !c.is_whitespace() {
            capitalize_next = false;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_sentences() {
        assert_eq!(
            capitalize_sentences("hello world"),
            "Hello world"
        );
        assert_eq!(
            capitalize_sentences("hello. how are you"),
            "Hello. How are you"
        );
        assert_eq!(
            capitalize_sentences("hello! what's up? not much"),
            "Hello! What's up? Not much"
        );
    }

    #[test]
    fn test_process_text_disabled() {
        assert_eq!(
            process_text("hello world", false),
            "hello world"
        );
    }

    #[test]
    fn test_process_text_enabled() {
        assert_eq!(
            process_text("hello world", true),
            "Hello world"
        );
    }
}
