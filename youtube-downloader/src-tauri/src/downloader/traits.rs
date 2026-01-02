// Downloader backend trait definition

use async_trait::async_trait;
use tauri::Emitter;

use super::errors::DownloadError;
use super::models::{DownloadOptions, DownloadProgress, VideoInfo};

/// Trait for downloader backend implementations
#[async_trait]
pub trait DownloaderBackend: Send + Sync {
    /// Name of the backend (for logging)
    fn name(&self) -> &'static str;

    /// Get video information from URL
    async fn get_video_info(&self, url: &str) -> Result<VideoInfo, DownloadError>;

    /// Download video with progress updates
    async fn download(
        &self,
        url: &str,
        options: DownloadOptions,
        app_handle: tauri::AppHandle,
    ) -> Result<(), DownloadError>;
}

/// Progress emitter helper
pub struct ProgressEmitter {
    app_handle: tauri::AppHandle,
}

impl ProgressEmitter {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }

    pub fn emit(&self, progress: DownloadProgress) {
        let _ = self.app_handle.emit("download-progress", progress);
    }
}
