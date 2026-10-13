#!/usr/bin/env python3
"""Compiles docs/llms.txt and docs/llms-full.txt for autonomous AI coding agents."""

import os
import re
from pathlib import Path

DOCS_DIR = Path("docs")

LLMS_TXT_CONTENT = """# Cartograph

> Ultra-Fast Native AST Code Graph Intelligence Engine & Autonomous AI Agent Architecture

Cartograph parses 7 polyglot languages (Rust, TypeScript/JS, Python, Ruby, Go, SQL, Markdown) into a unified SQLite code graph with sub-millisecond query latency.

## Key Capabilities
- **cartograph locate**: 1-turn AST subgraph retrieval replacing grep (<3ms).
- **cartograph impact**: Blast-radius calculation and breaking change detection (<25ms).
- **cartograph test-selector**: Predictive test selection and execution (<15ms).
- **cartograph routes**: Full-stack HTTP route and database ORM mapping.
- **cartograph lint-arch**: Architectural boundary and circular dependency checker.
- **cartograph complexity**: Cognitive complexity heatmap and God-class detection.
- **cartograph deadcode**: Mark-and-sweep reachability and dead code purging.
- **cartograph ai-review**: Contract-aware PR reviews and patch application.
- **cartograph extract-logic**: Business validation rules & Gherkin extractor.
- **cartograph diagnose**: Stack trace AST slicer & test reproducer generator.
- **cartograph diagram**: Visual architecture and callgraph diagram exporter.
- **cartograph skills**: Multi-IDE agent skills installer (Claude, Cursor, Windsurf, Antigravity).
- **cartograph verify**: Pre-commit closed-loop self-healing verification gate (<200ms).

## Documentation Links
- Full Documentation Corpus: /llms-full.txt
- CLI Reference: /reference/cli/
- Python SDK Reference: /reference/python/
"""

def clean_markdown_for_agent(text: str) -> str:
    # Remove Sphinx directives like ```{grid} ... ``` or ```{toctree}
    text = re.sub(r'```\{toctree\}[\s\S]*?```', '', text)
    text = re.sub(r'```\{grid\}[\s\S]*?```', '', text)
    text = re.sub(r'```\{eval-rst\}[\s\S]*?```', '', text)
    # Strip HTML tags
    text = re.sub(r'<div[^>]*>[\s\S]*?</div>', '', text)
    return text.strip()

def main():
    DOCS_DIR.mkdir(parents=True, exist_ok=True)

    # 1. Write docs/llms.txt
    llms_path = DOCS_DIR / "llms.txt"
    llms_path.write_text(LLMS_TXT_CONTENT.strip() + "\n", encoding="utf-8")
    print(f"✔ Generated {llms_path} ({len(LLMS_TXT_CONTENT.split())} words)")

    # 2. Compile docs/llms-full.txt
    full_corpus = ["# Cartograph Complete Documentation Corpus\n\n"]
    full_corpus.append(LLMS_TXT_CONTENT + "\n\n---\n\n")

    search_dirs = [
        DOCS_DIR / "getting-started",
        DOCS_DIR / "core-concepts",
        DOCS_DIR / "cookbooks",
        DOCS_DIR / "agent-integration",
        DOCS_DIR / "python-sdk",
        DOCS_DIR / "devops-ci-cd",
        DOCS_DIR / "polyglot-reference",
    ]

    for s_dir in search_dirs:
        if s_dir.exists():
            for md_file in sorted(s_dir.glob("*.md")):
                content = md_file.read_text(encoding="utf-8")
                cleaned = clean_markdown_for_agent(content)
                full_corpus.append(f"## File: {md_file.name}\n\n{cleaned}\n\n---\n\n")

    full_path = DOCS_DIR / "llms-full.txt"
    full_path.write_text("".join(full_corpus), encoding="utf-8")
    print(f"✔ Generated {full_path} ({len(''.join(full_corpus))} bytes)")

if __name__ == "__main__":
    main()
