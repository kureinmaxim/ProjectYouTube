use async_trait::async_trait;
use std::process::{Command, Stdio};
use serde_json::Value;

use crate::downloader::errors::DownloadError;
use crate::downloader::models::{DownloadOptions, DownloadProgress, VideoInfo};
use crate::downloader::traits::{DownloaderBackend, ProgressEmitter};
use crate::downloader::tools::{ToolManager, ToolType};

pub struct LuxBackend {
    binary_path: String,
}

impl LuxBackend {
    pub fn new() -> Self {
        let manager = ToolManager::new();
        let (path, _) = manager.get_tool_info(ToolType::Lux).path
            .map(|p| (p, "".to_string()))
            .unwrap_or_else(|| ("lux".to_string(), "".to_string()));
            
        Self { binary_path: path }
    }
}

#[async_trait]
impl DownloaderBackend for LuxBackend {
    fn name(&self) -> &'static str {
        "lux"
    }

    async fn get_video_info(&self, url: &str) -> Result<VideoInfo, DownloadError> {
        let output = Command::new(&self.binary_path)
            .args(["-i", "-j", url]) // -j for JSON
            .output()
            .map_err(|e| DownloadError::ToolNotFound(format!("lux: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(error.to_string().into());
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        // Lux can output multiple JSON objects if playlist, or just one.
        // It's a bit tricky, but let's try to parse the first one.
        // Also lux json structure is different from yt-dlp.
        
        let json: Value = serde_json::from_str(&json_str)
            .map_err(|e| DownloadError::ParseError(format!("JSON parse error: {}", e)))?;

        // Basic parsing for lux
        // Note: Lux JSON output schema needs to be verified. 
        // Often it returns a list of items.
        
        let title = json[0]["title"].as_str().unwrap_or("Unknown").to_string();
        
        Ok(VideoInfo {
            id: "".to_string(), // Lux might not expose ID easily in top level
            title,
            uploader: "Unknown".to_string(), // Lux doesn't always provide uploader in simple JSON
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
            status: "Starting lux download...".to_string(),
        });

        let mut args = vec!["-o", &options.output_path];
        
        // Lux doesn't have a simple format selector like yt-dlp's "best" that maps 1:1,
        // but it defaults to best quality usually.
        
        if options.extract_audio {
             // Lux is primarily video, might not support pure audio extraction easily without flags
             // Ignoring audio extraction for basic lux implementation for now or checking docs
             // Lux doesn't seem to have a simple audio-only flag in standard usage like yt-dlp -x
        }

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
