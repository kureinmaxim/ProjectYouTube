use async_trait::async_trait;
use std::process::{Command, Stdio};
use serde_json::Value;

use crate::downloader::errors::DownloadError;
use crate::downloader::models::{DownloadOptions, DownloadProgress, VideoInfo};
use crate::downloader::traits::{DownloaderBackend, ProgressEmitter};
use crate::downloader::tools::{ToolManager, ToolType};

pub struct YouGetBackend {
    binary_path: String,
}

impl YouGetBackend {
    pub fn new() -> Self {
        let manager = ToolManager::new();
        let (path, _) = manager.get_tool_info(ToolType::YouGet).path
            .map(|p| (p, "".to_string()))
            .unwrap_or_else(|| ("you-get".to_string(), "".to_string()));
            
        Self { binary_path: path }
    }
}

#[async_trait]
impl DownloaderBackend for YouGetBackend {
    fn name(&self) -> &'static str {
        "you-get"
    }

    async fn get_video_info(&self, url: &str) -> Result<VideoInfo, DownloadError> {
        let output = Command::new(&self.binary_path)
            .args(["--json", url])
            .output()
            .map_err(|e| DownloadError::ToolNotFound(format!("you-get: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(error.to_string().into());
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
        emitter.emit(DownloadProgress {
            percent: 0.0,
            status: "Starting you-get download...".to_string(),
        });

        // you-get -o DIR URL
        let mut args = vec!["-o", &options.output_path];
        args.push(url);

        let output = Command::new(&self.binary_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| DownloadError::ExecutionError(format!("Download failed: {}", e)))?;

        if output.status.success() {
            emitter.emit(DownloadProgress {
                percent: 100.0,
                status: "Download complete!".to_string(),
            });
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(error.to_string().into())
        }
    }
}
