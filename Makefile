.PHONY: build test run clean lint fmt check dev doc

# Build release binary
build:
	cargo build --release

# Build debug binary
dev:
	cargo build

# Run all tests
test:
	cargo test --all

# Run the application
run:
	cargo run -- $(ARGS)

# Clean build artifacts
clean:
	cargo clean

# Run clippy lints
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Check formatting
fmt:
	cargo fmt --all -- --check

# Format code
fmt-fix:
	cargo fmt --all

# Run all checks (lint + fmt + test)
check: fmt lint test

# Generate documentation
doc:
	cargo doc --no-deps --open

# Run Tauri development server
tauri-dev:
	cd ui && npm run tauri dev

# Build Tauri application
tauri-build:
	cd ui && npm run tauri build

