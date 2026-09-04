# Network: VPN, DNS, and Tailscale

Language: **English** · [Русский](docs/NETWORK_SETUP_ru.md)

**Version:** 1.6.1 | **Updated:** 2026-08-30

The app depends entirely on name resolution: yt-dlp looks up YouTube, the status bar looks up your external IP, a build talks to npm and crates.io. When system DNS does not answer, it looks like a YouTube block, even though the machine itself is misconfigured.

This document says where each setting lives and which configuration actually works.

---

## Where each setting lives

The usual confusion: Tailscale settings are split across three places, and the one you need — picking an exit node — is not where people look.

| Setting | Where | What it affects |
|---------|-------|-----------------|
| Global nameservers, Override DNS servers | admin console → **DNS** | every device on the tailnet |
| MagicDNS, Search domains | admin console → **DNS** | every device on the tailnet |
| Use Tailscale DNS settings | app → Settings → General | this computer only |
| **Picking** an exit node | **menu-bar icon** → Exit Node | this computer only |
| Run as exit node | app → Settings → Exit Nodes | sharing traffic **from** this Mac |
| Allow local network access | app → Settings → Exit Nodes | LAN access while an exit node is on |
| Use Tailscale subnets | app → Settings → General | routes into advertised subnets |

**There is no exit-node picker in the Settings window.** “Run as exit node” is the opposite — share *your* link with other devices. The switch you want is only in the menu bar (Tailscale icon next to the clock) → **Exit Node**.

The **Manage…** button next to “Use Tailscale DNS settings” opens a dialog with a single toggle. Nameservers are not edited there — they live in the admin console, which the dialog says: “DNS can be configured in the admin console”.

---

## Working configuration

### 1. Admin console → DNS

1. **Nameservers** → **Add nameserver** → **Custom** → `1.1.1.1`
2. Optionally a second one — `9.9.9.9`
3. Turn on **Override DNS servers**

Why: if no global resolver is set, an active exit node makes Tailscale inject the LAN DNS — usually a private router address like `192.168.x.1`. Through the exit node that address is unreachable, and every lookup hangs until timeout. A public resolver works from any point in the tunnel.

Do not put a private address (`192.168.x.x`, `10.x.x.x`) in Global nameservers: it only answers when you are physically on that network.

### 2. App on the Mac

Settings → General → **Use Tailscale DNS settings** — on.

Equivalent in the terminal:

```bash
sudo tailscale set --accept-dns=true
```

### 3. Exit node

Menu-bar icon → **Exit Node** → the node you want, or **None**.

```bash
sudo tailscale set --exit-node=<node-name>
sudo tailscale set --exit-node=
```

After any change, flush the resolver cache:

```bash
sudo dscacheutil -flushcache; sudo killall -HUP mDNSResponder
```

---

## Check

```bash
scutil --dns | head -8
```

`resolver #1` should be `100.100.100.100` (MagicDNS).

```bash
curl -sS -o /dev/null -w "%{http_code}\n" --max-time 10 https://www.youtube.com
```

`200` — the setup is correct. Check this **with the exit node on**: that is the mode that usually falls apart.

Current client state:

```bash
tailscale status
tailscale debug prefs | grep -iE "corpdns|exitnode"
```

`CorpDNS` is “Use Tailscale DNS settings”. A non-empty `ExitNodeID` means an exit node is active.

---

## Diagnosis: one failure, three symptoms

| Where | What you see |
|-------|----------------|
| App | `Network timeout (possible IP throttling)`, status bar `IP: N/A` |
| Build | `Could not resolve host: static.crates.io`, `make build` hangs on crates |
| Before 1.5.1 | blank white window — the font request hit the same dead DNS |

The app hint “No proxy detected, try enabling XRAY/Clash” is the wrong lead here: the proxy is fine, name resolution is not.

### DNS vs a real block — 30 seconds

```bash
dig +time=3 +tries=1 @1.1.1.1 www.youtube.com +short
curl -sS -o /dev/null -w "%{http_code}\n" --max-time 10 https://www.youtube.com
```

**The explicit resolver returns addresses, `curl` says `Resolving timed out`** — this is DNS, not YouTube and not the ISP.

Who replaced the resolver:

```bash
scutil --dns | head -8
```

A line `if_index : NN (utunN)` means DNS is forced by the tunnel. Addresses set with `networksetup -setdnsservers Wi-Fi …` are simply ignored: the tunnel-bound resolver has higher priority.

Whose tunnel:

```bash
ifconfig utun4 | grep inet
```

`100.64.x.x` and `fd7a:115c:a1e0::` are Tailscale. Another address is a third-party VPN client — fix it in that client.

---

## Common configuration mistakes

| Mistake | What happens |
|---------|----------------|
| Global nameservers empty + exit node on | LAN DNS is pulled onto the tunnel, where it is unreachable — everything hangs |
| Private address in Global nameservers | Works only on that LAN, breaks when you travel |
| Changing DNS via `networksetup` on Wi-Fi | Does nothing: the tunnel resolver wins |
| Relying on `--accept-dns=false` | Does not help: the exit node still overrides the resolver without Tailscale DNS |

---

## Temporary workarounds

Until you can reach the admin console — drop the exit node; DNS falls back to Wi-Fi settings:

```bash
sudo tailscale set --exit-node=
```

Punch specific hosts past the resolver — useful when you need to build *right now*:

```bash
for h in github.com codeload.github.com registry.npmjs.org static.crates.io index.crates.io; do ip=$(dig +short +time=3 +tries=1 @1.1.1.1 "$h" | grep -E '^[0-9.]+$' | head -1); [ -n "$ip" ] && printf "%s %s\n" "$ip" "$h"; done | sudo tee -a /etc/hosts
```

Remove those lines after DNS is fixed — CDN addresses change, and they will start breaking access instead of helping:

```bash
sudo sed -i '' -E '/^[0-9.]+[[:space:]]+(github\.com|codeload\.github\.com|registry\.npmjs\.org|static\.crates\.io|index\.crates\.io)$/d' /etc/hosts
```

---

## Related

- [MACOS_SETUP.md](MACOS_SETUP.md) — install, build, blank white window
- [YOUTUBE_BLOCKING.md](YOUTUBE_BLOCKING.md) — real YouTube blocks: SABR, 403, PO Token
