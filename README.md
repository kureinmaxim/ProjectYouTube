# YouTube Downloader v1.5.1

Language: **English** · [Русский](README_ru.md)

<p align="center">
  <img src="youtube-downloader/src-tauri/icons/128x128@2x.png" alt="YouTube Downloader" width="128">
</p>

<p align="center">
  <strong>A local desktop app for downloading YouTube videos.</strong><br>
  Paste a link, pick quality, save the file — Tauri + Rust + yt-dlp, no account, no cloud.
</p>

<p align="center">
  <a href="https://github.com/kureinmaxim/ProjectYouTube/releases"><img src="https://img.shields.io/github/v/release/kureinmaxim/ProjectYouTube?style=flat-square" alt="Latest release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-0F766E?style=flat-square" alt="MIT License"></a>
  <img src="https://img.shields.io/badge/tauri-2-FFC131?style=flat-square" alt="Tauri 2">
  <img src="https://img.shields.io/badge/yt--dlp-required-FF0000?style=flat-square" alt="yt-dlp">
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows-1F2937?style=flat-square" alt="macOS and Windows">
</p>

YouTube Downloader is a native window, not a website. Downloads run on your machine through [yt-dlp](https://github.com/yt-dlp/yt-dlp). The UI is English, dark, and works offline (fonts and styles are bundled).

## What you get

| | |
|---|---|
| **Paste and download** | Preview the video, pick Best / 1080p / 720p / 480p / MP3, choose a folder, watch a live progress bar. |
| **Quality and codec** | H.264, VP9, or AV1 when the stream offers them. |
| **Private / age-gated** | Optional Chrome cookies (`--cookies-from-browser chrome`). |
| **YouTube blocks** | Auto fallback across player clients (android → tv → web). Tools panel: player client, PO Token. |
| **Network status** | TUN / SOCKS5 / system proxy / direct, plus external IP and yt-dlp freshness. |
| **Offline UI** | No Google Fonts. A blocked network cannot leave a blank white window. |

## Quick start

**Needs:** Node.js 18+, Rust 1.70+, [yt-dlp](https://github.com/yt-dlp/yt-dlp), [ffmpeg](https://ffmpeg.org/) (to mux video + audio). Chrome is optional (cookies).

### macOS

```bash
git clone https://github.com/kureinmaxim/ProjectYouTube.git
cd ProjectYouTube

brew install node yt-dlp ffmpeg
# Rust: https://rustup.rs/

cd youtube-downloader
npm install
cd ..
make dev
```

Release `.app` / `.dmg`: `make build`. Pin the copy from `/Applications` (`make install-app`), not the bundle under `target/`.

First-time macOS checklist: [MACOS_SETUP.md](MACOS_SETUP.md).

### Windows

```powershell
git clone https://github.com/kureinmaxim/ProjectYouTube.git
cd ProjectYouTube\youtube-downloader

npm install
npm run tauri dev
```

Install yt-dlp and ffmpeg so they are on `PATH` (Chocolatey: `choco install yt-dlp ffmpeg`). Visual Studio Build Tools with the C++ workload are required to compile Rust.

First-time Windows checklist: [WINDOWS_SETUP.md](WINDOWS_SETUP.md).

## Use it

1. Open the app.
2. Paste a YouTube URL.
3. Click **Get Info** — title and thumbnail appear.
4. Pick quality (default 720p) and a save folder.
5. Click **Download** and wait for the progress bar.

If a download hangs on SABR / 403, open **Tools**: set **Player client** to `all`, and if YouTube asks for a GVS token, paste a **PO Token** (`mweb`). Official guide: [yt-dlp PO Token](https://github.com/yt-dlp/yt-dlp/wiki/PO-Token-Guide). Symptom list: [YOUTUBE_BLOCKING.md](YOUTUBE_BLOCKING.md).

If the status bar shows `IP: N/A` and everything times out, that is usually **DNS**, not a YouTube ban. See [NETWORK_SETUP.md](NETWORK_SETUP.md).

## Docs

| Doc | Language |
|---|---|
| [Build guide](BUILD.md) · [RU](BUILD_ru.md) | EN / RU |
| [macOS setup](MACOS_SETUP.md) · [RU](docs/MACOS_SETUP_ru.md) | EN / RU |
| [Windows setup](WINDOWS_SETUP.md) · [RU](docs/WINDOWS_SETUP_ru.md) | EN / RU |
| [Version management](VERSION_MANAGEMENT.md) · [RU](VERSION_MANAGEMENT_ru.md) | EN / RU |
| [Network / DNS / VPN](NETWORK_SETUP.md) · [RU](docs/NETWORK_SETUP_ru.md) | EN / RU |
| [YouTube blocking (SABR, 403, PO Token)](YOUTUBE_BLOCKING.md) | English |
| [Changelog](CHANGELOG.md) | English |
| [Documentation index](docs/INDEX_ru.md) | Russian (architecture, roadmap, deep dives) |

Bugs: [Issues](https://github.com/kureinmaxim/ProjectYouTube/issues). Questions: open a discussion or an issue.

## Project layout

```text
ProjectYouTube/
├── youtube-downloader/     # Tauri app
│   ├── src/                # TypeScript UI
│   ├── src-tauri/          # Rust backend (yt-dlp, network, commands)
│   └── index.html
├── scripts/version.py      # Keep package.json / Cargo.toml / tauri.conf in sync
├── Makefile                # macOS: dev, build, install-app, version-*
└── docs/                   # Russian long-form docs
```

## License

[MIT](LICENSE)

---

Kurein M.N.
