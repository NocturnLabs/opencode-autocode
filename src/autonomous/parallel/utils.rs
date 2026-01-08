/// Convert a description to a URL-safe slug
pub fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .take(5) // Limit length
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("User authentication"), "user-authentication");
        assert_eq!(slugify("API: Login endpoint"), "api-login-endpoint");
        assert_eq!(
            slugify("very long feature description that goes on and on"),
            "very-long-feature-description-that"
        );
    }
}
