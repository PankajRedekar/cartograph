use cartograph_core::config::DiagnoseConfig;
use cartograph_core::diagnose::{
    DiagnosticExecutionMode, FaultCandidate, FaultCategory, ProvenanceSlice, SuggestedFix,
    TracebackFrame,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmDiagnosisResponse {
    pub root_cause_explanation: Option<String>,
    pub suggested_patch_code: Option<String>,
    pub unified_diff: Option<String>,
}

pub struct DiagnosticLlmClient {
    config: DiagnoseConfig,
    http: reqwest::Client,
}

impl DiagnosticLlmClient {
    pub fn new(config: DiagnoseConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { config, http }
    }

    /// Augments the diagnosis with AI reasoning or falls back to offline mode.
    pub async fn synthesize_ai_diagnosis(
        &self,
        crash_type: &str,
        crash_message: &str,
        frames: &[TracebackFrame],
        provenance: Option<&ProvenanceSlice>,
        root_cause: Option<&FaultCandidate>,
        category: FaultCategory,
    ) -> (Option<String>, Option<SuggestedFix>, DiagnosticExecutionMode) {
        if self.config.offline {
            return (None, None, DiagnosticExecutionMode::OfflineDeterministic);
        }

        let context_xml = Self::build_diagnostic_context_xml(
            crash_type,
            crash_message,
            frames,
            provenance,
            root_cause,
            category,
        );

        match self.call_provider(&context_xml).await {
            Ok(resp) => {
                let fix = if let (Some(code), Some(diff)) =
                    (resp.suggested_patch_code, resp.unified_diff)
                {
                    Some(SuggestedFix {
                        replacement_code: code,
                        unified_diff_patch: diff,
                        explanation: resp
                            .root_cause_explanation
                            .clone()
                            .unwrap_or_else(|| "AI-synthesized defensive fix".to_string()),
                    })
                } else {
                    None
                };

                (
                    resp.root_cause_explanation,
                    fix,
                    DiagnosticExecutionMode::AiAugmented {
                        provider: self.config.provider.clone(),
                        model: self.config.model.clone(),
                    },
                )
            }
            Err(_) => {
                // Graceful fallback to offline
                (None, None, DiagnosticExecutionMode::OfflineDeterministic)
            }
        }
    }

    /// Compiles a token-optimized `<diagnostic_context>` XML (<1,500 tokens).
    pub fn build_diagnostic_context_xml(
        crash_type: &str,
        crash_message: &str,
        frames: &[TracebackFrame],
        provenance: Option<&ProvenanceSlice>,
        root_cause: Option<&FaultCandidate>,
        category: FaultCategory,
    ) -> String {
        let mut xml = String::new();
        xml.push_str("<diagnostic_context>\n");
        xml.push_str(&format!(
            "  <crash type=\"{crash_type}\" category=\"{category}\">\n    <message>{crash_message}</message>\n  </crash>\n"
        ));

        if let Some(rc) = root_cause {
            xml.push_str(&format!(
                "  <suspected_root_cause file=\"{}\" line=\"{}\" func=\"{}\" score=\"{:.2}\">\n    <reason>{}</reason>\n  </suspected_root_cause>\n",
                rc.file_path.display(),
                rc.line_number,
                rc.function_name,
                rc.suspicion_score,
                rc.reason
            ));
        }

        if let Some(prov) = provenance {
            xml.push_str(&format!(
                "  <variable_provenance var=\"{}\" origin_file=\"{}\" origin_line=\"{}\">\n    <expr>{}</expr>\n  </variable_provenance>\n",
                prov.variable_name,
                prov.origin_file.display(),
                prov.origin_line,
                prov.origin_expression
            ));
        }

        xml.push_str("  <call_path>\n");
        for f in frames {
            if f.is_in_project {
                xml.push_str(&format!(
                    "    <frame file=\"{}\" line=\"{}\" func=\"{}\" is_leaf=\"{}\">\n",
                    f.file_path.display(),
                    f.line_number,
                    f.function_name,
                    f.is_leaf_panic
                ));
                if let Some(snippet) = &f.code_snippet {
                    xml.push_str(&format!("      <snippet>\n{snippet}\n      </snippet>\n"));
                }
                xml.push_str("    </frame>\n");
            }
        }
        xml.push_str("  </call_path>\n");
        xml.push_str("</diagnostic_context>\n");
        xml
    }

    async fn call_provider(&self, context_xml: &str) -> Result<LlmDiagnosisResponse, String> {
        let provider = self.config.provider.to_lowercase();
        let prompt = format!(
            "You are Cartograph's AI Root-Cause Diagnostic Engine.\nAnalyze the following AST diagnostic context and return a JSON object with keys:\n- root_cause_explanation: string\n- suggested_patch_code: string (code to insert)\n- unified_diff: string (unified diff patch format)\n\nContext:\n{context_xml}"
        );

        match provider.as_str() {
            "anthropic" => {
                let api_key = env::var("ANTHROPIC_API_KEY")
                    .map_err(|_| "ANTHROPIC_API_KEY environment variable not set".to_string())?;

                let payload = serde_json::json!({
                    "model": self.config.model,
                    "max_tokens": 1500,
                    "temperature": 0.2,
                    "messages": [{"role": "user", "content": prompt}]
                });

                let res = self
                    .http
                    .post("https://api.anthropic.com/v1/messages")
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| format!("Anthropic request failed: {e}"))?;

                let body: serde_json::Value = res
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {e}"))?;

                let content = body["content"][0]["text"].as_str().unwrap_or("{}");
                Self::extract_json_response(content)
            }
            "openai" | "ollama" => {
                let endpoint = self.config.api_base.clone().unwrap_or_else(|| {
                    if provider == "ollama" {
                        "http://localhost:11434/v1/chat/completions".to_string()
                    } else {
                        "https://api.openai.com/v1/chat/completions".to_string()
                    }
                });

                let api_key = env::var("OPENAI_API_KEY").unwrap_or_default();
                let mut req = self.http.post(&endpoint).json(&serde_json::json!({
                    "model": self.config.model,
                    "messages": [{"role": "user", "content": prompt}]
                }));

                if !api_key.is_empty() {
                    req = req.header("Authorization", format!("Bearer {api_key}"));
                }

                let res = req
                    .send()
                    .await
                    .map_err(|e| format!("LLM request failed: {e}"))?;

                let body: serde_json::Value = res
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {e}"))?;

                let content = body["choices"][0]["message"]["content"].as_str().unwrap_or("{}");
                Self::extract_json_response(content)
            }
            _ => Err(format!("Unsupported provider: {provider}")),
        }
    }

    fn extract_json_response(content: &str) -> Result<LlmDiagnosisResponse, String> {
        let json_str = if let Some(start) = content.find('{') {
            if let Some(end) = content.rfind('}') {
                &content[start..=end]
            } else {
                content
            }
        } else {
            content
        };

        serde_json::from_str::<LlmDiagnosisResponse>(json_str)
            .map_err(|e| format!("JSON parsing failed: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_build_diagnostic_context_xml() {
        let frame = TracebackFrame {
            frame_index: 0,
            file_path: PathBuf::from("src/main.py"),
            line_number: 12,
            column: None,
            function_name: "run".to_string(),
            code_snippet: Some("service.process()".to_string()),
            is_in_project: true,
            is_leaf_panic: true,
            ast_symbol_id: None,
            ast_node_kind: None,
            language: "python".to_string(),
        };

        let xml = DiagnosticLlmClient::build_diagnostic_context_xml(
            "AttributeError",
            "'NoneType' object has no attribute 'x'",
            &[frame],
            None,
            None,
            FaultCategory::NullDeref,
        );

        assert!(xml.contains("<diagnostic_context>"));
        assert!(xml.contains("AttributeError"));
        assert!(xml.contains("src/main.py"));
        assert!(xml.contains("</diagnostic_context>"));
    }
}
