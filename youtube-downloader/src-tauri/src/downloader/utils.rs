// Helper functions for backend implementations

use crate::downloader::models::NetworkConfig;
use std::net::TcpStream;
use std::time::Duration;

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
