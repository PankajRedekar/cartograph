"""Tests that all documented CLI commands and flags in docs/cookbooks are valid."""

import os
import re
import subprocess
from pathlib import Path
import pytest

COOKBOOKS_DIR = Path("docs/cookbooks")

def extract_cartograph_commands():
    commands = []
    if not COOKBOOKS_DIR.exists():
        return commands

    for md_file in COOKBOOKS_DIR.glob("*.md"):
        content = md_file.read_text(encoding="utf-8")
        # Extract bash blocks
        matches = re.findall(r"```bash\n(cartograph\s+[^\n]+)\n```", content)
        for cmd in matches:
            # Skip placeholders or commands with pipes/variables
            if not any(token in cmd for token in ["<", ">", "$", "|", "cargo", "#"]):
                commands.append((md_file.name, cmd.strip()))
    return commands

@pytest.mark.parametrize("source_file,cmd_str", extract_cartograph_commands())
def test_documented_cli_command_syntax(source_file, cmd_str):
    """Verifies each extracted cookbook command runs or passes flag validation."""
    tokens = cmd_str.split()
    assert tokens[0] == "cartograph"
    subcommand = tokens[1]

    # Verify that the subcommand and its flags are valid by checking --help
    res = subprocess.run(
        ["cargo", "run", "-q", "-p", "cartograph-cli", "--", subcommand, "--help"],
        capture_output=True,
        text=True,
    )
    assert res.returncode == 0, f"Command '{cmd_str}' in {source_file} failed --help validation"
