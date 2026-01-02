// Helper functions for backend implementations

use crate::downloader::models::NetworkConfig;

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
