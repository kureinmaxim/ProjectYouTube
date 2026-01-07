// Helper functions for backend implementations

use crate::downloader::models::NetworkConfig;
use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command as TokioCommand;
use tokio::time::{timeout, Duration as TokioDuration};
use time::{format_description, Date, OffsetDateTime};

/// Network status information for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub proxy: Option<String>,
    pub mode: String,        // "direct", "proxy", "vpn"
    pub external_ip: Option<String>,
    pub proxy_reachable: bool,
    pub proxy_message: Option<String>,
    pub ytdlp_version: Option<String>,
    pub ytdlp_status: String,       // ok | stale | missing | unknown
    pub ytdlp_hint: Option<String>,
}

#[derive(Default)]
struct ProxyCheck {
    reachable: bool,
    message: Option<String>,
}

struct YtDlpFreshness {
    version: Option<String>,
    status: String,
    hint: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IpInfoResponse {
    ip: Option<String>,
    city: Option<String>,
    country: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SimpleIp {
    ip: String,
}

// 2ip.io response format
#[derive(Debug, Deserialize)]
struct TwoIpResponse {
    ip: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
}

/// Run command with timeout (shared utility)
pub async fn run_output_with_timeout(
    program: &str,
    args: Vec<String>,
    timeout_secs: u64,
) -> Result<std::process::Output, String> {
    let mut child = TokioCommand::new(program)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start {}: {}", program, e))?;

    let mut stdout_pipe = child
        .stdout
        .take()
        .ok_or_else(|| format!("Failed to capture stdout from {}", program))?;
    let mut stderr_pipe = child
        .stderr
        .take()
        .ok_or_else(|| format!("Failed to capture stderr from {}", program))?;

    let stdout_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        stdout_pipe
            .read_to_end(&mut buf)
            .await
            .map_err(|e| format!("Failed to read stdout: {}", e))?;
        Ok::<Vec<u8>, String>(buf)
    });
    let stderr_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        stderr_pipe
            .read_to_end(&mut buf)
            .await
            .map_err(|e| format!("Failed to read stderr: {}", e))?;
        Ok::<Vec<u8>, String>(buf)
    });

    let waited = timeout(TokioDuration::from_secs(timeout_secs), child.wait()).await;
    match waited {
        Ok(status_res) => {
            let status = status_res.map_err(|e| format!("Failed to wait for {}: {}", program, e))?;
            let stdout = stdout_task
                .await
                .map_err(|e| format!("stdout task failed: {}", e))??;
            let stderr = stderr_task
                .await
                .map_err(|e| format!("stderr task failed: {}", e))??;
            Ok(std::process::Output { status, stdout, stderr })
        }
        Err(_) => {
            let _ = child.kill().await;
            stdout_task.abort();
            stderr_task.abort();
            Err(format!("Timed out after {}s", timeout_secs))
        }
    }
}

/// Get external IP address via HTTP services (robust with proxy support)
pub async fn get_external_ip(proxy: Option<String>) -> Option<String> {
    eprintln!("[IpCheck] Starting IP check with proxy: {:?}", proxy);
    
    // Build NEW client each time to avoid connection pooling/caching
    let client_builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .pool_max_idle_per_host(0) // Don't reuse connections
        .tcp_nodelay(true);

    let client = if let Some(proxy_url) = proxy.as_deref() {
        if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
             match client_builder.proxy(proxy).build() {
                 Ok(c) => c,
                 Err(e) => {
                     eprintln!("[IpCheck] Failed to build proxy client: {}", e);
                     return None;
                 }
             }
        } else {
            eprintln!("[IpCheck] Invalid proxy URL: {}", proxy_url);
            // Fallback to direct connection if proxy is invalid? Or fail?
            // Better to fail or let the user know. For now, let's return None or try direct.
            // Let's try direct as fallback but log error.
            match client_builder.build() {
                Ok(c) => c,
                Err(_) => return None,
            }
        }
    } else {
        match client_builder.build() {
            Ok(c) => c,
            Err(_) => return None,
        }
    };

    // Try multiple services (2ip.io first for better RU compatibility)
    let services = [
        "https://2ip.io/json",
        "https://api.ipify.org?format=json",
        "https://ipinfo.io/json",
        "https://ifconfig.me/all.json",
    ];

    for service in services {
        eprintln!("[IpCheck] Trying service: {}", service);
        // Add cache-busting timestamp and no-cache headers
        let url = if service.contains('?') {
            format!("{}&_t={}", service, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis())
        } else {
            format!("{}?_t={}", service, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis())
        };
        match client.get(&url)
            .header("Cache-Control", "no-cache, no-store, must-revalidate")
            .header("Pragma", "no-cache")
            .send().await {
            Ok(response) => {
                if let Ok(text) = response.text().await {
                    eprintln!("[IpCheck] Response: {}", text.chars().take(200).collect::<String>());
                    
                    // Try parsing as 2ip.io format
                    if let Ok(info) = serde_json::from_str::<TwoIpResponse>(&text) {
                        if let Some(ip) = info.ip {
                            let country = info.country_code.or(info.country);
                            let ip_str = if let Some(c) = country {
                                format!("{} ({})", ip, c)
                            } else {
                                ip
                            };
                            eprintln!("[IpCheck] Success (2ip format): {}", ip_str);
                            return Some(ip_str);
                        }
                    }
                    
                    // Try parsing as ipinfo.io format
                    if let Ok(info) = serde_json::from_str::<IpInfoResponse>(&text) {
                        if let Some(ip) = info.ip {
                             let ip_str = if let Some(country) = info.country {
                                 format!("{} ({})", ip, country)
                             } else {
                                 ip
                             };
                             eprintln!("[IpCheck] Success (ipinfo format): {}", ip_str);
                             return Some(ip_str);
                        }
                    }
                    
                    // Try parsing as simple IP {"ip": "..."}
                    if let Ok(simple) = serde_json::from_str::<SimpleIp>(&text) {
                        eprintln!("[IpCheck] Success (simple format): {}", simple.ip);
                        return Some(simple.ip);
                    }
                    
                    eprintln!("[IpCheck] Could not parse response from {}", service);
                }
            }
            Err(e) => {
                eprintln!("[IpCheck] Service {} failed: {}", service, e);
                continue;
            }
        }
    }

    eprintln!("[IpCheck] All services failed");
    None
}

/// Get current network status (proxy, mode, IP)
pub async fn get_network_status_info(user_proxy: Option<String>) -> NetworkStatus {
    eprintln!("[NetworkStatus] Starting network status check...");
    
    // Determine mode using simple universal logic:
    // 1. TUN MODE: utun interface with 172.19.0.x address
    // 2. SOCKS5 MODE: sing-box running and listening on port
    // 3. DIRECT: neither
    
    let tun_active = is_tun_mode_active();
    let socks5_active = !tun_active && is_socks5_mode_active();
    
    let (mode, proxy, ip_check_proxy) = if tun_active {
        eprintln!("[NetworkStatus] ✅ TUN MODE detected");
        ("vpn".to_string(), None, None) // System routes through TUN
    } else if socks5_active {
        // Find working SOCKS5 proxy
        let detected_proxy = user_proxy.clone().or_else(auto_detect_proxy);
        eprintln!("[NetworkStatus] ✅ SOCKS5 MODE detected, proxy: {:?}", detected_proxy);
        let p = detected_proxy.clone();
        ("proxy".to_string(), detected_proxy, p)
    } else {
        eprintln!("[NetworkStatus] ❌ VPN OFF - direct mode");
        ("direct".to_string(), user_proxy.clone().or_else(auto_detect_proxy), None)
    };
    
    // Run checks in parallel with individual timeouts
    let proxy_clone = proxy.clone();
    
    let (proxy_check, external_ip, ytdlp) = tokio::join!(
        async {
            match timeout(TokioDuration::from_secs(5), check_proxy(&proxy_clone)).await {
                Ok(result) => result,
                Err(_) => {
                    eprintln!("[NetworkStatus] Proxy check timed out");
                    ProxyCheck { reachable: false, message: Some("Proxy check timed out".to_string()) }
                }
            }
        },
        async {
            match timeout(TokioDuration::from_secs(8), get_external_ip(ip_check_proxy)).await {
                Ok(result) => result,
                Err(_) => {
                    eprintln!("[NetworkStatus] IP check timed out");
                    None
                }
            }
        },
        check_ytdlp_freshness()
    );

    eprintln!("[NetworkStatus] Done. Mode={}, IP={:?}, proxy_ok={}", mode, external_ip, proxy_check.reachable);
    
    NetworkStatus {
        proxy,
        mode,
        external_ip,
        proxy_reachable: proxy_check.reachable,
        proxy_message: proxy_check.message,
        ytdlp_version: ytdlp.version,
        ytdlp_status: ytdlp.status,
        ytdlp_hint: ytdlp.hint,
    }
}

/// Quick TCP dial + simple HTTPS HEAD through proxy to see if it works.
async fn check_proxy(proxy: &Option<String>) -> ProxyCheck {
    let mut result = ProxyCheck::default();
    if proxy.is_none() {
        return result;
    }
    let proxy_url = proxy.clone().unwrap();

    // TCP check
    if let Some(port) = parse_port(&proxy_url) {
        let addr = format!("127.0.0.1:{}", port);
        if let Ok(sock_addr) = addr.parse() {
            if TcpStream::connect_timeout(&sock_addr, Duration::from_millis(300)).is_ok() {
                result.reachable = true;
            } else {
                result.message = Some(format!("Proxy port {} is closed", port));
                return result;
            }
        }
    }

    // HTTP HEAD check via reqwest (may fail if proxy blocks CONNECT)
    let mut client_builder = reqwest::Client::builder().timeout(Duration::from_secs(4));
    if let Ok(px) = reqwest::Proxy::all(&proxy_url) {
        client_builder = client_builder.proxy(px);
    }

    match client_builder.build() {
        Ok(c) => {
            let res = c
                .head("https://www.youtube.com/")
                .send()
                .await;
            match res {
                Ok(r) => {
                    if r.status().is_success() || r.status().is_redirection() {
                        result.reachable = true;
                        result.message = Some("Proxy reachable".to_string());
                    } else {
                        result.message = Some(format!("Proxy responded with status {}", r.status()));
                    }
                }
                Err(e) => {
                    result.message = Some(format!("Proxy HEAD failed: {}", shorten(&e.to_string(), 120)));
                }
            }
        }
        Err(e) => {
            result.message = Some(format!("Proxy build failed: {}", shorten(&e.to_string(), 120)));
        }
    }

    result
}

fn parse_port(proxy: &str) -> Option<u16> {
    let cleaned = proxy.replace("socks5h://", "")
        .replace("socks5://", "")
        .replace("http://", "")
        .replace("https://", "");
    let parts: Vec<&str> = cleaned.split(':').collect();
    if parts.len() >= 2 {
        parts[1].parse::<u16>().ok()
    } else {
        None
    }
}

fn shorten(text: &str, max: usize) -> String {
    if text.len() <= max {
        text.to_string()
    } else {
        format!("{}…", &text[..max])
    }
}

async fn check_ytdlp_freshness() -> YtDlpFreshness {
    // Find yt-dlp binary path first
    let ytdlp_path = find_ytdlp_path();
    eprintln!("[YtDlpCheck] Using path: {:?}", ytdlp_path);
    
    let path = match ytdlp_path {
        Some(p) => p,
        None => {
            eprintln!("[YtDlpCheck] yt-dlp not found in common paths");
            return YtDlpFreshness {
                version: None,
                status: "missing".to_string(),
                hint: Some("yt-dlp not found. Install: brew install yt-dlp".to_string()),
            };
        }
    };

    // Run yt-dlp --version synchronously in a blocking thread (more reliable)
    let path_clone = path.clone();
    let version_out = tokio::task::spawn_blocking(move || {
        std::process::Command::new(&path_clone)
            .arg("--version")
            .output()
    }).await;

    let version_out = match version_out {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            eprintln!("[YtDlpCheck] Command failed: {}", e);
            return YtDlpFreshness {
                version: None,
                status: "unknown".to_string(),
                hint: Some(format!("yt-dlp error: {}", e)),
            };
        }
        Err(e) => {
            eprintln!("[YtDlpCheck] Task failed: {}", e);
            return YtDlpFreshness {
                version: None,
                status: "unknown".to_string(),
                hint: Some("yt-dlp check failed".to_string()),
            };
        }
    };

    if version_out.status.success() {
        let v = String::from_utf8_lossy(&version_out.stdout).trim().to_string();
        eprintln!("[YtDlpCheck] Local version: {}", v);
        let local_date = parse_version_date(&v);

        // Skip GitHub check - it often times out and slows down UI
        // Just use local version age heuristic
        let status = if local_date.is_some() { "ok".to_string() } else { "unknown".to_string() };
        let hint = local_date.map(|d| format!("Version date: {}", d));

        return YtDlpFreshness {
            version: Some(v),
            status,
            hint,
        };
    }

    YtDlpFreshness {
        version: None,
        status: "error".to_string(),
        hint: Some("yt-dlp found but --version failed".to_string()),
    }
}

fn freshness_status(local: Option<Date>, upstream: Option<Date>) -> String {
    let today = OffsetDateTime::now_utc().date();

    // If no local version, already handled elsewhere.
    if let Some(local_date) = local {
        // If upstream is known and not newer → ok
        if let Some(up_date) = upstream {
            if up_date <= local_date {
                return "ok".to_string();
            }
            // Upstream newer
            let age = (today - local_date).whole_days();
            if age > 30 {
                return "stale".to_string();
            }
            return "ok".to_string();
        } else {
            // Upstream unknown: rely on age heuristic but softer wording
            let age = (today - local_date).whole_days();
            if age > 45 {
                return "stale".to_string();
            }
            return "ok".to_string();
        }
    }

    "missing".to_string()
}

fn build_hint(local: Option<Date>, upstream: Option<Date>, status: &str) -> Option<String> {
    if status == "missing" {
        return Some("yt-dlp not found. Install via brew or pip.".to_string());
    }
    let mut parts = Vec::new();
    if let Some(ld) = local {
        parts.push(format!("Local: {}", ld));
    }
    if let Some(ud) = upstream {
        parts.push(format!("Upstream: {}", ud));
    } else {
        parts.push("Upstream: unknown (GitHub unavailable)".to_string());
    }

    if status == "stale" {
        parts.push("Recommendation: brew upgrade yt-dlp".to_string());
    }

    Some(parts.join(" | "))
}

/// Find yt-dlp binary in common paths
fn find_ytdlp_path() -> Option<String> {
    let common_paths = vec![
        "/opt/homebrew/bin/yt-dlp",  // Homebrew on Apple Silicon
        "/usr/local/bin/yt-dlp",     // Homebrew on Intel Mac
        "/usr/bin/yt-dlp",           // System installation
    ];

    for path in common_paths {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }

    // Try PATH via which
    if let Ok(output) = std::process::Command::new("which").arg("yt-dlp").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(path);
            }
        }
    }

    None
}

fn parse_version_date(version: &str) -> Option<Date> {
    // yt-dlp version format is typically YYYY.MM.DD or YYYY.MM.DD.<commit>
    let date_part = version.split('.').take(3).collect::<Vec<_>>().join(".");
    if let Ok(fmt) = format_description::parse("[year].[month].[day]") {
        Date::parse(&date_part, &fmt).ok()
    } else {
        None
    }
}

async fn fetch_upstream_release_date() -> Option<Date> {
    // Best effort: fetch GitHub latest release published_at
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("youtube-downloader-tauri")
        .build()
        .ok()?;

    let resp = client
        .get("https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest")
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let json: serde_json::Value = resp.json().await.ok()?;
    let published = json["published_at"].as_str()?;
    // format: "2025-12-08T21:14:22Z"
    let parsed = OffsetDateTime::parse(published, &time::format_description::well_known::Rfc3339).ok()?;
    Some(parsed.date())
}

/// Check if TUN mode (system-wide VPN) is active
/// Logic: utun interface with 172.19.0.x address exists
fn is_tun_mode_active() -> bool {
    // Check ifconfig for utun interface with 172.19.0.x IP
    let ifconfig = std::process::Command::new("ifconfig")
        .output();
    
    if let Ok(output) = ifconfig {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("utun") {
                // Check next few lines for 172.19.0 IP
                for j in 1..=3 {
                    if i + j < lines.len() && lines[i + j].contains("172.19.0") {
                        eprintln!("[TunDetect] ✅ TUN MODE: {} with 172.19.0.x", 
                            line.split(':').next().unwrap_or("utun"));
                        return true;
                    }
                }
            }
        }
    }
    
    false
}

/// Check if SOCKS5 mode is active (sing-box running and listening)
fn is_socks5_mode_active() -> bool {
    // Check if sing-box process is running
    let pgrep = std::process::Command::new("pgrep")
        .arg("sing-box")
        .output();
    
    if !pgrep.map(|o| o.status.success()).unwrap_or(false) {
        return false;
    }
    
    // Check if sing-box is listening on any port
    let lsof = std::process::Command::new("lsof")
        .args(["-c", "sing-box", "-i", "-P"])
        .output();
    
    if let Ok(output) = lsof {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("LISTEN") {
            eprintln!("[TunDetect] ✅ SOCKS5 MODE: sing-box listening");
            return true;
        }
    }
    
    false
}

/// Find sing-box configuration file
fn find_singbox_config() -> Option<String> {
    let mut possible_paths: Vec<String> = vec![
        "/usr/local/etc/sing-box/config.json".to_string(),
        "/etc/sing-box/config.json".to_string(),
        "/opt/homebrew/etc/sing-box/config.json".to_string(),
    ];
    
    // Add home directory config path
    if let Some(home) = dirs::home_dir() {
        possible_paths.push(home.join(".config/sing-box/config.json").to_string_lossy().to_string());
    }
    
    for path in possible_paths {
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }
    
    None
}

/// Auto-detect local SOCKS5 proxy
/// Checks XRAY/sing-box config ports first, then common ports, then scans via lsof
pub fn auto_detect_proxy() -> Option<String> {
    // Collect all candidate ports: config ports first, then common ports
    let mut candidate_ports: Vec<u16> = Vec::new();
    
    // 1. Try to find XRAY/sing-box config ports
    let xray_ports = detect_all_xray_socks_ports();
    for port in &xray_ports {
        eprintln!("[ProxyDetect] Found SOCKS5 port in config: {}", port);
        candidate_ports.push(*port);
    }
    
    // 2. Try to find sing-box listening ports via lsof
    if let Some(singbox_ports) = detect_singbox_ports() {
        for port in singbox_ports {
            if !candidate_ports.contains(&port) {
                eprintln!("[ProxyDetect] Found sing-box port: {}", port);
                candidate_ports.push(port);
            }
        }
    }
    
    // 3. Add common SOCKS5 ports
    let common_ports = vec![
        2080,   // sing-box default
        1080,   // Standard SOCKS5
        7890,   // Clash
        7891,   // Clash SOCKS
        10808,  // V2RayN
        10809,  // V2RayN SOCKS
        1081,   // Alternative
        52838,  // XRAY
        52864,  // XRAY alternative
        9050,   // Tor
        9150,   // Tor Browser
    ];
    
    for port in common_ports {
        if !candidate_ports.contains(&port) {
            candidate_ports.push(port);
        }
    }
    
    // 4. Test each port and return first working one
    for port in &candidate_ports {
        if test_socks5_port(*port) {
            eprintln!("[ProxyDetect] ✓ Found working SOCKS5 on port {}", port);
            return Some(format!("socks5h://127.0.0.1:{}", port));
        }
    }
    
    // 5. Last resort: scan popular port range quickly
    eprintln!("[ProxyDetect] Scanning additional ports...");
    for port in [2081, 2082, 8080, 8118, 3128, 20170, 20171, 51837] {
        if !candidate_ports.contains(&port) && test_socks5_port(port) {
            eprintln!("[ProxyDetect] ✓ Found working SOCKS5 on port {}", port);
            return Some(format!("socks5h://127.0.0.1:{}", port));
        }
    }
    
    eprintln!("[ProxyDetect] No working SOCKS5 proxy detected");
    None
}

/// Detect sing-box listening ports via lsof
fn detect_singbox_ports() -> Option<Vec<u16>> {
    // Try to find sing-box process and its listening ports
    let output = std::process::Command::new("lsof")
        .args(["-i", "-P", "-n"])
        .output()
        .ok()?;
    
    if !output.status.success() {
        return None;
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut ports = Vec::new();
    
    for line in stdout.lines() {
        // Look for sing-box or any LISTEN on localhost
        let lower = line.to_lowercase();
        if (lower.contains("sing-box") || lower.contains("singbox") || lower.contains("xray") || lower.contains("v2ray"))
            && lower.contains("listen")
        {
            // Extract port from lines like: sing-box 1234 user 5u IPv4 ... TCP 127.0.0.1:2080 (LISTEN)
            if let Some(port) = extract_port_from_lsof_line(line) {
                if !ports.contains(&port) {
                    ports.push(port);
                }
            }
        }
    }
    
    if ports.is_empty() {
        None
    } else {
        Some(ports)
    }
}

fn extract_port_from_lsof_line(line: &str) -> Option<u16> {
    // Format: ... TCP 127.0.0.1:2080 (LISTEN) or *:2080
    for part in line.split_whitespace() {
        if part.contains(':') && (part.starts_with("127.0.0.1:") || part.starts_with("*:") || part.starts_with("localhost:")) {
            let port_str = part.split(':').last()?;
            // Remove (LISTEN) if present
            let port_str = port_str.trim_end_matches("(LISTEN)");
            return port_str.parse::<u16>().ok();
        }
    }
    None
}

/// Detect all XRAY SOCKS5 ports from config files (may return multiple)
fn detect_all_xray_socks_ports() -> Vec<u16> {
    let mut ports = Vec::new();
    
    // Common XRAY config locations on macOS
    let temp_path = std::env::temp_dir().join("apiai_xray_config.json");
    let home_path = dirs::home_dir().map(|h| h.join(".config/xray/config.json"));
    
    let mut config_paths: Vec<std::path::PathBuf> = vec![
        std::path::PathBuf::from("/var/folders/y_/dzbfyg5j0zsd130_ssss69k40000gn/T/apiai_xray_config.json"),
        std::path::PathBuf::from("/tmp/xray_config.json"),
        temp_path,
    ];
    
    if let Some(path) = home_path {
        config_paths.push(path);
    }
    
    // Also scan /tmp for any xray config files
    if let Ok(entries) = std::fs::read_dir("/tmp") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.contains("xray") && name.ends_with(".json") {
                    config_paths.push(path);
                }
            }
        }
    }
    
    // Also scan temp dir for xray configs
    let temp_dir = std::env::temp_dir();
    if let Ok(entries) = std::fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.contains("xray") && name.ends_with(".json") && !config_paths.contains(&path) {
                    config_paths.push(path);
                }
            }
        }
    }
    
    for path in config_paths {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(inbounds) = json["inbounds"].as_array() {
                    for inbound in inbounds {
                        if inbound["protocol"].as_str() == Some("socks") {
                            if let Some(port) = inbound["port"].as_u64() {
                                let port_u16 = port as u16;
                                if !ports.contains(&port_u16) {
                                    eprintln!("[ProxyDetect] Found SOCKS5 port {} in config: {:?}", port_u16, path);
                                    ports.push(port_u16);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    ports
}


/// Test if SOCKS5 proxy is running on port
fn test_socks5_port(port: u16) -> bool {
    let addr = format!("127.0.0.1:{}", port);
    
    match TcpStream::connect_timeout(
        &addr.parse().unwrap(),
        Duration::from_millis(200)
    ) {
        Ok(_) => {
            eprintln!("[ProxyDetect] Port {} is open", port);
            true
        }
        Err(_) => false,
    }
}

/// Build proxy arguments for yt-dlp
pub fn get_proxy_args(config: &NetworkConfig) -> Vec<String> {
    let mut args = Vec::new();
    
    if let Some(proxy) = &config.proxy {
        args.push("--proxy".to_string());
        args.push(proxy.clone());
    }
    
    args
}

/// Build timeout arguments for yt-dlp
pub fn get_timeout_args(config: &NetworkConfig) -> Vec<String> {
    let mut args = Vec::new();
    
    if let Some(timeout) = config.timeout {
        args.push("--socket-timeout".to_string());
        args.push(timeout.to_string());
    }
    
    args
}

