// InfoExtractor trait and common types

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::downloader::errors::DownloadError;

/// Extraction mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExtractorMode {
    /// Python module yt_dlp (better for YouTube, avoids bot detection)
    Python,
    /// CLI binary yt-dlp (faster, no Python dependency)
    Cli,
    /// Auto-select: Python → CLI fallback
    #[default]
    Auto,
}

impl fmt::Display for ExtractorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Python => write!(f, "python"),
            Self::Cli => write!(f, "cli"),
            Self::Auto => write!(f, "auto"),
        }
    }
}

/// Configuration for info extraction
#[derive(Debug, Clone)]
pub struct ExtractorConfig {
    /// Extraction mode (Python, CLI, or Auto)
    pub mode: ExtractorMode,
    /// SOCKS5/HTTP proxy URL
    pub proxy: Option<String>,
    /// Path to cookies.txt file
    pub cookies_path: Option<String>,
    /// Use cookies from browser (Chrome)
    pub cookies_from_browser: bool,
    /// Request timeout in seconds
    pub timeout_seconds: u32,
    /// YouTube player client (android, web, tv)
    pub player_client: Option<String>,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            mode: ExtractorMode::Auto,
            proxy: None,
            cookies_path: None,
            cookies_from_browser: true,
            timeout_seconds: 30,
            player_client: None,
        }
    }
}

impl ExtractorConfig {
    pub fn with_proxy(mut self, proxy: Option<String>) -> Self {
        self.proxy = proxy;
        self
    }

    pub fn with_cookies_path(mut self, path: Option<String>) -> Self {
        self.cookies_path = path;
        self
    }

    pub fn with_cookies_from_browser(mut self, enabled: bool) -> Self {
        self.cookies_from_browser = enabled;
        self
    }

    pub fn with_mode(mut self, mode: ExtractorMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_timeout(mut self, seconds: u32) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    pub fn with_player_client(mut self, client: Option<String>) -> Self {
        self.player_client = client;
        self
    }
}

/// Extended format information from yt-dlp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedFormat {
    /// Format ID (e.g., "137", "140")
    pub format_id: String,
    /// File extension (mp4, webm, m4a)
    pub ext: String,
    /// Resolution string (e.g., "1920x1080")
    pub resolution: Option<String>,
    /// Video width in pixels
    pub width: Option<u32>,
    /// Video height in pixels
    pub height: Option<u32>,
    /// Frames per second
    pub fps: Option<f32>,
    /// Video codec (avc1, vp9, av01, none)
    pub vcodec: Option<String>,
    /// Audio codec (mp4a, opus, none)
    pub acodec: Option<String>,
    /// File size in bytes
    pub filesize: Option<u64>,
    /// Approximate file size (when exact is unknown)
    pub filesize_approx: Option<u64>,
    /// Total bitrate in kbps
    pub tbr: Option<f32>,
    /// Audio bitrate in kbps
    pub abr: Option<f32>,
    /// Video bitrate in kbps
    pub vbr: Option<f32>,
    /// Format note (e.g., "1080p", "tiny")
    pub format_note: Option<String>,
    /// Whether this is video-only (no audio)
    pub video_only: bool,
    /// Whether this is audio-only (no video)
    pub audio_only: bool,
}

impl ExtendedFormat {
    /// Get effective file size (exact or approximate)
    pub fn effective_size(&self) -> Option<u64> {
        self.filesize.or(self.filesize_approx)
    }

    /// Check if format is H.264 (avc1)
    pub fn is_h264(&self) -> bool {
        self.vcodec
            .as_ref()
            .map_or(false, |v| v.starts_with("avc1"))
    }

    /// Check if format is VP9
    pub fn is_vp9(&self) -> bool {
        self.vcodec.as_ref().map_or(false, |v| v.starts_with("vp9"))
    }

    /// Check if format is AV1
    pub fn is_av1(&self) -> bool {
        self.vcodec
            .as_ref()
            .map_or(false, |v| v.starts_with("av01"))
    }

    /// Check if audio is AAC (m4a)
    pub fn is_aac(&self) -> bool {
        self.acodec
            .as_ref()
            .map_or(false, |a| a.starts_with("mp4a"))
    }
}

/// Extended video info with all formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedVideoInfo {
    pub id: String,
    pub title: String,
    pub uploader: String,
    pub duration_seconds: u64,
    pub thumbnail: String,
    pub webpage_url: String,
    pub formats: Vec<ExtendedFormat>,
}

/// Trait for info extractors
#[async_trait]
pub trait InfoExtractor: Send + Sync {
    /// Name of the extractor (for logging)
    fn name(&self) -> &'static str;

    /// Check if this extractor is available
    fn is_available(&self) -> bool;

    /// Extract video info with formats
    async fn extract(
        &self,
        url: &str,
        config: &ExtractorConfig,
    ) -> Result<ExtendedVideoInfo, DownloadError>;

    /// Extract only formats (lighter operation)
    async fn extract_formats(
        &self,
        url: &str,
        config: &ExtractorConfig,
    ) -> Result<Vec<ExtendedFormat>, DownloadError>;
}

