# macOS: first run

Language: **English** · [Русский](docs/MACOS_SETUP_ru.md)

**Version:** 1.5.1

Checklist to install tools and build YouTube Downloader on macOS 11+.

## Install

1. **Homebrew** — https://brew.sh
2. **Rust** — https://rustup.rs then restart the terminal. `rustc --version` → 1.70+.
3. **Node.js** — `brew install node` (`node` 18+, `npm` 8+).
4. **Python 3.10+** — `brew install python` (only for `scripts/version.py`).
5. **yt-dlp** — `brew install yt-dlp` (keep it current: `brew upgrade yt-dlp`).
6. **ffmpeg** — `brew install ffmpeg`.
7. **Xcode CLT** if `xcrun` fails — `xcode-select --install`.
8. **Chrome** (optional) — for cookies on private / age-restricted videos.

## First build

From the **repo root** (not a hardcoded home path):

```bash
cd youtube-downloader
npm install
cd ..
make build
# youtube-downloader/src-tauri/target/release/bundle/macos/youtube-downloader.app
# youtube-downloader/src-tauri/target/release/bundle/dmg/*.dmg
```

Daily development: `make dev` (Vite + Rust hot reload).

Pin **`/Applications/youtube-downloader.app`** in the Dock after `make install-app`. Do not pin `target/` or a `make dev` binary.

| Command | What it does |
|---|---|
| `make dev` | Dev window, hot reload |
| `make build` | Release `.app` + `.dmg` |
| `make install-app` | Copy `.app` to `/Applications` |
| `make run` / `make run-verbose` | Launch installed app |
| `make version-status` | Print synced versions |
| `make clean` | Delete build artifacts |

## Blank white window

The title bar says “Downloader” but the body is white. Rust started; the web UI did not paint. The real UI is dark — white means **nothing rendered**.

**Cause 1 (fixed in 1.5.1):** older builds loaded Inter from Google Fonts. A hanging DNS/proxy to `fonts.googleapis.com` blocked first paint. Rebuild on 1.5.1+ (`make build && make install-app`).

**Cause 2:** Dock pin to a **dev** binary (`target/debug/`). That UI comes from `http://localhost:1420`. Without `make dev`, the window is empty.

**Cause 3:** Dock pin into `target/release/bundle/macos/`. That folder is wiped on every rebuild.

If it still fails: `make run-verbose` and `make check-assets`.

## DNS looks like a YouTube block

`IP: N/A` in the status bar, `Network timeout`, or `Could not resolve host` during `make build` is often a **dead system resolver** (Tailscale exit node with a private DNS), not SABR.

```bash
dig +time=3 +tries=1 @1.1.1.1 www.youtube.com +short
curl -sS -o /dev/null -w "%{http_code}\n" --max-time 10 https://www.youtube.com
```

Explicit resolver works, `curl` says `Resolving timed out` → [NETWORK_SETUP.md](NETWORK_SETUP.md). Quick unblock: `sudo tailscale set --exit-node=` then flush DNS.

Real YouTube 403 / SABR: [YOUTUBE_BLOCKING.md](YOUTUBE_BLOCKING.md).

## Smoke test

1. Status bar: mode, IP, yt-dlp version.
2. Paste a public URL → **Get Info**.
3. Download 720p into a writable folder.

Full build notes: [BUILD.md](BUILD.md).
