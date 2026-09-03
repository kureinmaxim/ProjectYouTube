# Changelog

All notable changes to YouTube Downloader. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versioning: [SemVer](https://semver.org/).

## [1.5.1] - 2026-08-30

### Fixed
- Bundle Inter inside the app so a blocked or hanging `fonts.googleapis.com` request cannot leave a blank white window.
- Blank window now explains itself when the UI fails to start.
- `make install-app` copies the release `.app` to `/Applications` so Dock pins survive rebuilds.

### Docs
- [NETWORK_SETUP.md](NETWORK_SETUP.md): Tailscale DNS vs YouTube blocking; three symptoms of a dead resolver.

## [1.4.4] - 2026-01

### Added
- Video codec selection (H.264 / VP9 / AV1).
- Clearer download logging.

### Fixed
- Progress logs every third update (less terminal spam).
- HLS/DASH fragment handling.

## [1.4.3] - 2026-01

### Added
- Real-time download progress in the UI.

## [1.4.2] - 2026-01

### Improved
- Multi-client SABR bypass (try several yt-dlp player clients).

## [1.4.1] - 2026-01-07

### Added
- Detect macOS system HTTP/SOCKS proxy and pass it to yt-dlp.
- Network status bar: TUN / SOCKS5 / direct, external IP, yt-dlp freshness.

### Removed
- Legacy `lux` / `you-get` backends. yt-dlp is the only downloader.

## [1.2.1] - 2026-01

### Added
- DRM / restriction detection and clearer error hints.
- Network diagnostics and proxy auto-detect.

## [0.1.0] - 2026-01-02

### Added
- First release: dark UI, quality picker, progress bar, Chrome cookies, save-folder dialog.
