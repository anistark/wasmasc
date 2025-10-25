# WasmASC Development Justfile

default:
    @just --list

# Format code
format:
    cargo fmt

# Lint code
lint:
    cargo clippy -- -D warnings

# Run tests
test:
    cargo test

# Build with CLI feature
build-cli:
    cargo build --release --features cli

# Build library
build:
    cargo build --release

# Clean build artifacts
clean:
    cargo clean

# Install locally
install-cli:
    cargo install --path . --features cli --force

# Quick development cycle
dev: format lint test build-cli
    @echo "✅ Development cycle complete!"

# Publish to crates.io
publish: format lint test build
    @echo "📦 Publishing to crates.io..."
    cargo publish
