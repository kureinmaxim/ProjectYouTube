// Python InfoExtractor - uses `python3 -m yt_dlp`
//
// Advantages:
// - Better at bypassing YouTube bot detection
// - Works well with cookies/auth
// - Less likely to trigger 403/SABR blocks
//
// Disadvantages:
// - Requires Python 3 and yt-dlp module
// - Slightly slower than native binary

use async_trait::async_trait;
use std::process::Command as StdCommand;

use super::traits::{ExtendedFormat, ExtendedVideoInfo, ExtractorConfig, InfoExtractor};
use crate::downloader::errors::DownloadError;
use crate::downloader::utils::run_output_with_timeout;

/// Python-based info extractor using yt_dlp module
pub struct PythonInfoExtractor {
    python_cmd: String,
}

impl PythonInfoExtractor {
    pub fn new() -> Self {
        Self {
            python_cmd: Self::find_python(),
        }
    }

    /// Find Python interpreter
    fn find_python() -> String {
        // Allow override via environment variable
        if let Ok(custom) = std::env::var("YTDLP_PYTHON") {
            return custom;
        }

        // Check common paths
        let candidates = ["python3", "/opt/homebrew/bin/python3", "/usr/local/bin/python3"];

        for cmd in candidates {
            if let Ok(output) = StdCommand::new(cmd).arg("--version").output() {
                if output.status.success() {
                    return cmd.to_string();
                }
            }
        }

        "python3".to_string()
    }

    /// Check if yt_dlp module is installed
    fn has_ytdlp_module(&self) -> bool {
        let code = "import yt_dlp; print('ok')";
        match StdCommand::new(&self.python_cmd)
            .args(["-c", code])
            .output()
        {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }

    /// Build command arguments
    fn build_args(&self, url: &str, config: &ExtractorConfig) -> Vec<String> {
        let mut args = vec![
            "-m".to_string(),
            "yt_dlp".to_string(),
            "--dump-json".to_string(),
            "--no-playlist".to_string(),
            "--no-warnings".to_string(),
            "--socket-timeout".to_string(),
            config.timeout_seconds.to_string(),
            "--retries".to_string(),
            "2".to_string(),
        ];

        // Player client for YouTube
        if let Some(client) = &config.player_client {
            args.push("--extractor-args".to_string());
            args.push(format!("youtube:player_client={}", client));
        } else {
            // Default to web client for Python mode
            args.push("--extractor-args".to_string());
            args.push("youtube:player_client=web".to_string());
        }

        // Cookies
        if let Some(path) = &config.cookies_path {
            args.push("--cookies".to_string());
            args.push(path.clone());
        } else if config.cookies_from_browser {
            args.push("--cookies-from-browser".to_string());
            args.push("chrome".to_string());
        }

        // Proxy
        if let Some(proxy) = &config.proxy {
            args.push("--proxy".to_string());
            args.push(proxy.clone());
        }

        args.push(url.to_string());
        args
    }

    /// Parse JSON output into ExtendedVideoInfo
    fn parse_json(stdout: &[u8]) -> Result<ExtendedVideoInfo, DownloadError> {
        let json_str = String::from_utf8_lossy(stdout);
        let json: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| DownloadError::ParseError(format!("Invalid JSON: {}", e)))?;

        let formats = Self::parse_formats(&json)?;

        Ok(ExtendedVideoInfo {
            id: json["id"].as_str().unwrap_or("unknown").to_string(),
            title: json["title"].as_str().unwrap_or("Unknown").to_string(),
            uploader: json["uploader"].as_str().unwrap_or("Unknown").to_string(),
            duration_seconds: json["duration"].as_f64().unwrap_or(0.0) as u64,
            thumbnail: json["thumbnail"].as_str().unwrap_or("").to_string(),
            webpage_url: json["webpage_url"].as_str().unwrap_or("").to_string(),
            formats,
        })
    }

    /// Parse formats array from JSON
    fn parse_formats(json: &serde_json::Value) -> Result<Vec<ExtendedFormat>, DownloadError> {
        let formats_array = json["formats"]
            .as_array()
            .ok_or_else(|| DownloadError::ParseError("No formats array in JSON".to_string()))?;

        let mut formats = Vec::new();

        for f in formats_array {
            let vcodec = f["vcodec"].as_str().map(|s| s.to_string());
            let acodec = f["acodec"].as_str().map(|s| s.to_string());

            let video_only = vcodec.as_ref().map_or(false, |v| v != "none")
                && acodec.as_ref().map_or(true, |a| a == "none");
            let audio_only = acodec.as_ref().map_or(false, |a| a != "none")
                && vcodec.as_ref().map_or(true, |v| v == "none");

            formats.push(ExtendedFormat {
                format_id: f["format_id"].as_str().unwrap_or("").to_string(),
                ext: f["ext"].as_str().unwrap_or("").to_string(),
                resolution: f["resolution"].as_str().map(|s| s.to_string()),
                width: f["width"].as_u64().map(|w| w as u32),
                height: f["height"].as_u64().map(|h| h as u32),
                fps: f["fps"].as_f64().map(|fps| fps as f32),
                vcodec,
                acodec,
                filesize: f["filesize"].as_u64(),
                filesize_approx: f["filesize_approx"].as_u64(),
                tbr: f["tbr"].as_f64().map(|t| t as f32),
                abr: f["abr"].as_f64().map(|a| a as f32),
                vbr: f["vbr"].as_f64().map(|v| v as f32),
                format_note: f["format_note"].as_str().map(|s| s.to_string()),
                video_only,
                audio_only,
            });
        }

        Ok(formats)
    }
}

impl Default for PythonInfoExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InfoExtractor for PythonInfoExtractor {
    fn name(&self) -> &'static str {
        "python-yt-dlp"
    }

    fn is_available(&self) -> bool {
        self.has_ytdlp_module()
    }

    async fn extract(
        &self,
        url: &str,
        config: &ExtractorConfig,
    ) -> Result<ExtendedVideoInfo, DownloadError> {
        if !self.is_available() {
            return Err(DownloadError::ToolNotFound(
                "Python yt_dlp module not installed".to_string(),
            ));
        }

        let args = self.build_args(url, config);
        eprintln!(
            "[PythonExtractor] Running: {} {}",
            self.python_cmd,
            args.join(" ")
        );

        let output = run_output_with_timeout(&self.python_cmd, args, config.timeout_seconds as u64)
            .await
            .map_err(|e| DownloadError::ExecutionError(format!("Python yt-dlp error: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DownloadError::from(stderr.to_string()));
        }

        Self::parse_json(&output.stdout)
    }

    async fn extract_formats(
        &self,
        url: &str,
        config: &ExtractorConfig,
    ) -> Result<Vec<ExtendedFormat>, DownloadError> {
        let info = self.extract(url, config).await?;
        Ok(info.formats)
    }
}

