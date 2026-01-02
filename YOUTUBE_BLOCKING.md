# YouTube Blocking - Understanding and Solutions

**Last Updated:** 02.01.2026 20:16

---

## 🎯 What Is Happening

You're experiencing **IP reputation throttling** from YouTube CDN, not a ban or CAPTCHA.

### Technical Details

**What works:**
- ✅ TCP connection established
- ✅ TLS handshake successful
- ✅ Request sent to YouTube

**What doesn't work:**
- ❌ HTTP response body never arrives
- ❌ Read timeout after 20-30 seconds
- ❌ Same behavior for all client types (web/android/ios)
- ❌ Same behavior with or without cookies

**This is NOT:**
- ❌ A bug in this application
- ❌ A broken yt-dlp version
- ❌ Wrong command-line flags
- ❌ Missing permissions

**This IS:**
- ✅ Temporary IP-based soft block by YouTube
- ✅ Automatic CDN throttling
- ✅ Normal behavior during burst requests

---

## ⏱️ How Long Does It Last?

Typical durations:
- **Short:** 6-12 hours (most common)
- **Medium:** 24 hours
- **Recurring:** If you make many requests in a row

The block is **automatic** and will lift on its own.

---

## 🔍 How to Detect

### Symptoms in Application

```
Error: yt-dlp error: ERROR: [youtube] videoID: Unable to download API page:
HTTPSConnectionPool(host='www.youtube.com', port=443): Read timed out. 
(read timeout=20.0)
```

### Symptoms in Terminal

```bash
# This hangs for 20+ seconds then fails:
yt-dlp --dump-json "https://youtu.be/videoID"
```

---

## ✅ Solutions

### Option 1: Wait (Recommended for occasional use)

Simply wait 6-24 hours. The block will clear automatically.

**When to use:**
- You're not in a hurry
- First time encountering the block
- Occasional personal use

### Option 2: Use Proxy/VPN (Recommended for frequent use)

YouTube rarely blocks SOCKS5 proxy traffic.

**Setup:**

1. **If you have XRAY/V2Ray running:**
   ```bash
   # Your proxy is likely already running on:
   socks5://127.0.0.1:1080
   ```

2. **Enable in app:**
   - Go to Settings (when implemented)
   - Enable "Use Proxy"
   - Enter: `socks5://127.0.0.1:1080`

3. **Test manually:**
   ```bash
   yt-dlp --proxy socks5://127.0.0.1:1080 \
          --dump-json \
          "https://youtu.be/dQw4w9WgXcQ"
   ```

**When to use:**
- You need immediate access
- You frequently download videos
- You already have VPN/proxy setup

### Option 3: Try Different Network

Sometimes the block is ISP-specific:
- Switch from WiFi to mobile hotspot
- Try from a different location
- Use a different internet connection

### Option 4: Use YouTube Data API v3 (Metadata only)

For getting video information (not downloading):

1. Get free API key from [Google Cloud Console](https://console.cloud.google.com/)
2. Quota: 10,000 requests/day
3. **Limitation:** Cannot download videos, only get metadata

---

## 🚫 What DOESN'T Work

These will NOT bypass the block:
- ❌ Changing `--extractor-args` (web/android/ios)
- ❌ Using `--cookies-from-browser`
- ❌ Switching between binary and Python module
- ❌ Adding User-Agent headers
- ❌ Increasing timeout values
- ❌ Updating yt-dlp to latest version

Because the issue is **before** YouTube API responds, not in parsing the response.

---

## 🛡️ How to Avoid Future Blocks

1. **Don't make burst requests**
   - Wait 5-10 seconds between video info requests
   - Don't refresh repeatedly if it fails

2. **Use proxy from the start** if you download frequently

3. **Cache video information** (app does this automatically)

4. **Limit retries** when requests fail

---

## 🔧 For Developers

### Detecting the Block Programmatically

```rust
if stderr.contains("Read timed out") 
   && execution_time > 15_000 
   && output.stdout.is_empty() {
    return Err(DownloadError::BlockedByYouTube);
}
```

### User-Friendly Error Message

Instead of:
```
Error: Unable to download API page: HTTPSConnectionPool...
```

Show:
```
YouTube is temporarily blocking requests from your IP address.
This is normal and will resolve in 6-24 hours.

Solutions:
1. Wait and try again later
2. Enable Proxy/VPN in settings
3. Try from a different network
```

---

## 📚 Additional Resources

- [yt-dlp documentation](https://github.com/yt-dlp/yt-dlp)
- [YouTube Data API v3](https://developers.google.com/youtube/v3)
- [XRAY proxy setup](https://xtls.github.io/)

---

## ❓ FAQ

**Q: Is my IP banned permanently?**  
A: No, it's a temporary throttle that clears automatically.

**Q: Will VPN help?**  
A: Yes, SOCKS5 proxy is very effective.

**Q: Can I speed up the unblock?**  
A: No, you can only wait or use proxy. Making more requests makes it worse.

**Q: Is this illegal?**  
A: No, downloading public videos for personal use is generally legal. But check YouTube ToS and your local laws.

**Q: Why doesn't the app show better errors?**  
A: We're working on it! Enhanced error detection coming soon.

---

**Created:** 02.01.2026  
**Author:** Kurein Maxim
