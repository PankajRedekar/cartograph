# 🛠️ Aider, Roo Code, Cline & OpenHands Integration

Cartograph supports the broader autonomous coding agent ecosystem:

## Aider (`.aider.conf.yml`)

Configure Aider to automatically check architectural boundaries and tests:

```yaml
# .aider.conf.yml
test-cmd: "cartograph verify --staged"
```

## Roo Code & Cline (`.roomodes`, `.clinerules`)

Run the automated installer:

```bash
cartograph skills install --target roo --yes
cartograph skills install --target cline --yes
```

Inlines rules instructing the LLM to invoke `cartograph locate` to anchor file edits to exact coordinates.
