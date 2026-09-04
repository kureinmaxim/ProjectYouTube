# Version management — YouTube Downloader

Language: **English** · [Русский](VERSION_MANAGEMENT_ru.md)

How the app version is stored and how to change it safely. Source of truth: `youtube-downloader/package.json`.

---

## Quick reference

### macOS

```bash
# Via Make
make version-status
make version-sync
make version-bump-patch    # 1.6.0 → 1.6.1
make version-bump-minor    # 1.6.0 → 1.7.0
make version-bump-major    # 1.6.0 → 2.0.0
make version-set v=1.0.0

# Or Python directly
python3 scripts/version.py status
python3 scripts/version.py sync
python3 scripts/version.py bump patch
python3 scripts/version.py bump minor
python3 scripts/version.py bump major
python3 scripts/version.py set 1.0.0
```

---

## Current version: **1.6.5**

**Date:** 2026-09-04  
**Status:** `tauri build` works after `npm install` (Rust crates pinned to npm minors)

---

## Version files

| File | Role | Source |
|------|------|--------|
| `package.json` | npm package version | yes |
| `src-tauri/Cargo.toml` | Rust app version | |
| `src-tauri/tauri.conf.json` | Tauri config version | |

### Source of truth

```json
// package.json
{
  "name": "youtube-downloader",
  "version": "1.6.5"  // ← source of truth
}
```

---

## macOS: managing versions

### Check the current version

```bash
# From the repo root

# Via Make
make version-status

# Or Python
python3 scripts/version.py status
```

**Output:**
```
📦 YouTube Downloader Version Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  youtube-downloader/package.json              : 1.6.5
  youtube-downloader/src-tauri/Cargo.toml      : 1.6.5
  youtube-downloader/src-tauri/tauri.conf.json : 1.6.5

✓ All versions synchronized
```

### Sync the files

```bash
# Via Make
make version-sync

# Or Python
python3 scripts/version.py sync
```

Reads the version from `package.json` and updates the other files.

### Bump the version

**Patch (1.6.0 → 1.6.1):** bug fixes
```bash
make version-bump-patch
# or
python3 scripts/version.py bump patch
```

**Minor (1.6.0 → 1.7.0):** new features
```bash
make version-bump-minor
# or
python3 scripts/version.py bump minor
```

**Major (1.6.0 → 2.0.0):** breaking changes
```bash
make version-bump-major
# or
python3 scripts/version.py bump major
```

### Set a specific version

```bash
# Via Make
make version-set v=1.0.0

# Or Python
python3 scripts/version.py set 1.0.0
```

---

## Release process

### macOS

```bash
# 1. Bump the version
make version-bump-minor
# or for patch/major:
# make version-bump-patch
# make version-bump-major

# 2. Build the app
cd youtube-downloader
npm run tauri build

# 3. Check the version
make version-status

# 4. Commit
git add -A
git commit -m "chore: release v1.5.2"

# 5. Tag
git tag -a v1.5.2 -m "YouTube Downloader v1.5.2"
git push origin v1.5.2

# 6. GitHub Release (optional)
gh release create v1.5.2 \
  --title "YouTube Downloader v1.5.2" \
  --notes "Release notes here" \
  youtube-downloader/src-tauri/target/release/bundle/dmg/*.dmg
```

---

## Semantic Versioning

| Kind | When | Example |
|------|------|---------|
| **Patch** | Bug fixes, small UI polish | 1.6.0 → 1.6.1 |
| **Minor** | New features (playlists, history) | 1.6.0 → 1.7.0 |
| **Major** | Full UI / architecture rewrite | 1.6.0 → 2.0.0 |

---

## Manual version edits

### 1. package.json
```json
{
  "name": "youtube-downloader",
  "version": "X.Y.Z"
}
```

### 2. src-tauri/Cargo.toml
```toml
[package]
name = "youtube-downloader"
version = "X.Y.Z"
```

### 3. src-tauri/tauri.conf.json
```json
{
  "version": "X.Y.Z"
}
```

---

## Version history

### 0.1.0 (2026-01-02)
- First release
- Dark-mode UI
- YouTube video download
- Quality picker (Best, 1080p, 720p, 480p, MP3)
- Progress bar
- Chrome cookies
- Save-folder picker

---

## Troubleshooting

### Versions are out of sync

```bash
# Current state
make version-status
# or
python3 scripts/version.py status

# Sync every file
make version-sync
# or
python3 scripts/version.py sync
```

### `make` not found

Install via Homebrew:
```bash
brew install make
```

### `python3` not found

```bash
# Check Python
python3 --version

# Install if missing
brew install python3
```

### After a bump nothing changed

```bash
# Clean the cache and rebuild
cd youtube-downloader
npm run tauri build
```

The version script does not rewrite Markdown. Update `README.md` / `README_ru.md` in the same commit as the bump.

---

**Last updated:** 2026-09-03
