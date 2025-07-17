# Makefile for the project.
#
# This Makefile is designed to be cross-platform. It requires `make` and `cargo`
# to be installed and available in the system's PATH.

.DEFAULT_GOAL := help
CARGO := cargo

.PHONY: all build check clippy clean doc fmt help install release run test build-macos-arm64 build-linux-arm64 build-linux-amd64 cross-build-all

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
# Cross-compilation targets
# ==============================================================================
# Ensure you have the required toolchains installed via rustup, e.g.:
# rustup target add aarch64-apple-darwin
# rustup target add aarch64-unknown-linux-gnu
# rustup target add x86_64-unknown-linux-gnu

MACOS_ARM64_TARGET := aarch64-apple-darwin
LINUX_ARM64_TARGET := aarch64-unknown-linux-gnu
LINUX_AMD64_TARGET := x86_64-unknown-linux-gnu

build-macos-arm64:
	@echo "Building release for macOS (ARM64)..."
	@$(CARGO) build --release --target $(MACOS_ARM64_TARGET)

build-linux-arm64:
	@echo "Building release for Linux (ARM64)..."
	@$(CARGO) build --release --target $(LINUX_ARM64_TARGET)

build-linux-amd64:
	@echo "Building release for Linux (AMD64)..."
	@$(CARGO) build --release --target $(LINUX_AMD64_TARGET)

cross-build-all: build-macos-arm64 build-linux-arm64 build-linux-amd64
	@echo "All cross-compilation builds complete."

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
	@echo "  release    Build an optimized release version for the host machine."
	@echo "  run        Run the project. Pass arguments via ARGS."
	@echo "             Example: make run ARGS=\"-f ansi src/main.rs\""
	@echo "  test       Run all tests."
	@echo "  check      Check source code for errors without building."
	@echo ""
	@echo "Cross-compilation targets (release build):"
	@echo "  build-macos-arm64   Build for macOS (ARM64)."
	@echo "  build-linux-arm64   Build for Linux (ARM64)."
	@echo "  build-linux-amd64   Build for Linux (AMD64)."
	@echo "  cross-build-all     Build for all cross-compilation targets."
	@echo ""
	@echo "Utility targets:"
	@echo "  clippy     Run the clippy linter for code analysis."
	@echo "  fmt        Format code according to style guidelines."
	@echo "  doc        Generate and open project documentation."
	@echo "  install    Build release and install to Cargo's bin directory."
	@echo "  clean      Remove build artifacts."
	@echo "  help       Display this help message."
