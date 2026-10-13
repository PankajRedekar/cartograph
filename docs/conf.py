# docs/conf.py
"""Sphinx configuration for Cartograph Developer Documentation Portal."""

import os
import sys
from datetime import datetime

# Project Information
project = "Cartograph"
copyright = f"{datetime.now().year}, Cartograph Engineering Team"
author = "Cartograph Engineering Team"
release = "0.2.0"
version = "0.2"

# General Sphinx Configuration
extensions = [
    "myst_parser",
    "sphinx_design",
    "sphinxcontrib.mermaid",
    "sphinx_copybutton",
    "autoapi.extension",
    "sphinx.ext.doctest",
    "sphinx.ext.viewcode",
]

# Source File Suffixes
source_suffix = {
    ".rst": "restructuredtext",
    ".md": "markdown",
}
root_doc = "index"
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store", "venv", ".venv", "llms*.txt"]

# MyST-Parser Settings
myst_enable_extensions = [
    "colon_fence",
    "deflist",
    "fieldlist",
    "html_admonition",
    "html_image",
    "replacements",
    "smartquotes",
    "substitution",
    "tasklist",
]
myst_heading_anchors = 3

# Theme Options (Furo)
html_theme = "furo"
html_title = "Cartograph Documentation"
html_static_path = ["_static"]
html_css_files = ["css/custom.css"]
html_js_files = [
    "js/config-builder.js",
    "js/graph-explorer.js",
]

html_theme_options = {
    "light_logo": "images/logo-light.svg",
    "dark_logo": "images/logo-dark.svg",
    "sidebar_hide_name": False,
    "navigation_with_keys": True,
    "top_of_page_buttons": ["view", "edit"],
    "source_repository": "https://github.com/PankajRedekar/cartograph",
    "source_branch": "master",
    "source_directory": "docs/",
}

# Mermaid Configuration
mermaid_version = "10.9.1"
mermaid_init_js = """
mermaid.initialize({
    startOnLoad: true,
    theme: 'default',
    securityLevel: 'loose'
});
"""

# CopyButton Configuration (Strip terminal prompts)
copybutton_prompt_text = r">>> |\.\.\. |\$ |❯ "
copybutton_prompt_is_regexp = True

# Python AutoAPI Settings
autoapi_type = "python"
autoapi_dirs = ["../crates/cartograph-python/python/cartograph"]
autoapi_root = "reference/python"
autoapi_keep_files = False
autoapi_options = [
    "members",
    "undoc-members",
    "show-inheritance",
    "show-module-summary",
    "special-members",
    "imported-members",
]
autoapi_python_class_content = "both"
autoapi_member_order = "bysource"
suppress_warnings = ["autoapi.python_import_resolution"]

# UNIX Man Pages Configuration
man_pages = [
    (
        "index",
        "cartograph",
        "Cartograph - Ultra-fast native AST code graph & software intelligence engine",
        [author],
        1,
    ),
    (
        "reference/cli/cartograph-index",
        "cartograph-index",
        "Index repository code graph into SQLite database",
        [author],
        1,
    ),
    (
        "reference/cli/cartograph-locate",
        "cartograph-locate",
        "Rapid 1-turn AST symbol search replacing grep loops",
        [author],
        1,
    ),
    (
        "reference/cli/cartograph-impact",
        "cartograph-impact",
        "Blast-radius calculation and breaking change detection",
        [author],
        1,
    ),
    (
        "reference/cli/cartograph-verify",
        "cartograph-verify",
        "Atomic pre-commit closed-loop self-healing verification gate",
        [author],
        1,
    ),
]

# Linkcheck Configuration
linkcheck_retries = 3
linkcheck_timeout = 10
linkcheck_ignore = [
    r"http://localhost:\d+/?",
    r"https://github.com/PankajRedekar/cartograph/.*",
]
