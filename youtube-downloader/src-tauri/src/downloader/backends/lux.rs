use async_trait::async_trait;
use std::process::{Command, Stdio};
use serde_json::Value;

use crate::downloader::errors::DownloadError;
use crate::downloader::models::{DownloadOptions, DownloadProgress, VideoInfo};
use crate::downloader::traits::{DownloaderBackend, ProgressEmitter};
use crate::downloader::tools::{ToolManager, ToolType};
use crate::downloader::utils;

pub struct LuxBackend {
    binary_path: String,
    http_proxy: Option<String>,
}

impl LuxBackend {
    pub fn new() -> Self {
        let manager = ToolManager::new();
        let (path, _) = manager.get_tool_info(ToolType::Lux).path
            .map(|p| (p, "".to_string()))
            .unwrap_or_else(|| ("lux".to_string(), "".to_string()));
        
        // Auto-detect HTTP proxy (lux works better with HTTP proxy than SOCKS5)
        let http_proxy = Self::detect_http_proxy();
        if http_proxy.is_some() {
            eprintln!("[lux] Using HTTP proxy: {:?}", http_proxy);
        } else {
            // Fallback to SOCKS5 (may not work well with lux)
            let socks_proxy = utils::auto_detect_proxy();
            if socks_proxy.is_some() {
                eprintln!("[lux] Warning: Only SOCKS5 proxy found. Lux works better with HTTP proxy.");
            }
        }
            
        Self { binary_path: path, http_proxy }
    }
    
    /// Detect HTTP proxy from XRAY config
    fn detect_http_proxy() -> Option<String> {
        // Try to read XRAY config to find HTTP proxy port
        let xray_config_path = std::env::temp_dir().join("apiai_xray_config.json");
        if let Ok(content) = std::fs::read_to_string(&xray_config_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                // Look for HTTP inbound
                if let Some(inbounds) = json["inbounds"].as_array() {
                    for inbound in inbounds {
                        if inbound["protocol"].as_str() == Some("http") {
                            if let (Some(listen), Some(port)) = (
                                inbound["listen"].as_str(),
                                inbound["port"].as_u64()
                            ) {
                                return Some(format!("http://{}:{}", listen, port));
                            }
                        }
                    }
                }
            }
        }
        None
    }
    
    /// Configure command with proxy environment variables
    fn configure_proxy(&self, cmd: &mut Command) {
        if let Some(ref proxy) = self.http_proxy {
            // Lux uses standard HTTP proxy environment variables
            cmd.env("HTTP_PROXY", proxy);
            cmd.env("HTTPS_PROXY", proxy);
            cmd.env("http_proxy", proxy);
            cmd.env("https_proxy", proxy);
        }
    }
    
    /// Check if URL is YouTube (lux has issues with YouTube)
    fn is_youtube_url(url: &str) -> bool {
        url.contains("youtube.com") || url.contains("youtu.be")
    }
}

#[async_trait]
impl DownloaderBackend for LuxBackend {
    fn name(&self) -> &'static str {
        "lux"
    }

    async fn get_video_info(&self, url: &str) -> Result<VideoInfo, DownloadError> {
        // Warn about YouTube compatibility issues
        if Self::is_youtube_url(url) {
            eprintln!("[lux] Warning: YouTube support is broken in lux v0.24.1");
        }
        
        let mut cmd = Command::new(&self.binary_path);
        cmd.args(["-i", "-j", url]); // -i info only, -j JSON output
        
        // Configure proxy via environment variables
        self.configure_proxy(&mut cmd);
        
        let output = cmd
            .output()
            .map_err(|e| DownloadError::ToolNotFound(format!("lux: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // Build detailed error message
            let error_msg = if !stderr.is_empty() {
                stderr.to_string()
            } else if !stdout.is_empty() && stdout.contains("error") {
                stdout.to_string()
            } else {
                format!("lux failed with exit code: {:?}", output.status.code())
            };
            
            return Err(Self::parse_lux_error(&error_msg, url));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        
        // Lux outputs JSON array for videos
        let json: Value = serde_json::from_str(&json_str)
            .map_err(|e| DownloadError::ParseError(format!("JSON parse error: {}", e)))?;

        // Handle both array and object formats
        let video = if json.is_array() {
            json.get(0)
        } else {
            Some(&json)
        };
        
        let title = video
            .and_then(|v| v["title"].as_str())
            .unwrap_or("Unknown")
            .to_string();
        
        Ok(VideoInfo {
            id: video.and_then(|v| v["id"].as_str()).unwrap_or("").to_string(),
            title,
            uploader: video.and_then(|v| v["uploader"].as_str()).unwrap_or("Unknown").to_string(),
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
        
        // Check YouTube compatibility
        if Self::is_youtube_url(url) {
            emitter.emit(DownloadProgress {
                percent: 0.0,
                status: "lux: YouTube may not work (use yt-dlp)".to_string(),
            });
        }
        
        let proxy_status = if self.http_proxy.is_some() { "http-proxy=on" } else { "proxy=off" };
        emitter.emit(DownloadProgress {
            percent: 0.0,
            status: format!("lux: starting download ({})", proxy_status),
        });

        let mut args: Vec<&str> = vec!["-o", &options.output_path];
        
        // Audio-only mode
        if options.extract_audio {
            args.push("--audio-only");
            emitter.emit(DownloadProgress {
                percent: 1.0,
                status: "lux: audio-only mode".to_string(),
            });
        }

        args.push(url);

        let mut cmd = Command::new(&self.binary_path);
        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        // Configure proxy via environment variables
        self.configure_proxy(&mut cmd);
        
        let output = cmd
            .output()
            .map_err(|e| DownloadError::ExecutionError(format!("Download failed: {}", e)))?;

        if output.status.success() {
            emitter.emit(DownloadProgress {
                percent: 100.0,
                status: "lux: download complete!".to_string(),
            });
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // Build detailed error message
            let error_msg = if !stderr.is_empty() {
                stderr.to_string()
            } else if !stdout.is_empty() {
                stdout.to_string()
            } else {
                format!("lux failed with exit code: {:?}", output.status.code())
            };
            
            Err(Self::parse_lux_error(&error_msg, url))
        }
    }
}

impl LuxBackend {
    /// Parse lux error and return appropriate DownloadError
    fn parse_lux_error(error: &str, url: &str) -> DownloadError {
        // YouTube cipher error (lux is outdated for YouTube)
        if error.contains("cipher not found") || error.contains("cipher") {
            return DownloadError::UnsupportedUrl(
                format!(
                    "Lux cannot decode YouTube cipher (v0.24.1 is outdated).\n\
                     YouTube changes encryption frequently.\n\
                     Solution: Use yt-dlp instead — it's actively maintained."
                )
            );
        }
        
        // Network timeout
        if error.contains("operation timed out") || error.contains("timeout") || error.contains("timed out") {
            if Self::is_youtube_url(url) {
                return DownloadError::NetworkError(
                    "Network timeout. YouTube may be blocked. Use VPN/proxy.".to_string()
                );
            }
            return DownloadError::NetworkTimeout;
        }
        
        // Access denied
        if error.contains("403") || error.contains("Forbidden") {
            return DownloadError::NetworkError(
                "Access denied (403). Try yt-dlp instead.".to_string()
            );
        }
        
        // Unsupported site
        if error.contains("not support") || error.contains("unsupported") {
            return DownloadError::UnsupportedUrl(
                "This URL is not supported by lux. Try yt-dlp.".to_string()
            );
        }
        
        // Generic error
        DownloadError::Unknown(error.to_string())
    }
}
