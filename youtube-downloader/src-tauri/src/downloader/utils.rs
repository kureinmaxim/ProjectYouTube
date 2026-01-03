// Helper functions for backend implementations

use crate::downloader::models::NetworkConfig;
use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command as TokioCommand;
use tokio::time::{timeout, Duration as TokioDuration};

/// Network status information for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub proxy: Option<String>,
    pub mode: String,        // "direct", "proxy", "vpn"
    pub external_ip: Option<String>,
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
    
    // Build client with optional proxy
    let client_builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(10));

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

    // Try multiple services
    let services = [
        "https://ipinfo.io/json",
        "https://api.ipify.org?format=json",
        "https://ifconfig.me/all.json",
    ];

    for service in services {
        eprintln!("[IpCheck] Trying service: {}", service);
        match client.get(service).send().await {
            Ok(response) => {
                if let Ok(text) = response.text().await {
                    // Try parsing as full info first
                    if let Ok(info) = serde_json::from_str::<IpInfoResponse>(&text) {
                        if let Some(ip) = info.ip {
                             let ip_str = if let Some(country) = info.country {
                                 format!("{} ({})", ip, country)
                             } else {
                                 ip
                             };
                             eprintln!("[IpCheck] Success: {}", ip_str);
                             return Some(ip_str);
                        }
                    }
                    // Try parsing as simple IP
                    if let Ok(simple) = serde_json::from_str::<SimpleIp>(&text) {
                        eprintln!("[IpCheck] Success Simple: {}", simple.ip);
                        return Some(simple.ip);
                    }
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
    // Determine proxy - user-supplied or auto-detected
    let proxy = user_proxy.or_else(auto_detect_proxy);
    
    // Determine mode based on proxy
    let mode = match &proxy {
        Some(p) if p.contains("socks") => "proxy".to_string(),
        Some(_) => "proxy".to_string(),
        None => "direct".to_string(),
    };
    
    // Get external IP (async) pass the proxy!
    let external_ip = get_external_ip(proxy.clone()).await;
    
    NetworkStatus {
        proxy,
        mode,
        external_ip,
    }
}


/// Auto-detect local SOCKS5 proxy
/// Checks common ports and XRAY config
pub fn auto_detect_proxy() -> Option<String> {
    // Try to find XRAY config first (most reliable)
    if let Some(xray_port) = detect_xray_socks_port() {
        let proxy_url = format!("socks5h://127.0.0.1:{}", xray_port);
        eprintln!("[ProxyDetect] Found XRAY SOCKS5 on port {}", xray_port);
        return Some(proxy_url);
    }
    
    // Common SOCKS5 ports to check
    let common_ports = vec![
        1080,   // Standard SOCKS5
        7890,   // Clash
        10808,  // V2RayN
        1081,   // Alternative
        7891,   // Alternative Clash
    ];
    
    for port in common_ports {
        if test_socks5_port(port) {
            eprintln!("[ProxyDetect] Found SOCKS5 on common port {}", port);
            return Some(format!("socks5h://127.0.0.1:{}", port));
        }
    }
    
    eprintln!("[ProxyDetect] No SOCKS5 proxy detected");
    None
}

/// Detect XRAY SOCKS5 port from config file
fn detect_xray_socks_port() -> Option<u16> {
    // Common XRAY config locations on macOS
    let temp_path = std::env::temp_dir().join("apiai_xray_config.json");
    let home_path = dirs::home_dir().map(|h| h.join(".config/xray/config.json"));
    
    let mut config_paths = vec![
        "/var/folders/y_/dzbfyg5j0zsd130_ssss69k40000gn/T/apiai_xray_config.json",
        "/tmp/xray_config.json",
    ];
    
    if let Some(ref path) = home_path {
        if let Some(path_str) = path.to_str() {
            config_paths.push(path_str);
        }
    }
    
    for path in config_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(inbounds) = json["inbounds"].as_array() {
                    for inbound in inbounds {
                        if inbound["protocol"].as_str() == Some("socks") {
                            if let Some(port) = inbound["port"].as_u64() {
                                eprintln!("[ProxyDetect] Found SOCKS5 in XRAY config: {}", path);
                                return Some(port as u16);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Also try temp directory
    if let Ok(content) = std::fs::read_to_string(&temp_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(inbounds) = json["inbounds"].as_array() {
                for inbound in inbounds {
                    if inbound["protocol"].as_str() == Some("socks") {
                        if let Some(port) = inbound["port"].as_u64() {
                            eprintln!("[ProxyDetect] Found SOCKS5 in XRAY temp config");
                            return Some(port as u16);
                        }
                    }
                }
            }
        }
    }
    
    None
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
