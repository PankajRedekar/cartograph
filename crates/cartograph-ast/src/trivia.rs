/// Trivia binder and signature normalizer for cleaning docstrings, comments, and signatures.
pub struct TriviaBinder;

impl TriviaBinder {
    /// Clean a raw docstring string literal (e.g. Python triple quotes `"""..."""` or `'''...'''`)
    pub fn clean_python_docstring(raw: &str) -> String {
        let trimmed = raw.trim();
        let stripped = if ((trimmed.starts_with("\"\"\"") && trimmed.ends_with("\"\"\""))
            || (trimmed.starts_with("'''") && trimmed.ends_with("'''")))
            && trimmed.len() >= 6
        {
            &trimmed[3..trimmed.len() - 3]
        } else if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            if trimmed.len() >= 2 {
                &trimmed[1..trimmed.len() - 1]
            } else {
                trimmed
            }
        } else {
            trimmed
        };

        Self::normalize_indentation(stripped)
    }

    /// Clean JSDoc or block comments (`/** ... */`)
    pub fn clean_jsdoc(raw: &str) -> String {
        let trimmed = raw.trim();
        let without_delimiters = if trimmed.starts_with("/**") && trimmed.ends_with("*/") {
            &trimmed[3..trimmed.len() - 2]
        } else if trimmed.starts_with("/*") && trimmed.ends_with("*/") {
            &trimmed[2..trimmed.len() - 2]
        } else {
            trimmed
        };

        let mut lines = Vec::new();
        for line in without_delimiters.lines() {
            let l = line.trim();
            let cleaned = if let Some(stripped) = l.strip_prefix('*') {
                stripped.trim()
            } else {
                l
            };
            if !cleaned.is_empty() {
                lines.push(cleaned);
            }
        }

        lines.join("\n")
    }

    /// Clean contiguous line comments (e.g. Rust `///`, Ruby `#`, Go `//`)
    pub fn clean_line_comments(raw: &str, prefix: &str) -> String {
        let mut lines = Vec::new();
        for line in raw.lines() {
            let l = line.trim();
            if let Some(stripped) = l.strip_prefix(prefix) {
                let content = stripped.strip_prefix(' ').unwrap_or(stripped);
                lines.push(content);
            }
        }
        lines.join("\n")
    }

    fn normalize_indentation(s: &str) -> String {
        let lines: Vec<&str> = s.lines().collect();
        if lines.is_empty() {
            return String::new();
        }

        // Find min indentation of non-empty lines (skip first line if inline)
        let min_indent = lines
            .iter()
            .skip(1)
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
            .min()
            .unwrap_or(0);

        let mut cleaned = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if i == 0 {
                cleaned.push(line.trim());
            } else if line.trim().is_empty() {
                cleaned.push("");
            } else if line.len() >= min_indent {
                cleaned.push(&line[min_indent..]);
            } else {
                cleaned.push(line.trim_start());
            }
        }

        cleaned.join("\n").trim().to_string()
    }
}

/// Normalizer for function and class signatures.
pub struct SignatureNormalizer;

impl SignatureNormalizer {
    /// Normalize a raw multiline signature into a clean, canonical single-line representation.
    pub fn normalize(raw: &str) -> String {
        let mut result = String::with_capacity(raw.len());
        let mut in_whitespace = false;

        for c in raw.chars() {
            if c.is_whitespace() {
                if !in_whitespace && !result.is_empty() {
                    result.push(' ');
                    in_whitespace = true;
                }
            } else {
                result.push(c);
                in_whitespace = false;
            }
        }

        result.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_python_docstring() {
        let raw = "\"\"\"\n        Authenticate the user.\n        Returns a bool.\n        \"\"\"";
        let cleaned = TriviaBinder::clean_python_docstring(raw);
        assert_eq!(cleaned, "Authenticate the user.\nReturns a bool.");
    }

    #[test]
    fn test_clean_jsdoc() {
        let raw = "/**\n * Fetches user profile\n * @param id User ID\n */";
        let cleaned = TriviaBinder::clean_jsdoc(raw);
        assert_eq!(cleaned, "Fetches user profile\n@param id User ID");
    }

    #[test]
    fn test_clean_line_comments() {
        let raw = "/// Line 1\n/// Line 2";
        let cleaned = TriviaBinder::clean_line_comments(raw, "///");
        assert_eq!(cleaned, "Line 1\nLine 2");
    }

    #[test]
    fn test_signature_normalizer() {
        let raw = "pub fn calculate(\n    x: i32,\n    y: i32,\n) -> Result<i32>";
        let norm = SignatureNormalizer::normalize(raw);
        assert_eq!(norm, "pub fn calculate( x: i32, y: i32, ) -> Result<i32>");
    }
}
