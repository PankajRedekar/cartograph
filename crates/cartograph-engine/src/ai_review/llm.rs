use cartograph_core::config::AiReviewConfig;
use cartograph_core::review::{
    KeepAChangelog, PrChangelogSummary, ReviewExecutionMode, ReviewFinding, SuggestedFix,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmReviewResponse {
    pub summary: Option<String>,
    pub findings_insights: Vec<LlmFindingInsight>,
    pub changelog: Option<LlmChangelog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmFindingInsight {
    pub finding_id: String,
    pub ai_reasoning: String,
    pub improved_replacement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmChangelog {
    pub title: String,
    pub overview: String,
    pub added: Vec<String>,
    pub changed: Vec<String>,
    pub deprecated: Vec<String>,
    pub removed: Vec<String>,
    pub fixed: Vec<String>,
    pub security: Vec<String>,
}

pub struct LlmClient {
    config: AiReviewConfig,
    http: reqwest::Client,
}

impl LlmClient {
    pub fn new(config: AiReviewConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { config, http }
    }

    /// Augment findings with AI reasoning or deterministic fallback
    pub async fn review_and_augment(
        &self,
        context_xml: &str,
        findings: &mut [ReviewFinding],
        offline: bool,
    ) -> (ReviewExecutionMode, Option<PrChangelogSummary>) {
        if offline || self.config.provider == "offline" {
            Self::apply_offline_deterministic(findings);
            return (ReviewExecutionMode::OfflineDeterministic, None);
        }

        // Check for API key in config or environment
        let api_key = match self.config.provider.to_lowercase().as_str() {
            "anthropic" => env::var("ANTHROPIC_API_KEY").ok(),
            "openai" => env::var("OPENAI_API_KEY").ok(),
            "gemini" => env::var("GEMINI_API_KEY").ok(),
            "ollama" => Some("ollama".to_string()),
            _ => env::var("LLM_API_KEY").ok(),
        };

        let Some(key) = api_key else {
            Self::apply_offline_deterministic(findings);
            return (ReviewExecutionMode::OfflineDeterministic, None);
        };

        match self.call_llm(&key, context_xml).await {
            Ok(resp) => {
                // Apply insights to findings
                for insight in resp.findings_insights {
                    if let Some(f) = findings.iter_mut().find(|f| f.id == insight.finding_id) {
                        f.ai_reasoning = Some(insight.ai_reasoning);
                        if let Some(imp) = insight.improved_replacement {
                            if let Some(ref mut fix) = f.suggested_fix {
                                fix.replacement_code = imp;
                            } else {
                                f.suggested_fix = Some(SuggestedFix {
                                    replacement_code: imp,
                                    unified_diff_patch: String::new(),
                                    explanation: "AI generated contextual remediation".to_string(),
                                });
                            }
                        }
                    }
                }

                let changelog = resp.changelog.map(|c| PrChangelogSummary {
                    title: c.title,
                    pr_overview: c.overview,
                    api_surface_changes: Vec::new(),
                    business_rule_deltas: Vec::new(),
                    changelog: KeepAChangelog {
                        added: c.added,
                        changed: c.changed,
                        deprecated: c.deprecated,
                        removed: c.removed,
                        fixed: c.fixed,
                        security: c.security,
                    },
                });

                (
                    ReviewExecutionMode::AiAugmented {
                        provider: self.config.provider.clone(),
                        model: self.config.model.clone(),
                        prompt_tokens: context_xml.len() / 4,
                        completion_tokens: 300,
                    },
                    changelog,
                )
            }
            Err(_) => {
                Self::apply_offline_deterministic(findings);
                (ReviewExecutionMode::OfflineDeterministic, None)
            }
        }
    }

    /// Dispatch LLM call based on configured provider
    async fn call_llm(
        &self,
        api_key: &str,
        context_xml: &str,
    ) -> Result<LlmReviewResponse, Box<dyn std::error::Error + Send + Sync>> {
        let system_prompt = "You are Cartograph AI Reviewer. Analyze the provided <review_context> XML containing AST graph structural evidence, breaking contracts, downstream hazard call sites, and risk metrics. Return JSON adhering to schema: { \"summary\": string, \"findings_insights\": [{\"finding_id\": string, \"ai_reasoning\": string, \"improved_replacement\": string}], \"changelog\": {\"title\": string, \"overview\": string, \"added\": [], \"changed\": [], \"deprecated\": [], \"removed\": [], \"fixed\": [], \"security\": []} }";

        let provider = self.config.provider.to_lowercase();
        let body = if provider == "anthropic" {
            let url = "https://api.anthropic.com/v1/messages";
            let payload = serde_json::json!({
                "model": self.config.model,
                "max_tokens": 1500,
                "system": system_prompt,
                "messages": [{"role": "user", "content": context_xml}]
            });

            let resp = self
                .http
                .post(url)
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&payload)
                .send()
                .await?;

            let json: serde_json::Value = resp.json().await?;
            let text = json["content"][0]["text"].as_str().unwrap_or("{}");
            Self::clean_json_str(text)
        } else if provider == "gemini" {
            let model = &self.config.model;
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model, api_key
            );
            let payload = serde_json::json!({
                "contents": [{"parts": [{"text": format!("{}\n\n{}", system_prompt, context_xml)}]}]
            });

            let resp = self
                .http
                .post(&url)
                .header("content-type", "application/json")
                .json(&payload)
                .send()
                .await?;

            let json: serde_json::Value = resp.json().await?;
            let text = json["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .unwrap_or("{}");
            Self::clean_json_str(text)
        } else if provider == "ollama" {
            let url = self
                .config
                .api_base
                .as_deref()
                .unwrap_or("http://localhost:11434/api/generate");
            let payload = serde_json::json!({
                "model": self.config.model,
                "prompt": format!("{}\n\n{}", system_prompt, context_xml),
                "format": "json",
                "stream": false
            });

            let resp = self.http.post(url).json(&payload).send().await?;

            let json: serde_json::Value = resp.json().await?;
            let text = json["response"].as_str().unwrap_or("{}");
            Self::clean_json_str(text)
        } else {
            // OpenAI or compatible
            let url = self
                .config
                .api_base
                .as_deref()
                .unwrap_or("https://api.openai.com/v1/chat/completions");
            let payload = serde_json::json!({
                "model": self.config.model,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": context_xml}
                ],
                "response_format": {"type": "json_object"}
            });

            let resp = self
                .http
                .post(url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("content-type", "application/json")
                .json(&payload)
                .send()
                .await?;

            let json: serde_json::Value = resp.json().await?;
            let text = json["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("{}");
            Self::clean_json_str(text)
        };

        let parsed: LlmReviewResponse = serde_json::from_str(&body)?;
        Ok(parsed)
    }

    /// Strip markdown ```json code fences if present
    fn clean_json_str(raw: &str) -> String {
        let trimmed = raw.trim();
        if let Some(stripped) = trimmed.strip_prefix("```json")
            && let Some(end) = stripped.rfind("```")
        {
            return stripped[..end].trim().to_string();
        }
        if let Some(stripped) = trimmed.strip_prefix("```")
            && let Some(end) = stripped.rfind("```")
        {
            return stripped[..end].trim().to_string();
        }
        trimmed.to_string()
    }

    /// Populate deterministic explanation when running offline
    fn apply_offline_deterministic(findings: &mut [ReviewFinding]) {
        for f in findings.iter_mut() {
            if f.ai_reasoning.is_none() {
                f.ai_reasoning = Some(format!(
                    "[Deterministic Offline Analysis] Flagged based on AST structural invariants: {}. Severity: {:?}.",
                    f.title, f.severity
                ));
            }
        }
    }
}
