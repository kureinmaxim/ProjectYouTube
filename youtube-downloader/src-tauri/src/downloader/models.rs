// Common data models for downloader

use serde::{Deserialize, Serialize};

/// Video information extracted from YouTube
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub uploader: String,
    pub duration: String,
    pub thumbnail: String,
}

/// Video format details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFormat {
    pub format_id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub filesize: Option<u64>,
}

/// Download options
#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub quality: String,
    pub output_path: String,
    pub extract_audio: bool,
    pub audio_format: Option<String>,
    pub proxy: Option<String>,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            quality: "720p".to_string(),
            output_path: dirs::download_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .to_string_lossy()
                .to_string(),
            extract_audio: false,
            audio_format: None,
            proxy: None,
        }
    }
}

/// Download progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub percent: f32,
    pub status: String,
}

/// Network configuration for backends
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// SOCKS5 proxy URL (e.g., "socks5://127.0.0.1:1080")
    pub proxy: Option<String>,
    
    /// Timeout in seconds
    pub timeout: Option<u32>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            proxy: None,
            timeout: Some(30),
        }
    }
}
