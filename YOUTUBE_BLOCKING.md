# YouTube Blocking / SABR / 403

Language: **English** (this file) · Russian network/DNS notes: [docs/NETWORK_SETUP_ru.md](docs/NETWORK_SETUP_ru.md)

**Last Updated:** 2026-08-30  
**App Version:** 1.5.1

This doc is written for this project's desktop app (Tauri) and the realities of YouTube in 2026: **SABR streaming**, **PO Token**, **bot protection**, and **IP reputation throttling**.

> 💡 **New in 1.2.0**: Built-in blocking diagnostics that automatically detects the reason for failure and suggests solutions.

---

## What’s happening (high-level)

YouTube failures usually fall into one of these buckets:

- **CDN throttling / soft IP block (timeouts)**  
  Connection works, request is sent, but the response body never arrives → timeouts after ~20–30s.

- **SABR / 403 Forbidden on “web” client**  
  YouTube hides/blocks normal HTTPS formats for some clients and returns `HTTP Error 403: Forbidden`.

- **Bot protection / auth required**  
  `yt-dlp` needs authentication/cookies even if the video plays in a browser.

Important nuance: **“Video plays in browser with VPN” does not mean the app uses the same VPN.**  
If your VPN is a **browser extension** or you use **split tunneling**, the app/yt-dlp may still go through your normal IP and get blocked.

---

## How to recognize the problem quickly

### First: rule out broken DNS (looks identical, fixed differently)

A dead system resolver produces the same timeouts as an IP block, and the app's
own hint ("No proxy detected, try enabling XRAY/Clash") points the wrong way.
The giveaway is `IP: N/A` in the status bar - the app could not even look up its
own external address.

```bash
dig +time=3 +tries=1 @1.1.1.1 www.youtube.com +short
curl -sS -o /dev/null -w "%{http_code}\n" --max-time 10 https://www.youtube.com
```

Explicit resolver answers but curl says `Resolving timed out` -> this is not
YouTube, it is DNS. A VPN tunnel (Tailscale exit node, for one) can bind an
unreachable nameserver to its interface, and that resolver outranks anything set
on Wi-Fi. Fix it first: [NETWORK_SETUP.md](NETWORK_SETUP.md).

### Timeout / soft IP block (typical)

- App log shows `Timed out after ...` or `Read timed out`
- Terminal hangs ~20–30 seconds then fails

### SABR / 403 (typical)

You’ll see one of:

- `YouTube is forcing SABR streaming for this client`
- `ERROR: unable to download video data: HTTP Error 403: Forbidden`

### PO Token (mweb) (newer)

You’ll see:

- `mweb client https formats require a GVS PO Token ...`

The project tracks that situation here: [PO Token guide](https://github.com/yt-dlp/yt-dlp/wiki/PO-Token-Guide)

---

## What the app already does (our implemented mitigations)

This is the current behavior of the app:

### Core Features

- **Progress / "still working" signals**  
  The UI shows heartbeat logs during slow operations.

- **Timeout protection**  
  `yt-dlp` calls have timeouts; the process is killed if it hangs.

- **Proxy support**
  - Auto-detects local SOCKS5 (XRAY/Clash/V2Ray common ports)
  - Manual proxy input in **Tools → Proxy**

- **Cookies support**
  - `Chrome (logged-in)` → uses `--cookies-from-browser chrome`
  - `cookies.txt file` → uses `--cookies /path/to/cookies.txt`

- **Client/strategy switching**
  When YouTube blocks a client:
  - tries different clients / strategies
  - can switch from cookies-on to cookies-off
  - can try audio-only fallback (MP3) when video is blocked

- **Tool fallback (optional)**
  If enabled, the app tries multiple tools (yt-dlp → lux → you-get) until one succeeds.
  You can disable this via **Mode → Auto fallback**.

### New in v1.2.0: Production-Grade Architecture

- **Dual InfoExtractor Mode**
  - **Python mode** (`python3 -m yt_dlp`) — better for YouTube, avoids bot detection
  - **CLI mode** (`yt-dlp` binary) — faster, no Python dependency
  - Auto-switch: Python first for YouTube, CLI first for other sites

- **Blocking Diagnostics**
  The app now automatically detects the blocking reason:

  | Blocking Type | Detection | Suggestion |
  |---------------|-----------|------------|
  | `Http403Forbidden` | "403", "Forbidden" | Use VPN/Proxy, update cookies |
  | `SabrStreaming` | "SABR", "forcing SABR" | Python mode + cookies, audio-only |
  | `PoTokenRequired` | "PO Token", "GVS" | Cookies from logged-in browser |
  | `AgeRestricted` | "age-restricted", "sign in" | Cookies from authorized account |
  | `GeoBlocked` | "not available in your country" | VPN with different country |
  | `NetworkTimeout` | "timeout", "timed out" | Check connection, use proxy |
  | `RateLimited` | "429", "rate limit" | Wait 10-15 min, change IP |
  | `BotDetection` | "captcha", "unusual traffic" | Python mode + cookies |
  | `PrivateVideo` | "private video" | Cookies from authorized account |
  | `VideoUnavailable` | "unavailable", "removed" | Video deleted, nothing to do |

- **FormatSelector**
  - Unified quality selection with codec info (H.264/VP9/AV1)
  - Estimated file sizes (video + audio combined)
  - H.264 preference for maximum compatibility

---

## What to do (best order)

### 1) Try in-app Proxy first (recommended)

If you have XRAY/Clash/V2Ray:

- Run it locally
- In the app open **Tools → Proxy**
- Enter something like:
  - `socks5h://127.0.0.1:1080`
  - or your local port (example: `socks5h://127.0.0.1:49506`)

Why this works: YouTube is much less aggressive against SOCKS5 traffic, and you get a different routing/IP.

### 2) Try cookies (recommended when auth/bot protection triggers)

If YouTube requires login/captcha-like checks for automated traffic, cookies help:

- Open Chrome
- Ensure you’re logged in on YouTube (avatar visible)
- In the app set **Tools → Cookies → Chrome (logged-in)**

If it still fails, export cookies to a file:

- In Chrome export cookies to `cookies.txt` (via a cookies exporter extension)
- In the app choose **Tools → Cookies → cookies.txt file → Pick**

### 3) Try audio-only (often allowed)

Some cases block video but allow audio. In the app choose **Quality → MP3**.

### 4) Try a different network / real system VPN

- Mobile hotspot
- Different Wi‑Fi
- System VPN app (not a browser extension)

---

## What usually does NOT help

- Randomly increasing timeouts
- Changing user-agent only
- Spamming retries quickly (often makes throttling worse)

---

## Future-proof solution: use a remote server (most reliable)

This is the “always works” path when YouTube is aggressive on your home IP/ISP. Below are **three practical approaches**, from simplest to most product-grade.

### Option A — Download directly on a VPS over SSH (fastest)

Best when you just need the file quickly.

1) SSH into your server:

```bash
ssh user@SERVER_IP
```

2) Install tools:

```bash
sudo apt update
sudo apt install -y yt-dlp ffmpeg
```

3) Download on the server:

```bash
yt-dlp -f "bv*+ba/b" -o "%(title)s.%(ext)s" "https://youtu.be/VIDEO_ID"
```

4) Copy the result back to your Mac:

```bash
scp user@SERVER_IP:/home/user/*.mp4 ~/Downloads/
```

Why it works: different IP/ASN, YouTube throttles/blocks that path differently.

### Option B — Use the server as a SOCKS5 proxy via SSH (elegant + perfect for our app)

This is the best “semi-product” approach: no XRAY, no VPN client, just SSH.

1) Start a SOCKS5 tunnel locally (keep it running):

```bash
ssh -D 1080 user@SERVER_IP
```

2) Use it with yt-dlp (manual test):

```bash
yt-dlp --proxy socks5h://127.0.0.1:1080 -f "bv*+ba/b" "https://youtu.be/VIDEO_ID"
```

3) Use it in the app:

- Open **Tools → Proxy**
- Enter: `socks5h://127.0.0.1:1080`

Why it’s ideal: DNS goes through the server (`socks5h`), the app gets server IP, and you don’t need extra software.

### Option C — HTTP download service (production-grade architecture)

Best if you want a robust “desktop app → server → YouTube” pipeline:

**Flow**

Desktop App → Server API → YouTube  
                      ↓  
                  MP4/MP3 file  
                      ↓  
Desktop App ← downloads via HTTPS

**Minimal server idea**

- Receive `{ url, quality }`
- Run `yt-dlp` server-side
- Store file (local disk / S3)
- Return a download link

Implementation options:

- FastAPI (Python)
- Actix-web (Rust)
- Nginx for static hosting + queue worker for downloads

This approach:
- avoids local IP throttling entirely
- is scalable
- is the most reliable long-term solution

---

## Quick Test: Get Formats with Cookies (Python)

If you want to test what formats are available for a video:

```bash
# From the repo root
python3 formats.py "https://youtu.be/VIDEO_ID"
```

This script:
1. Uses cookies from `cookies.txt` (export from browser)
2. Shows all available formats with resolution, codec, size
3. Recommends yt-dlp commands for downloading

---

## Architecture Reference

See [docs/ARCHITECTURE_2026_ru.md](docs/ARCHITECTURE_2026_ru.md) for:
- InfoExtractor trait design
- Python vs CLI mode comparison
- FormatSelector implementation
- Full module structure

---

## References

- yt-dlp: `https://github.com/yt-dlp/yt-dlp`  
- PO Token guide: `https://github.com/yt-dlp/yt-dlp/wiki/PO-Token-Guide`  
- YouTube Data API v3: `https://developers.google.com/youtube/v3`
- Project architecture: [docs/ARCHITECTURE_2026_ru.md](docs/ARCHITECTURE_2026_ru.md)
