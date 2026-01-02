# YouTube Downloader - Makefile
# Convenience commands for development and version management

.PHONY: help dev build clean version-status version-sync version-bump-patch version-bump-minor version-bump-major version-set

# Default target
help:
	@echo "YouTube Downloader - Available Commands"
	@echo "========================================"
	@echo ""
	@echo "Development:"
	@echo "  make dev              - Run in development mode (hot-reload)"
	@echo "  make build            - Build release version"
	@echo "  make clean            - Clean build artifacts"
	@echo ""
	@echo "Version Management:"
	@echo "  make version-status        - Show current version"
	@echo "  make version-sync          - Sync all version files"
	@echo "  make version-bump-patch    - Bump patch version (0.1.0 → 0.1.1)"
	@echo "  make version-bump-minor    - Bump minor version (0.1.0 → 0.2.0)"
	@echo "  make version-bump-major    - Bump major version (0.1.0 → 1.0.0)"
	@echo "  make version-set v=X.Y.Z   - Set specific version"
	@echo ""

# Development
dev:
	@echo "🚀 Starting development mode..."
	cd youtube-downloader && npm run tauri dev

# Build
build:
	@echo "🔨 Building release version..."
	cd youtube-downloader && npm run tauri build
	@echo "✓ Build complete!"
	@echo "📦 Output:"
	@echo "   - youtube-downloader/src-tauri/target/release/bundle/macos/youtube-downloader.app"
	@echo "   - youtube-downloader/src-tauri/target/release/bundle/dmg/*.dmg"

# Clean
clean:
	@echo "🧹 Cleaning build artifacts..."
	cd youtube-downloader && cargo clean
	cd youtube-downloader && rm -rf node_modules/.vite
	@echo "✓ Clean complete!"

# Version Management
version-status:
	@python3 scripts/version.py status

version-sync:
	@python3 scripts/version.py sync

version-bump-patch:
	@python3 scripts/version.py bump patch

version-bump-minor:
	@python3 scripts/version.py bump minor

version-bump-major:
	@python3 scripts/version.py bump major

version-set:
ifndef v
	@echo "❌ Error: version not specified"
	@echo "Usage: make version-set v=X.Y.Z"
	@echo "Example: make version-set v=1.0.0"
	@exit 1
endif
	@python3 scripts/version.py set $(v)

# Install dependencies
install:
	@echo "📦 Installing dependencies..."
	cd youtube-downloader && npm install
	@echo "✓ Dependencies installed!"

# Test
test:
	@echo "🧪 Running tests..."
	cd youtube-downloader/src-tauri && cargo test

# Lint
lint:
	@echo "🔍 Linting code..."
	cd youtube-downloader/src-tauri && cargo clippy -- -D warnings

# Format
format:
	@echo "✨ Formatting code..."
	cd youtube-downloader/src-tauri && cargo fmt
