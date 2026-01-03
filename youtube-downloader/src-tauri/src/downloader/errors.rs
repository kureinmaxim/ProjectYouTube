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
    
    /// Failed to parse yt-dlp JSON output
    ParseError(String),
    
    /// Command execution failed
    ExecutionError(String),
    
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
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            Self::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for DownloadError {}

// Convert from String for backward compatibility
impl From<String> for DownloadError {
    fn from(s: String) -> Self {
        // Smart detection of error types
        
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
        if s.contains("429") || s.contains("bot") || s.contains("blocked") {
            return Self::BlockedByYouTube;
        }
        
        // Tool not found
        if s.contains("not found") || s.contains("No such file") || s.contains("command not found") {
            return Self::ToolNotFound(s);
        }
        
        // Parse errors
        if s.contains("parse") || s.contains("JSON") || s.contains("Invalid JSON") {
            return Self::ParseError(s);
        }
        
        // Invalid URLs
        if s.contains("Invalid URL") || s.contains("Unsupported URL") {
            return Self::InvalidUrl(s);
        }
        
        // Everything else
        Self::Unknown(s)
    }
}
