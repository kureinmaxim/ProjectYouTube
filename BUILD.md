# YouTube Downloader — build and development

Language: **English** · [Русский](BUILD_ru.md)

Guide to developing, building, and running YouTube Downloader.

---

## For developers (Quick Start)

### macOS — Dev Mode

```bash
# Dev mode — fast rebuild with hot-reload
cd youtube-downloader
npm run tauri dev

# The app starts automatically
# Frontend: http://localhost:1420/
# Backend: Rust with hot-reload
```

### macOS — Build Mode (release)

```bash
# Full build — .app and .dmg
cd youtube-downloader
npm run tauri build

# Output:
# src-tauri/target/release/bundle/macos/youtube-downloader.app
# src-tauri/target/release/bundle/dmg/youtube-downloader_X.X.X_aarch64.dmg
```

### Useful dev commands

```bash
# Install dependencies
cd youtube-downloader
npm install

# Check Rust
cd src-tauri
cargo check
cargo clippy -- -D warnings
cargo fmt

# Tests
cargo test

# Clean
cargo clean
```

---

## Project layout

```
youtube-downloader/
├── index.html              # HTML UI
├── package.json            # NPM dependencies
├── src/                    # Frontend
│   ├── main.ts            # TypeScript logic
│   └── styles.css         # CSS
├── src-tauri/              # Rust backend
│   ├── Cargo.toml         # Rust dependencies
│   ├── tauri.conf.json    # Tauri config
│   └── src/
│       ├── lib.rs         # Entry module
│       └── ytdlp.rs       # yt-dlp integration
└── vite.config.ts         # Vite config
```

---

## Requirements

### macOS

```bash
# Check the tools you need
rustc --version    # Rust 1.70+
cargo --version    # Cargo
node --version     # Node.js 18+
npm --version      # npm 8+
yt-dlp --version   # yt-dlp (downloads)
ffmpeg -version    # ffmpeg (mux video + audio)
```

### Installing missing tools

> **First time on this project?** Use the platform guides:
>
> - **macOS:** [MACOS_SETUP.md](MACOS_SETUP.md) — step-by-step install
> - **Windows:** [WINDOWS_SETUP.md](WINDOWS_SETUP.md) — step-by-step install

Those guides cover every tool and the first build.

#### Quick install (macOS)

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js (Homebrew)
brew install node

# yt-dlp
brew install yt-dlp
# or
curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o ~/bin/yt-dlp
chmod +x ~/bin/yt-dlp

# ffmpeg (mux video + audio)
brew install ffmpeg
```

---

## First-time setup

### Clone and install

```bash
# 1. Enter the app folder
cd youtube-downloader

# 2. Install npm dependencies
npm install

# 3. First build (sanity check)
npm run tauri build
```

---

## Frontend development

### Stack
- **HTML/CSS** — structure and styles
- **TypeScript** — app logic
- **Vite** — dev server with hot-reload
- **Tauri API** — backend integration

### Start the dev server

```bash
cd youtube-downloader
npm run dev  # Frontend only, no Tauri

# or

npm run tauri dev  # Frontend + Tauri backend
```

### Editing styles

All styles live in `src/styles.css`. Changes apply on save.

```css
/* Main CSS variables */
:root {
  --color-primary: #8b5cf6;     /* Purple */
  --color-secondary: #ec4899;   /* Pink */
  --bg-primary: #0a0a0f;        /* Dark background */
  /* ... */
}
```

---

## Backend development (Rust)

### Main files

**lib.rs** — entry point
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

**ytdlp.rs** — yt-dlp integration
- `get_video_info()` — fetch video metadata
- `download_video()` — download with progress
- `get_formats()` — available formats

### Adding a new command

```rust
// 1. Add a function in ytdlp.rs
#[tauri::command]
pub async fn new_command(param: String) -> Result<String, String> {
    Ok("Result".to_string())
}

// 2. Register it in lib.rs
.invoke_handler(tauri::generate_handler![
    get_video_info,
    download_video,
    get_formats,
    new_command,  // ← add
])

// 3. Call it from the frontend (main.ts)
const result = await invoke("new_command", { param: "value" });
```

---

## Testing

### Manual testing in dev mode

```bash
cd youtube-downloader
npm run tauri dev

# In the window:
# 1. Paste a YouTube URL
# 2. Click "Get Info"
# 3. Check the video preview
# 4. Pick quality and folder
# 5. Download
```

### Unit tests (Rust)

```bash
cd src-tauri
cargo test

# Verbose
cargo test -- --nocapture
```

### Code checks

```bash
# Lint
cargo clippy -- -D warnings

# Format
cargo fmt --check
```

---

## Release build

### macOS

```bash
cd youtube-downloader
npm run tauri build

# Output:
# src-tauri/target/release/bundle/macos/youtube-downloader.app
# src-tauri/target/release/bundle/dmg/youtube-downloader_X.X.X_aarch64.dmg
```

### Testing the release build

```bash
# Run the .app
open src-tauri/target/release/bundle/macos/youtube-downloader.app

# Or install the .dmg
open src-tauri/target/release/bundle/dmg/youtube-downloader_X.X.X_aarch64.dmg
```

---

## Configuration

### tauri.conf.json

Main app settings:

```json
{
  "productName": "youtube-downloader",
  "version": "1.5.1",
  "identifier": "com.olgazaharova.youtube-downloader",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

### Cargo.toml

Rust dependencies:

```toml
[dependencies]
tauri = { version = "2", features = ["devtools"] }
tauri-plugin-dialog = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
```

---

## Common problems

### "yt-dlp not found"

```bash
# Check the install
yt-dlp --version

# Install if missing
brew install yt-dlp

# Or hard-code the path in ytdlp.rs
Command::new("/usr/local/bin/yt-dlp")
```

### Chrome cookies do not work

```bash
# Chrome must be installed and you must be signed in to YouTube
# yt-dlp looks in:
# ~/Library/Application Support/Google/Chrome/Default/Cookies (macOS)
```

### Rust compile error

```bash
# Clean and rebuild
cd src-tauri
cargo clean
cd ..
npm run tauri build
```

### Frontend does not refresh

```bash
# Clear the Vite cache
rm -rf node_modules/.vite
npm run tauri dev
```

### Permission denied while downloading

```bash
# Check Downloads permissions
ls -la ~/Downloads

# Or pick another writable folder
```

---

## Performance monitoring

### Dev Tools

In dev mode, Chrome DevTools are available:
- Right click → Inspect Element
- Or F12

### Logging

```rust
// In Rust
println!("Debug: {:?}", value);
eprintln!("Error: {}", error);

// In TypeScript
console.log("Info:", info);
console.error("Error:", error);
```

---

## Optimization

### App size

```bash
# Bundle size
du -sh src-tauri/target/release/bundle/macos/youtube-downloader.app

# To shrink it:
# 1. Use strip in Cargo.toml
# 2. Enable LTO (Link Time Optimization)
```

### Cargo.toml optimizations

```toml
[profile.release]
strip = true          # Strip debug symbols
lto = true           # Link Time Optimization
codegen-units = 1    # Better optimization
opt-level = "s"      # Optimize for size
```

---

## Project dependencies

### NPM packages

```json
{
  "@tauri-apps/api": "^2.x",
  "@tauri-apps/plugin-dialog": "^2.x"
}
```

### Rust crates

```toml
tauri = "2"
tauri-plugin-dialog = "2"
serde = "1"
serde_json = "1"
tokio = "1"
```

### External tools

- **yt-dlp** — video download
- **Google Chrome** — cookies (optional)

---

## Workflow

### Daily development

```bash
# 1. Start dev mode
cd youtube-downloader
npm run tauri dev

# 2. Edit code
# - main.ts for logic
# - styles.css for styles
# - ytdlp.rs for backend

# 3. Test (hot-reload)

# 4. Commit
git add -A
git commit -m "feat: add a new feature"
git push
```

### Preparing a release

```bash
# 1. Bump the version
# - package.json
# - src-tauri/Cargo.toml
# - src-tauri/tauri.conf.json

# 2. Build
npm run tauri build

# 3. Test the .app

# 4. Tag the release
git tag -a v1.5.2 -m "Release v1.5.2"
git push origin v1.5.2
```

---

## Support

If something is wrong:
1. Confirm yt-dlp is installed: `yt-dlp --version`
2. Confirm Chrome is installed (for cookies)
3. Clean the cache: `cargo clean`
4. Rebuild: `npm run tauri build`
5. Check the terminal logs

**Developer:** Kurein M.N.

---

## Customization

### Change the color scheme

In `src/styles.css`:

```css
:root {
  --color-primary: #8b5cf6;      /* Purple → your color */
  --color-secondary: #ec4899;    /* Pink → your color */
  --bg-primary: #0a0a0f;         /* Dark background → your color */
}
```

### Add a new quality preset

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
<select id="quality-select">
  <option value="custom">🎬 Custom Quality</option>
</select>
```

---

Happy hacking.
