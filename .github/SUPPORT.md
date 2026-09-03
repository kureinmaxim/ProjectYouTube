# Support

## Docs

- [README](../README.md) — install, use, docs index
- [README (Russian)](../README_ru.md)
- [Build](../BUILD.md)
- [YouTube blocking](../YOUTUBE_BLOCKING.md)
- [Network / DNS](../NETWORK_SETUP.md)
- [Changelog](../CHANGELOG.md)

## Common problems

### Blank white window (macOS)

Pin `/Applications/youtube-downloader.app` after `make install-app`. Do not pin `target/` or a `make dev` binary. Details: [MACOS_SETUP.md](../MACOS_SETUP.md).

### Timeouts / `IP: N/A`

Usually broken DNS (VPN / Tailscale exit node), not a YouTube ban. [NETWORK_SETUP.md](../NETWORK_SETUP.md).

### 403 / SABR / hang

Try **Tools → Player client = all**, then a PO Token (`mweb`). [YOUTUBE_BLOCKING.md](../YOUTUBE_BLOCKING.md).

### `yt-dlp not found`

Install yt-dlp and ffmpeg so they are on `PATH`. Open a new terminal after installing.

## Where to ask

- Bugs: [Issues](https://github.com/kureinmaxim/ProjectYouTube/issues)
- Docs: [docs/INDEX_ru.md](../docs/INDEX_ru.md) (Russian long-form)

When you open an issue, include app version, OS, status-bar line (mode / IP / yt-dlp), and steps to reproduce.
