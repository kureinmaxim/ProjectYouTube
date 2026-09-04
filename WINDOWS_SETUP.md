# Windows: first run

Language: **English** · [Русский](docs/WINDOWS_SETUP_ru.md)

**Version:** 1.6.3 | **Updated:** 2026-09-04

Short guide for the first YouTube Downloader build on Windows.

## Install checklist

### 1. Rust
- [ ] Download [rustup-init.exe](https://rustup.rs/)
- [ ] Install (default options)
- [ ] Restart PowerShell
- [ ] Check: `rustc --version` (1.70+)

### 2. Node.js (build **and** YouTube downloads)
- [ ] Download LTS from [nodejs.org](https://nodejs.org/)
- [ ] Install with “Add to PATH”
- [ ] Restart PowerShell
- [ ] Check: `node --version` (v18+)
- [ ] Check: `npm --version` (8+)

yt-dlp 2026 needs a JavaScript runtime to solve YouTube’s n-challenge. It enables
only Deno by default. This app passes `--js-runtimes` for Node / Deno / Bun when
it finds one. Without that, Get Info may work and every download strategy fails
with `Requested format is not available` — it looks like a block, it is not.
Deno is an alternative (`scoop install deno`).

### 3. Python (version scripts)
- [ ] Download 3.10+ from [python.org](https://www.python.org/downloads/)
- [ ] Install with “Add Python to PATH”
- [ ] Restart PowerShell
- [ ] Check: `python --version` or `py --version`

### 4. yt-dlp and ffmpeg — the app installs these

Nothing to do before the first run. Launch the app, open the **Tools** panel and click
**Install** next to each tool. Binaries are downloaded from GitHub releases into
`%LOCALAPPDATA%\youtube-downloader\bin` — no package manager, no admin rights, no PATH
change and no restart. ffmpeg is ~170 MB, so its progress is shown in the log.

ffmpeg is what merges video and audio; without it, quality above 720p will not download.

Prefer to install them yourself? Any of these work, and the app finds and uses them.
Scoop shims (`scoop\shims\ffmpeg.exe`) are followed to the real binary — do not
point `--ffmpeg-location` at the shim folder yourself.

- [ ] `winget install yt-dlp.yt-dlp` and `winget install Gyan.FFmpeg`
- [ ] `choco install yt-dlp ffmpeg`
- [ ] `scoop install yt-dlp ffmpeg`
- [ ] Or put the `.exe` files anywhere on PATH
- [ ] Check: `yt-dlp --version` and `ffmpeg -version`

**Update** in the Tools panel only replaces a copy the app installed itself. If the tool
came from winget / Chocolatey / Scoop, the app names the owner and shows the command that
updates it rather than downloading over it.

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
# yt-dlp and ffmpeg are optional here — the app can install them itself

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
| `yt-dlp` not found | Open **Tools** and click **Install** (or add the folder with yt-dlp.exe to PATH) |
| Quality above 720p fails | ffmpeg is missing — install it from the **Tools** panel |
| `ffmpeg is not installed` but `ffmpeg -version` works | Scoop shim. 1.6.3+ unwraps it. Or install ffmpeg from **Tools** |
| Get Info works, every strategy fails (`format is not available` / `[Errno 22]`) | Not a YouTube block. Need Node.js or Deno at runtime, and build 1.6.3+. See [YOUTUBE_BLOCKING.md](YOUTUBE_BLOCKING.md) |
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
