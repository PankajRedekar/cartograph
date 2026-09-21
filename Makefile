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

.PHONY: all ci test lint fmt fmt-check build gates index lint-arch deadcode complexity config-validate clean help

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

## clean: Remove build artifacts, target folder, and local cartograph database
clean:
	@echo -e "$(YELLOW)==> Cleaning build artifacts and local cache...$(NC)"
	$(CARGO) clean
	rm -rf .cartograph/

## help: Display this help message
help:
	@echo -e "Usage: make [target]"
	@echo -e ""
	@echo -e "Available targets:"
	@grep -E '^## ' $(MAKEFILE_LIST) | sed -e 's/## /  /' | awk -F': ' '{printf "  \033[1;36m%-18s\033[0m %s\n", $$1, $$2}'
