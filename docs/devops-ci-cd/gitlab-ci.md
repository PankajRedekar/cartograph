# 🦊 GitLab CI Pipeline Configuration

```yaml
# .gitlab-ci.yml
stages:
  - analyze

cartograph_gate:
  stage: analyze
  image: rust:latest
  script:
    - cargo install --path crates/cartograph-cli
    - cartograph index .
    - cartograph lint-arch --strict --fail-on error
    - cartograph deadcode --fail-on-deadcode
```
