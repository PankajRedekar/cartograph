use crate::ai_review::publishers::ReviewPublisher;
use crate::ai_review::publishers::markdown::MarkdownPublisher;
use cartograph_core::review::ReviewReport;
use std::env;
use std::path::Path;
use std::time::Duration;

pub struct GitHubPublisher {
    pub repo: Option<String>,
    pub pr_number: Option<u64>,
    pub token: Option<String>,
}

impl GitHubPublisher {
    pub fn new(repo: Option<String>, pr_number: Option<u64>, token: Option<String>) -> Self {
        Self {
            repo,
            pr_number,
            token,
        }
    }
}

impl ReviewPublisher for GitHubPublisher {
    fn name(&self) -> &'static str {
        "github"
    }

    async fn publish(
        &self,
        report: &ReviewReport,
        _repo_root: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let repo = self
            .repo
            .clone()
            .or_else(|| env::var("GITHUB_REPOSITORY").ok())
            .ok_or("GITHUB_REPOSITORY not specified")?;

        let pr_num = self
            .pr_number
            .or_else(|| {
                env::var("GITHUB_PR_NUMBER")
                    .or_else(|_| env::var("PR_NUMBER"))
                    .ok()
                    .and_then(|s| s.parse().ok())
            })
            .ok_or("GitHub PR number not specified")?;

        let token = self
            .token
            .clone()
            .or_else(|| env::var("GITHUB_TOKEN").ok())
            .ok_or("GITHUB_TOKEN not provided")?;

        let md_body = MarkdownPublisher::render_markdown(report);
        let url = format!(
            "https://api.github.com/repos/{}/issues/{}/comments",
            repo, pr_num
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()?;

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "cartograph-ai-review")
            .header("Accept", "application/vnd.github+json")
            .json(&serde_json::json!({ "body": md_body }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            return Err(
                format!("GitHub API comment failed (HTTP {}): {}", status, err_text).into(),
            );
        }

        Ok(())
    }
}
