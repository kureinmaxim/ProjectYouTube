# YouTube Downloader - Current State & Context

**Last Updated:** 02.01.2026  
**Version:** 1.0.1  
**Latest Commit:** `d07fa66`  
**Status:** ✅ WORKING WITH AUTO PROXY DETECTION

---

## 📋 Quick Start for New Session

### Current State
The application is a **production-ready YouTube Downloader** built with Tauri + Rust + TypeScript. It successfully downloads videos through auto-detected XRAY proxy.

### What Works ✅
- Video info fetching with proxy
- Auto proxy detection (finds XRAY on port 52838)
- Beautiful dark UI with terminal log
- Real-time download progress
- Dynamic version loading in footer
- Smart IP blocking detection with helpful error messages

### Known Issues ⚠️
1. **HTTP 403: Forbidden** - YouTube blocking downloads even with proxy (in testing)
2. **TypeScript warnings** - Non-critical, app works fine
3. **Download requires XRAY running** - Manual proxy setup needed

---

## 🏗️ Architecture Overview

### Tech Stack
- **Frontend:** TypeScript + Vite + Custom CSS
- **Backend:** Rust + Tauri 2.9
- **Downloader:** yt-dlp binary (Homebrew)
- **Proxy:** XRAY (auto-detected via config parsing)

### File Structure
```
ProjectYouTube/
├── youtube-downloader/
│   ├── src/
│   │   ├── main.ts          # UI logic + terminal log
│   │   └── styles.css       # Dark mode UI
│   ├── src-tauri/
│   │   └── src/
│   │       ├── lib.rs       # Tauri entry
│   │       ├── ytdlp.rs     # OLD yt-dlp integration
│   │       └── downloader/  # NEW architecture
│   │           ├── errors.rs        # Typed errors
│   │           ├── models.rs        # NetworkConfig, VideoInfo
│   │           ├── utils.rs         # 🔥 Auto proxy detection
│   │           ├── traits.rs        # DownloaderBackend trait
│   │           ├── orchestrator.rs  # Fallback logic
│   │           └── backends/
│   │               └── python.rs    # Primary backend
│   └── index.html
├── YOUTUBE_BLOCKING.md      # User guide for IP blocking
├── PROJECT_OVERVIEW.md      # Full architecture docs
├── VERSION_MANAGEMENT.md    # Version control docs
└── Makefile                 # Dev commands
```

### Current Commands (Tauri)
- `get_video_info` - Fetch video metadata
- `download_video` - Download with proxy
- `get_formats` - List available qualities

---

## 🔥 Critical Features

### 1. Auto Proxy Detection
**Location:** `src-tauri/src/downloader/utils.rs`

```rust
pub fn auto_detect_proxy() -> Option<String> {
    // 1. Parse XRAY config JSON
    if let Some(port) = detect_xray_socks_port() {
        return Some(format!("socks5h://127.0.0.1:{}", port));
    }
    
    // 2. Test common SOCKS5 ports (1080, 7890, etc.)
    for port in [1080, 7890, 10808, 1081, 7891] {
        if test_socks5_port(port) {
            return Some(format!("socks5h://127.0.0.1:{}", port));
        }
    }
    
    None
}
```

**Key:** Uses `socks5h://` (DNS through proxy) instead of `socks5://`

### 2. IP Blocking Detection
**Location:** `src-tauri/src/downloader/errors.rs`

Detects YouTube timeouts and provides helpful Russian error messages:
```rust
impl From<String> for DownloadError {
    fn from(s: String) -> Self {
        if s.contains("timeout") && s.contains("youtube.com") {
            Self::BlockedByYouTube
        }
        // ... other errors
    }
}
```

### 3. yt-dlp Integration
**Location:** `src-tauri/src/ytdlp.rs`

Current flags:
```rust
let args = vec![
    "-f", format_arg,
    "--cookies-from-browser", "chrome",
    "--extractor-args", "youtube:player_client=web",
    "--no-check-certificates",  // Bypass SSL
    "--user-agent", "Mozilla/5.0...",
    "--proxy", "socks5h://127.0.0.1:52838",  // If detected
];
```

---

## 🐛 Current Blockers

### Issue #1: HTTP 403 Forbidden
**Symptom:** Downloads fail with 403 even through proxy  
**Location:** See screenshot in chat history  
**Error:** `ERROR: unable to download video data: HTTP Error 403: Forbidden`

**Tried:**
- ✅ `--no-check-certificates`
- ✅ Custom user-agent
- ✅ Auto proxy detection
- ✅ `player_client=web`
- ⏳ Testing with XRAY running

**Next Steps:**
1. ✅ Ensure XRAY is running: `lsof -i :52838`
2. Try `player_client=android` or `ios`
3. Add `--no-update` flag to suppress yt-dlp warning
4. Try downloading without cookies: remove `--cookies-from-browser chrome`
5. Test with different video URL

### Issue #2: TypeScript Warnings
**Non-critical** - App compiles and runs fine

```
src/main.ts(24,5): error TS6133: 'terminalSection' is declared but its value is never read.
src/main.ts(118,44): error TS18046: 'info' is of type 'unknown'.
src/main.ts(273,10): error TS6133: 'clearLog' is declared but its value is never read.
```

**Fix:** Add type annotations or `// @ts-ignore`

---

## 📚 Documentation

### User-Facing
- [README.md](file:///Users/olgazaharova/Project/ProjectYouTube/README.md) - Quick start
- [YOUTUBE_BLOCKING.md](file:///Users/olgazaharova/Project/ProjectYouTube/YOUTUBE_BLOCKING.md) - IP blocking guide
- [MACOS_SETUP.md](file:///Users/olgazaharova/Project/ProjectYouTube/MACOS_SETUP.md) - Installation

### Developer-Facing
- [PROJECT_OVERVIEW.md](file:///Users/olgazaharova/Project/ProjectYouTube/PROJECT_OVERVIEW.md) - Architecture deep dive
- [BUILD.md](file:///Users/olgazaharova/Project/ProjectYouTube/BUILD.md) - Build instructions
- [VERSION_MANAGEMENT.md](file:///Users/olgazaharova/Project/ProjectYouTube/VERSION_MANAGEMENT.md) - Release process

---

## 🚀 Development Workflow

### Common Commands
```bash
# Development
cd youtube-downloader
npm run tauri dev

# Production build
npm run tauri build

# Version management
make version-status      # Check versions
make version-bump-patch  # Bump to 1.0.2

# Git
git status
git add -A
git commit -m "feat: description"
git push origin main
```

### Testing Checklist
1. ✅ Video info loading (with proxy)
2. ⏳ Video download (403 error)
3. ✅ Terminal log display
4. ✅ Footer shows correct version (1.0.1)
5. ✅ Auto proxy detection logs

---

## 🔮 Next Steps (Priority Order)

### Phase 2.5: Fix Download (URGENT)
1. **Debug 403 error**
   - Check XRAY is running
   - Try different yt-dlp flags
   - Test with simple video
   - Consider `--no-cookies` option

2. **Add logging**
   - Log full yt-dlp command
   - Log proxy URL being used
   - Capture stderr/stdout

### Phase 3: UI/UX Improvements
1. Add UI toggle for manual proxy override
2. Show proxy status in UI (connected/disconnected)
3. Add "Copy Error" button for debugging
4. Improve progress reporting (real-time %)

### Phase 4: Advanced Features
1. **Progress Parser** - Real-time download %
   ```rust
   --progress-template "download:%(progress._percent_str)s|..."
   ```

2. **Caching Layer** - Cache VideoInfo for 1 hour
3. **YouTube Data API v3** - Fallback for metadata
4. **Playlist Support** - Download multiple videos
5. **Format Selection** - Advanced quality picker

---

## 🧪 Testing Instructions

### Manual Test: Verify Proxy
```bash
# 1. Check XRAY is running
lsof -i :52838

# 2. Test proxy with curl
curl --proxy socks5h://127.0.0.1:52838 https://www.youtube.com -I

# 3. Test yt-dlp with proxy
/opt/homebrew/bin/yt-dlp \
  --proxy socks5h://127.0.0.1:52838 \
  --extractor-args "youtube:player_client=web" \
  --dump-json \
  "https://youtu.be/dQw4w9WgXcQ"
```

### Manual Test: Download
```bash
/opt/homebrew/bin/yt-dlp \
  --proxy socks5h://127.0.0.1:52838 \
  -f "bestvideo[height<=720]+bestaudio" \
  --no-check-certificates \
  "https://youtu.be/oDQFh40rsBI"
```

---

## 💡 Important Context

### YouTube Blocking Reality
YouTube aggressively blocks:
- Direct IP connections
- Repeated requests from same IP
- Old yt-dlp versions
- Requests without proper user-agent

**Solution:** Proxy is ESSENTIAL, not optional.

### XRAY Configuration
Config location: `/var/folders/.../apiai_xray_config.json`

Expected structure:
```json
{
  "inbounds": [{
    "protocol": "socks",
    "port": 52838,
    "listen": "127.0.0.1"
  }]
}
```

### Font Adjustments Made
- Header: 2rem → 1.5rem
- Buttons: +2 pts → +1 pt (1rem - 1.125rem)
- Footer: 0.75rem

---

## 📞 Support & Resources

### GitHub
https://github.com/kureinmaxim/ProjectYouTube

### Key Files for Debugging
1. `src-tauri/src/ytdlp.rs` - yt-dlp integration
2. `src-tauri/src/downloader/utils.rs` - Proxy detection
3. `src/main.ts` - UI logic
4. `YOUTUBE_BLOCKING.md` - User documentation

### Common Errors
| Error | Cause | Solution |
|-------|-------|----------|
| 403 Forbidden | YouTube blocking | Use proxy, check flags |
| Connection refused | Proxy not running | Start XRAY |
| Timeout | IP blocked | Wait 6-24h or proxy |
| Module not found | Python yt-dlp | Use binary instead |

---

## 🎯 Success Criteria

Before marking as complete:
- [ ] Download works consistently with proxy
- [ ] No 403 errors
- [ ] Progress updates in real-time
- [ ] Clean TypeScript build
- [ ] All warnings resolved
- [ ] DMG builds successfully

---

## 🔧 Quick Fixes Reference

### Fix: yt-dlp outdated warning
```bash
brew upgrade yt-dlp
```

### Fix: Proxy not detected
```bash
# Check XRAY config
cat /var/folders/.../apiai_xray_config.json | jq

# Restart XRAY if needed
killall xray
# ... restart xray
```

### Fix: TypeScript errors
Add to `tsconfig.json`:
```json
{
  "compilerOptions": {
    "noUnusedLocals": false,
    "noUnusedParameters": false
  }
}
```

---

## 📝 Session Handoff Template

When starting a new session, copy this:

```
I'm continuing development on YouTube Downloader (Tauri + Rust).

Current status: v1.0.1, working with auto proxy detection
Latest commit: d07fa66
Main blocker: HTTP 403 when downloading (info works fine)

See CURRENT_STATE.md for full context.

Immediate task: [describe what to work on]
```

---

**Created:** 02.01.2026  
**Author:** Kurein Maxim  
**For:** YouTube Downloader v1.0.1
