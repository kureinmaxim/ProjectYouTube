use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use tauri::Emitter;

// Find yt-dlp executable in common paths
fn find_ytdlp() -> String {
    // Common paths where yt-dlp might be installed
    let common_paths = vec![
        "/opt/homebrew/bin/yt-dlp",  // Homebrew on Apple Silicon
        "/usr/local/bin/yt-dlp",     // Homebrew on Intel Mac
        "/usr/bin/yt-dlp",            // System installation
        "yt-dlp",                     // In PATH
    ];

    for path in common_paths {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }

    // Fallback: try to find in PATH
    if let Ok(output) = Command::new("which").arg("yt-dlp").output() {
        if output.status.success() {
            if let Ok(path) = String::from_utf8(output.stdout) {
                let trimmed = path.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }

    // Last resort: hope it's in PATH
    "yt-dlp".to_string()
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoInfo {
    pub title: String,
    pub duration: String,
    pub thumbnail: String,
    pub uploader: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadProgress {
    pub percent: f32,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormatInfo {
    pub format_id: String,
    pub quality: String,
    pub ext: String,
}

// Get video information with dual backend approach
#[tauri::command]
pub async fn get_video_info(url: String) -> Result<VideoInfo, String> {
    // Try Python module first (most reliable in 2025)
    match get_video_info_python(&url).await {
        Ok(info) => {
            eprintln!("[yt-dlp] Successfully fetched via Python module");
            return Ok(info);
        }
        Err(e) => {
            eprintln!("[yt-dlp] Python module failed: {}, trying native binary...", e);
        }
    }
    
    // Fallback to native binary
    get_video_info_native(&url).await
}

// Primary method: Python module (most reliable)
async fn get_video_info_python(url: &str) -> Result<VideoInfo, String> {
    let output = Command::new("python3")
        .args([
            "-m", "yt_dlp",
            "--dump-json",
            "--no-playlist",
            "--no-warnings",
            "--extractor-args", "youtube:player_client=web",
            url,
        ])
        .output()
        .map_err(|e| format!("Failed to execute python -m yt_dlp: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python yt-dlp error: {}", error));
    }

    parse_video_info(&output.stdout)
}

// Fallback method: Native binary
async fn get_video_info_native(url: &str) -> Result<VideoInfo, String> {
    let ytdlp_path = find_ytdlp();
    
    let output = Command::new(&ytdlp_path)
        .args([
            "--dump-json",
            "--no-playlist",
            "--no-warnings",
            "--user-agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
            "--extractor-args", "youtube:player_client=web",
            url,
        ])
        .output()
        .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp error: {}", error));
    }

    parse_video_info(&output.stdout)
}

// Shared JSON parsing logic
fn parse_video_info(stdout: &[u8]) -> Result<VideoInfo, String> {
    let json_str = String::from_utf8_lossy(stdout);
    let json: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let duration_secs = json["duration"].as_f64().unwrap_or(0.0) as i64;
    let minutes = duration_secs / 60;
    let seconds = duration_secs % 60;
    let duration = format!("{}:{:02}", minutes, seconds);

    Ok(VideoInfo {
        title: json["title"].as_str().unwrap_or("Unknown").to_string(),
        duration,
        thumbnail: json["thumbnail"].as_str().unwrap_or("").to_string(),
        uploader: json["uploader"].as_str().unwrap_or("Unknown").to_string(),
    })
}

// Download video
#[tauri::command]
pub async fn download_video(
    url: String,
    quality: String,
    output_path: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    // Determine format based on quality selection
    let format_arg = match quality.as_str() {
        "best" => "bestvideo+bestaudio/best",
        "1080p" => "bestvideo[height<=1080]+bestaudio/best[height<=1080]",
        "720p" => "bestvideo[height<=720]+bestaudio/best[height<=720]",
        "480p" => "bestvideo[height<=480]+bestaudio/best[height<=480]",
        "audio" => "bestaudio/best",
        _ => "best",
    };

    let mut args = vec![
        "-m".to_string(),
        "yt_dlp".to_string(),
        "-f".to_string(),
        format_arg.to_string(),
        "--cookies-from-browser".to_string(),
        "chrome".to_string(),
        "--extractor-args".to_string(),
        "youtube:player_client=web".to_string(),
        "--newline".to_string(),
        "--no-playlist".to_string(),
        "-P".to_string(),
        output_path.clone(),
    ];

    // Add audio format conversion if audio only
    if quality == "audio" {
        args.extend(vec![
            "-x".to_string(),
            "--audio-format".to_string(),
            "mp3".to_string(),
        ]);
    }

    args.push(url.clone());

    let child = Command::new("python3")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start download: {}", e))?;

    // Emit progress updates
    let _ = app_handle.emit("download-progress", DownloadProgress {
        percent: 0.0,
        status: "Starting download...".to_string(),
    });

    let output = child.wait_with_output()
        .map_err(|e| format!("Failed to wait for download: {}", e))?;

    if output.status.success() {
        let _ = app_handle.emit("download-progress", DownloadProgress {
            percent: 100.0,
            status: "Download complete!".to_string(),
        });
        Ok("Download completed successfully!".to_string())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("Download failed: {}", error))
    }
}

// Get available formats
#[tauri::command]
pub async fn get_formats(url: String) -> Result<Vec<FormatInfo>, String> {
    let ytdlp_path = find_ytdlp();
    
    let output = Command::new(&ytdlp_path)
        .args([
            "--list-formats",
            "--cookies-from-browser", "chrome",
            &url,
        ])
        .output()
        .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp error: {}", error));
    }

    // Return simplified format list
    Ok(vec![
        FormatInfo {
            format_id: "best".to_string(),
            quality: "Best Quality".to_string(),
            ext: "mp4".to_string(),
        },
        FormatInfo {
            format_id: "1080p".to_string(),
            quality: "1080p".to_string(),
            ext: "mp4".to_string(),
        },
        FormatInfo {
            format_id: "720p".to_string(),
            quality: "720p".to_string(),
            ext: "mp4".to_string(),
        },
        FormatInfo {
            format_id: "480p".to_string(),
            quality: "480p".to_string(),
            ext: "mp4".to_string(),
        },
        FormatInfo {
            format_id: "audio".to_string(),
            quality: "Audio Only (MP3)".to_string(),
            ext: "mp3".to_string(),
        },
    ])
}
