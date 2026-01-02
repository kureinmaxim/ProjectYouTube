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
            Self::NetworkTimeout => write!(f, "Network timeout: YouTube не отвечает"),
            Self::BlockedByYouTube => write!(f, "YouTube заблокировал запрос"),
            Self::ToolNotFound(tool) => write!(f, "Инструмент не найден: {}", tool),
            Self::InvalidUrl(url) => write!(f, "Неверный URL: {}", url),
            Self::ParseError(msg) => write!(f, "Ошибка парсинга: {}", msg),
            Self::ExecutionError(msg) => write!(f, "Ошибка выполнения: {}", msg),
            Self::Unknown(msg) => write!(f, "Неизвестная ошибка: {}", msg),
        }
    }
}

impl std::error::Error for DownloadError {}

// Convert from String for backward compatibility
impl From<String> for DownloadError {
    fn from(s: String) -> Self {
        // Try to detect error type from message
        if s.contains("timeout") || s.contains("timed out") {
            Self::NetworkTimeout
        } else if s.contains("429") || s.contains("bot") {
            Self::BlockedByYouTube
        } else if s.contains("not found") || s.contains("No such file") {
            Self::ToolNotFound(s)
        } else if s.contains("parse") || s.contains("JSON") {
            Self::ParseError(s)
        } else {
            Self::Unknown(s)
        }
    }
}
