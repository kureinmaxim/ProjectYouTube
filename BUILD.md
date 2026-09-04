# 🔨 YouTube Downloader — build and development

Language: **English** · [Русский](BUILD_ru.md)

**Version:** 1.6.5

---

## 📑 Table of contents

- [Requirements](#-requirements)
- [macOS: install and first build](#-macos-install-and-first-build)
- [Windows: install and first build](#-windows-install-and-first-build)
- [Development mode (dev)](#-development-mode-dev)
- [Release build](#-release-build)
- [Version management](#-version-management)
- [Project layout](#-project-layout)
- [Frontend development](#-frontend-development)
- [Backend development (Rust)](#-backend-development-rust)
- [Testing and code checks](#-testing-and-code-checks)
- [Configuration](#-configuration)
- [Optimization](#-optimization)
- [Customization](#-customization)
- [Common problems](#-common-problems)

---

## 🛠️ Requirements

| Tool | Why | Version |
|---|---|---|
| **Node.js** + npm | Frontend (Vite), Tauri CLI, and YouTube n-challenge at runtime (yt-dlp 2026) | 18+ / 8+ |
| **Rust** + Cargo | Native backend | 1.70+ |
| **yt-dlp** | Video downloads | latest |
| **ffmpeg** | Mux video + audio | any |
| **Python** | Only for `scripts/version.py` | 3.10+ |
| **Chrome** | Cookies for private videos (optional) | any |

**Platform-specific:**

| | macOS | Windows |
|---|---|---|
| Compiler | Xcode CLT (`xcode-select --install`) | VS Build Tools → "Desktop development with C++" |
| Package manager | Homebrew | Chocolatey (optional) |

> 👉 Full step-by-step checklists:
> - [MACOS_SETUP.md](MACOS_SETUP.md)
> - [WINDOWS_SETUP.md](WINDOWS_SETUP.md)

---

## 🍎 macOS: install and first build

```bash
# 1. Install tools (if not already present)
brew install node yt-dlp ffmpeg
# Rust: https://rustup.rs/

# 2. Clone and install dependencies
git clone https://github.com/kureinmaxim/ProjectYouTube.git
cd ProjectYouTube/youtube-downloader
npm install

# 3. First build (sanity check)
npm run tauri build

# Output:
# src-tauri/target/release/bundle/macos/youtube-downloader.app
# src-tauri/target/release/bundle/dmg/*.dmg
```

Install into `/Applications` and pin to the Dock:

```bash
cd ..          # back to ProjectYouTube root
make install-app
```

> ⚠️ Do not pin the `.app` from `target/` — it is deleted on every rebuild.

---

## 🪟 Windows: install and first build

### What to install

1. **Rust** — [rustup-init.exe](https://rustup.rs/), default options → **restart PowerShell**
2. **Node.js LTS** — [nodejs.org](https://nodejs.org/), check "Add to PATH" → **restart PowerShell**. Also required at **runtime**: yt-dlp 2026 will not extract YouTube formats without Deno or Node.
3. **Visual Studio Build Tools** — [visualstudio.microsoft.com](https://visualstudio.microsoft.com/downloads/) → select **"Desktop development with C++"**
4. **yt-dlp** — `choco install yt-dlp` or download `yt-dlp.exe` and add to PATH
5. **ffmpeg** — `choco install ffmpeg` or add `ffmpeg.exe` to PATH
6. **Python 3.10+** — [python.org](https://www.python.org/downloads/), "Add to PATH" (only needed for `scripts/version.py`)
7. **Chrome** (optional) — for cookies

### Verify

```powershell
rustc --version    # 1.70+
node --version     # v18+
npm --version      # 8+
yt-dlp --version
python --version   # 3.10+
```

### First build

```powershell
git clone https://github.com/kureinmaxim/ProjectYouTube.git
cd ProjectYouTube\youtube-downloader
npm install
npm run tauri build
```

Output:

```
src-tauri\target\release\youtube-downloader.exe
src-tauri\target\release\bundle\msi\youtube-downloader_*_x64_en-US.msi
```

---

## 🚀 Development mode (dev)

Hot-reload: TypeScript/CSS changes apply instantly; Rust recompiles automatically.

### macOS

```bash
# from the ProjectYouTube root
make dev
```

### Windows

```powershell
cd youtube-downloader
npm run tauri dev
```

The app opens automatically. Frontend: `http://localhost:1420/`.

Frontend only (no Tauri window):

```bash
cd youtube-downloader
npm run dev
```

---

## 📦 Release build

### macOS

```bash
make build
# → youtube-downloader/src-tauri/target/release/bundle/macos/youtube-downloader.app
# → youtube-downloader/src-tauri/target/release/bundle/dmg/*.dmg

make install-app   # copy to /Applications
make run           # launch
make run-verbose   # launch with terminal logs (blank-window debugging)
```

### Windows

```powershell
cd youtube-downloader
npm run tauri build
# → src-tauri\target\release\youtube-downloader.exe
# → src-tauri\target\release\bundle\msi\*.msi
```

---

## 🔢 Version management

Source of truth: `youtube-downloader/package.json`. The script syncs `Cargo.toml` and `tauri.conf.json`.

### macOS (Make)

```bash
make version-status          # current version
make version-bump-patch      # 1.6.0 → 1.6.1
make version-bump-minor      # 1.6.0 → 1.7.0
make version-set v=2.0.0     # specific version
```

### Windows / without Make

```powershell
python scripts\version.py status
python scripts\version.py bump patch
python scripts\version.py set 2.0.0
```

After bumping: update [CHANGELOG.md](CHANGELOG.md), build, tag.

Details: [VERSION_MANAGEMENT.md](VERSION_MANAGEMENT.md)

---

## 📂 Project layout

```
ProjectYouTube/
├── youtube-downloader/           # Tauri app
│   ├── index.html               # HTML UI
│   ├── package.json             # NPM dependencies
│   ├── vite.config.ts           # Vite config
│   ├── src/                     # Frontend
│   │   ├── main.ts             # TypeScript logic
│   │   └── styles.css          # CSS styles
│   └── src-tauri/               # Rust backend
│       ├── Cargo.toml           # Rust dependencies
│       ├── tauri.conf.json      # Tauri config
│       └── src/
│           ├── lib.rs           # Entry module
│           ├── ytdlp.rs         # yt-dlp integration + fallback
│           └── downloader/      # Download module
│               ├── utils.rs     # Network detection (TUN/SOCKS5/IP)
│               ├── tools.rs     # yt-dlp management
│               ├── commands.rs  # Tauri commands
│               └── backends/    # Download backends
├── scripts/
│   └── version.py               # Version management
├── Makefile                     # macOS commands (dev, build, version-*)
└── docs/                        # Documentation (Russian)
```

---

## 🎨 Frontend development

**Stack:** HTML/CSS + TypeScript + Vite + Tauri API

### Files

| File | Purpose |
|---|---|
| `index.html` | UI markup |
| `src/main.ts` | All logic: URL → info → download → progress |
| `src/styles.css` | All styles (CSS variables, dark mode) |

### CSS variables (theme)

```css
:root {
  --color-primary: #8b5cf6;     /* Purple */
  --color-secondary: #ec4899;   /* Pink */
  --bg-primary: #0a0a0f;        /* Dark background */
}
```

Changes apply instantly in `npm run tauri dev`.

---

## 🦀 Backend development (Rust)

### Main files

**`lib.rs`** — entry point, command registration:

```rust
mod ytdlp;
use ytdlp::{get_video_info, download_video, get_formats};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_video_info,
            download_video,
            get_formats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**`ytdlp.rs`** — yt-dlp integration:

| Function | Purpose |
|---|---|
| `get_video_info()` | Fetch video metadata |
| `download_video()` | Download with progress |
| `get_formats()` | Available formats |

### Adding a new command

```rust
// 1. New function in ytdlp.rs
#[tauri::command]
pub async fn new_command(param: String) -> Result<String, String> {
    Ok("Result".to_string())
}

// 2. Register in lib.rs
.invoke_handler(tauri::generate_handler![
    get_video_info, download_video, get_formats,
    new_command,  // ← add
])

// 3. Call from the frontend (main.ts)
const result = await invoke("new_command", { param: "value" });
```

---

## 🧪 Testing and code checks

### Unit tests (Rust)

```bash
cd youtube-downloader/src-tauri
cargo test
cargo test -- --nocapture   # verbose
```

### Lint and format

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

### Manual testing

1. `npm run tauri dev` (or `make dev`)
2. Paste a YouTube URL → **Get Info**
3. Pick quality and folder → **Download**
4. Check the progress bar and the saved file

### Offline asset check (macOS)

```bash
make check-assets   # confirm the UI loads nothing over the network
```

---

## 🔧 Configuration

### tauri.conf.json

```json
{
  "productName": "youtube-downloader",
  "version": "1.6.0",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.icns", "icons/icon.ico"]
  }
}
```

### Cargo.toml (main dependencies)

```toml
[dependencies]
tauri = { version = "2.11", features = ["devtools"] }
tauri-plugin-dialog = "2.7"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
```

---

## 🚀 Optimization

### Smaller binary

```toml
# Cargo.toml
[profile.release]
strip = true          # strip debug symbols
lto = true            # Link Time Optimization
codegen-units = 1     # better optimization
opt-level = "s"       # optimize for size
```

### DevTools

In dev mode Chrome DevTools are available: right-click → Inspect Element or F12.

### Logging

```rust
// Rust
println!("Debug: {:?}", value);
eprintln!("Error: {}", error);
```

```typescript
// TypeScript
console.log("Info:", info);
console.error("Error:", error);
```

---

## 🎨 Customization

### Color scheme

In `src/styles.css`:

```css
:root {
  --color-primary: #8b5cf6;      /* your color */
  --color-secondary: #ec4899;    /* your color */
  --bg-primary: #0a0a0f;         /* your color */
}
```

### New quality preset

In `src-tauri/src/ytdlp.rs`:

```rust
let format_arg = match quality.as_str() {
    "best" => "bestvideo+bestaudio/best",
    "1080p" => "bestvideo[height<=1080]+bestaudio/best[height<=1080]",
    "custom" => "YOUR_FORMAT_HERE",  // ← add
    _ => "best",
};
```

In `index.html`:

```html
<option value="custom">🎬 Custom Quality</option>
```

---

## 🐛 Common problems

### General

| Problem | Fix |
|---|---|
| `yt-dlp not found` | Install and verify: `yt-dlp --version` |
| Chrome cookies fail | Chrome must be installed and signed into YouTube |
| Rust compile error | `cd src-tauri && cargo clean`, then rebuild |
| `version mismatched Tauri packages` | npm and Cargo drifted. Rust crates in `src-tauri/Cargo.toml` must share a minor with `@tauri-apps/*` in `package.json` (e.g. `tauri = "2.11"` with `@tauri-apps/api` `^2.11`). Then `cargo update -p tauri -p tauri-plugin-dialog -p tauri-plugin-opener`. |
| Frontend does not refresh | Delete `node_modules/.vite`, restart `npm run tauri dev` |
| Permission denied on download | Pick another writable folder |

### macOS

| Problem | Fix |
|---|---|
| Blank white window | See [MACOS_SETUP.md](MACOS_SETUP.md) → "Blank white window" |
| `xcrun: error` | `xcode-select --install` |
| `command not found: rustc` | Restart the terminal after installing Rust |
| `IP: N/A` / timeouts | Broken DNS — [NETWORK_SETUP.md](NETWORK_SETUP.md) |

### Windows

| Problem | Fix |
|---|---|
| `rustc` not found | Restart PowerShell after installing rustup |
| `npm` not found | Restart PowerShell after installing Node.js |
| Download fails: `format is not available` / `[Errno 22]` | Node.js (or Deno) is required at runtime, not only to build. Use 1.6.3+. See [WINDOWS_SETUP.md](WINDOWS_SETUP.md) |
| MSVC / linker not found | Install VS Build Tools → "Desktop development with C++" |
| `python` not found | Try `py` instead of `python` |

---

**Developer:** Kurein M.N.
