# Changelog

All notable changes to YouTube Downloader. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versioning: [SemVer](https://semver.org/).

## [1.6.3] - 2026-09-04

### Fixed
- **Windows downloads failed while the same video worked on macOS.** Three local faults stacked and the UI reported "Unknown blocking reason":
  - yt-dlp 2026 enables only Deno for YouTube n-challenge solving. Deno is common on Homebrew Macs; on Windows Node is often installed but unused unless `--js-runtimes` is passed. Without it, web clients return "Only images are available" / "Requested format is not available".
  - A Russian Windows console uses cp1251. yt-dlp writing a Cyrillic title into a GUI pipe raised `[Errno 22] Invalid argument`. The app now forces UTF-8 and `--windows-filenames`.
  - `--ffmpeg-location` pointed at `scoop\shims`. That stub works in a terminal and fails from the GUI (`ffmpeg is not installed`). The app follows the `.shim` file to the real binary under `scoop\apps\...`.

### Docs
- [YOUTUBE_BLOCKING.md](YOUTUBE_BLOCKING.md) and [WINDOWS_SETUP.md](WINDOWS_SETUP.md): these three look like a YouTube block. They are not. Node.js (or Deno) is required at runtime, not only to build.

## [1.6.2] - 2026-09-04

### Fixed
- **Downloads failed on every Windows install.** The default output folder came from a hardcoded macOS home path (`/Users/<developer>/Downloads`), which Windows cannot create — `C:\Users` needs elevation. All seven strategies ran anyway and the last one's diagnosis, "PO Token required", was reported instead of the real cause. The folder now comes from the OS, is tested with a real write before a download starts, and is repaired automatically when unusable.
- A cookie failure no longer burns three strategies. Once the browser refuses its cookie database, later attempts drop cookies instead of repeating the identical error.
- Failures report what yt-dlp actually said. Its output went to stderr only, so the in-app log could not tell two different faults apart.
- `ERROR:` lines now outrank warnings when diagnosing. yt-dlp warns about SABR and PO tokens on clients that still work, and those warnings were being reported as the failure.
- The per-attempt status no longer shows 🍪 for attempts that had already dropped cookies.

### Added
- Every download logs the yt-dlp version and path in use, and warns when ffmpeg is missing.

### Docs
- [YOUTUBE_BLOCKING.md](YOUTUBE_BLOCKING.md): read the yt-dlp line first and rule out the local causes that imitate a block. Updating yt-dlp is no longer presented as the most common fix — it was not the cause here, and version proved irrelevant.

## [1.6.1] - 2026-09-04

### Fixed
- **YouTube info fetch was failing for every video.** The pinned player clients (`web`, `web_safari`, `ios`) that used to bypass SABR now return "Requested format is not available" for anything, so all three info strategies failed. yt-dlp's own default client list is used first, with `all` as fallback; the download path leads with `all` for the same reason.
- A browser cookie failure no longer masks the real error. `Could not copy Chrome cookie database` is diagnosed as its own reason with advice that fits it (close Chrome, or switch Cookies to None), instead of "Unknown blocking reason - try a VPN".
- Attempts that only tripped on cookies no longer overwrite a more informative earlier error.

## [1.6.0] - 2026-09-04

### Added
- **ffmpeg is now a managed tool.** It has its own status and Install button in the Tools panel. Without it, merging above 720p fails — and the app never mentioned it before.
- Install progress in the log (`⬇️ ffmpeg 47% (76.3/162.8 MB)`); the ffmpeg archive is ~170 MB.

### Fixed
- **Install on Windows.** The Install button demanded Homebrew before checking the platform, so it could never succeed there. Windows now downloads yt-dlp and ffmpeg from GitHub releases into `%LOCALAPPDATA%\youtube-downloader\bin` — no package manager, no admin rights, no PATH edit, no restart.
- **Tool detection on Windows.** Discovery hardcoded `/opt/homebrew` and `/usr/local/bin`, omitted the `.exe` suffix, and used `which` for PATH lookup — a command absent from a clean Windows box. PATH is now walked in Rust honouring `PATHEXT`, and winget / Chocolatey / Scoop locations are searched.
- Update no longer downloads over a copy the app did not install: a Scoop / Chocolatey / winget install is named, left alone, and the command that updates it is shown.
- `scripts/version.py` crashed on a Windows console using a non-UTF-8 codepage.

### Changed
- All five duplicated "find yt-dlp" implementations replaced by `downloader/platform.rs`, the single place for platform differences.

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
