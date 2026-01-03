// Error types for downloader backends

use std::fmt;

#[derive(Debug, Clone)]
pub enum DownloadError {
    /// Network timeout while connecting to YouTube
    NetworkTimeout,
    
    /// YouTube blocked the request (429, bot detection, etc.)
    BlockedByYouTube,
    
    /// yt-dlp or python not found in system
    ToolNotFound(String),
    
    /// Invalid YouTube URL format
    InvalidUrl(String),
    
    /// URL not supported by the tool
    UnsupportedUrl(String),
    
    /// Network error (connection, proxy, etc.)
    NetworkError(String),
    
    /// Failed to parse yt-dlp JSON output
    ParseError(String),
    
    /// Command execution failed
    ExecutionError(String),
    
    /// DRM-protected content - permanent restriction, not an error
    /// This includes: YouTube Premium content, purchased/rented movies, YouTube Music
    DrmProtected(String),
    
    /// Members-only content requiring channel membership
    MembersOnly(String),
    
    /// Unknown error with details
    Unknown(String),
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NetworkTimeout => write!(f, "Network timeout: YouTube is not responding"),
            Self::BlockedByYouTube => write!(
                f,
                "YouTube is temporarily throttling requests from your IP address.\n\
                 This is normal and usually resolves on its own in 6–24 hours.\n\n\
                 What you can do:\n\
                 1) Wait and try again later\n\
                 2) Enable Proxy/VPN\n\
                 3) Try a different network\n\n\
                 More details: see YOUTUBE_BLOCKING.md"
            ),
            Self::ToolNotFound(tool) => write!(f, "Tool not found: {}", tool),
            Self::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            Self::UnsupportedUrl(msg) => write!(f, "URL not supported: {}", msg),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            Self::DrmProtected(content_type) => write!(
                f,
                "🔒 DRM-Protected Content\n\n\
                 This {} is protected by DRM and cannot be downloaded.\n\n\
                 ✔ Available offline in YouTube app (with Premium)\n\
                 ✔ Can be screen-recorded\n\
                 ✖ Cannot be downloaded as a file\n\n\
                 This is a content protection measure, not an error.\n\
                 Direct download is blocked by DRM encryption.",
                content_type
            ),
            Self::MembersOnly(channel) => write!(
                f,
                "🎫 Members-Only Content\n\n\
                 This video requires {} channel membership.\n\n\
                 ✔ Available if you're a member\n\
                 ✖ Cannot be downloaded without membership\n\n\
                 Try using cookies from a browser where you're logged in as a member.",
                channel
            ),
            Self::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for DownloadError {}

// Convert from String for backward compatibility
impl From<String> for DownloadError {
    fn from(s: String) -> Self {
        let lower = s.to_lowercase();
        
        // DRM detection (most important - permanent restriction, check first)
        if lower.contains("drm")
            || lower.contains("widevine")
            || lower.contains("playready")
            || lower.contains("fairplay")
            || lower.contains("encrypted media")
            || lower.contains("content is protected")
            || lower.contains("requires purchase")
            || lower.contains("rental")
            || lower.contains("pay to watch")
            || lower.contains("this video requires payment")
        {
            // Try to identify content type
            let content_type = if lower.contains("music") {
                "YouTube Music track"
            } else if lower.contains("movie") || lower.contains("film") {
                "movie/film"
            } else if lower.contains("premium") {
                "YouTube Premium content"
            } else {
                "video"
            };
            return Self::DrmProtected(content_type.to_string());
        }
        
        // Members-only content
        if lower.contains("members only")
            || lower.contains("members-only")
            || lower.contains("join this channel")
            || lower.contains("membership required")
            || lower.contains("available to members")
        {
            return Self::MembersOnly("a".to_string());
        }
        
        // IP blocking detection (most important)
        if (s.contains("timeout") || s.contains("timed out")) 
            && s.contains("youtube.com") {
            return Self::BlockedByYouTube;
        }
        
        // Generic network timeout
        if s.contains("timeout") || s.contains("timed out") {
            return Self::NetworkTimeout;
        }
        
        // Explicit blocks
        if s.contains("429") || lower.contains("bot") || lower.contains("blocked") {
            return Self::BlockedByYouTube;
        }
        
        // Network errors
        if s.contains("connection") || s.contains("Connection") 
            || s.contains("network") || s.contains("Network")
            || s.contains("tcp") || s.contains("socket") {
            return Self::NetworkError(s);
        }
        
        // Tool not found
        if s.contains("not found") || s.contains("No such file") || s.contains("command not found") {
            return Self::ToolNotFound(s);
        }
        
        // Parse errors
        if s.contains("parse") || s.contains("JSON") || s.contains("Invalid JSON") {
            return Self::ParseError(s);
        }
        
        // Unsupported URLs
        if s.contains("not support") || s.contains("unsupported") {
            return Self::UnsupportedUrl(s);
        }
        
        // Invalid URLs
        if s.contains("Invalid URL") || s.contains("Unsupported URL") {
            return Self::InvalidUrl(s);
        }
        
        // Everything else
        Self::Unknown(s)
    }
}
