#!/usr/bin/env python3
"""Builds an offline Dash / Zeal .docset from compiled Sphinx HTML documentation."""

import os
import shutil
import sqlite3
from pathlib import Path

DOCS_BUILD_HTML = Path("docs/_build/html")
DOCSET_DIR = Path("docs/_build/Cartograph.docset")
DOCUMENTS_DIR = DOCSET_DIR / "Contents" / "Resources" / "Documents"

INFO_PLIST = """<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>cartograph</string>
    <key>CFBundleName</key>
    <string>Cartograph</string>
    <key>DocSetPlatformFamily</key>
    <string>cartograph</string>
    <key>isDashDocset</key>
    <true/>
    <key>dashIndexFilePath</key>
    <string>index.html</string>
</dict>
</plist>
"""

def main():
    if not DOCS_BUILD_HTML.exists():
        print(f"Error: {DOCS_BUILD_HTML} does not exist. Run 'make docs' first.")
        return

    # Clean existing docset
    if DOCSET_DIR.exists():
        shutil.rmtree(DOCSET_DIR)

    DOCUMENTS_DIR.mkdir(parents=True, exist_ok=True)

    # Copy HTML files
    print("==> Copying HTML documentation assets...")
    shutil.copytree(DOCS_BUILD_HTML, DOCUMENTS_DIR, dirs_exist_ok=True)

    # Write Info.plist
    plist_path = DOCSET_DIR / "Contents" / "Info.plist"
    plist_path.write_text(INFO_PLIST, encoding="utf-8")

    # Create SQLite docSet.dsidx index
    db_path = DOCSET_DIR / "Contents" / "Resources" / "docSet.dsidx"
    conn = sqlite3.connect(db_path)
    cur = conn.cursor()
    cur.execute("CREATE TABLE searchIndex(id INTEGER PRIMARY KEY, name TEXT, type TEXT, path TEXT);")
    cur.execute("CREATE UNIQUE INDEX anchor ON searchIndex (name, type, path);")

    # Index command pages
    for html_file in DOCUMENTS_DIR.glob("**/*.html"):
        rel_path = html_file.relative_to(DOCUMENTS_DIR)
        name = html_file.stem
        entry_type = "Command" if "cartograph-" in name else "Guide"
        cur.execute(
            "INSERT OR IGNORE INTO searchIndex(name, type, path) VALUES (?, ?, ?);",
            (name, entry_type, str(rel_path)),
        )

    conn.commit()
    conn.close()

    print(f"✔ Successfully packaged Dash/Zeal docset at: {DOCSET_DIR}")

if __name__ == "__main__":
    main()
