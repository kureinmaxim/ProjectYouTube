# YouTube Downloader build guide

Language: **English** · [Русский](BUILD_ru.md)

How to develop and ship the **macOS** `.app` / `.dmg` and the **Windows** `.exe` / `.msi`.

Related: [MACOS_SETUP.md](MACOS_SETUP.md) · [WINDOWS_SETUP.md](WINDOWS_SETUP.md) · [VERSION_MANAGEMENT.md](VERSION_MANAGEMENT.md)

## Requirements

| Tool | Why |
|---|---|
| **Node.js 18+** / npm | Frontend (Vite) and Tauri CLI |
| **Rust 1.70+** / Cargo | Native backend |
| **yt-dlp** on `PATH` | Downloads |
| **ffmpeg** on `PATH` | Mux video + audio |
| **Python 3.10+** | `scripts/version.py` only |
| **Xcode CLT** (macOS) | `xcode-select --install` |
| **VS Build Tools** (Windows) | C++ workload for Rust |

Chrome is optional (cookies for private / age-restricted videos).

## Dev (hot reload)

From the repo root:

```bash
# macOS
make dev
```

```powershell
# Windows
cd youtube-downloader
npm install
npm run tauri dev
```

Frontend: `http://localhost:1420/`. Rust rebuilds on change.

Frontend only (no window): `cd youtube-downloader && npm run dev`.

## Release build

### macOS

```bash
make build
# youtube-downloader/src-tauri/target/release/bundle/macos/youtube-downloader.app
# youtube-downloader/src-tauri/target/release/bundle/dmg/*.dmg

make install-app   # copy .app to /Applications
make run           # open the installed copy
make run-verbose   # logs in the terminal (blank-window debugging)
```

Pin the **`/Applications`** copy in the Dock. The bundle under `target/` is deleted on every rebuild and on `make clean`.

### Windows

```powershell
cd youtube-downloader
npm install
npm run tauri build
```

Typical output:

- `src-tauri\target\release\youtube-downloader.exe`
- `src-tauri\target\release\bundle\msi\youtube-downloader_*_x64_en-US.msi`

## Version bump (do not edit by hand)

Source of truth: `youtube-downloader/package.json`. The script updates `Cargo.toml` and `tauri.conf.json`.

```bash
make version-status
make version-bump-patch    # 1.5.1 → 1.5.2
make version-set v=1.6.0
```

Windows / without Make:

```text
python scripts/version.py status
python scripts/version.py bump patch
python scripts/version.py set 1.6.0
```

Then add a `[X.Y.Z]` heading in [CHANGELOG.md](CHANGELOG.md) and tag the release.

## Checks

```bash
cd youtube-downloader/src-tauri
cargo check
cargo clippy -- -D warnings
cargo fmt --check
cargo test
```

macOS: `make check-assets` — built UI must not load render-blocking files from the network.

## Layout

```text
youtube-downloader/
├── index.html
├── src/                 # TypeScript + CSS
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/             # lib.rs, ytdlp.rs, downloader/
└── vite.config.ts
```

## Common problems

**`yt-dlp not found`** — install it and confirm `yt-dlp --version` in a new terminal.

**Chrome cookies fail** — Chrome installed, signed into YouTube. macOS cookies: `~/Library/Application Support/Google/Chrome/Default/Cookies`.

**Rust compile error** — `cd youtube-downloader/src-tauri && cargo clean`, then rebuild.

**Frontend does not refresh** — delete `youtube-downloader/dist` / Vite cache and restart `npm run tauri dev`.

**Blank white window (macOS)** — [MACOS_SETUP.md](MACOS_SETUP.md) (“Blank white window”). Usually a Dock pin to `target/` or a pre-1.5.1 build waiting on Google Fonts.

**`IP: N/A` / `Could not resolve host`** — broken system DNS, often a VPN/Tailscale exit node. [NETWORK_SETUP.md](NETWORK_SETUP.md).

## Support

1. `yt-dlp --version` and `ffmpeg -version`
2. `make clean` (or `cargo clean`) and rebuild
3. [Issues](https://github.com/kureinmaxim/ProjectYouTube/issues) with OS, app version, and logs
