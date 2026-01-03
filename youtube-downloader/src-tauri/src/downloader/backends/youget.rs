use async_trait::async_trait;
use std::process::{Command, Stdio};
use serde_json::Value;

use crate::downloader::errors::DownloadError;
use crate::downloader::models::{DownloadOptions, DownloadProgress, VideoInfo};
use crate::downloader::traits::{DownloaderBackend, ProgressEmitter};
use crate::downloader::tools::{ToolManager, ToolType};
use crate::downloader::utils;

pub struct YouGetBackend {
    binary_path: String,
    http_proxy: Option<String>,
}

impl YouGetBackend {
    pub fn new() -> Self {
        let manager = ToolManager::new();
        let (path, _) = manager.get_tool_info(ToolType::YouGet).path
            .map(|p| (p, "".to_string()))
            .unwrap_or_else(|| ("you-get".to_string(), "".to_string()));
        
        // Auto-detect HTTP proxy (you-get supports -x flag)
        let http_proxy = Self::detect_http_proxy();
        if http_proxy.is_some() {
            eprintln!("[you-get] Using HTTP proxy: {:?}", http_proxy);
        } else {
            // Check for SOCKS proxy (requires PySocks library)
            let socks = utils::auto_detect_proxy();
            if socks.is_some() {
                eprintln!("[you-get] Warning: SOCKS5 proxy found but requires PySocks library");
            }
        }
            
        Self { binary_path: path, http_proxy }
    }
    
    /// Detect HTTP proxy from XRAY config
    fn detect_http_proxy() -> Option<String> {
        let xray_config_path = std::env::temp_dir().join("apiai_xray_config.json");
        if let Ok(content) = std::fs::read_to_string(&xray_config_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                if let Some(inbounds) = json["inbounds"].as_array() {
                    for inbound in inbounds {
                        if inbound["protocol"].as_str() == Some("http") {
                            if let (Some(listen), Some(port)) = (
                                inbound["listen"].as_str(),
                                inbound["port"].as_u64()
                            ) {
                                return Some(format!("{}:{}", listen, port));
                            }
                        }
                    }
                }
            }
        }
        None
    }
    
    /// Check if URL is YouTube
    fn is_youtube_url(url: &str) -> bool {
        url.contains("youtube.com") || url.contains("youtu.be")
    }
    
    /// Build proxy args for you-get
    fn get_proxy_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if let Some(ref proxy) = self.http_proxy {
            args.push("-x".to_string());
            args.push(proxy.clone());
        }
        args
    }
}

#[async_trait]
impl DownloaderBackend for YouGetBackend {
    fn name(&self) -> &'static str {
        "you-get"
    }

    async fn get_video_info(&self, url: &str) -> Result<VideoInfo, DownloadError> {
        // Warn about YouTube compatibility
        if Self::is_youtube_url(url) {
            eprintln!("[you-get] Warning: YouTube support is unreliable in you-get");
        }
        
        let mut args = vec!["--json".to_string()];
        args.extend(self.get_proxy_args());
        args.push(url.to_string());
        
        let output = Command::new(&self.binary_path)
            .args(&args)
            .output()
            .map_err(|e| DownloadError::ToolNotFound(format!("you-get: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            let error_msg = if !stderr.is_empty() {
                stderr.to_string()
            } else if !stdout.is_empty() {
                stdout.to_string()
            } else {
                format!("you-get failed with exit code: {:?}", output.status.code())
            };
            
            return Err(Self::parse_youget_error(&error_msg, url));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let json: Value = serde_json::from_str(&json_str)
            .map_err(|e| DownloadError::ParseError(format!("JSON parse error: {}", e)))?;

        let title = json["title"].as_str().unwrap_or("Unknown").to_string();
        
        Ok(VideoInfo {
            id: "".to_string(),
            title,
            uploader: "Unknown".to_string(),
            duration: "0:00".to_string(),
            thumbnail: "".to_string(),
        })
    }

    async fn download(
        &self,
        url: &str,
        options: DownloadOptions,
        app_handle: tauri::AppHandle,
    ) -> Result<(), DownloadError> {
        let emitter = ProgressEmitter::new(app_handle);
        
        // Warn about YouTube
        if Self::is_youtube_url(url) {
            emitter.emit(DownloadProgress {
                percent: 0.0,
                status: "you-get: YouTube may not work (use yt-dlp)".to_string(),
            });
        }
        
        let proxy_status = if self.http_proxy.is_some() { "proxy=on" } else { "proxy=off" };
        emitter.emit(DownloadProgress {
            percent: 0.0,
            status: format!("you-get: starting download ({})", proxy_status),
        });

        // Build args: you-get -o DIR [-x PROXY] URL
        let mut args = vec!["-o".to_string(), options.output_path.clone()];
        args.extend(self.get_proxy_args());
        args.push(url.to_string());

        let output = Command::new(&self.binary_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| DownloadError::ExecutionError(format!("Download failed: {}", e)))?;

        if output.status.success() {
            emitter.emit(DownloadProgress {
                percent: 100.0,
                status: "you-get: download complete!".to_string(),
            });
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            let error_msg = if !stderr.is_empty() {
                stderr.to_string()
            } else if !stdout.is_empty() {
                stdout.to_string()
            } else {
                format!("you-get failed with exit code: {:?}", output.status.code())
            };
            
            Err(Self::parse_youget_error(&error_msg, url))
        }
    }
}

impl YouGetBackend {
    /// Parse you-get error and return appropriate DownloadError
    fn parse_youget_error(error: &str, url: &str) -> DownloadError {
        // Connection refused / network issues
        if error.contains("Connection refused") || error.contains("urlopen error") {
            if Self::is_youtube_url(url) {
                return DownloadError::NetworkError(
                    "Network error. YouTube may be blocked.\n\
                     You-get has issues with proxy for HTTPS.\n\
                     Solution: Use yt-dlp instead.".to_string()
                );
            }
            return DownloadError::NetworkError(
                "Connection refused. Check your network or proxy settings.".to_string()
            );
        }
        
        // Generic "oops" error from you-get
        if error.contains("oops, something went wrong") {
            if Self::is_youtube_url(url) {
                return DownloadError::UnsupportedUrl(
                    "You-get failed for YouTube.\n\
                     You-get has limited YouTube support.\n\
                     Solution: Use yt-dlp instead — it's actively maintained.".to_string()
                );
            }
            return DownloadError::Unknown(
                "You-get encountered an error. Try updating: pipx upgrade you-get".to_string()
            );
        }
        
        // Timeout
        if error.contains("timed out") || error.contains("timeout") {
            return DownloadError::NetworkTimeout;
        }
        
        // 403 Forbidden
        if error.contains("403") || error.contains("Forbidden") {
            return DownloadError::NetworkError(
                "Access denied (403). Try yt-dlp instead.".to_string()
            );
        }
        
        // Generic error
        DownloadError::Unknown(error.to_string())
    }
}
