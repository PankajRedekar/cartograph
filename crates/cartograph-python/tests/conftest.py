"""Pytest fixtures for cartograph Python SDK tests."""
from __future__ import annotations

import tempfile
from pathlib import Path
import pytest
from cartograph import CodeGraph


@pytest.fixture
def sample_repo(tmp_path: Path) -> Path:
    """Create a multi-file sample codebase for graph indexing tests."""
    pkg = tmp_path / "app"
    pkg.mkdir()
    (pkg / "__init__.py").write_text("")

    math_mod = pkg / "math_ops.py"
    math_mod.write_text(
        "def add(a: int, b: int) -> int:\n"
        "    return a + b\n\n"
        "def multiply(a: int, b: int) -> int:\n"
        "    return a * b\n"
    )

    service_mod = pkg / "service.py"
    service_mod.write_text(
        "from app.math_ops import add, multiply\n\n"
        "class CalculatorService:\n"
        "    def compute(self, x: int, y: int) -> int:\n"
        "        sum_val = add(x, y)\n"
        "        return multiply(sum_val, 2)\n"
    )

    test_mod = tmp_path / "test_app.py"
    test_mod.write_text(
        "from app.service import CalculatorService\n\n"
        "def test_compute():\n"
        "    svc = CalculatorService()\n"
        "    assert svc.compute(1, 2) == 6\n"
    )

    return tmp_path


@pytest.fixture
def in_memory_graph(sample_repo: Path) -> CodeGraph:
    """Provide an indexed in-memory CodeGraph instance."""
    cg = CodeGraph(path=sample_repo, in_memory=True)
    cg.index(force=True)
    yield cg
    cg.close()
