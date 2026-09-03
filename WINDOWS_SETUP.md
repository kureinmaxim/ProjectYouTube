# Windows: first run

Language: **English** · [Русский](docs/WINDOWS_SETUP_ru.md)

**Version:** 1.5.1 | **Updated:** 2026-09-03

Short guide for the first YouTube Downloader build on Windows.

## Install checklist

### 1. Rust
- [ ] Download [rustup-init.exe](https://rustup.rs/)
- [ ] Install (default options)
- [ ] Restart PowerShell
- [ ] Check: `rustc --version` (1.70+)

### 2. Node.js
- [ ] Download LTS from [nodejs.org](https://nodejs.org/)
- [ ] Install with “Add to PATH”
- [ ] Restart PowerShell
- [ ] Check: `node --version` (v18+)
- [ ] Check: `npm --version` (8+)

### 3. Python (version scripts)
- [ ] Download 3.10+ from [python.org](https://www.python.org/downloads/)
- [ ] Install with “Add Python to PATH”
- [ ] Restart PowerShell
- [ ] Check: `python --version` or `py --version`

### 4. yt-dlp (the only download tool)
- [ ] Install via Chocolatey: `choco install yt-dlp`
- [ ] Or download from [yt-dlp releases](https://github.com/yt-dlp/yt-dlp/releases)
- [ ] Put `yt-dlp.exe` in `C:\Windows\` or on PATH
- [ ] Check: `yt-dlp --version`
- [ ] **Keep yt-dlp current** (`choco upgrade yt-dlp`) — the app shows how fresh the binary is

### 5. Google Chrome (optional, cookies)
- [ ] Install from [google.com/chrome](https://www.google.com/chrome/)
- [ ] Sign in to YouTube for private videos

### 6. Visual Studio Build Tools (to compile Rust)
- [ ] Download [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
- [ ] Install “Desktop development with C++”
- [ ] Or install full Visual Studio (Community)

## First build

Open PowerShell in the project folder:

```powershell
# 1. Go to the repo root
cd path\to\ProjectYouTube

# 2. Check the tools
rustc --version    # 1.70+
node --version     # v18+
npm --version      # 8+
python --version   # or: py --version
yt-dlp --version   # should print a version

# 3. Install npm dependencies
cd youtube-downloader
npm install

# 4. First Rust build (can take several minutes)
npm run tauri build

# Output:
# src-tauri\target\release\youtube-downloader.exe
# src-tauri\target\release\bundle\msi\youtube-downloader_*_x64_en-US.msi
```

## Verify

After a successful build:

```powershell
# Check artifacts
dir src-tauri\target\release\youtube-downloader.exe
dir src-tauri\target\release\bundle\msi\*.msi

# Launch the app
.\src-tauri\target\release\youtube-downloader.exe
```

## Quick commands

```powershell
# Dev mode (hot reload) — daily work
cd youtube-downloader
npm run tauri dev

# Production build — for a release
npm run tauri build

# Version (Python script)
python scripts\version.py status

# Clean artifacts
cd src-tauri
cargo clean
```

## Dev mode

For day-to-day work use dev mode:

```powershell
cd path\to\ProjectYouTube\youtube-downloader
npm run tauri dev
```

**What happens:**
- Vite dev server with hot-reload at http://localhost:1420/
- Rust backend compiles automatically
- HTML/CSS/JS changes apply immediately
- The app window opens on its own

## Common problems

| Problem | Fix |
|---------|-----|
| `rustc` not found | Restart PowerShell after rustup |
| `npm` not found | Install Node.js and restart PowerShell |
| `yt-dlp` not found | Add the folder with yt-dlp.exe to PATH |
| Python not found | Use `py` instead of `python` |
| MSVC not found | Install Visual Studio Build Tools |
| `Permission denied` | Run PowerShell as Administrator |
| Chrome cookies fail | Chrome installed and signed in to YouTube |
| `Failed to compile` | `cargo clean` and try again |

## PATH (if you need it)

### yt-dlp

```powershell
# Add the yt-dlp folder to PATH
$env:Path += ";C:\path\to\yt-dlp-folder"

# Or permanently: System Properties > Environment Variables
```

### Python

```powershell
# If you installed without "Add to PATH"
# System Properties > Environment Variables > Path > Add:
# C:\Users\YOUR_USERNAME\AppData\Local\Programs\Python\Python311\
# C:\Users\YOUR_USERNAME\AppData\Local\Programs\Python\Python311\Scripts\
```

## Smoke-test the app

After install, exercise the main path:

1. **Launch** (dev or release)
2. **Check the Network Status Bar** at the top:
   - Mode: `direct` / `proxy` / `vpn`
   - External IP: your public IP
   - yt-dlp: version and freshness
3. **Paste a YouTube URL**, e.g. `https://youtu.be/dQw4w9WgXcQ`
4. **Click "Get Info"** — video info should appear
5. **Pick quality** (720p default)
6. **Pick a save folder**
7. **Click "Download"**
8. **Check the progress bar** and the saved file

### YouTube blocks
- Turn on **Auto fallback** in Tools → Mode
- The app tries different strategies (android / tv / web clients)

## Installer (Inno Setup)

To build an `.exe` installer:

1. **Install Inno Setup:**
   ```powershell
   choco install innosetup
   ```

2. **Create an `.iss` file** (example to be added later)

3. **Compile:**
   ```powershell
   iscc installer\youtube-downloader.iss
   ```

## More

- Full build guide: [BUILD.md](BUILD.md)
- Version management: [VERSION_MANAGEMENT.md](VERSION_MANAGEMENT.md)
- Main docs: [README.md](README.md)

## Tips

- **For development** always use `npm run tauri dev` — it is faster
- **For a release** use `npm run tauri build` — it creates `.exe` and `.msi`
- **To bump the version** use `python scripts\version.py bump patch`
- **If it breaks** try `cargo clean`, then rebuild

## Done

YouTube Downloader is ready to use on Windows.
