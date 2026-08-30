# YouTube Downloader - Makefile
# Convenience commands for development and version management

.PHONY: help dev build check-assets install-app run run-verbose clean version-status version-sync version-bump-patch version-bump-minor version-bump-major version-set

# Default target
help:
	@echo "YouTube Downloader - Available Commands"
	@echo "========================================"
	@echo ""
	@echo "Development:"
	@echo "  make dev              - Run in development mode (hot-reload)"
	@echo "  make build            - Build release version"
	@echo "  make check-assets     - Verify the built UI loads nothing over the network"
	@echo "  make install-app      - Install the release .app into /Applications"
	@echo "  make run              - Launch the installed app"
	@echo "  make run-verbose      - Launch the installed app with logs in terminal"
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
	@./scripts/check-no-external-assets.sh
	@echo "✓ Build complete!"
	@echo "📦 Output:"
	@echo "   - youtube-downloader/src-tauri/target/release/bundle/macos/youtube-downloader.app"
	@echo "   - youtube-downloader/src-tauri/target/release/bundle/dmg/*.dmg"

# Verify the built UI has no render-blocking external assets
check-assets:
	@./scripts/check-no-external-assets.sh

# Paths
APP_NAME    := youtube-downloader
APP_BUNDLE  := youtube-downloader/src-tauri/target/release/bundle/macos/$(APP_NAME).app
INSTALL_DIR := /Applications
INSTALLED   := $(INSTALL_DIR)/$(APP_NAME).app

# Install the release build into /Applications.
# Pin THAT copy to the Dock: the bundle under target/ is deleted by every
# rebuild and by `make clean`, which leaves the Dock icon pointing at nothing
# (or at a half-written bundle that opens as an empty white window).
install-app:
	@test -d "$(APP_BUNDLE)" || { \
		echo "❌ Release bundle not found: $(APP_BUNDLE)"; \
		echo "   Run 'make build' first."; \
		exit 1; \
	}
	@echo "📦 Installing to $(INSTALLED)..."
	@rm -rf "$(INSTALLED)"
	@cp -R "$(APP_BUNDLE)" "$(INSTALL_DIR)/"
	@echo "✓ Installed: $(INSTALLED)"
	@echo "   Drag it to the Dock from /Applications (remove any older Dock icon first)."

# Launch the installed app
run:
	@test -d "$(INSTALLED)" || { echo "❌ Not installed. Run 'make install-app' first."; exit 1; }
	open "$(INSTALLED)"

# Launch the installed app in the foreground so startup errors are visible.
# Use this when the window opens empty - the reason shows up here.
run-verbose:
	@test -x "$(INSTALLED)/Contents/MacOS/$(APP_NAME)" || { echo "❌ Not installed. Run 'make install-app' first."; exit 1; }
	"$(INSTALLED)/Contents/MacOS/$(APP_NAME)"

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
