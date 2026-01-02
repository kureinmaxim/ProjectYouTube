use async_trait::async_trait;
use std::process::{Command, Stdio};

use crate::downloader::errors::DownloadError;
use crate::downloader::models::{DownloadOptions, DownloadProgress, VideoInfo};
use crate::downloader::traits::{DownloaderBackend, ProgressEmitter};

pub struct PythonYtDlp {
    ytdlp_bin: String,
}

impl PythonYtDlp {
    pub fn new() -> Self {
        // Use Homebrew binary (most common on macOS)
        // Falls back to system paths
        let ytdlp_bin = if std::path::Path::new("/opt/homebrew/bin/yt-dlp").exists() {
            "/opt/homebrew/bin/yt-dlp".to_string()
        } else if std::path::Path::new("/usr/local/bin/yt-dlp").exists() {
            "/usr/local/bin/yt-dlp".to_string()
        } else {
            "yt-dlp".to_string()
        };
        
        Self { ytdlp_bin }
    }
}

impl Default for PythonYtDlp {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DownloaderBackend for PythonYtDlp {
    fn name(&self) -> &'static str {
        "yt-dlp-python"
    }

    async fn get_video_info(&self, url: &str) -> Result<VideoInfo, DownloadError> {
        let output = Command::new(&self.ytdlp_bin)
            .args([
                "--dump-json",
                "--no-playlist",
                "--no-warnings",
                "--extractor-args", "youtube:player_client=web",
                url,
            ])
            .output()
            .map_err(|e| DownloadError::ToolNotFound(format!("yt-dlp: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(error.to_string().into());
        }

        parse_video_info(&output.stdout)
    }

    async fn download(
        &self,
        url: &str,
        options: DownloadOptions,
        app_handle: tauri::AppHandle,
    ) -> Result<(), DownloadError> {
        let format_arg = match options.quality.as_str() {
            "best" => "bestvideo+bestaudio/best",
            "1080p" => "bestvideo[height<=1080]+bestaudio/best[height<=1080]",
            "720p" => "bestvideo[height<=720]+bestaudio/best[height<=720]",
            "480p" => "bestvideo[height<=480]+bestaudio/best[height<=480]",
            "audio" => "bestaudio/best",
            _ => "best",
        };

        let mut args = vec![
            "-f", format_arg,
            "--cookies-from-browser", "chrome",
            "--extractor-args", "youtube:player_client=web",
            "--newline",
            "--no-playlist",
            "-P", &options.output_path,
        ];

        let audio_args: Vec<String>;
        if options.extract_audio {
            audio_args = vec![
                "-x".to_string(),
                "--audio-format".to_string(),
                options.audio_format.unwrap_or_else(|| "mp3".to_string()),
            ];
            for arg in &audio_args {
                args.push(arg);
            }
        }

        args.push(url);

        let emitter = ProgressEmitter::new(app_handle);
        emitter.emit(DownloadProgress {
            percent: 0.0,
            status: "Starting download...".to_string(),
        });

        let output = Command::new(&self.ytdlp_bin)
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

fn parse_video_info(stdout: &[u8]) -> Result<VideoInfo, DownloadError> {
    let json_str = String::from_utf8_lossy(stdout);
    let json: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| DownloadError::ParseError(format!("JSON parse error: {}", e)))?;

    let duration_secs = json["duration"].as_f64().unwrap_or(0.0) as i64;
    let minutes = duration_secs / 60;
    let seconds = duration_secs % 60;

    Ok(VideoInfo {
        id: json["id"].as_str().unwrap_or("").to_string(),
        title: json["title"].as_str().unwrap_or("Unknown").to_string(),
        uploader: json["uploader"].as_str().unwrap_or("Unknown").to_string(),
        duration: format!("{}:{:02}", minutes, seconds),
        thumbnail: json["thumbnail"].as_str().unwrap_or("").to_string(),
    })
}
