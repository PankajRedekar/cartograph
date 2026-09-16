use std::collections::HashMap;

/// Extracts architectural tags declared directly in source code docstrings, comments, or attributes
pub struct ArchTagExtractor;

impl ArchTagExtractor {
    /// Extract key-value architectural properties from comment/docstring text
    ///
    /// Examples supported:
    /// - `/// @arch layer=domain`
    /// - `//! @arch layer=domain`
    /// - `/** @arch layer=domain */`
    /// - `// @arch layer=domain`
    /// - `# @arch: layer=domain`
    /// - `#[cartograph(layer = "domain")]`
    pub fn extract_tags(text: &str) -> HashMap<String, String> {
        let mut tags = HashMap::new();

        for line in text.lines() {
            let trimmed = line.trim();

            // Check for @arch marker
            if let Some(pos) = trimmed.find("@arch") {
                let rest = &trimmed[pos + 5..];
                // Clean up leading colons or spaces
                let cleaned = rest.trim_start_matches([':', ' ']);
                Self::parse_key_values(cleaned, &mut tags);
            }
            // Check for #[cartograph(...)] attribute
            else if trimmed.contains("cartograph(") {
                if let Some(start) = trimmed.find("cartograph(") {
                    let inner = &trimmed[start + 11..];
                    if let Some(end) = inner.find(')') {
                        let attrs = &inner[..end];
                        Self::parse_attribute_key_values(attrs, &mut tags);
                    }
                }
            }
        }

        tags
    }

    /// Extract the primary layer name if declared in the given text
    pub fn extract_layer(text: &str) -> Option<String> {
        Self::extract_tags(text).get("layer").cloned()
    }

    fn parse_key_values(input: &str, tags: &mut HashMap<String, String>) {
        for token in input.split_whitespace() {
            let token = token.trim_end_matches(['*', '/', ';', ',']);
            if let Some((k, v)) = token.split_once('=') {
                let key = k.trim().to_lowercase();
                let val = v.trim().trim_matches(['"', '\'']).to_string();
                if !key.is_empty() && !val.is_empty() {
                    tags.insert(key, val);
                }
            }
        }
    }

    fn parse_attribute_key_values(input: &str, tags: &mut HashMap<String, String>) {
        for item in input.split(',') {
            if let Some((k, v)) = item.split_once('=') {
                let key = k.trim().to_lowercase();
                let val = v.trim().trim_matches(['"', '\'']).to_string();
                if !key.is_empty() && !val.is_empty() {
                    tags.insert(key, val);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_docstring() {
        let doc = r#"
            /// Service for user authentication.
            /// @arch layer=domain
            pub struct AuthService;
        "#;
        assert_eq!(ArchTagExtractor::extract_layer(doc), Some("domain".to_string()));
    }

    #[test]
    fn test_extract_typescript_comment() {
        let comment = r#"
            /**
             * React Checkout Component
             * @arch layer=presentation
             */
        "#;
        assert_eq!(ArchTagExtractor::extract_layer(comment), Some("presentation".to_string()));
    }

    #[test]
    fn test_extract_python_comment() {
        let comment = "# @arch: layer=infrastructure";
        assert_eq!(ArchTagExtractor::extract_layer(comment), Some("infrastructure".to_string()));
    }

    #[test]
    fn test_extract_attribute() {
        let attr = r#"#[cartograph(layer = "application", role = "service")]"#;
        let tags = ArchTagExtractor::extract_tags(attr);
        assert_eq!(tags.get("layer"), Some(&"application".to_string()));
        assert_eq!(tags.get("role"), Some(&"service".to_string()));
    }
}
