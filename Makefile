# agent-pmo:3140e31
# =============================================================================
# Standard Makefile — dart_mutant
# Cross-platform: Linux, macOS, Windows (via GNU Make)
# Language: Rust
# =============================================================================

.PHONY: build test lint fmt clean ci setup

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

# =============================================================================
# Coverage thresholds — read from `coverage-thresholds.json` at the repo root.
# See REPO-STANDARDS-SPEC [COVERAGE-THRESHOLDS-JSON].
# DO NOT hardcode a threshold here. DO NOT read from a CI env var.
# =============================================================================
COVERAGE_THRESHOLDS_FILE := coverage-thresholds.json

# =============================================================================
# PRIMARY TARGETS — exactly 7. Do not add others.
# =============================================================================

## build: Compile release artifacts
build:
	@echo "==> Building..."
	cargo build --release

## test: FAIL-FAST tests + coverage + threshold enforcement (ONLY test entry point).
##       Stops at the first failing test. Collects coverage. Asserts measured %
##       against `coverage-thresholds.json`. Exits non-zero on any failure.
##       See REPO-STANDARDS-SPEC [TEST-RULES] and [COVERAGE-THRESHOLDS-JSON].
test:
	@echo "==> Testing (fail-fast + coverage + threshold)..."
	rustup component add llvm-tools-preview 2>/dev/null || true
	cargo llvm-cov --workspace --all-targets --lcov --output-path lcov.info
	$(MAKE) _coverage_check

## lint: Run all linters (read-only, no formatting). Fails on any warning.
lint:
	@echo "==> Linting..."
	cargo fmt --all --check
	cargo clippy --release --all-targets -- -D warnings

## fmt: Format all code in-place
fmt:
	@echo "==> Formatting..."
	cargo fmt --all

## clean: Remove all build artifacts
clean:
	@echo "==> Cleaning..."
	cargo clean
	$(RM) lcov.info

## ci: lint + test + build (full CI simulation)
ci: lint test build

## setup: Post-create dev environment setup
setup:
	@echo "==> Setting up development environment..."
	cargo install cargo-llvm-cov
	rustup component add llvm-tools-preview rustfmt clippy
	@echo "==> Setup complete. Run 'make ci' to validate."

# =============================================================================
# PRIVATE HELPERS — not public targets
# =============================================================================

# _coverage_check: Assert coverage >= threshold from coverage-thresholds.json.
#                  Called by `test`. Never call directly.
_coverage_check:
	@if [ ! -f "$(COVERAGE_THRESHOLDS_FILE)" ]; then echo "FAIL: $(COVERAGE_THRESHOLDS_FILE) not found"; exit 1; fi; \
	THRESHOLD=$$(jq -r '.default_threshold' "$(COVERAGE_THRESHOLDS_FILE)"); \
	LH=$$(grep '^LH:' lcov.info | awk -F: '{sum+=$$2} END{print sum+0}'); \
	LF=$$(grep '^LF:' lcov.info | awk -F: '{sum+=$$2} END{print sum+0}'); \
	if [ "$$LF" -eq 0 ]; then echo "FAIL: No lines in lcov.info"; exit 1; fi; \
	PCT=$$(awk "BEGIN{printf \"%.1f\", $$LH/$$LF*100}"); \
	PCT_INT=$$(awk "BEGIN{printf \"%d\", $$LH/$$LF*100}"); \
	echo "Line coverage: $${PCT}% (threshold: $${THRESHOLD}%)"; \
	if [ "$$PCT_INT" -lt "$${THRESHOLD}" ]; then \
	  echo "FAIL: $${PCT}% < $${THRESHOLD}%"; exit 1; \
	else \
	  echo "OK: $${PCT}% >= $${THRESHOLD}%"; \
	fi

# =============================================================================
# HELP
# =============================================================================
help:
	@echo "Available targets:"
	@echo "  build  - Compile release artifacts"
	@echo "  test   - Fail-fast tests + coverage + threshold enforcement"
	@echo "  lint   - All linters/analyzers (read-only, no formatting)"
	@echo "  fmt    - Format all code in-place"
	@echo "  clean  - Remove build artifacts"
	@echo "  ci     - lint + test + build (full CI simulation)"
	@echo "  setup  - Install dev tooling (cargo-llvm-cov, components)"
