# 🛡️ SARIF 2.1.0 & GitHub Code Scanning

Export architectural violations and security risks directly into GitHub's Security Code Scanning alert tab:

```bash
cartograph lint-arch --sarif > results.sarif
```

Upload in GitHub Actions:

```yaml
- name: Upload SARIF report
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: results.sarif
```
