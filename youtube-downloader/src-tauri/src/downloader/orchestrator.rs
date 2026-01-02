// Orchestrator with fallback logic

use super::errors::DownloadError;
use super::models::{DownloadOptions, VideoInfo};
use super::traits::DownloaderBackend;

pub struct Downloader {
    backends: Vec<Box<dyn DownloaderBackend>>,
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
        }
    }

    pub fn add_backend(&mut self, backend: Box<dyn DownloaderBackend>) {
        self.backends.push(backend);
    }

    pub async fn get_video_info(&self, url: &str) -> Result<VideoInfo, DownloadError> {
        for backend in &self.backends {
            eprintln!("[Downloader] Trying backend: {}", backend.name());
            
            match backend.get_video_info(url).await {
                Ok(info) => {
                    eprintln!("[Downloader] ✓ Success with {}", backend.name());
                    return Ok(info);
                }
                Err(e) => {
                    eprintln!("[Downloader] ✗ {} failed: {}", backend.name(), e);
                }
            }
        }
        
        Err(DownloadError::Unknown(
            "All backends failed".to_string()
        ))
    }

    pub async fn download(
        &self,
        url: &str,
        options: DownloadOptions,
        app_handle: tauri::AppHandle,
    ) -> Result<(), DownloadError> {
        for backend in &self.backends {
            eprintln!("[Downloader] Trying download with: {}", backend.name());
            
            match backend.download(url, options.clone(), app_handle.clone()).await {
                Ok(()) => {
                    eprintln!("[Downloader] ✓ Download success with {}", backend.name());
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("[Downloader] ✗ {} download failed: {}", backend.name(), e);
                }
            }
        }
        
        Err(DownloadError::Unknown(
            "All backends failed to download".to_string()
        ))
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}
