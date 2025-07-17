# Makefile for the project.
#
# This Makefile is designed to be cross-platform. It requires `make` and `cargo`
# to be installed and available in the system's PATH.

.DEFAULT_GOAL := help
CARGO := cargo

.PHONY: all build check clippy clean doc fmt help install release run test

# ==============================================================================
# Main targets
# ==============================================================================

all: build

build:
	@echo "Building debug version..."
	@$(CARGO) build

release:
	@echo "Building release version..."
	@$(CARGO) build --release

run:
	@$(CARGO) run -- $(ARGS)

test:
	@echo "Running tests..."
	@$(CARGO) test

check:
	@echo "Checking source code..."
	@$(CARGO) check

# ==============================================================================
# Utility targets
# ==============================================================================

clippy:
	@echo "Running clippy linter..."
	@$(CARGO) clippy

fmt:
	@echo "Formatting code..."
	@$(CARGO) fmt

doc:
	@echo "Generating documentation..."
	@$(CARGO) doc --no-deps --open

install: release
	@echo "Installing release binary..."
	@$(CARGO) install --path .

clean:
	@echo "Cleaning project..."
	@$(CARGO) clean

# ==============================================================================
# Help
# ==============================================================================

help:
	@echo "Cross-platform Makefile for the project."
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Main targets:"
	@echo "  all        Build the project (default)."
	@echo "  build      Build the project for debugging."
	@echo "  release    Build an optimized release version."
	@echo "  run        Run the project. Pass arguments via ARGS."
	@echo "             Example: make run ARGS=\"-f ansi src/main.rs\""
	@echo "  test       Run all tests."
	@echo "  check      Check source code for errors without building."
	@echo ""
	@echo "Utility targets:"
	@echo "  clippy     Run the clippy linter for code analysis."
	@echo "  fmt        Format code according to style guidelines."
	@echo "  doc        Generate and open project documentation."
	@echo "  install    Build release and install to Cargo's bin directory."
	@echo "  clean      Remove build artifacts."
	@echo "  help       Display this help message."
