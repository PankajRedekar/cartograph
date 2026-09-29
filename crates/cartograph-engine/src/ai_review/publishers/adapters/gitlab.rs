use crate::ai_review::publishers::ReviewPublisher;
use crate::ai_review::publishers::markdown::MarkdownPublisher;
use cartograph_core::review::ReviewReport;
use std::env;
use std::path::Path;
use std::time::Duration;

pub struct GitLabPublisher {
    pub project_id: Option<String>,
    pub mr_iid: Option<u64>,
    pub token: Option<String>,
    pub host: Option<String>,
}

impl GitLabPublisher {
    pub fn new(
        project_id: Option<String>,
        mr_iid: Option<u64>,
        token: Option<String>,
        host: Option<String>,
    ) -> Self {
        Self {
            project_id,
            mr_iid,
            token,
            host,
        }
    }
}

impl ReviewPublisher for GitLabPublisher {
    fn name(&self) -> &'static str {
        "gitlab"
    }

    async fn publish(
        &self,
        report: &ReviewReport,
        _repo_root: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let project_id = self
            .project_id
            .clone()
            .or_else(|| env::var("CI_PROJECT_ID").ok())
            .ok_or("CI_PROJECT_ID not specified")?;

        let mr_iid = self
            .mr_iid
            .or_else(|| {
                env::var("CI_MERGE_REQUEST_IID")
                    .ok()
                    .and_then(|s| s.parse().ok())
            })
            .ok_or("GitLab MR IID not specified")?;

        let token = self
            .token
            .clone()
            .or_else(|| {
                env::var("GITLAB_TOKEN")
                    .or_else(|_| env::var("CI_JOB_TOKEN"))
                    .ok()
            })
            .ok_or("GITLAB_TOKEN not provided")?;

        let host = self
            .host
            .clone()
            .or_else(|| env::var("CI_SERVER_URL").ok())
            .unwrap_or_else(|| "https://gitlab.com".to_string());

        let md_body = MarkdownPublisher::render_markdown(report);
        let url = format!(
            "{}/api/v4/projects/{}/merge_requests/{}/notes",
            host.trim_end_matches('/'),
            project_id,
            mr_iid
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()?;

        let resp = client
            .post(&url)
            .header("PRIVATE-TOKEN", token)
            .header("User-Agent", "cartograph-ai-review")
            .json(&serde_json::json!({ "body": md_body }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("GitLab API note failed (HTTP {}): {}", status, err_text).into());
        }

        Ok(())
    }
}
