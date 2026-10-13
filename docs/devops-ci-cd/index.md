# 🚢 CI/CD & DevOps Automation

Integrate Cartograph quality gates into automated build and deployment pipelines.

::::{grid} 1 2 2 2
:gutter: 3

:::{grid-item-card} 🐙 GitHub Actions
:link: github-actions
:link-type: doc

Automated structural quality gates, branch diff checks, and fail-fast predictive tests.
:::

:::{grid-item-card} 🦊 GitLab CI
:link: gitlab-ci
:link-type: doc

Native containerized pipelines with SQLite caching and architectural gate stages.
:::

:::{grid-item-card} 🪝 Local Pre-Commit Hooks
:link: pre-commit
:link-type: doc

Fast staged verification (<200ms) with .pre-commit-config.yaml preventing bad commits.
:::

:::{grid-item-card} 🛡️ SARIF & Security Scanning
:link: sarif-security
:link-type: doc

Exporting architectural violations to the GitHub Code Scanning security tab.
:::
::::

```{toctree}
:maxdepth: 2
:hidden:

github-actions
gitlab-ci
pre-commit
sarif-security
```
