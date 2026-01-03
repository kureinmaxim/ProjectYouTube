// CLI InfoExtractor - uses native `yt-dlp` binary
//
// Advantages:
// - Faster than Python mode
// - No Python dependency
// - Easier for CI/CD and distribution
//
// Disadvantages:
// - More likely to trigger YouTube bot detection
// - May require different player clients

use async_trait::async_trait;
use std::process::Command as StdCommand;

use super::traits::{ExtendedFormat, ExtendedVideoInfo, ExtractorConfig, InfoExtractor};
use crate::downloader::errors::DownloadError;
use crate::downloader::utils::run_output_with_timeout;

/// CLI-based info extractor using yt-dlp binary
pub struct CliInfoExtractor {
    ytdlp_path: String,
}

impl CliInfoExtractor {
    pub fn new() -> Self {
        Self {
            ytdlp_path: Self::find_ytdlp(),
        }
    }

    /// Find yt-dlp binary
    fn find_ytdlp() -> String {
        let common_paths = vec![
            "/opt/homebrew/bin/yt-dlp", // Homebrew on Apple Silicon
            "/usr/local/bin/yt-dlp",    // Homebrew on Intel Mac
            "/usr/bin/yt-dlp",          // System installation
            "yt-dlp",                   // In PATH
        ];

        for path in common_paths {
            if std::path::Path::new(path).exists() {
                return path.to_string();
            }
        }

        // Try to find via `which`
        if let Ok(output) = StdCommand::new("which").arg("yt-dlp").output() {
            if output.status.success() {
                if let Ok(path) = String::from_utf8(output.stdout) {
                    let trimmed = path.trim();
                    if !trimmed.is_empty() {
                        return trimmed.to_string();
                    }
                }
            }
        }

        "yt-dlp".to_string()
    }

    /// Check if yt-dlp binary is available
    fn has_ytdlp_binary(&self) -> bool {
        match StdCommand::new(&self.ytdlp_path)
            .arg("--version")
            .output()
        {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }

    /// Build command arguments
    fn build_args(&self, url: &str, config: &ExtractorConfig, client: &str) -> Vec<String> {
        let mut args = vec![
            "--dump-json".to_string(),
            "--no-playlist".to_string(),
            "--no-warnings".to_string(),
            "--socket-timeout".to_string(),
            config.timeout_seconds.to_string(),
            "--retries".to_string(),
            "2".to_string(),
            "--user-agent".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36".to_string(),
        ];

        // Player client for YouTube
        let is_youtube = url.to_lowercase().contains("youtube.com")
            || url.to_lowercase().contains("youtu.be");

        if is_youtube {
            args.push("--extractor-args".to_string());
            args.push(format!("youtube:player_client={}", client));
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

    /// Parse JSON output (same as Python extractor)
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

    /// Try extraction with multiple player clients
    async fn try_with_clients(
        &self,
        url: &str,
        config: &ExtractorConfig,
        clients: &[&str],
    ) -> Result<ExtendedVideoInfo, DownloadError> {
        let mut last_error = DownloadError::Unknown("No clients to try".to_string());

        for client in clients {
            let args = self.build_args(url, config, client);
            eprintln!(
                "[CliExtractor] Trying client '{}': {} {}",
                client,
                self.ytdlp_path,
                args.join(" ")
            );

            let output =
                run_output_with_timeout(&self.ytdlp_path, args, config.timeout_seconds as u64)
                    .await;

            match output {
                Ok(out) if out.status.success() => {
                    eprintln!("[CliExtractor] Success with client '{}'", client);
                    return Self::parse_json(&out.stdout);
                }
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    eprintln!("[CliExtractor] Client '{}' failed: {}", client, stderr);
                    last_error = DownloadError::from(stderr.to_string());
                }
                Err(e) => {
                    eprintln!("[CliExtractor] Client '{}' error: {}", client, e);
                    last_error = DownloadError::ExecutionError(e);
                }
            }
        }

        Err(last_error)
    }
}

impl Default for CliInfoExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InfoExtractor for CliInfoExtractor {
    fn name(&self) -> &'static str {
        "cli-yt-dlp"
    }

    fn is_available(&self) -> bool {
        self.has_ytdlp_binary()
    }

    async fn extract(
        &self,
        url: &str,
        config: &ExtractorConfig,
    ) -> Result<ExtendedVideoInfo, DownloadError> {
        if !self.is_available() {
            return Err(DownloadError::ToolNotFound(
                "yt-dlp binary not found".to_string(),
            ));
        }

        let is_youtube = url.to_lowercase().contains("youtube.com")
            || url.to_lowercase().contains("youtu.be");

        // For YouTube, try multiple clients
        // android is faster and less likely to be blocked
        // web is fallback for age-gated content
        let clients: Vec<&str> = if is_youtube {
            if config.cookies_path.is_some() || config.cookies_from_browser {
                // With cookies, prefer web client
                vec!["web", "android"]
            } else {
                // Without cookies, prefer android (less blocking)
                vec!["android", "tv", "web"]
            }
        } else {
            vec!["web"]
        };

        self.try_with_clients(url, config, &clients).await
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

