# Windows: first run

Language: **English** · [Русский](docs/WINDOWS_SETUP_ru.md)

**Version:** 1.5.1

Checklist to install tools and build YouTube Downloader on Windows 10/11.

## Install

1. **Rust** — [rustup-init.exe](https://rustup.rs/), default options, then **restart PowerShell**. `rustc --version` → 1.70+.
2. **Node.js LTS** — [nodejs.org](https://nodejs.org/), “Add to PATH”. `node` 18+, `npm` 8+.
3. **Python 3.10+** — [python.org](https://www.python.org/downloads/), “Add Python to PATH” (`scripts/version.py`).
4. **yt-dlp** — `choco install yt-dlp` or put `yt-dlp.exe` on `PATH`. Keep it current.
5. **ffmpeg** — `choco install ffmpeg` or add `ffmpeg.exe` to `PATH`.
6. **Visual Studio Build Tools** — “Desktop development with C++” (required to compile Rust).
7. **Chrome** (optional) — cookies for private / age-restricted videos.

There is no project `Makefile` workflow on Windows. Use npm / Python directly.

## First build

```powershell
cd youtube-downloader
npm install
npm run tauri build
```

Typical output:

```text
src-tauri\target\release\youtube-downloader.exe
src-tauri\target\release\bundle\msi\youtube-downloader_*_x64_en-US.msi
```

Dev (hot reload):

```powershell
cd youtube-downloader
npm run tauri dev
```

Versions:

```powershell
python scripts\version.py status
python scripts\version.py bump patch
```

## Common problems

| Symptom | What to try |
|---|---|
| `rustc` / `npm` not found | Restart PowerShell after install; confirm PATH |
| `yt-dlp` not found | Install and open a **new** terminal |
| Link / compile errors | Install the VS C++ workload; `cargo clean` in `src-tauri` |
| Cookies fail | Chrome installed and signed into YouTube |
| Timeouts, `IP: N/A` | DNS / VPN — [NETWORK_SETUP.md](NETWORK_SETUP.md) |
| 403 / SABR | [YOUTUBE_BLOCKING.md](YOUTUBE_BLOCKING.md) |

Full build notes: [BUILD.md](BUILD.md).
