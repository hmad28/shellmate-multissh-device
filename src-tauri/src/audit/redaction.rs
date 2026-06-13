/// Apply redaction patterns to a payload string.
/// Patterns are regex-like strings that replace matched content with `[REDACTED]`.
/// Uses simple string matching for MVP (full regex deferred).
pub fn apply_patterns(payload: &str, patterns: &[String]) -> String {
    if patterns.is_empty() {
        return payload.to_string();
    }

    let mut result = payload.to_string();

    for pattern in patterns {
        // Simple pattern matching: look for `key=value` or `key: value` patterns.
        // For MVP, we use case-insensitive substring matching.
        let lower = result.to_lowercase();
        let pattern_lower = pattern.to_lowercase();

        // Find all occurrences and replace.
        let mut offset = 0;
        while let Some(pos) = lower[offset..].find(&pattern_lower) {
            let abs_pos = offset + pos;
            // Find the end of the value (next space, newline, or end of string).
            let value_start = abs_pos + pattern.len();
            let value_end = result[value_start..]
                .find(|c: char| c.is_whitespace() || c == ',' || c == ';')
                .map(|i| value_start + i)
                .unwrap_or(result.len());

            if value_end > value_start {
                result = format!(
                    "{}{}{}",
                    &result[..value_start],
                    "[REDACTED]",
                    &result[value_end..]
                );
                offset = value_start + "[REDACTED]".len();
            } else {
                offset = abs_pos + 1;
            }

            if offset >= result.len() {
                break;
            }
        }
    }

    result
}

/// Built-in redaction patterns for common secret formats.
pub fn default_patterns() -> Vec<String> {
    vec![
        "password=".into(),
        "password: ".into(),
        "passwd=".into(),
        "secret=".into(),
        "token=".into(),
        "api_key=".into(),
        "apikey=".into(),
        "authorization: bearer ".into(),
        "private_key".into(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_patterns_returns_original() {
        let result = apply_patterns("hello world", &[]);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn redacts_password_value() {
        let patterns = vec!["password=".into()];
        let result = apply_patterns("user=admin password=secret123 host=localhost", &patterns);
        assert_eq!(result, "user=admin password=[REDACTED] host=localhost");
    }

    #[test]
    fn redacts_token_value() {
        let patterns = vec!["token=".into()];
        let result = apply_patterns("token=abc123 other=value", &patterns);
        assert_eq!(result, "token=[REDACTED] other=value");
    }

    #[test]
    fn multiple_patterns() {
        let patterns = vec!["password=".into(), "secret=".into()];
        let result = apply_patterns("password=abc secret=xyz normal=ok", &patterns);
        assert_eq!(result, "password=[REDACTED] secret=[REDACTED] normal=ok");
    }
}
