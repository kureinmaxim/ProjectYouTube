// Common data models for downloader

use serde::{Deserialize, Serialize};

/// Types of content restrictions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestrictionType {
    /// No restriction - content is freely downloadable
    None,
    /// DRM-protected (Widevine, PlayReady, FairPlay)
    Drm,
    /// Requires YouTube Premium subscription
    Premium,
    /// Requires channel membership
    MembersOnly,
    /// Requires purchase or rental
    PaidContent,
    /// Age-restricted (needs login)
    AgeRestricted,
    /// Geographic restriction
    GeoBlocked,
    /// Private video
    Private,
}

impl RestrictionType {
    /// Check if this restriction allows download with workarounds
    pub fn has_workaround(&self) -> bool {
        matches!(
            self,
            Self::None | Self::AgeRestricted | Self::GeoBlocked | Self::MembersOnly
        )
    }

    /// Check if this is a permanent restriction (no workaround possible)
    pub fn is_permanent(&self) -> bool {
        matches!(self, Self::Drm | Self::Premium | Self::PaidContent)
    }
}

/// Content restriction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRestriction {
    /// Type of restriction
    pub restriction_type: RestrictionType,
    /// Human-readable reason
    pub reason: String,
    /// Whether download is possible at all
    pub is_downloadable: bool,
    /// Suggestions for the user
    pub suggestions: Vec<String>,
}

impl ContentRestriction {
    /// Create a "no restriction" instance
    pub fn none() -> Self {
        Self {
            restriction_type: RestrictionType::None,
            reason: String::new(),
            is_downloadable: true,
            suggestions: Vec::new(),
        }
    }

    /// Create a DRM restriction
    pub fn drm(content_type: &str) -> Self {
        Self {
            restriction_type: RestrictionType::Drm,
            reason: format!("This {} is DRM-protected", content_type),
            is_downloadable: false,
            suggestions: vec![
                "✔ Available offline in YouTube app (with Premium)".to_string(),
                "✔ Can be screen-recorded".to_string(),
                "✖ Cannot be downloaded as a file".to_string(),
            ],
        }
    }

    /// Create a Premium restriction
    pub fn premium() -> Self {
        Self {
            restriction_type: RestrictionType::Premium,
            reason: "This content requires YouTube Premium".to_string(),
            is_downloadable: false,
            suggestions: vec![
                "✔ Available offline in YouTube app (with Premium subscription)".to_string(),
                "✖ Cannot be downloaded as a file".to_string(),
            ],
        }
    }

    /// Create a members-only restriction
    pub fn members_only(channel: &str) -> Self {
        Self {
            restriction_type: RestrictionType::MembersOnly,
            reason: format!("This video requires {} channel membership", channel),
            is_downloadable: true, // Can be downloaded with proper cookies
            suggestions: vec![
                "✔ Use cookies from a browser where you're a member".to_string(),
                "✖ Cannot be downloaded without membership".to_string(),
            ],
        }
    }

    /// Create a paid content restriction
    pub fn paid_content() -> Self {
        Self {
            restriction_type: RestrictionType::PaidContent,
            reason: "This content requires purchase or rental".to_string(),
            is_downloadable: false,
            suggestions: vec![
                "This is paid content (movie/rental)".to_string(),
                "✖ Cannot be downloaded - DRM protection".to_string(),
            ],
        }
    }

    /// Create an age restriction
    pub fn age_restricted() -> Self {
        Self {
            restriction_type: RestrictionType::AgeRestricted,
            reason: "Age-restricted content".to_string(),
            is_downloadable: true,
            suggestions: vec![
                "✔ Use cookies from a logged-in browser".to_string(),
                "Your account must be 18+".to_string(),
            ],
        }
    }
}

/// Video information extracted from YouTube
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub uploader: String,
    pub duration: String,
    pub thumbnail: String,
}

/// Video format details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFormat {
    pub format_id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub filesize: Option<u64>,
}

/// Download options
#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub quality: String,
    pub output_path: String,
    pub extract_audio: bool,
    pub audio_format: Option<String>,
    pub proxy: Option<String>,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            quality: "720p".to_string(),
            output_path: dirs::download_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .to_string_lossy()
                .to_string(),
            extract_audio: false,
            audio_format: None,
            proxy: None,
        }
    }
}

/// Download progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub percent: f32,
    pub status: String,
}

/// Network configuration for backends
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// SOCKS5 proxy URL (e.g., "socks5://127.0.0.1:1080")
    pub proxy: Option<String>,
    
    /// Timeout in seconds
    pub timeout: Option<u32>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            proxy: None,
            timeout: Some(30),
        }
    }
}
