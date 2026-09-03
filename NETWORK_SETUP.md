# Network, DNS, and VPN

Language: **English** · [Русский](docs/NETWORK_SETUP_ru.md)

The app needs working **name resolution**. yt-dlp looks up YouTube, the status bar looks up your external IP, and a build talks to npm / crates.io. A dead system DNS looks exactly like a YouTube IP block.

If `dig @1.1.1.1` works but `curl https://www.youtube.com` says `Resolving timed out`, this is **not** SABR. For real 403 / PO Token issues see [YOUTUBE_BLOCKING.md](YOUTUBE_BLOCKING.md).

## Fast check

Status bar **`IP: N/A`** means the app could not even resolve an IP-echo host.

```bash
dig +time=3 +tries=1 @1.1.1.1 www.youtube.com +short
curl -sS -o /dev/null -w "%{http_code}\n" --max-time 10 https://www.youtube.com
```

On macOS, `scutil --dns` with `if_index : NN (utunN)` means a VPN tunnel (often **Tailscale exit node**) injected a resolver that outranks Wi-Fi DNS. That resolver is frequently a LAN address such as `192.168.x.1`, which is unreachable through the tunnel.

## What usually fixes it

1. Tailscale **exit node** is chosen from the **menu-bar icon**, not from Settings. “Run as exit node” is the opposite (share *your* connection).
2. In the Tailscale admin console → **DNS**: set public nameservers (`1.1.1.1`, optionally `9.9.9.9`) and enable **Override DNS servers**.
3. Temporary unblock: `sudo tailscale set --exit-node=` then flush the DNS cache.

Step-by-step Tailscale screens (admin console vs app vs menu bar): [docs/NETWORK_SETUP_ru.md](docs/NETWORK_SETUP_ru.md) (Russian).

macOS blank window and build failures that share this DNS bug: [MACOS_SETUP.md](MACOS_SETUP.md).
