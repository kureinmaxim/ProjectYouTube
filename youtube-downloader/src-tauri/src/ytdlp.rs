use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tauri::Emitter;
use std::process::Command as StdCommand;

use crate::downloader::utils;
use crate::downloader::utils::run_output_with_timeout;

fn python_cmd() -> String {
    // Allow overriding python interpreter (e.g. venv) to avoid Homebrew PEP 668 limitations.
    // Example: export YTDLP_PYTHON="/path/to/venv/bin/python"
    std::env::var("YTDLP_PYTHON").unwrap_or_else(|_| "python3".to_string())
}

fn python_has_module(module: &str) -> bool {
    // Quick check: avoid noisy stderr and wasted time when module is missing.
    // We intentionally allow overriding the interpreter via YTDLP_PYTHON.
    let code = format!("import {}", module);
    let py = python_cmd();
    match StdCommand::new(&py).args(["-c", &code]).output() {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

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
pub async fn get_video_info(
    url: String,
    proxy: Option<String>,
    cookies_from_browser: Option<bool>,
    cookies_path: Option<String>,
) -> Result<VideoInfo, String> {
    // Try Python module first only if it is installed (avoid noisy failures)
    if python_has_module("yt_dlp") {
        match get_video_info_python(&url, proxy.clone(), cookies_from_browser, cookies_path.clone()).await {
            Ok(info) => {
                eprintln!("[yt-dlp] Successfully fetched via Python module");
                return Ok(info);
            }
            Err(e) => {
                eprintln!("[yt-dlp] Python module failed: {}, trying native binary...", e);
            }
        }
    } else {
        eprintln!("[yt-dlp] Python module yt_dlp is not installed — OK. Continuing with native yt-dlp...");
    }
    
    // Fallback to native binary
    get_video_info_native(&url, proxy, cookies_from_browser, cookies_path).await
}

// Primary method: Python module (most reliable)
async fn get_video_info_python(
    url: &str,
    proxy: Option<String>,
    cookies_from_browser: Option<bool>,
    cookies_path: Option<String>,
) -> Result<VideoInfo, String> {
    let py = python_cmd();
    let mut args = vec![
        "-m".to_string(),
        "yt_dlp".to_string(),
        "--dump-json".to_string(),
        "--no-playlist".to_string(),
        "--no-warnings".to_string(),
        "--socket-timeout".to_string(),
        "15".to_string(),
        "--retries".to_string(),
        "2".to_string(),
        "--extractor-args".to_string(),
        "youtube:player_client=web".to_string(),
        url.to_string(),
    ];
    if let Some(path) = cookies_path {
        args.push("--cookies".to_string());
        args.push(path);
    } else if cookies_from_browser.unwrap_or(false) {
        args.push("--cookies-from-browser".to_string());
        args.push("chrome".to_string());
    }
    if let Some(p) = proxy {
        args.push("--proxy".to_string());
        args.push(p);
    }

    let output = run_output_with_timeout(&py, args, 30).await
        .map_err(|e| format!("Python yt-dlp error: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python yt-dlp error: {}", error));
    }

    parse_video_info(&output.stdout)
}

// Fallback method: Native binary
async fn get_video_info_native(
    url: &str,
    proxy: Option<String>,
    cookies_from_browser: Option<bool>,
    cookies_path: Option<String>,
) -> Result<VideoInfo, String> {
    let ytdlp_path = find_ytdlp();

    // Auto-detect proxy (same strategy as download_video)
    let proxy = proxy.or_else(utils::auto_detect_proxy);
    
    let args = vec![
        "--dump-json".to_string(),
        "--no-playlist".to_string(),
        "--no-warnings".to_string(),
        "--socket-timeout".to_string(),
        "15".to_string(),
        "--retries".to_string(),
        "2".to_string(),
        "--user-agent".to_string(),
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36"
            .to_string(),
        "--extractor-args".to_string(),
        "youtube:player_client=web".to_string(),
        url.to_string(),
    ];

    // Attach proxy if detected
    let mut args = args;
    if let Some(path) = cookies_path {
        args.push("--cookies".to_string());
        args.push(path);
    } else if cookies_from_browser.unwrap_or(false) {
        args.push("--cookies-from-browser".to_string());
        args.push("chrome".to_string());
    }
    if let Some(proxy_url) = &proxy {
        eprintln!("[yt-dlp] Using proxy for info: {}", proxy_url);
        args.push("--proxy".to_string());
        args.push(proxy_url.clone());
    }

    let output = match run_output_with_timeout(&ytdlp_path, args, 30).await {
        Ok(out) => out,
        Err(e) => {
            let lower = url.to_lowercase();
            let is_youtube = lower.contains("youtube.com") || lower.contains("youtu.be");

            // Friendly error for the common YouTube soft-block case
            if is_youtube && e.contains("Timed out after") {
                let mut msg = String::from(
                    "YouTube is temporarily throttling requests from your IP (CDN soft-block).\n\
This is not a bug in the app, and it usually resolves on its own.\n\n\
What you can do:\n\
1) Wait 6–24 hours and try again\n\
2) Use a Proxy/VPN (SOCKS5 works best)\n\
3) Try a different network (mobile hotspot / another Wi‑Fi)\n\n",
                );

                match proxy {
                    Some(p) => {
                        msg.push_str(&format!(
                            "Detected local proxy: {}\n\
It was applied automatically, but the request still timed out.\n\
Please verify your proxy/VPN is actually working.\n",
                            p
                        ));
                    }
                    None => {
                        msg.push_str(
                            "No local SOCKS5 proxy was detected.\n\
If you have XRAY/Clash/V2Ray, start it and try again (common ports: 1080 / 7890 / 10808).\n\
Example manual test:\n\
yt-dlp --proxy socks5h://127.0.0.1:1080 --dump-json <URL>\n",
                        );
                    }
                }

                return Err(msg);
            }

            return Err(e);
        }
    };

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

use crate::downloader::traits::DownloaderBackend;
use crate::downloader::backends::{LuxBackend, YouGetBackend};
use crate::downloader::models::DownloadOptions;
use crate::downloader::tools::{ToolManager, ToolType};

async fn try_download_with_lux(
    url: &str,
    options: DownloadOptions,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let backend = LuxBackend::new();
    backend
        .download(url, options, app_handle)
        .await
        .map_err(|e| format!("Lux error: {}", e))
}

async fn try_download_with_youget(
    url: &str,
    options: DownloadOptions,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let backend = YouGetBackend::new();
    backend
        .download(url, options, app_handle)
        .await
        .map_err(|e| format!("You-Get error: {}", e))
}

fn tool_label(tool: &str) -> &'static str {
    match tool {
        "yt-dlp" => "yt-dlp",
        "lux" => "lux",
        "you-get" => "you-get",
        _ => "yt-dlp",
    }
}

fn tool_available(tool: &str) -> bool {
    let manager = ToolManager::new();
    match tool {
        "yt-dlp" => manager.get_tool_info(ToolType::YtDlp).is_available,
        "lux" => manager.get_tool_info(ToolType::Lux).is_available,
        "you-get" => manager.get_tool_info(ToolType::YouGet).is_available,
        _ => false,
    }
}

async fn try_download_with_ytdlp(
    url: &str,
    quality: &str,
    output_path: &str,
    proxy_override: Option<String>,
    cookies_from_browser: bool,
    cookies_path: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // Determine format based on quality selection
    let format_arg = match quality {
        // Prefer separate video+audio (DASH) but always provide robust fallbacks.
        "best" => "bv*+ba/best",
        "1080p" => "bv*[height<=1080]+ba/b[height<=1080]/bv*+ba/best",
        "720p" => "bv*[height<=720]+ba/b[height<=720]/bv*+ba/best",
        "480p" => "bv*[height<=480]+ba/b[height<=480]/bv*+ba/best",
        "audio" => "ba/b",
        _ => "bv*+ba/best",
    };

    let ytdlp_path = find_ytdlp();

    // Auto-detect proxy
    let proxy = proxy_override.or_else(utils::auto_detect_proxy);
    let is_youtube = {
        let lower = url.to_lowercase();
        lower.contains("youtube.com") || lower.contains("youtu.be")
    };

    let build_args = |player_client: &str,
                      format_override: Option<&str>,
                      use_cookies: bool,
                      force_audio: bool| -> Vec<String> {
        let mut args = vec![
            "-f".to_string(),
            format_override.unwrap_or(format_arg).to_string(),
            "--no-playlist".to_string(),
            "--newline".to_string(),
            // keep stderr less noisy; we surface actionable messages ourselves
            "--no-update".to_string(),
            "--socket-timeout".to_string(),
            "30".to_string(),
            "--retries".to_string(),
            "3".to_string(),
            "-P".to_string(),
            output_path.to_string(),
            // Default yt-dlp template is "%(title)s [%(id)s].%(ext)s" — override to remove [id]
            "-o".to_string(),
            "%(title)s.%(ext)s".to_string(),
            "--no-check-certificates".to_string(),
            "--user-agent".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36".to_string(),
        ];

        // Cookies / auth (helps with bot protection / age gates)
        if use_cookies {
            if let Some(path) = &cookies_path {
                args.push("--cookies".to_string());
                args.push(path.clone());
            } else if cookies_from_browser {
                args.push("--cookies-from-browser".to_string());
                args.push("chrome".to_string());
            }
        }

        if is_youtube {
            // Helps when IPv6 ranges are throttled/blocked by Google/CDNs
            args.push("--force-ipv4".to_string());
            // Make merged output predictable on macOS players
            args.push("--merge-output-format".to_string());
            args.push("mp4".to_string());
            args.push("--extractor-args".to_string());
            args.push(format!("youtube:player_client={}", player_client));
        }

        // Add proxy if detected
        if let Some(proxy_url) = &proxy {
            eprintln!("[download_video] Using proxy: {}", proxy_url);
            args.push("--proxy".to_string());
            args.push(proxy_url.clone());
        }

        // Add audio format conversion if audio only
        if quality == "audio" || force_audio {
            args.extend(vec![
                "-x".to_string(),
                "--audio-format".to_string(),
                "mp3".to_string(),
            ]);
        }

        args.push(url.to_string());
        args
    };

    let cookies_enabled = cookies_path.is_some() || cookies_from_browser;

    // Helper: run attempts for a given client list and cookie mode; return last stderr on failure.
    // NOTE: this function runs sync processes (StdCommand::output), so it doesn't need to be async.
    let run_attempts = |clients: Vec<&str>, use_cookies: bool, force_audio: bool| -> Result<(), String> {
        let mut last_stderr = String::new();
        for (idx, client) in clients.iter().enumerate() {
            let attempt = idx + 1;
            let total = clients.len();

            let _ = app_handle.emit(
                "download-progress",
                DownloadProgress {
                    percent: 0.0,
                    status: format!(
                        "yt-dlp attempt {}/{} (client: {}; cookies: {})...",
                        attempt,
                        total,
                        client,
                        if use_cookies { "on" } else { "off" }
                    ),
                },
            );

            let args = build_args(client, None, use_cookies, force_audio);
            let output = StdCommand::new(&ytdlp_path)
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .map_err(|e| format!("Failed to start yt-dlp: {}", e))?;

            if output.status.success() {
                let mode = if quality == "audio" || force_audio { "audio" } else { "video" };
                let cookies_label = if use_cookies { "on" } else { "off" };
                let success = format!("yt-dlp succeeded (client: {}; cookies: {}; mode: {})", client, cookies_label, mode);
                eprintln!("[download_video] {}", success);
                let _ = app_handle.emit(
                    "download-progress",
                    DownloadProgress {
                        percent: 100.0,
                        status: success.clone(),
                    },
                );
                return Ok(());
            }

            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            last_stderr = stderr.clone();

            // Short reason for UI + terminal
            let important_lines: Vec<&str> = stderr
                .lines()
                .map(|l| l.trim())
                .filter(|s| {
                    s.starts_with("ERROR:")
                        || s.starts_with("WARNING:")
                        || s.contains("HTTP Error")
                        || s.contains("Forbidden")
                        || s.contains("SABR")
                        || s.contains("PO Token")
                        || s.contains("Requested format is not available")
                        || s.contains("Skipping client")
                        || s.contains("Sign in")
                        || s.contains("cookies")
                        || s.contains("Captcha")
                        || s.contains("CAPTCHA")
                })
                .take(3)
                .collect();
            let preview = if !important_lines.is_empty() {
                important_lines.join(" | ")
            } else {
                stderr
                    .lines()
                    .rev()
                    .find(|l| !l.trim().is_empty())
                    .unwrap_or("Unknown yt-dlp error")
                    .trim()
                    .chars()
                    .take(220)
                    .collect::<String>()
            };
            eprintln!("[download_video] yt-dlp client {} error: {}", client, preview);
            let _ = app_handle.emit(
                "download-progress",
                DownloadProgress {
                    percent: 0.0,
                    status: format!("yt-dlp client {} failed: {}", client, preview),
                },
            );

            let retryable =
                is_youtube
                    && (stderr.contains("HTTP Error 403")
                        || stderr.contains("Forbidden")
                        || stderr.contains("SABR")
                        || stderr.contains("PO Token")
                        || stderr.contains("Requested format is not available")
                        || stderr.contains("Skipping client"));

            if retryable && attempt < total {
                eprintln!(
                    "[download_video] yt-dlp failed with client {} (retrying next client).",
                    client
                );
                continue;
            }
            break;
        }

        // Special case: quality not available -> retry best
        if last_stderr.contains("Requested format is not available") && quality != "best" && !force_audio {
            let _ = app_handle.emit(
                "download-progress",
                DownloadProgress {
                    percent: 0.0,
                    status: "Requested quality not available. Falling back to Best…".to_string(),
                },
            );

            let args = build_args("web", Some("bv*+ba/best"), use_cookies, false);
            let output = StdCommand::new(&ytdlp_path)
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .map_err(|e| format!("Failed to start yt-dlp: {}", e))?;

            if output.status.success() {
                return Ok(());
            }
            last_stderr = String::from_utf8_lossy(&output.stderr).to_string();
        }

        Err(last_stderr)
    };

    // Phase 1: if cookies enabled -> try cookie-compatible clients first.
    // NOTE: yt-dlp may skip some clients when cookies are enabled.
    if cookies_enabled {
        eprintln!("[download_video] yt-dlp strategy: cookies=on (try web)");
        let _ = app_handle.emit(
            "download-progress",
            DownloadProgress { percent: 0.0, status: "yt-dlp strategy: cookies=on (web)".to_string() },
        );
        let clients = vec!["web"];
        if run_attempts(clients, true, false).is_ok() {
            return Ok(());
        }

        // If cookies-path/browser is enabled but web hits SABR/403, try without cookies next.
        eprintln!("[download_video] yt-dlp strategy switch: cookies=on -> cookies=off");
        let _ = app_handle.emit(
            "download-progress",
            DownloadProgress {
                percent: 0.0,
                status: "Authenticated download failed. Retrying without cookies (android/tv)…".to_string(),
            },
        );
    }

    // Phase 2: no-cookies strategy for YouTube (often better for android/tv).
    let clients_no_cookies: Vec<&str> = if is_youtube {
        vec!["android", "tv", "web"]
    } else {
        vec!["web"]
    };

    eprintln!("[download_video] yt-dlp strategy: cookies=off (try android/tv/web)");
    let _ = app_handle.emit(
        "download-progress",
        DownloadProgress { percent: 0.0, status: "yt-dlp strategy: cookies=off (android/tv/web)".to_string() },
    );
    if run_attempts(clients_no_cookies, false, false).is_ok() {
        return Ok(());
    }

    // Phase 3: last resort — audio-only (often allowed even when video is blocked).
    if quality != "audio" {
        eprintln!("[download_video] yt-dlp strategy: audio-only fallback");
        let _ = app_handle.emit(
            "download-progress",
            DownloadProgress {
                percent: 0.0,
                status: "Video download blocked. Trying audio-only (MP3) fallback…".to_string(),
            },
        );

        let clients_audio: Vec<&str> = if is_youtube { vec!["web", "android"] } else { vec!["web"] };
        // Prefer cookies if enabled; otherwise no cookies.
        if cookies_enabled {
            eprintln!("[download_video] yt-dlp audio fallback: cookies=on (web/android)");
            let _ = run_attempts(clients_audio.clone(), true, true);
        }
        eprintln!("[download_video] yt-dlp audio fallback: cookies=off (web/android)");
        if run_attempts(clients_audio, false, true).is_ok() {
            return Ok(());
        }
    }

    Err("yt-dlp download failed after multiple strategies (cookies/no-cookies/audio fallback).".to_string())
}

// Download video
#[tauri::command]
pub async fn download_video(
    url: String,
    quality: String,
    output_path: String,
    tool: Option<String>,
    proxy: Option<String>,
    allow_fallback: Option<bool>,
    cookies_from_browser: Option<bool>,
    cookies_path: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    eprintln!("[download_video] Tool selected: {:?}", tool);
    
    let selected = tool.as_deref().unwrap_or("yt-dlp");
    let allow_fallback = allow_fallback.unwrap_or(true);

    // Tool fallback order (or strict single-tool mode)
    let tools_to_try: Vec<&str> = if !allow_fallback {
        vec![selected]
    } else {
        match selected {
            "lux" => vec!["lux", "yt-dlp", "you-get"],
            "you-get" => vec!["you-get", "yt-dlp", "lux"],
            _ => vec!["yt-dlp", "lux", "you-get"],
        }
    };

    let mut errors: Vec<String> = Vec::new();

    for (idx, t) in tools_to_try.iter().enumerate() {
        let label = tool_label(t);

        if !tool_available(t) {
            let msg = format!("Skipping {} (not installed)", label);
            let _ = app_handle.emit(
                "download-progress",
                DownloadProgress { percent: 0.0, status: msg.clone() },
            );
            errors.push(msg);
            continue;
        }

        let _ = app_handle.emit(
            "download-progress",
            DownloadProgress {
                percent: 0.0,
                status: format!("Trying tool {}/{}: {}...", idx + 1, tools_to_try.len(), label),
            },
        );
        eprintln!("[download_video] Trying tool {}/{}: {}", idx + 1, tools_to_try.len(), label);

        let base_options = DownloadOptions {
            output_path: output_path.clone(),
            quality: quality.clone(),
            extract_audio: quality == "audio",
            audio_format: Some("mp3".to_string()),
            proxy: None,
        };

        let res = match *t {
            "yt-dlp" => try_download_with_ytdlp(
                &url,
                &quality,
                &output_path,
                proxy.clone(),
                cookies_from_browser.unwrap_or(true),
                cookies_path.clone(),
                app_handle.clone(),
            )
            .await,
            "lux" => try_download_with_lux(&url, base_options, app_handle.clone()).await,
            "you-get" => try_download_with_youget(&url, base_options, app_handle.clone()).await,
            _ => try_download_with_ytdlp(
                &url,
                &quality,
                &output_path,
                proxy.clone(),
                cookies_from_browser.unwrap_or(true),
                cookies_path.clone(),
                app_handle.clone(),
            )
            .await,
        };

        match res {
            Ok(()) => {
                let _ = app_handle.emit(
                    "download-progress",
                    DownloadProgress { percent: 100.0, status: format!("Download complete ({}).", label) },
                );
                return Ok(format!("Download completed successfully with {}!", label));
            }
            Err(e) => {
                let msg = format!("{} failed: {}", label, e);
                errors.push(msg.clone());
                eprintln!("[download_video] {} failed: {}", label, e);
                let _ = app_handle.emit(
                    "download-progress",
                    DownloadProgress {
                        percent: 0.0,
                        status: format!("{} failed, trying next tool…", label),
                    },
                );
            }
        }
    }

    Err(format!(
        "All tools failed.\n\n{}",
        errors
            .into_iter()
            .map(|e| format!("- {}", e))
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

// Get available formats
#[tauri::command]
pub async fn get_formats(url: String) -> Result<Vec<FormatInfo>, String> {
    let ytdlp_path = find_ytdlp();
    
    let output = StdCommand::new(&ytdlp_path)
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
