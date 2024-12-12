#[cfg(test)]
mod tests {
    use super::engines::{apply_pattern, EngineChoice};

    #[test]
    fn test_builtin_engine_valid_pattern() {
        let pattern = "ab.";
        let text = "abc abx aby";
        let result = apply_pattern(pattern, text, &EngineChoice::Builtin);
        assert!(
            result.contains("Matches: [\"abc\", \"abx\", \"aby\"]"),
            "Expected three matches for 'ab.'"
        );
    }

    #[test]
    fn test_builtin_engine_invalid_pattern() {
        let pattern = "("; // invalid pattern
        let text = "abc";
        let result = apply_pattern(pattern, text, &EngineChoice::Builtin);
        assert!(
            result.contains("Invalid pattern:"),
            "Expected invalid pattern error."
        );
    }

    #[test]
    fn test_custom_engine_with_valid_pattern() {
        let pattern = "a";
        let text = "abc a";
        let result = apply_pattern(pattern, text, &EngineChoice::Custom);
        assert!(
            result.contains("Matches:"),
            "Expected at least one match from CustomRegex."
        );
    }

    #[test]
    fn test_custom_engine_with_invalid_pattern() {
        let pattern = ""; // empty pattern is considered invalid in CustomRegex
        let text = "abc";
        let result = apply_pattern(pattern, text, &EngineChoice::Custom);
        assert!(
            result.contains("Invalid pattern:"),
            "Expected invalid pattern error from CustomRegex."
        );
    }

    #[test]
    fn test_custommeta_engine_fallback() {
        let pattern = "z";
        let text = "abc";
        let result = apply_pattern(pattern, text, &EngineChoice::Custommeta);
        assert!(
            result.contains("No matches found."),
            "Expected no matches found on fallback."
        );

        let pattern2 = "a";
        let text2 = "abc";
        let result2 = apply_pattern(pattern2, text2, &EngineChoice::Custommeta);
        assert!(
            result2.contains("Matches:"),
            "Expected matches from customMeta engine."
        );
    }
}
