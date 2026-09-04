# macOS: first run

Language: **English** · [Русский](docs/MACOS_SETUP_ru.md)

**Version:** 1.6.1 | **Updated:** 2026-09-04

Short guide for the first YouTube Downloader build on macOS.

## Install checklist

### 1. Homebrew
- [ ] Install Homebrew: `/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"`
- [ ] Check: `brew --version`

### 2. Rust
- [ ] Install rustup: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- [ ] Choose the default installation
- [ ] Restart the terminal
- [ ] Check: `rustc --version` (1.70+)

### 3. Node.js
- [ ] Install via Homebrew: `brew install node`
- [ ] Check: `node --version` (v18+)
- [ ] Check: `npm --version` (8+)

### 4. Python (version scripts)
- [ ] Install via Homebrew: `brew install python@3.11`
- [ ] Check: `python3 --version` (3.10+)

### 5. yt-dlp (the only download tool)
- [ ] Install via Homebrew: `brew install yt-dlp`
- [ ] Or manually:
  ```bash
  curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o ~/bin/yt-dlp
  chmod +x ~/bin/yt-dlp
  ```
- [ ] Check: `yt-dlp --version`
- [ ] **Keep yt-dlp current** (`brew upgrade yt-dlp`) — the app shows how fresh the binary is

### 6. ffmpeg (mux video + audio)
- [ ] Install via Homebrew: `brew install ffmpeg`
- [ ] Check: `ffmpeg -version`

Both tools also appear in the app's **Tools** panel with their status, and **Install**
there runs the Homebrew command for you. **Update** only touches a copy the app installed
itself — a Homebrew install is named and left to `brew upgrade`.

### 7. Google Chrome (optional, cookies)
- [ ] Install from [google.com/chrome](https://www.google.com/chrome/)
- [ ] Sign in to YouTube for private videos

## First build

Open Terminal in the project folder:

```bash
# 1. From the repo root

# 2. Check the tools
rustc --version    # 1.70+
node --version     # v18+
npm --version      # 8+
python3 --version  # 3.10+
yt-dlp --version   # should print a version
ffmpeg -version    # should print a version

# 3. Install npm dependencies
cd youtube-downloader
npm install

# 4. First Rust build (can take several minutes)
npm run tauri build

# Output:
# src-tauri/target/release/bundle/macos/youtube-downloader.app
# src-tauri/target/release/bundle/dmg/*.dmg
```

## Verify

After a successful build:

```bash
# Check artifacts
ls -lh src-tauri/target/release/bundle/macos/youtube-downloader.app
ls -lh src-tauri/target/release/bundle/dmg/*.dmg

# Launch the app
open src-tauri/target/release/bundle/macos/youtube-downloader.app
```

## Quick commands (Makefile)

```bash
# Dev mode (hot reload) — daily work
# from the repo root
make dev

# Production build — for a release
make build

# Install the .app into /Applications (pin THAT copy in the Dock)
make install-app

# Launch the installed app
make run

# Launch with logs in the terminal (blank window)
make run-verbose

# Version
make version-status

# Clean artifacts
make clean
```

## Dev mode

For day-to-day work use dev mode:

```bash
# from the repo root
make dev

# Or directly:
cd youtube-downloader
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
| `command not found: rustc` | Restart the terminal after rustup |
| `command not found: npm` | Install Node.js: `brew install node` |
| `command not found: yt-dlp` | Install: `brew install yt-dlp` |
| `Permission denied` | Check permissions or `chmod +x` |
| `xcrun: error` | Install Xcode CLT: `xcode-select --install` |
| Chrome cookies fail | Chrome installed and signed in to YouTube |
| `Failed to compile` | Clean: `cd youtube-downloader && cargo clean` |
| `Could not resolve host` / `IP: N/A` in the app | Broken system DNS — [NETWORK_SETUP.md](NETWORK_SETUP.md) |
| Blank white window on launch | See “The app opens as a blank white window” |

## The app opens as a blank white window

The title bar says “Downloader”, but the body is empty and white. Rust started; the web UI did not paint. The real UI is dark, so **white means the page never rendered at all** (broken styles would show black text on white).

### Cause 1: the network blocks an external font (fixed in v1.5.1)

Before 1.5.1, `index.html` loaded Inter from `fonts.googleapis.com`. That `<link rel="stylesheet">` **blocks first paint**: until it loads or fails, the engine draws nothing.

What you see depends on *how* the network fails:

| Network behavior | What you see |
|------------------|--------------|
| Google is reachable | UI in ~0.15 s |
| DNS immediately says “no such host” | UI in ~0.15 s (system font) |
| The request **hangs** (DPI filter, dead proxy, VPN tunnel with no exit) | **white window until timeout** |

The third case is typical on Russian ISPs and a half-up VPN. That is why it “works on one Wi-Fi and not the other”: the app did not change, the answer to Google did.

**Fix:** Inter now ships inside the app (`@fontsource-variable/inter`). The UI no longer needs a network to paint. Update and rebuild:

```bash
# from the repo root
git pull
make build
make install-app
```

**Emergency workaround for an already-installed old build** (no rebuild) — make the request fail immediately instead of hanging:

```bash
printf "0.0.0.0 fonts.googleapis.com\n0.0.0.0 fonts.gstatic.com\n" | sudo tee -a /etc/hosts
sudo dscacheutil -flushcache; sudo killall -HUP mDNSResponder
```

The UI appears at once, with the system font. After updating to 1.5.1+ you can remove those lines from `/etc/hosts`.

### Cause 2: the Dock pin is a dev build

`make dev` launches a binary from `target/debug/` that loads the UI from the Vite server at `http://localhost:1420`. Pin that binary, launch it without the dev server — there is nothing to load, empty window again.

Check what is running (while the empty window is open):

```bash
ps -Ao pid,command | grep -i youtube-downloader | grep -v grep
```

- path contains `/target/debug/` → dev build, you need `make dev` running;
- path contains `/Applications/` or `/target/release/` → release, see causes 1 and 3.

### Cause 3: the Dock icon points into `target/`

The `.app` under `src-tauri/target/release/bundle/macos/` is deleted on every rebuild and on `make clean` — the Dock link points at nothing or a half-written bundle.

**Fix:** `make install-app` and pin the copy from `/Applications`; it survives rebuilds.

### If that did not help

```bash
# launch in the terminal — startup errors show immediately
make run-verbose

# the bundle must not load render-blocking external assets
make check-assets

# is the bundle intact
ls -l /Applications/youtube-downloader.app/Contents/MacOS/
```

If the UI loaded but JS never ran, the app writes that in the window (“Interface did not start”) instead of a blank screen.

## System DNS does not resolve (VPN / Tailscale exit node)

One failure, three symptoms — easy to treat as three different bugs:

| Where | What you see |
|-------|----------------|
| App | `Network timeout (possible IP throttling)`, status bar `IP: N/A` |
| Build | `Could not resolve host: static.crates.io`, `make build` hangs on crates |
| Before 1.5.1 | blank white window (the font request hit the same dead DNS) |

30-second check:

```bash
dig +time=3 +tries=1 @1.1.1.1 www.youtube.com +short
```
```bash
curl -sS -o /dev/null -w "%{http_code}\n" --max-time 10 https://www.youtube.com
```

**The explicit resolver returns addresses, `curl` says `Resolving timed out`** — the network is fine, the system resolver is broken. See who replaced it:

```bash
scutil --dns | head -8
```

A line `if_index : NN (utunN)` means DNS is forced by a VPN tunnel, and addresses from `networksetup -setdnsservers Wi-Fi …` are ignored — the tunnel resolver has higher priority.

Where the settings live and how to fix it for good: **[NETWORK_SETUP.md](NETWORK_SETUP.md)**.

Quick unblock if you need to build right now:

```bash
sudo tailscale set --exit-node=
```
```bash
sudo dscacheutil -flushcache; sudo killall -HUP mDNSResponder
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

## More

- Network, VPN, DNS: [NETWORK_SETUP.md](NETWORK_SETUP.md)
- Full build guide: [BUILD.md](BUILD.md)
- Version management: [VERSION_MANAGEMENT.md](VERSION_MANAGEMENT.md)
- Main docs: [README.md](README.md)

## Tips

- **For development** always use `make dev` — it is faster
- **For a release** use `make build` — it creates `.app` and `.dmg`
- **To bump the version** use `make version-bump-*`
- **If it breaks** try `make clean`, then rebuild

## Done

YouTube Downloader is ready to use on macOS.
