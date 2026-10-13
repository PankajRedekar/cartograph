# 📄 AI Agent Endpoints (`llms.txt`, `llms-full.txt` & Headless Lookups)

Cartograph provides dedicated endpoints designed specifically for ingestion by Large Language Models and autonomous coding agents.

## 1. `llms.txt` (Curated Token Roadmap)

Hosted at `/llms.txt` on the documentation site:
- Adheres to the emerging `llms.txt` standard.
- Delivers an ultra-dense, curated overview of Cartograph's capabilities, CLI syntax, and skills architecture in **<500 tokens**.
- Ideal for agents to read on turn 1 of an engineering session.

## 2. `llms-full.txt` (Concatenated Corpus)

Hosted at `/llms-full.txt`:
- Concatenates the entire documentation into a single clean Markdown file.
- Strips out HTML navigation menus, headers, footers, and interactive web widgets.
- Preserves code examples, XML schemas, and parameter tables.
- Enables models with large context windows (128k–1M tokens) to ingest the complete documentation corpus in a single turn.

## 3. Headless CLI Docs Retrieval (`cartograph docs --for-agent`)

Coding agents with terminal execution tools can look up targeted documentation without scraping the web:

```bash
cartograph docs --for-agent --section impact --max-tokens 500
```

Emitted output:
```xml
<cartograph_docs section="impact" budget="500">
  <summary>Analyze blast radius, breaking changes, and risk of code modifications in &lt;25ms.</summary>
  <usage>cartograph impact --symbol &lt;NAME&gt; [--depth &lt;D&gt;] [--fail-on &lt;LEVEL&gt;] [--format xml|json]</usage>
  <options>
    <option flag="--symbol" type="string" desc="Target symbol query"/>
    <option flag="--depth" type="int" default="3" desc="Maximum graph traversal depth"/>
    <option flag="--verify-semver" type="string" desc="Enforce SemVer policy (patch, minor, major)"/>
    <option flag="--fail-on" type="string" desc="Fail CI if risk meets threshold (low, medium, high, critical)"/>
  </options>
</cartograph_docs>
```
