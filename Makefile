# agent-pmo:5547fd2
# =============================================================================
# Standard Makefile — dart_mutant
# Cross-platform: Linux, macOS, Windows (via GNU Make)
# Language: Rust
# =============================================================================

.PHONY: build test lint fmt fmt-check clean check ci coverage coverage-check setup help

# -----------------------------------------------------------------------------
# OS Detection — portable commands for Linux, macOS, and Windows
# -----------------------------------------------------------------------------
ifeq ($(OS),Windows_NT)
  SHELL := powershell.exe
  .SHELLFLAGS := -NoProfile -Command
  RM = Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
  MKDIR = New-Item -ItemType Directory -Force
  HOME ?= $(USERPROFILE)
else
  RM = rm -rf
  MKDIR = mkdir -p
endif

# Coverage threshold (override via env var or per-repo CI variable)
COVERAGE_THRESHOLD ?= 85

# =============================================================================
# PRIMARY TARGETS (uniform interface — do not rename)
# =============================================================================

## build: Compile release artifacts
build:
	@echo "==> Building..."
	cargo build --release

## test: Run full test suite with coverage collection
test:
	@echo "==> Testing..."
	rustup component add llvm-tools-preview 2>/dev/null || true
	cargo llvm-cov --workspace --all-targets --lcov --output-path lcov.info
	$(MAKE) coverage-check

## lint: Run all linters (fails on any warning)
lint:
	@echo "==> Linting..."
	$(MAKE) fmt-check
	cargo clippy --release --all-targets -- -D warnings

## fmt: Format all code in-place
fmt:
	@echo "==> Formatting..."
	cargo fmt --all

## fmt-check: Check formatting without modifying (CI uses this — fails hard)
fmt-check:
	@echo "==> Checking format..."
	cargo fmt --all --check

## clean: Remove all build artifacts
clean:
	@echo "==> Cleaning..."
	cargo clean
	$(RM) lcov.info

## check: lint + test (pre-commit)
check: lint test

## ci: lint + test + build (full CI simulation)
ci: lint test build

## coverage: Generate HTML coverage report
coverage:
	@echo "==> Coverage report..."
	cargo llvm-cov report --html --output-dir target/llvm-cov/html
	@echo "==> HTML report: target/llvm-cov/html/index.html"

## coverage-check: Assert thresholds (exits non-zero if below)
coverage-check:
	@echo "==> Checking coverage thresholds..."
	@LH=$$(grep '^LH:' lcov.info | awk -F: '{sum+=$$2} END{print sum+0}'); \
	LF=$$(grep '^LF:' lcov.info | awk -F: '{sum+=$$2} END{print sum+0}'); \
	if [ "$$LF" -eq 0 ]; then echo "FAIL: No lines in lcov.info"; exit 1; fi; \
	PCT=$$(awk "BEGIN{printf \"%.1f\", $$LH/$$LF*100}"); \
	PCT_INT=$$(awk "BEGIN{printf \"%d\", $$LH/$$LF*100}"); \
	echo "Line coverage: $${PCT}% (threshold: $(COVERAGE_THRESHOLD)%)"; \
	if [ "$$PCT_INT" -lt "$(COVERAGE_THRESHOLD)" ]; then \
	  echo "FAIL: $${PCT}% < $(COVERAGE_THRESHOLD)%"; exit 1; \
	else \
	  echo "OK: $${PCT}% >= $(COVERAGE_THRESHOLD)%"; \
	fi

## setup: Post-create dev environment setup
setup:
	@echo "==> Setting up development environment..."
	cargo install cargo-llvm-cov
	rustup component add llvm-tools-preview rustfmt clippy
	@echo "==> Setup complete. Run 'make ci' to validate."

# =============================================================================
# HELP
# =============================================================================
help:
	@echo "Available targets:"
	@echo "  build          - Compile release artifacts"
	@echo "  test           - Run full test suite with coverage"
	@echo "  lint           - Run rustfmt check + clippy (errors mode)"
	@echo "  fmt            - Format all code in-place"
	@echo "  fmt-check      - Check formatting (no modification, CI)"
	@echo "  clean          - Remove build artifacts"
	@echo "  check          - lint + test (pre-commit)"
	@echo "  ci             - lint + test + build (full CI)"
	@echo "  coverage       - Generate HTML coverage report"
	@echo "  coverage-check - Assert coverage thresholds"
	@echo "  setup          - Install dev tooling (cargo-llvm-cov, components)"
