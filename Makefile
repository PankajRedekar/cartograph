# 🗺️ Cartograph - Local CI/CD Makefile
# Run all CI test suites, linters, and quality gates locally before pushing.

SHELL := /bin/bash
export PATH := $(HOME)/.cargo/bin:$(PATH)
CARGO := $(shell which cargo 2>/dev/null || echo $(HOME)/.cargo/bin/cargo)
CARTOGRAPH ?= target/debug/cartograph

# Colors for terminal output
BLUE := \033[1;34m
GREEN := \033[1;32m
YELLOW := \033[1;33m
RED := \033[1;31m
NC := \033[0m

.PHONY: all ci test lint fmt fmt-check build gates index lint-arch deadcode complexity config-validate clean help docs docs-serve docs-check docs-agent docs-man docs-clean python-build python-test python-publish

all: ci

## ci: Run the entire CI suite locally (formatting, clippy, unit tests, and quality gates)
ci: fmt-check lint test gates
	@echo -e "\n$(GREEN)========================================$(NC)"
	@echo -e "$(GREEN)✔ All local CI checks passed cleanly!$(NC)"
	@echo -e "$(GREEN)========================================$(NC)"

## fmt-check: Verify code formatting without making modifications
fmt-check:
	@echo -e "$(BLUE)==> [1/6] Checking code formatting...$(NC)"
	$(CARGO) fmt --all -- --check

## fmt: Automatically format all code across the workspace
fmt:
	@echo -e "$(BLUE)==> Formatting codebase...$(NC)"
	$(CARGO) fmt --all

## lint: Run Clippy linter with -D warnings across all targets
lint:
	@echo -e "$(BLUE)==> [2/6] Running Clippy linter...$(NC)"
	$(CARGO) clippy --workspace --all-targets -- -D warnings

## test: Run all workspace unit and integration tests
test:
	@echo -e "$(BLUE)==> [3/6] Running full test suite...$(NC)"
	$(CARGO) test --workspace --all-targets

## build: Compile all workspace crates and the cartograph CLI binary
build:
	@echo -e "$(BLUE)==> Building Cartograph workspace...$(NC)"
	$(CARGO) build --workspace --all-targets

## gates: Run all Cartograph self-dogfooding quality gates
gates: build config-validate index lint-arch deadcode complexity
	@echo -e "$(GREEN)✔ Cartograph quality gates passed!$(NC)"

## config-validate: Validate cartograph.toml configuration syntax and rules
config-validate: build
	@echo -e "$(BLUE)==> [4/6] Validating cartograph.toml...$(NC)"
	$(CARTOGRAPH) config validate

## index: Index the repository code graph into .cartograph/cartograph.db
index: build
	@echo -e "$(BLUE)==> [5/6] Indexing codebase graph...$(NC)"
	$(CARTOGRAPH) index . --json

## lint-arch: Run architectural boundary, encapsulation, and circular dependency checks
lint-arch: build
	@echo -e "$(BLUE)==> [6/6] Checking architectural boundaries & cycles...$(NC)"
	$(CARTOGRAPH) lint-arch --strict --fail-on error

## deadcode: Run dead code and abandoned component detector
deadcode: build
	@echo -e "$(BLUE)==> Scanning for dead code & zombie components...$(NC)"
	$(CARTOGRAPH) deadcode --json > /dev/null
	@echo -e "$(GREEN)✔ Dead code engine finished cleanly.$(NC)"

## complexity: Run code complexity, hotspot, and maintainability heatmap
complexity: build
	@echo -e "$(BLUE)==> Analyzing complexity & risk hotspots...$(NC)"
	$(CARTOGRAPH) complexity --top 10

## docs: Build Sphinx documentation portal into docs/_build/html
docs:
	@echo -e "$(BLUE)==> Generating CLI reference & agent endpoints...$(NC)"
	cargo run -q -p cartograph-cli -- docs generate-cli
	.venv/bin/python scripts/generate_llms_txt.py
	@echo -e "$(BLUE)==> Building Cartograph Sphinx documentation portal...$(NC)"
	.venv/bin/sphinx-build -b html -W --keep-going docs docs/_build/html
	@echo -e "$(GREEN)✔ Documentation built cleanly in docs/_build/html$(NC)"

## docs-serve: Launch live-reloading local documentation server on http://localhost:8080
docs-serve:
	@echo -e "$(BLUE)==> Launching live-reloading documentation server...$(NC)"
	.venv/bin/sphinx-autobuild docs docs/_build/html --port 8080 --open-browser --ignore "*/_build/*"

## docs-check: Run automated link checker, Python doctests, and CLI snippet tests
docs-check:
	@echo -e "$(BLUE)==> Verifying CLI cookbook code snippets...$(NC)"
	.venv/bin/pytest tests/test_docs_cli.py
	@echo -e "$(BLUE)==> Running documentation linkcheck...$(NC)"
	.venv/bin/sphinx-build -b linkcheck docs docs/_build/linkcheck
	@echo -e "$(BLUE)==> Running documentation doctests...$(NC)"
	.venv/bin/sphinx-build -b doctest docs docs/_build/doctest
	@echo -e "$(GREEN)✔ All documentation checks passed cleanly!$(NC)"

## docs-agent: Generate llms.txt and llms-full.txt machine endpoints
docs-agent:
	@echo -e "$(BLUE)==> Compiling llms.txt and llms-full.txt agent endpoints...$(NC)"
	.venv/bin/python scripts/generate_llms_txt.py
	@echo -e "$(GREEN)✔ Agent documentation endpoints generated.$(NC)"

## docs-man: Generate POSIX UNIX man pages into docs/_build/man/
docs-man:
	@echo -e "$(BLUE)==> Building UNIX man pages...$(NC)"
	.venv/bin/sphinx-build -b man docs docs/_build/man
	@echo -e "$(GREEN)✔ Man pages generated in docs/_build/man/$(NC)"

## docs-clean: Clean documentation build artifacts
docs-clean:
	@echo -e "$(YELLOW)==> Cleaning documentation build artifacts...$(NC)"
	rm -rf docs/_build/ docs/llms.txt docs/llms-full.txt

## python-build: Build cartograph-python source distribution and release wheel
python-build:
	@echo -e "$(BLUE)==> Building cartograph-python sdist and wheel...$(NC)"
	rm -rf dist/
	.venv/bin/maturin sdist --manifest-path crates/cartograph-python/Cargo.toml --out dist
	.venv/bin/maturin build --manifest-path crates/cartograph-python/Cargo.toml --release --out dist
	@echo -e "$(GREEN)✔ Built artifacts in dist/$(NC)"

## python-test: Run cartograph-python unit and integration test suite
python-test:
	@echo -e "$(BLUE)==> Running Python SDK tests...$(NC)"
	.venv/bin/maturin develop --manifest-path crates/cartograph-python/Cargo.toml
	.venv/bin/pytest crates/cartograph-python/tests

## python-publish: Publish cartograph-python artifacts in dist/ to PyPI
python-publish: python-build
	@echo -e "$(BLUE)==> Publishing cartograph-python to PyPI...$(NC)"
	.venv/bin/maturin publish --manifest-path crates/cartograph-python/Cargo.toml

## clean: Remove build artifacts, target folder, and local cartograph database
clean:
	@echo -e "$(YELLOW)==> Cleaning build artifacts and local cache...$(NC)"
	$(CARGO) clean
	rm -rf .cartograph/ docs/_build/ docs/llms.txt docs/llms-full.txt

## help: Display this help message
help:
	@echo -e "Usage: make [target]"
	@echo -e ""
	@echo -e "Available targets:"
	@grep -E '^## ' $(MAKEFILE_LIST) | sed -e 's/## /  /' | awk -F': ' '{printf "  \033[1;36m%-18s\033[0m %s\n", $$1, $$2}'
