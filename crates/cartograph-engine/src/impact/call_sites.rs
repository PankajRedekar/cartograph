use std::fs;
use std::path::PathBuf;
use cartograph_core::model::{CallSiteLocation, Node};

pub struct CallSiteExtractor {
    repo_root: PathBuf,
}

impl CallSiteExtractor {
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }

    /// Extract invocation call-sites within caller_node that reference target_symbol_name
    pub fn find_call_sites(
        &self,
        caller_node: &Node,
        target_symbol_name: &str,
        remediation_hint: Option<String>,
    ) -> Vec<CallSiteLocation> {
        let full_path = self.repo_root.join(&caller_node.filepath);
        let content = match fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let start_line = caller_node.start_line.unwrap_or(1);
        let end_line = caller_node.end_line.unwrap_or(u32::MAX);

        let mut call_sites = Vec::new();
        let target_ident = target_symbol_name.split("::").last().unwrap_or(target_symbol_name);

        for (idx, line_str) in content.lines().enumerate() {
            let line_num = (idx + 1) as u32;
            if line_num >= start_line && line_num <= end_line {
                if let Some(col_idx) = line_str.find(target_ident) {
                    let trimmed = line_str.trim();
                    call_sites.push(CallSiteLocation {
                        caller_node_id: caller_node.id.clone(),
                        filepath: caller_node.filepath.clone(),
                        line: line_num,
                        column: (col_idx + 1) as u32,
                        invocation_snippet: trimmed.to_string(),
                        remediation_hint: remediation_hint.clone(),
                    });
                }
            }
        }

        call_sites
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::model::NodeKind;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_find_call_sites() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("service.rs");

        let code = r#"
pub fn process_order() {
    let period = AccrualPeriod::from_date();
    save();
}
"#;
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(code.as_bytes()).unwrap();

        let caller = Node {
            id: "test::process_order".to_string(),
            name: "process_order".to_string(),
            kind: NodeKind::Function,
            filepath: "service.rs".to_string(),
            scope: "".to_string(),
            start_line: Some(2),
            start_col: Some(1),
            end_line: Some(5),
            end_col: Some(1),
            signature: None,
            docstring: None,
            checksum: None,
            metadata: None,
        };

        let extractor = CallSiteExtractor::new(dir.path().to_path_buf());
        let sites = extractor.find_call_sites(&caller, "AccrualPeriod", Some("Add parameter".to_string()));

        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].line, 3);
        assert!(sites[0].invocation_snippet.contains("AccrualPeriod::from_date"));
        assert_eq!(sites[0].remediation_hint.as_deref(), Some("Add parameter"));
    }
}
