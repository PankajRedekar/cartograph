use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const CARTOGRAPH_START_DELIMITER: &str = "<!-- CARTOGRAPH SKILLS START -->";
pub const CARTOGRAPH_END_DELIMITER: &str = "<!-- CARTOGRAPH SKILLS END -->";

pub const USER_CUSTOM_START: &str = "<!-- USER CUSTOMIZATIONS START -->";
pub const USER_CUSTOM_END: &str = "<!-- USER CUSTOMIZATIONS END -->";

pub struct TemplateEngine;

impl TemplateEngine {
    /// Merges `new_block` into `original_content` using Cartograph delimiters.
    /// If delimiters already exist, the delimited block is replaced in-place.
    /// Otherwise, the delimited block is appended to the end of the file.
    pub fn merge_delimited_content(original_content: &str, new_block_body: &str) -> String {
        let full_block = format!(
            "{}\n{}\n{}",
            CARTOGRAPH_START_DELIMITER,
            new_block_body.trim(),
            CARTOGRAPH_END_DELIMITER
        );

        if let Some(start_idx) = original_content.find(CARTOGRAPH_START_DELIMITER)
            && let Some(end_idx) = original_content.find(CARTOGRAPH_END_DELIMITER)
        {
            let actual_end = end_idx + CARTOGRAPH_END_DELIMITER.len();
            if start_idx <= end_idx {
                let mut result = String::with_capacity(original_content.len() + full_block.len());
                result.push_str(&original_content[..start_idx]);
                result.push_str(&full_block);
                result.push_str(&original_content[actual_end..]);
                return result;
            }
        }

        // Delimiters not present or malformed: append
        if original_content.trim().is_empty() {
            full_block
        } else {
            format!("{}\n\n{}", original_content.trim_end(), full_block)
        }
    }

    /// Extracts any user customizations delimited by `<!-- USER CUSTOMIZATIONS START -->` and `<!-- USER CUSTOMIZATIONS END -->`.
    pub fn extract_user_customizations(content: &str) -> Option<String> {
        if let Some(start_idx) = content.find(USER_CUSTOM_START) {
            let body_start = start_idx + USER_CUSTOM_START.len();
            if let Some(end_idx) = content[body_start..].find(USER_CUSTOM_END) {
                let custom = &content[body_start..body_start + end_idx];
                return Some(custom.trim().to_string());
            }
        }
        None
    }

    /// Injects extracted user customizations into a new template.
    pub fn inject_user_customizations(new_template: &str, customizations: &str) -> String {
        if customizations.trim().is_empty() {
            return new_template.to_string();
        }

        let custom_block = format!(
            "{}\n{}\n{}",
            USER_CUSTOM_START,
            customizations.trim(),
            USER_CUSTOM_END
        );

        if let Some(start_idx) = new_template.find(USER_CUSTOM_START)
            && let Some(end_idx) = new_template.find(USER_CUSTOM_END)
        {
            let actual_end = end_idx + USER_CUSTOM_END.len();
            let mut result = String::new();
            result.push_str(&new_template[..start_idx]);
            result.push_str(&custom_block);
            result.push_str(&new_template[actual_end..]);
            return result;
        }

        // If template doesn't have customization delimiters, append before decision checklist or at end
        format!(
            "{}\n\n## Custom Repository Guidelines\n{}\n",
            new_template.trim_end(),
            custom_block
        )
    }

    /// Expands `{{variable}}` placeholders in template text.
    pub fn expand_variables(template: &str, vars: &[(&str, &str)]) -> String {
        let mut result = template.to_string();
        for (key, val) in vars {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, val);
        }
        result
    }

    /// Atomically writes content to a file via a temporary sibling file and rename.
    pub fn write_file_atomic(path: &Path, content: &str) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp_path = match path.file_name() {
            Some(name) => {
                let rand_suffix = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0);
                path.with_file_name(format!(".{}.tmp_{}", name.to_string_lossy(), rand_suffix))
            }
            None => path.with_extension("tmp"),
        };

        fs::write(&tmp_path, content)?;

        if let Err(e) = fs::rename(&tmp_path, path) {
            // Attempt cleanup of temporary file if rename failed
            let _ = fs::remove_file(&tmp_path);
            return Err(e);
        }

        Ok(())
    }

    /// Creates a backup file (.bak) if the file currently exists.
    pub fn backup_file(path: &Path) -> io::Result<Option<PathBuf>> {
        if !path.exists() {
            return Ok(None);
        }

        let bak_path = path.with_extension("bak");
        fs::copy(path, &bak_path)?;
        Ok(Some(bak_path))
    }
}
