use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::io::{BufRead, BufReader};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use std::env;
use std::path::Path;
use tauri::Emitter;
use std::process::Command as StdCommand;
use regex::Regex;

use crate::downloader::platform;
use crate::downloader::utils;
use crate::downloader::utils::run_output_with_timeout;

// New architecture (v1.2.0)
use crate::downloader::extractors::{diagnose_error, is_cookie_extraction_failure, BlockingReason};

/// Generate user-friendly suggestion based on blocking reason
fn get_blocking_suggestion(reason: &BlockingReason, proxy: Option<&str>) -> String {
    let mut suggestion = match reason {
        BlockingReason::Http403Forbidden => {
            "What to try:\n\
             1) Use a VPN/Proxy (SOCKS5)\n\
             2) Update cookies (re-login to YouTube)\n\
             3) Wait and try again later".to_string()
        }
        BlockingReason::SabrStreaming => {
            "YouTube SABR protection active.\n\
             What to try:\n\
             1) Enable Auto fallback (uses multiple player clients)\n\
             2) Use cookies from logged-in Chrome\n\
             3) Update yt-dlp: brew upgrade yt-dlp\n\
             4) Use a proxy/VPN".to_string()
        }
        BlockingReason::PoTokenRequired => {
            "YouTube requires PO Token.\n\
             What to try:\n\
             1) Use cookies from logged-in browser\n\
             2) See: github.com/yt-dlp/yt-dlp/wiki/PO-Token-Guide".to_string()
        }
        BlockingReason::AgeRestricted => {
            "Video is age-restricted.\n\
             What to try:\n\
             1) Enable 'Chrome (logged-in)' in Tools → Cookies\n\
             2) Or export cookies.txt from logged-in browser".to_string()
        }
        BlockingReason::GeoBlocked => {
            "Video is blocked in your country.\n\
             What to try:\n\
             1) Use a VPN with a different country\n\
             2) Use a proxy server in allowed region".to_string()
        }
        BlockingReason::NetworkTimeout => {
            "Network timeout (possible IP throttling).\n\
             What to try:\n\
             1) Check your internet connection\n\
             2) Use a proxy/VPN\n\
             3) Try again later".to_string()
        }
        BlockingReason::RateLimited => {
            "YouTube is rate-limiting requests.\n\
             What to try:\n\
             1) Wait 10-15 minutes\n\
             2) Use a different IP (VPN/proxy)".to_string()
        }
        BlockingReason::BotDetection => {
            "YouTube detected automated access.\n\
             What to try:\n\
             1) Use cookies from logged-in Chrome\n\
             2) Use a fresh proxy/VPN".to_string()
        }
        BlockingReason::PrivateVideo => {
            "Video is private.\n\
             You need:\n\
             1) Cookies from an authorized account\n\
             2) Access permission from the uploader".to_string()
        }
        BlockingReason::VideoUnavailable => {
            "Video is unavailable.\n\
             It may have been:\n\
             - Deleted by the uploader\n\
             - Removed for copyright\n\
             - Made private".to_string()
        }
        BlockingReason::DrmProtected => {
            "🔒 DRM-Protected Content\n\n\
             This video is protected by DRM and cannot be downloaded.\n\n\
             ✔ Available offline in YouTube app (with Premium)\n\
             ✔ Can be screen-recorded\n\
             ✖ Cannot be downloaded as a file\n\n\
             This is a content protection measure, not an error.\n\
             Direct download is blocked by DRM encryption.".to_string()
        }
        BlockingReason::MembersOnly => {
            "🎫 Members-Only Content\n\n\
             This video requires channel membership.\n\n\
             ✔ Available if you're a member\n\
             ✖ Cannot be downloaded without membership\n\n\
             Try using cookies from a browser where you're logged in as a member.".to_string()
        }
        BlockingReason::CookiesUnavailable => {
            "Browser cookies could not be read.\n\
             yt-dlp copies the browser's cookie database before reading it, and that fails\n\
             while the browser holds it open - seen on both Windows and macOS.\n\
             What to try:\n\
             1) Close Chrome completely and retry\n\
             2) Or set Tools -> Cookies to 'None' (fine for public videos)\n\
             3) Or export cookies.txt and select it in Tools -> Cookies".to_string()
        }
        BlockingReason::Unknown => {
            "Unknown error.\n\
             What to try:\n\
             1) Check the video URL\n\
             2) Try again later\n\
             3) Use a VPN/proxy".to_string()
        }
    };

    // Add proxy info (but not for permanent restrictions)
    if !reason.is_permanent() {
        if let Some(p) = proxy {
            suggestion.push_str(&format!("\n\nProxy in use: {}", p));
        } else if reason.proxy_might_help() {
            suggestion.push_str("\n\n💡 Tip: No proxy detected. Try enabling XRAY/Clash.");
        }
    }

    suggestion
}

/// Lines yt-dlp reported as fatal, if any.
///
/// yt-dlp warns about SABR on clients that still download fine, so warnings must
/// not decide the diagnosis when real ERROR lines are present.
fn fatal_lines(stderr: &str) -> Option<String> {
    let errors: Vec<&str> = stderr
        .lines()
        .map(|l| l.trim())
        .filter(|l| l.starts_with("ERROR:"))
        .collect();
    if errors.is_empty() {
        None
    } else {
        Some(errors.join(" | "))
    }
}

fn python_cmd() -> String {
    // Allow overriding python interpreter (e.g. venv) to avoid Homebrew PEP 668 limitations.
    // Example: export YTDLP_PYTHON="/path/to/venv/bin/python"
    std::env::var("YTDLP_PYTHON").unwrap_or_else(|_| "python3".to_string())
}

fn clamp_u64(value: u64, min: u64, max: u64) -> u64 {
    value.max(min).min(max)
}

fn env_u64(key: &str, default: u64, min: u64, max: u64) -> u64 {
    let parsed = env::var(key).ok().and_then(|v| v.parse::<u64>().ok());
    clamp_u64(parsed.unwrap_or(default), min, max)
}

/// Parse yt-dlp progress line like:
/// [download]   6.2% of ~ 343.72MiB at  420.30KiB/s ETA 12:32 (frag 29/454)
/// Returns (percent, status_string)
fn parse_ytdlp_progress(line: &str) -> Option<(f32, String)> {
    // Match progress line pattern
    // Example: [download]  12.5% of ~ 310.04MiB at  374.36KiB/s ETA 11:59 (frag 56/454)
    lazy_static::lazy_static! {
        static ref PROGRESS_RE: Regex = Regex::new(
            r"\[download\]\s+(\d+\.?\d*)%\s+of\s+~?\s*(\d+\.?\d*\s*\w+)\s+at\s+(\d+\.?\d*\s*\w+/s)(?:\s+ETA\s+(\S+))?(?:\s+\(frag\s+(\d+)/(\d+)\))?"
        ).unwrap();
        static ref DEST_RE: Regex = Regex::new(r"\[download\]\s+Destination:\s+(.+)").unwrap();
        static ref MERGE_RE: Regex = Regex::new(r"\[Merger?\]\s+Merging").unwrap();
        static ref ALREADY_RE: Regex = Regex::new(r"has already been downloaded").unwrap();
    }

    if let Some(caps) = PROGRESS_RE.captures(line) {
        let percent: f32 = caps.get(1)?.as_str().parse().ok()?;
        let size = caps.get(2).map(|m| m.as_str()).unwrap_or("?");
        let speed = caps.get(3).map(|m| m.as_str()).unwrap_or("?");
        let eta = caps.get(4).map(|m| m.as_str()).unwrap_or("");
        let frag_current = caps.get(5).map(|m| m.as_str());
        let frag_total = caps.get(6).map(|m| m.as_str());

        let status = if let (Some(fc), Some(ft)) = (frag_current, frag_total) {
            format!("⬇️ {:.1}% of {} @ {} ETA {} (frag {}/{})", percent, size, speed, eta, fc, ft)
        } else if !eta.is_empty() {
            format!("⬇️ {:.1}% of {} @ {} ETA {}", percent, size, speed, eta)
        } else {
            format!("⬇️ {:.1}% of {} @ {}", percent, size, speed)
        };

        return Some((percent, status));
    }

    // Check for destination (starting download)
    if let Some(caps) = DEST_RE.captures(line) {
        let filename = caps.get(1).map(|m| m.as_str()).unwrap_or("file");
        // Extract just filename, not full path
        let short_name: String = filename.split('/').last().unwrap_or(filename)
            .chars().take(50).collect();
        return Some((0.0, format!("📥 Starting: {}...", short_name)));
    }

    // Check for merging
    if MERGE_RE.is_match(line) {
        return Some((99.0, "🔄 Merging video and audio...".to_string()));
    }

    // Check for already downloaded
    if ALREADY_RE.is_match(line) {
        return Some((100.0, "✅ File already downloaded".to_string()));
    }

    None
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

fn resolve_executable_dir(path: &str) -> Option<String> {
    let p = Path::new(path);
    if p.is_dir() {
        return Some(path.to_string());
    }
    p.parent()
        .and_then(|parent| parent.to_str())
        .map(|s| s.to_string())
}

// Find ffmpeg directory for yt-dlp merging
fn find_ffmpeg_dir() -> Option<String> {
    let env_override = env::var("YTDLP_FFMPEG_PATH")
        .ok()
        .or_else(|| env::var("FFMPEG_PATH").ok());
    if let Some(path) = env_override {
        if Path::new(&path).exists() {
            return resolve_executable_dir(&path);
        }
    }

    platform::resolve_tool("ffmpeg").and_then(|p| resolve_executable_dir(&p))
}

// Find yt-dlp executable in common paths
fn find_ytdlp() -> String {
    platform::resolve_tool_or_bare("yt-dlp")
}


/// Content restriction information for UI display
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RestrictionInfo {
    /// Type of restriction: "none", "drm", "premium", "members_only", "paid", "age_restricted"
    pub restriction_type: String,
    /// Whether the content can be downloaded
    pub is_downloadable: bool,
    /// User-friendly message explaining the restriction
    pub message: String,
    /// Suggestions for the user
    pub suggestions: Vec<String>,
}

impl RestrictionInfo {
    /// No restriction - content is freely downloadable
    pub fn none() -> Self {
        Self {
            restriction_type: "none".to_string(),
            is_downloadable: true,
            message: String::new(),
            suggestions: Vec::new(),
        }
    }

    /// DRM-protected content
    pub fn drm(content_type: &str) -> Self {
        Self {
            restriction_type: "drm".to_string(),
            is_downloadable: false,
            message: format!("🔒 This {} is DRM-protected and cannot be downloaded.", content_type),
            suggestions: vec![
                "✔ Available offline in YouTube app (with Premium)".to_string(),
                "✔ Can be screen-recorded".to_string(),
                "✖ Cannot be downloaded as a file".to_string(),
            ],
        }
    }

    /// Premium content
    pub fn premium() -> Self {
        Self {
            restriction_type: "premium".to_string(),
            is_downloadable: false,
            message: "🔒 This content requires YouTube Premium.".to_string(),
            suggestions: vec![
                "✔ Available offline in YouTube app (Premium subscription)".to_string(),
                "✖ Cannot be downloaded as a file".to_string(),
            ],
        }
    }

    /// Members-only content
    pub fn members_only() -> Self {
        Self {
            restriction_type: "members_only".to_string(),
            is_downloadable: true, // Can be downloaded with proper cookies
            message: "🎫 This video requires channel membership.".to_string(),
            suggestions: vec![
                "✔ Use cookies from a browser where you're a member".to_string(),
                "✖ Cannot be downloaded without membership".to_string(),
            ],
        }
    }

    /// Paid content (rental/purchase)
    pub fn paid() -> Self {
        Self {
            restriction_type: "paid".to_string(),
            is_downloadable: false,
            message: "💳 This content requires purchase or rental.".to_string(),
            suggestions: vec![
                "This is paid content (movie/rental)".to_string(),
                "✖ Cannot be downloaded - DRM protection".to_string(),
            ],
        }
    }

    /// Age-restricted content
    pub fn age_restricted() -> Self {
        Self {
            restriction_type: "age_restricted".to_string(),
            is_downloadable: true, // Can be downloaded with login
            message: "🔞 This video is age-restricted.".to_string(),
            suggestions: vec![
                "✔ Use cookies from a logged-in browser".to_string(),
                "Your account must be 18+".to_string(),
            ],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoInfo {
    pub title: String,
    pub duration: String,
    pub thumbnail: String,
    pub uploader: String,
    pub formats: Vec<FormatOption>,
    /// Content restriction information (DRM, Premium, etc.)
    pub restriction: RestrictionInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormatOption {
    pub label: String,
    pub value: String,
    pub size: Option<String>,
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
        // No forced player_client: yt-dlp's own defaults track YouTube changes,
        // while a pinned list goes stale and stops returning usable formats.
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
    let proxy = proxy.or_else(utils::auto_detect_proxy);
    let is_youtube = url.to_lowercase().contains("youtube.com") || url.to_lowercase().contains("youtu.be");

    // Strategies, in order. `None` means: pass no player_client at all and let
    // yt-dlp pick.
    //
    // This used to force web,web_safari,ios to bypass SABR. That list rotted:
    // by 2026 every one of those clients returns "Requested format is not
    // available" for any video, so info fetch failed outright. yt-dlp keeps its
    // own default list working as YouTube changes, so defer to it first and
    // keep `all` as the broad fallback.
    let has_cookies = cookies_path.is_some() || cookies_from_browser.unwrap_or(false);
    let mut strategies: Vec<(Option<&str>, bool)> = vec![(None, false)];
    if has_cookies {
        strategies.push((None, true)); // age-gated / private videos
    }
    if is_youtube {
        strategies.push((Some("all"), false));
        if has_cookies {
            strategies.push((Some("all"), true));
        }
    }

    let mut last_error = String::new();
    // A cookie-copy failure says nothing about the video. Keep the first error
    // that does, so it is not masked by a later attempt that only tripped on
    // cookies — which is exactly how this bug hid itself.
    let mut informative_error: Option<String> = None;
    let mut saw_cookie_failure = false;

    for (client, allow_cookies) in strategies {
        let mut args = vec![
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
            url.to_string(),
        ];

        if let Some(c) = client {
            args.push("--extractor-args".to_string());
            args.push(format!("youtube:player_client={}", c));
        }

        let mut using_cookies = false;
        if allow_cookies {
            if let Some(path) = &cookies_path {
                args.push("--cookies".to_string());
                args.push(path.clone());
                using_cookies = true;
            } else if cookies_from_browser.unwrap_or(false) {
                args.push("--cookies-from-browser".to_string());
                args.push("chrome".to_string());
                using_cookies = true;
            }
        }
        
        if let Some(proxy_url) = &proxy {
            if client.is_none() { // Only log once, on the first strategy
                 eprintln!("[yt-dlp] Using proxy for info: {}", proxy_url);
            }
            args.push("--proxy".to_string());
            args.push(proxy_url.clone());
        }

        let output_res = run_output_with_timeout(&ytdlp_path, args, 30).await;
        
        match output_res {
            Ok(output) => {
                if output.status.success() {
                     eprintln!("[yt-dlp] Info fetched successfully with client: {} (cookies: {})",
                        client.unwrap_or("yt-dlp default"), using_cookies);
                     return parse_video_info(&output.stdout);
                }
                last_error = String::from_utf8_lossy(&output.stderr).to_string();
            }
            Err(e) => {
                last_error = e;
            }
        }

        if is_cookie_extraction_failure(&last_error.to_lowercase()) {
            saw_cookie_failure = true;
        } else if informative_error.is_none() {
            informative_error = Some(last_error.clone());
        }
        
        // If not success, try next strategy...
    }

    // Diagnose the error that actually describes the video, not a cookie hiccup.
    let reported = informative_error.unwrap_or(last_error);
    let cookie_note = if saw_cookie_failure {
        "\n\nNote: browser cookies could not be read, so those attempts were skipped. \
Close Chrome, or set Cookies to 'None' in Tools if the video is public."
    } else {
        ""
    };

    if let Some(reason) = diagnose_error(&reported) {
        let suggestion = get_blocking_suggestion(&reason, proxy.as_deref());
        return Err(format!(
            "{}\n\n{}\n\nDetails: {}{}",
            reason.description(),
            suggestion,
            reported.lines().take(3).collect::<Vec<_>>().join(" | "),
            cookie_note
        ));
    }

    Err(format!("yt-dlp info failed: {}{}", reported, cookie_note))
}

/// Detect content restriction from video JSON
fn detect_restriction(json: &serde_json::Value) -> RestrictionInfo {
    // Check availability status
    let availability = json["availability"].as_str().unwrap_or("");
    let is_live = json["is_live"].as_bool().unwrap_or(false);
    let live_status = json["live_status"].as_str().unwrap_or("");
    
    // Check for various restriction indicators
    let categories = json["categories"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_default();
    
    let description = json["description"].as_str().unwrap_or("").to_lowercase();
    let title = json["title"].as_str().unwrap_or("").to_lowercase();
    
    // Check age restriction
    if json["age_limit"].as_u64().unwrap_or(0) >= 18 || availability == "needs_auth" {
        // Age-restricted but downloadable with cookies
        return RestrictionInfo::age_restricted();
    }
    
    // Check for DRM indicators in formats
    let formats = json["formats"].as_array();
    let has_drm = formats.map_or(false, |fmts| {
        fmts.iter().any(|f| {
            // Check for DRM-related fields
            f["drm"].as_bool().unwrap_or(false)
                || f["has_drm"].as_bool().unwrap_or(false)
                || f.get("_drm_scheme").is_some()
                || f["protocol"].as_str().map_or(false, |p| p.contains("drm"))
        })
    });
    
    // Check for paid content (movies, rentals)
    let is_paid = json["is_paid_video"].as_bool().unwrap_or(false)
        || json["requires_payment"].as_bool().unwrap_or(false)
        || json["paid_content"].as_bool().unwrap_or(false)
        || availability == "premium_only"
        || categories.iter().any(|c| c.to_lowercase().contains("movie"));
    
    // Check for YouTube Premium content
    let is_premium = json["is_premium"].as_bool().unwrap_or(false)
        || json["requires_premium"].as_bool().unwrap_or(false)
        || description.contains("youtube premium")
        || title.contains("premium");
    
    // Check for members-only content
    let is_members_only = availability == "subscriber_only"
        || json["subscriber_only"].as_bool().unwrap_or(false)
        || json["is_member_only"].as_bool().unwrap_or(false)
        || description.contains("members only")
        || description.contains("members-only");
    
    // Check for YouTube Music (often DRM protected)
    let is_music_premium = json["extractor"].as_str().map_or(false, |e| {
        e.contains("music") || e == "youtube:music"
    }) && is_premium;
    
    // Check for no downloadable formats (strong DRM indicator)
    let no_formats = formats.map_or(true, |fmts| {
        fmts.iter().all(|f| {
            // Format is not downloadable if:
            // - It's manifest-only (m3u8/mpd without direct URL)
            // - Or has DRM
            let protocol = f["protocol"].as_str().unwrap_or("");
            let url = f["url"].as_str().unwrap_or("");
            (protocol == "m3u8_native" || protocol == "http_dash_segments")
                && url.is_empty()
        })
    });

    // Determine restriction type
    if has_drm || no_formats {
        let content_type = if is_music_premium {
            "YouTube Music track"
        } else if categories.iter().any(|c| c.to_lowercase().contains("movie")) {
            "movie"
        } else {
            "video"
        };
        return RestrictionInfo::drm(content_type);
    }
    
    if is_paid {
        return RestrictionInfo::paid();
    }
    
    if is_premium || is_music_premium {
        return RestrictionInfo::premium();
    }
    
    if is_members_only {
        return RestrictionInfo::members_only();
    }
    
    // Check if it's a live stream (not an error, just info)
    if is_live || live_status == "is_live" {
        // Live streams are generally not downloadable in real-time
        // but we don't mark them as restricted
    }
    
    // No restriction detected
    RestrictionInfo::none()
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

    let formats = extract_format_options(&json);
    
    // Detect content restrictions (DRM, Premium, etc.)
    let restriction = detect_restriction(&json);
    
    // Log restriction if detected
    if restriction.restriction_type != "none" {
        eprintln!(
            "[yt-dlp] Content restriction detected: {} - {}",
            restriction.restriction_type, restriction.message
        );
    }

    Ok(VideoInfo {
        title: json["title"].as_str().unwrap_or("Unknown").to_string(),
        duration,
        thumbnail: json["thumbnail"].as_str().unwrap_or("").to_string(),
        uploader: json["uploader"].as_str().unwrap_or("Unknown").to_string(),
        formats,
        restriction,
    })
}

fn extract_format_options(json: &serde_json::Value) -> Vec<FormatOption> {
    let mut options = Vec::new();
    let formats = match json["formats"].as_array() {
        Some(f) => f,
        None => return options,
    };

    // Helper to get size
    let get_size = |f: &serde_json::Value| -> u64 {
        f["filesize"].as_u64()
            .or_else(|| f["filesize_approx"].as_u64())
            .unwrap_or(0)
    };

    // Find best audio size
    let best_audio_size = formats.iter()
        .filter(|f| f["vcodec"].as_str().map_or(false, |v| v == "none"))
        .map(|f| get_size(f))
        .max()
        .unwrap_or(0);

    // Format size string
    let format_size = |bytes: u64| -> Option<String> {
        if bytes == 0 { return None; }
        let mb = bytes as f64 / 1_048_576.0;
        if mb >= 1024.0 {
            Some(format!("{:.1} GB", mb / 1024.0))
        } else {
            Some(format!("{:.0} MB", mb))
        }
    };

    // Define standard targets
    let targets = vec![
        ("1080p", 1080),
        ("720p", 720),
        ("480p", 480),
        ("360p", 360),
    ];

    // 1. Best (Max video + Max Audio)
    // Fix: Don't prioritize "video only" (acodec=none) blindly.
    // Instead, find the format with the largest Height (resolution). 
    // If heights are equal, pick the largest filesize.
    let best_f = formats.iter()
        .filter(|f| f["vcodec"].as_str().map_or(false, |v| v != "none")) // Must have video
        .max_by(|a, b| {
            let h_a = a["height"].as_u64().unwrap_or(0);
            let h_b = b["height"].as_u64().unwrap_or(0);
            match h_a.cmp(&h_b) {
                std::cmp::Ordering::Equal => {
                    let s_a = get_size(a);
                    let s_b = get_size(b);
                    s_a.cmp(&s_b)
                }
                other => other,
            }
        });

    if let Some(f) = best_f {
        let size = get_size(f);
        let w = f["width"].as_u64().unwrap_or(0);
        let h = f["height"].as_u64().unwrap_or(0);
        
        // If it's video-only, add audio size. If it's merged, take size as is.
        let is_video_only = f["acodec"].as_str().map_or(false, |a| a == "none");
        let total = if is_video_only && size > 0 { 
            size + best_audio_size 
        } else { 
            size 
        };

        let label = if w > 0 && h > 0 {
             format!("Best Quality ({}x{})", w, h)
        } else {
             "Best Quality".to_string()
        };

        options.push(FormatOption {
            label,
            value: "best".to_string(), // Keep "best" as value for download logic
            size: format_size(total),
        });
    } else {
        options.push(FormatOption {
            label: "Best Quality".to_string(),
            value: "best".to_string(),
            size: None,
        });
    }

    // 2. Specific resolutions
    for (base_label, target_h) in targets {
        // Check if any format matches this resolution
        let matches: Vec<&serde_json::Value> = formats.iter().filter(|f| {
             let h = f["height"].as_u64().unwrap_or(0);
             h >= target_h * 9 / 10 && h <= target_h * 11 / 10
        }).collect();

        // Find "best" among matches (largest size)
        let best_match = matches.iter().max_by_key(|f| get_size(f));

        if let Some(&f) = best_match {
             let size = get_size(f);
             let w = f["width"].as_u64().unwrap_or(0);
             let h = f["height"].as_u64().unwrap_or(0);
             // If video size is 0 (unknown), result is 0 (unknown)
             let total = if size > 0 { size + best_audio_size } else { 0 };
             
             let label = if w > 0 && h > 0 {
                  format!("{} ({}x{})", base_label, w, h)
             } else {
                  base_label.to_string()
             };
             
             options.push(FormatOption {
                 label,
                 value: base_label.to_string(),
                 size: format_size(total),
             });
        }
    }

    // 3. Audio Only
    options.push(FormatOption {
        label: "Audio Only (MP3)".to_string(),
        value: "audio".to_string(),
        size: format_size(best_audio_size),
    });

    options
}

async fn try_download_with_ytdlp(
    url: &str,
    quality: &str,
    codec: &str,
    output_path: &str,
    proxy_override: Option<String>,
    cookies_from_browser: bool,
    cookies_path: Option<String>,
    player_client_override: Option<String>,
    po_token: Option<String>,
    po_token_client: Option<String>,
    allow_fallback: bool,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // Fail fast on an unusable output folder. Without this, every strategy
    // runs and fails on the same directory error, and the last one's
    // diagnosis - PO Token, SABR - gets reported instead of the real cause.
    if let Err(msg) = crate::downloader::platform::check_writable_dir(output_path) {
        return Err(format!(
            "{}

Pick a different folder in the app before downloading.",
            msg
        ));
    }

    // Determine format based on quality and codec selection
    let format_arg = if codec == "h264" {
        // H.264 (avc1) + AAC for QuickTime/macOS compatibility
        match quality {
            "best" => "bv*[vcodec^=avc1]+ba[acodec^=mp4a]/bv*[vcodec^=avc]+ba/bv*+ba/best",
            "1080p" => "bv*[height<=1080][vcodec^=avc1]+ba[acodec^=mp4a]/bv*[height<=1080]+ba/best",
            "720p" => "bv*[height<=720][vcodec^=avc1]+ba[acodec^=mp4a]/bv*[height<=720]+ba/best",
            "480p" => "bv*[height<=480][vcodec^=avc1]+ba[acodec^=mp4a]/bv*[height<=480]+ba/best",
            "audio" => "ba[acodec^=mp4a]/ba/b",
            _ => "bv*[vcodec^=avc1]+ba[acodec^=mp4a]/bv*+ba/best",
        }
    } else {
        // VP9/AV1 - best quality (needs VLC or other players)
        match quality {
            "best" => "bv*+ba/best",
            "1080p" => "bv*[height<=1080]+ba/best",
            "720p" => "bv*[height<=720]+ba/best",
            "480p" => "bv*[height<=480]+ba/best",
            "audio" => "ba/b",
            _ => "bv*+ba/best",
        }
    };

    let ytdlp_path = find_ytdlp();
    let ffmpeg_dir = find_ffmpeg_dir();

    // Auto-detect proxy - ALWAYS try to use SOCKS for yt-dlp
    // Even in TUN mode, CLI apps may not route through system TUN
    let has_proxy_override = proxy_override.is_some();
    let mut proxy = proxy_override.or_else(|| {
        let detected = utils::auto_detect_proxy();
        if detected.is_some() {
            eprintln!("[download_video] Using detected proxy for yt-dlp");
        } else {
            eprintln!("[download_video] No proxy detected - yt-dlp will use direct connection");
        }
        detected
    });
    let is_youtube = {
        let lower = url.to_lowercase();
        lower.contains("youtube.com") || lower.contains("youtu.be")
    };

    let socket_timeout = env_u64("YTDLP_SOCKET_TIMEOUT_SECS", 60, 5, 600);
    let download_timeout_secs = env_u64("YTDLP_DOWNLOAD_TIMEOUT_SECS", 15 * 60, 60, 60 * 60);
    let stall_timeout_secs = env_u64("YTDLP_STALL_TIMEOUT_SECS", 120, 30, 30 * 60);
    let player_client_override_env = env::var("YTDLP_PLAYER_CLIENT_OVERRIDE").ok();
    let extra_extractor_args = env::var("YTDLP_EXTRACTOR_ARGS").ok();
    let po_token_env = env::var("YTDLP_PO_TOKEN").ok();
    let po_token_client_env = env::var("YTDLP_PO_TOKEN_CLIENT").ok();

    let player_client_override = player_client_override
        .and_then(|v| {
            let trimmed = v.trim().to_string();
            if trimmed.is_empty() { None } else { Some(trimmed) }
        })
        .or_else(|| player_client_override_env.and_then(|v| {
            let trimmed = v.trim().to_string();
            if trimmed.is_empty() { None } else { Some(trimmed) }
        }));

    let po_token = po_token
        .and_then(|v| {
            let trimmed = v.trim().to_string();
            if trimmed.is_empty() { None } else { Some(trimmed) }
        })
        .or_else(|| po_token_env.and_then(|v| {
            let trimmed = v.trim().to_string();
            if trimmed.is_empty() { None } else { Some(trimmed) }
        }));

    let po_token_client = po_token_client
        .and_then(|v| {
            let trimmed = v.trim().to_string();
            if trimmed.is_empty() { None } else { Some(trimmed) }
        })
        .or(po_token_client_env)
        .unwrap_or_else(|| "mweb".to_string());

    // Validate proxy before attempting download
    if proxy.is_some() {
        let (reachable, message) = utils::check_proxy_reachable(&proxy).await;
        if !reachable {
            let msg = message.unwrap_or_else(|| "Proxy unreachable".to_string());
            eprintln!("[download_video] Proxy check failed: {}", msg);
            let _ = app_handle.emit(
                "download-progress",
                DownloadProgress {
                    percent: 0.0,
                    status: format!("⚠️ Proxy issue: {}", msg),
                },
            );
            if has_proxy_override {
                return Err(format!("Proxy unreachable: {}", msg));
            } else {
                proxy = None;
                eprintln!("[download_video] Falling back to direct connection");
            }
        }
    }

    let build_args = |player_client: &str,
                      format_override: Option<&str>,
                      use_cookies: bool,
                      force_audio: bool| -> Vec<String> {
        let effective_client = player_client_override.as_deref().unwrap_or(player_client);
        let mut args = vec![
            "-f".to_string(),
            format_override.unwrap_or(format_arg).to_string(),
            "--no-playlist".to_string(),
            "--newline".to_string(),
            // keep stderr less noisy; we surface actionable messages ourselves
            "--no-update".to_string(),
            "--socket-timeout".to_string(),
            socket_timeout.to_string(),
            "--retries".to_string(),
            "5".to_string(),
            // Fragment handling for HLS/DASH streams
            "--fragment-retries".to_string(),
            "50".to_string(),  // Retry failed fragments up to 50 times
            "--file-access-retries".to_string(),
            "10".to_string(),
            // Skip unavailable fragments instead of failing entire download
            "--skip-unavailable-fragments".to_string(),
            // Use native HLS downloader (more reliable)
            "--hls-prefer-native".to_string(),
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
            let mut extractor_args = format!("youtube:player_client={}", effective_client);
            if let Some(token) = po_token.as_deref() {
                if effective_client.contains(&po_token_client) {
                    extractor_args.push_str(&format!(";po_token={}.gvs+{}", po_token_client, token));
                }
            }
            if let Some(extra) = extra_extractor_args.as_deref() {
                extractor_args.push(';');
                extractor_args.push_str(extra);
            }
            args.push(extractor_args);
            // Remux to fix mp4 structure for QuickTime compatibility
            args.push("--ppa".to_string());
            args.push("Merger+ffmpeg:-c copy -movflags +faststart".to_string());
        }

        // Ensure yt-dlp can find ffmpeg for merging
        if let Some(dir) = &ffmpeg_dir {
            eprintln!("[download_video] Using ffmpeg from: {}", dir);
            args.push("--ffmpeg-location".to_string());
            args.push(dir.clone());
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
    let has_po_token = po_token.is_some();
    let has_player_override = player_client_override.is_some();

    // Helper: run a single yt-dlp attempt with real-time progress streaming
    let run_with_progress = |args: Vec<String>, client: &str, use_cookies: bool, force_audio: bool| -> Result<(), String> {
        let mode_label = if force_audio { "🎵 audio" } else { "🎬 video" };
        let cookies_label = if use_cookies { "🍪" } else { "🔓" };
        
        eprintln!("[download_video] Starting yt-dlp: client={}, cookies={}", client, use_cookies);
        
        // Spawn process with piped stdout for real-time progress
        // Log the exact command. When the app fails at something that works
        // from a shell, the argv is the only way to see what actually differs.
        let printable: String = args
            .iter()
            .map(|a| if a.contains(' ') { format!("\"{}\"", a) } else { a.clone() })
            .collect::<Vec<_>>()
            .join(" ");
        eprintln!("[download_video] {} {}", ytdlp_path, printable);
        let _ = app_handle.emit(
            "download-progress",
            DownloadProgress {
                percent: 0.0,
                status: format!("▶ {} {}", ytdlp_path, printable.chars().take(700).collect::<String>()),
            },
        );

        let mut child = StdCommand::new(&ytdlp_path)
            .args(&args)
            // A GUI process has no usable stdin; handing the child an invalid
            // handle is a known source of odd Windows errors in child tools.
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start yt-dlp: {}", e))?;

        // Read stdout line by line for progress updates
        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        // Shared stderr buffer for error reporting
        let stderr_lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        // Spawn thread to read stderr and send lines to main loop
        let (tx, rx) = mpsc::channel::<String>();
        let stderr_lines_clone = Arc::clone(&stderr_lines);
        let tx_err = tx.clone();
        let stderr_handle = std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                if let Ok(mut locked) = stderr_lines_clone.lock() {
                    locked.push(line.clone());
                }
                let _ = tx_err.send(line);
            }
        });

        // Spawn thread to read stdout and send lines to main loop
        let stdout_handle = std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });

        // Timeouts: total download and progress stall
        let download_timeout = Duration::from_secs(download_timeout_secs);
        let stall_timeout = Duration::from_secs(stall_timeout_secs);
        let start_time = Instant::now();
        let mut last_progress = Instant::now();

        // Main loop: consume stdout, emit progress, and watch for timeouts
        let status = loop {
            if start_time.elapsed() > download_timeout {
                let _ = child.kill();
                let _ = stdout_handle.join();
                let _ = stderr_handle.join();
                return Err("Download timed out (15 min). Try again or use VPN/proxy.".to_string());
            }

            if last_progress.elapsed() > stall_timeout {
                let _ = child.kill();
                let _ = stdout_handle.join();
                let _ = stderr_handle.join();
                return Err("Download stalled (no progress for 5 min). Try again.".to_string());
            }

            match rx.recv_timeout(Duration::from_millis(250)) {
                Ok(line) => {
                    let is_download_line = line.contains("[download]")
                        || line.contains("Destination")
                        || line.contains("frag ");
                    if is_download_line {
                        last_progress = Instant::now();
                    }

                    // Parse and emit progress
                    if let Some((percent, status)) = parse_ytdlp_progress(&line) {
                        last_progress = Instant::now();
                        let _ = app_handle.emit(
                            "download-progress",
                            DownloadProgress { percent, status },
                        );
                    }
                    // Also log important lines
                    if line.contains("[download]") || line.contains("[Merger]") || line.contains("Destination") {
                        eprintln!("[yt-dlp] {}", line);
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {}
            }

            if let Some(status) = child.try_wait().map_err(|e| format!("Process error: {}", e))? {
                break status;
            }
        };

        let _ = stdout_handle.join();
        let _ = stderr_handle.join();
        let stderr_output = stderr_lines
            .lock()
            .map(|lines| lines.join("\n"))
            .unwrap_or_default();

        if status.success() {
            let success = format!("✅ Success! client={}, {}, {}", client, cookies_label, mode_label);
            eprintln!("[download_video] {}", success);
            let _ = app_handle.emit(
                "download-progress",
                DownloadProgress { percent: 100.0, status: success },
            );
            return Ok(());
        }

        Err(stderr_output)
    };

    // What yt-dlp actually said, kept across strategies. Without this the final
    // error was a fixed sentence and the UI log only ever showed our own label,
    // which made two separate failures indistinguishable from the log alone.
    let observed_error = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let observed_error_for_attempts = observed_error.clone();
    // Once the browser refuses to hand over its cookies, every later strategy
    // asking for them fails identically and burns an attempt. Retry those
    // without cookies instead — several clients work fine without.
    let cookies_broken = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let cookies_broken_for_attempts = cookies_broken.clone();
    // Full yt-dlp output for every attempt. The UI shows two lines per attempt,
    // which was not enough to explain why a command that works in a shell fails
    // inside the app.
    let transcript = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let transcript_for_attempts = transcript.clone();

    // Helper: run attempts for a given client list and cookie mode
    let run_attempts = |clients: Vec<&str>, use_cookies: bool, force_audio: bool| -> Result<(), String> {
        let mut last_stderr = String::new();
        for (idx, client) in clients.iter().enumerate() {
            let attempt = idx + 1;
            let total = clients.len();

            // Resolve the effective cookie mode before reporting it, or the
            // status line keeps showing 🍪 for attempts that already dropped them.
            let cookies_ok = !cookies_broken_for_attempts.load(std::sync::atomic::Ordering::Relaxed);
            let use_cookies = use_cookies && cookies_ok;

            // Emit user-friendly status with mode info
            let mode_emoji = if force_audio { "🎵" } else { "🎬" };
            let cookies_emoji = if use_cookies { "🍪" } else { "🔓" };
            let _ = app_handle.emit(
                "download-progress",
                DownloadProgress {
                    percent: 0.0,
                    status: format!(
                        "{} {} client={} | attempt {}/{}",
                        mode_emoji, cookies_emoji, client, attempt, total
                    ),
                },
            );

            let args = build_args(client, None, use_cookies, force_audio);
            let args_line = args.join(" ");

            match run_with_progress(args, client, use_cookies, force_audio) {
                Ok(()) => return Ok(()),
                Err(stderr) => {
                    last_stderr = stderr.clone();
                    
                    // Short reason for UI + terminal.
                    //
                    // ERROR lines first. yt-dlp emits warnings that mention SABR
                    // and "HTTP Error" on clients that are merely degraded, and
                    // taking the first matches showed those while dropping the
                    // actual failure further down.
                    let errors: Vec<&str> = stderr
                        .lines()
                        .map(|l| l.trim())
                        .filter(|s| s.starts_with("ERROR:"))
                        .collect();
                    let important_lines: Vec<&str> = if !errors.is_empty() {
                        errors.into_iter().take(2).collect()
                    } else {
                        stderr
                            .lines()
                            .map(|l| l.trim())
                            .filter(|s| {
                                s.contains("HTTP Error")
                                    || s.contains("Forbidden")
                                    || s.contains("SABR")
                                    || s.contains("Requested format is not available")
                            })
                            .take(2)
                            .collect()
                    };
                    
                    let preview = if !important_lines.is_empty() {
                        important_lines.join(" | ")
                    } else {
                        stderr.lines().rev().find(|l| !l.trim().is_empty())
                            .unwrap_or("Unknown error").chars().take(100).collect()
                    };
                    
                    eprintln!("[download_video] client {} error: {}", client, preview);
                    // Keep the first error that describes the video. A cookie
                    // copy failure says nothing about it, and later strategies
                    // kept overwriting the useful one.
                    if let Ok(mut log) = transcript_for_attempts.lock() {
                        log.push_str(&format!(
                            "===== client={} cookies={} audio={} =====
{} {}

{}

",
                            client, use_cookies, force_audio, ytdlp_path, args_line, stderr
                        ));
                    }

                    let cookie_only = is_cookie_extraction_failure(&preview.to_lowercase());
                    if cookie_only {
                        cookies_broken_for_attempts
                            .store(true, std::sync::atomic::Ordering::Relaxed);
                        eprintln!("[download_video] cookies unusable; later strategies drop them");
                    }
                    if let Ok(mut slot) = observed_error_for_attempts.lock() {
                        let have_useful = !slot.is_empty()
                            && !is_cookie_extraction_failure(&slot.to_lowercase());
                        if !have_useful && (!cookie_only || slot.is_empty()) {
                            *slot = preview.clone();
                        }
                    }

                    // Diagnose on ERROR lines when there are any: yt-dlp emits a
                    // SABR *warning* on clients that still work, and letting a
                    // warning name the reason hides the real failure.
                    let diagnosable = fatal_lines(&stderr).unwrap_or_else(|| stderr.clone());
                    let short = preview.chars().take(160).collect::<String>();
                    let diag_msg = if let Some(reason) = diagnose_error(&diagnosable) {
                        format!("⚠️ {} | client={} — {}", reason.description(), client, short)
                    } else {
                        format!("❌ client={} failed — {}", client, short)
                    };
                    
                    let _ = app_handle.emit(
                        "download-progress",
                        DownloadProgress { percent: 0.0, status: diag_msg },
                    );

                    // Don't let a cookie failure end the strategy: move on to the
                    // next client, which the flag above now runs without cookies.
                    if cookie_only && use_cookies {
                        eprintln!("[download_video] cookies failed on {}; next client goes without", client);
                        continue;
                    }

                    let retryable = is_youtube && (
                        stderr.contains("HTTP Error 403")
                        || stderr.contains("Forbidden")
                        || stderr.contains("SABR")
                        || stderr.contains("Requested format is not available")
                    );

                    if retryable && attempt < total {
                        eprintln!("[download_video] Retrying next client...");
                        continue;
                    }
                    break;
                }
            }
        }

        // Special case: quality not available -> retry best
        if last_stderr.contains("Requested format is not available") && quality != "best" && !force_audio {
            let _ = app_handle.emit(
                "download-progress",
                DownloadProgress {
                    percent: 0.0,
                    status: "⚠️ Quality not available. Trying best...".to_string(),
                },
            );

            let args = build_args("web,web_safari", Some("bv*+ba/best"), use_cookies, false);
            if run_with_progress(args, "web,web_safari", use_cookies, false).is_ok() {
                return Ok(());
            }
        }

        Err(last_stderr)
    };

    if !allow_fallback {
        eprintln!("[download_video] Fallback disabled: single yt-dlp attempt (multi-client)");
        let _ = app_handle.emit(
            "download-progress",
            DownloadProgress {
                percent: 0.0,
                status: "Single attempt: yt-dlp (web+web_safari+ios)".to_string(),
            },
        );

        // Even without fallback, use multi-client for best SABR bypass
        let primary_clients: Vec<&str> = if is_youtube { vec!["web,web_safari,ios"] } else { vec!["web"] };
        return run_attempts(primary_clients, cookies_enabled, quality == "audio")
            .map_err(|e| format!("yt-dlp failed (fallback off): {}", e));
    }

    // Phase 1: Multi-client strategy (best for bypassing SABR protection)
    // `all` lets yt-dlp try every client it knows, and keeps working as YouTube
    // changes. The pinned web,web_safari,ios list used to lead here, but by 2026
    // it returns no usable formats at all, so it is only a fallback now.
    let clients_multi: Vec<&str> = if is_youtube {
        vec!["all"]
    } else {
        vec!["web"]
    };

    eprintln!("[download_video] yt-dlp strategy: player_client=all");
    let _ = app_handle.emit(
        "download-progress",
        DownloadProgress { percent: 0.0, status: "🌐 Strategy 1: player_client=all".to_string() },
    );
    if run_attempts(clients_multi, false, false).is_ok() {
        return Ok(());
    }

    // Phase 2: legacy pinned clients, unless the user overrides the client
    if !has_player_override && is_youtube {
        eprintln!("[download_video] yt-dlp strategy: multi-client (web,web_safari,ios)");
        let _ = app_handle.emit(
            "download-progress",
            DownloadProgress { percent: 0.0, status: "🌐 Strategy 2: Multi-client (web+web_safari+ios)".to_string() },
        );
        let clients_all: Vec<&str> = vec!["web,web_safari,ios"];
        if run_attempts(clients_all, false, false).is_ok() {
            return Ok(());
        }
    }

    // Phase 3: If failed and cookies enabled -> Try with cookies (ios doesn't support cookies)
    if cookies_enabled {
        eprintln!("[download_video] yt-dlp strategy: cookies=on (web,web_safari)");
        let _ = app_handle.emit(
            "download-progress",
            DownloadProgress { percent: 0.0, status: "🍪 Strategy 3: With cookies (web+web_safari)".to_string() },
        );
        let clients = vec!["web,web_safari"];
        if run_attempts(clients, true, false).is_ok() {
            return Ok(());
        }
        
        eprintln!("[download_video] Authenticated download failed. Proceeding to fallbacks...");
    }

    // Phase 4: Optional PO Token path (mweb)
    if has_po_token && is_youtube {
        eprintln!("[download_video] yt-dlp strategy: PO Token (mweb)");
        let _ = app_handle.emit(
            "download-progress",
            DownloadProgress { percent: 0.0, status: "🧩 Strategy 4: PO Token (mweb)".to_string() },
        );
        let clients_po: Vec<&str> = vec!["default,mweb"];
        if run_attempts(clients_po, cookies_enabled, false).is_ok() {
            return Ok(());
        }
    }

    // Phase 5: TV/embedded clients (often bypass SABR)
    let clients_tv: Vec<&str> = if is_youtube {
        vec!["tv", "web_embedded"]
    } else {
        vec!["web"]
    };
    eprintln!("[download_video] yt-dlp strategy: tv/embedded");
    let _ = app_handle.emit(
        "download-progress",
        DownloadProgress { percent: 0.0, status: "📺 Strategy 5: TV/Embedded clients".to_string() },
    );
    if run_attempts(clients_tv, false, false).is_ok() {
        return Ok(());
    }

    // Phase 6: Fallback single clients (android/web)
    let clients_fallback: Vec<&str> = if is_youtube {
        vec!["android", "web"]
    } else {
        vec!["web"]
    };
    
    eprintln!("[download_video] yt-dlp strategy: single client fallback (android/web)");
    let _ = app_handle.emit(
        "download-progress",
        DownloadProgress { percent: 0.0, status: "🔄 Strategy 6: Single client fallback".to_string() },
    );
    if run_attempts(clients_fallback, cookies_enabled, false).is_ok() {
        return Ok(());
    }

    // Phase 7: last resort — audio-only (often allowed even when video is blocked)
    if quality != "audio" {
        eprintln!("[download_video] yt-dlp strategy: audio-only fallback");
        let _ = app_handle.emit(
            "download-progress",
            DownloadProgress {
                percent: 0.0,
                status: "🎵 Strategy 7: Audio-only fallback".to_string(),
            },
        );

        let clients_audio: Vec<&str> = if is_youtube { vec!["web,web_safari", "web"] } else { vec!["web"] };
        if run_attempts(clients_audio, cookies_enabled, true).is_ok() {
            return Ok(());
        }
    }

    let detail = observed_error
        .lock()
        .ok()
        .map(|s| s.clone())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "no output captured from yt-dlp".to_string());

    let saved = transcript.lock().ok().and_then(|log| {
        if log.trim().is_empty() {
            return None;
        }
        let name = format!(
            "download-{}.log",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        );
        crate::downloader::platform::write_transcript(&name, &log)
    });

    let where_to_look = match saved {
        Some(path) => format!("

Full yt-dlp output: {}", path.display()),
        None => String::new(),
    };

    Err(format!(
        "yt-dlp download failed after multiple strategies (cookies/no-cookies/audio fallback).

yt-dlp said: {}{}",
        detail, where_to_look
    ))
}

// Download video
#[tauri::command]
pub async fn download_video(
    url: String,
    quality: String,
    codec: Option<String>,
    output_path: String,
    tool: Option<String>,
    proxy: Option<String>,
    allow_fallback: Option<bool>,
    cookies_from_browser: Option<bool>,
    cookies_path: Option<String>,
    player_client_override: Option<String>,
    po_token: Option<String>,
    po_token_client: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    eprintln!("[download_video] Tool selected: {:?}, codec: {:?}", tool, codec);
    let selected = tool.as_deref().unwrap_or("yt-dlp");
    let allow_fallback = allow_fallback.unwrap_or(true);
    let codec = codec.unwrap_or_else(|| "h264".to_string());
    
    if selected != "yt-dlp" {
        eprintln!(
            "[download_video] {} requested, but only yt-dlp is supported now. Forcing yt-dlp.",
            selected
        );
    }

    let result = try_download_with_ytdlp(
                &url,
                &quality,
                &codec,
                &output_path,
                proxy.clone(),
                cookies_from_browser.unwrap_or(true),
                cookies_path.clone(),
                player_client_override,
                po_token,
                po_token_client,
        allow_fallback,
                app_handle.clone(),
            )
    .await;

    match result {
        Ok(()) => Ok("Download completed successfully with yt-dlp!".to_string()),
        Err(err) => {
            let diagnosis = if let Some(reason) = diagnose_error(&err) {
        format!(
            "\n\n⚠️ Detected: {}\n{}",
            reason.description(),
            get_blocking_suggestion(&reason, proxy.as_deref())
        )
    } else {
        String::new()
    };

    Err(format!(
                "yt-dlp download failed.{}\n\nDetails:\n{}",
                diagnosis, err
            ))
        }
    }
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
