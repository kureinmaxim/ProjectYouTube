// Blocking diagnostics - identifies YouTube blocking reasons
//
// Analyzes error messages to determine:
// - Type of blocking (403, SABR, PO Token, etc.)
// - Recommended action for the user
// - Whether retry with different settings might help

use serde::{Deserialize, Serialize};

/// Reasons why YouTube might block a request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockingReason {
    /// HTTP 403 Forbidden - general access denied
    Http403Forbidden,

    /// SABR (Segmented Adaptive Bitrate Restreaming) protection
    /// YouTube's streaming protection that hides formats
    SabrStreaming,

    /// PO Token (Proof of Origin) required
    /// New anti-bot measure requiring browser verification
    PoTokenRequired,

    /// Age-restricted content requiring login
    AgeRestricted,

    /// Geographic restriction
    GeoBlocked,

    /// Network timeout (soft IP block)
    NetworkTimeout,

    /// Rate limiting (429 or similar)
    RateLimited,

    /// Bot detection triggered
    BotDetection,

    /// Private video requiring authorization
    PrivateVideo,

    /// Video deleted or unavailable
    VideoUnavailable,

    /// DRM-protected content (YouTube Premium, Music, Movies)
    /// Cannot be downloaded - this is a permanent restriction, not an error
    DrmProtected,

    /// Member-only content (requires channel membership)
    MembersOnly,

    /// Generic/unknown blocking
    Unknown,
}

impl BlockingReason {
    /// Check if this reason is retryable with different settings
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Http403Forbidden
                | Self::SabrStreaming
                | Self::PoTokenRequired
                | Self::NetworkTimeout
                | Self::RateLimited
                | Self::BotDetection
        )
    }

    /// Check if cookies might help
    pub fn cookies_might_help(&self) -> bool {
        matches!(
            self,
            Self::Http403Forbidden
                | Self::SabrStreaming
                | Self::PoTokenRequired
                | Self::AgeRestricted
                | Self::BotDetection
                | Self::PrivateVideo
                | Self::MembersOnly
        )
    }

    /// Check if proxy might help
    pub fn proxy_might_help(&self) -> bool {
        matches!(
            self,
            Self::Http403Forbidden
                | Self::GeoBlocked
                | Self::NetworkTimeout
                | Self::RateLimited
                | Self::BotDetection
        )
    }

    /// Check if audio-only fallback might work
    pub fn audio_fallback_might_work(&self) -> bool {
        matches!(self, Self::SabrStreaming | Self::Http403Forbidden)
    }

    /// Check if this is a permanent restriction (no workaround)
    pub fn is_permanent(&self) -> bool {
        matches!(
            self,
            Self::DrmProtected | Self::VideoUnavailable
        )
    }

    /// Check if this is DRM-related
    pub fn is_drm(&self) -> bool {
        matches!(self, Self::DrmProtected)
    }

    /// Human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Http403Forbidden => "Access denied (HTTP 403)",
            Self::SabrStreaming => "SABR streaming protection active",
            Self::PoTokenRequired => "Proof of Origin token required",
            Self::AgeRestricted => "Age-restricted content",
            Self::GeoBlocked => "Geographic restriction",
            Self::NetworkTimeout => "Network timeout (possible IP throttling)",
            Self::RateLimited => "Rate limited by YouTube",
            Self::BotDetection => "Bot detection triggered",
            Self::PrivateVideo => "Private video",
            Self::VideoUnavailable => "Video unavailable",
            Self::DrmProtected => "DRM-protected content",
            Self::MembersOnly => "Members-only content",
            Self::Unknown => "Unknown blocking reason",
        }
    }

    /// Get user-friendly explanation for DRM/permanent restrictions
    pub fn user_explanation(&self) -> Option<&'static str> {
        match self {
            Self::DrmProtected => Some(
                "This video is DRM-protected and cannot be downloaded.\n\n\
                 ✔ Available offline in YouTube app (Premium)\n\
                 ✔ Can be screen-recorded\n\
                 ✖ Cannot be downloaded as a file\n\n\
                 This is a content protection measure, not an error."
            ),
            Self::MembersOnly => Some(
                "This video requires a channel membership.\n\n\
                 ✔ Available if you're a member (use cookies)\n\
                 ✖ Cannot be downloaded without membership\n\n\
                 Try using cookies from a browser where you're a member."
            ),
            Self::VideoUnavailable => Some(
                "This video has been removed or is no longer available."
            ),
            _ => None,
        }
    }
}

/// Detailed diagnostics information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockingDiagnostics {
    /// Primary blocking reason
    pub reason: BlockingReason,

    /// Additional context from error message
    pub context: Option<String>,

    /// Whether cookies are recommended
    pub recommend_cookies: bool,

    /// Whether proxy is recommended
    pub recommend_proxy: bool,

    /// Whether audio-only might work
    pub recommend_audio_only: bool,

    /// Severity level (1-5, 5 being most severe)
    pub severity: u8,

    /// Raw error patterns that matched
    pub matched_patterns: Vec<String>,
}

impl BlockingDiagnostics {
    pub fn new(reason: BlockingReason, context: Option<String>) -> Self {
        let severity = match reason {
            BlockingReason::DrmProtected => 5, // Permanent - no workaround
            BlockingReason::VideoUnavailable => 5,
            BlockingReason::PrivateVideo => 4,
            BlockingReason::GeoBlocked => 4,
            BlockingReason::MembersOnly => 4,
            BlockingReason::AgeRestricted => 3,
            BlockingReason::PoTokenRequired => 3,
            BlockingReason::SabrStreaming => 3,
            BlockingReason::Http403Forbidden => 2,
            BlockingReason::BotDetection => 2,
            BlockingReason::RateLimited => 2,
            BlockingReason::NetworkTimeout => 1,
            BlockingReason::Unknown => 1,
        };

        Self {
            reason,
            context,
            recommend_cookies: reason.cookies_might_help(),
            recommend_proxy: reason.proxy_might_help(),
            recommend_audio_only: reason.audio_fallback_might_work(),
            severity,
            matched_patterns: Vec::new(),
        }
    }

    pub fn with_patterns(mut self, patterns: Vec<String>) -> Self {
        self.matched_patterns = patterns;
        self
    }

    /// Check if this is a permanent restriction (DRM, etc.)
    pub fn is_permanent(&self) -> bool {
        self.reason.is_permanent()
    }

    /// Check if this is DRM-protected content
    pub fn is_drm(&self) -> bool {
        self.reason.is_drm()
    }

    /// Get user-friendly explanation for this restriction
    pub fn user_explanation(&self) -> Option<&'static str> {
        self.reason.user_explanation()
    }
}

/// Analyze error message and return blocking reason
pub fn diagnose_error(error: &str) -> Option<BlockingReason> {
    let lower = error.to_lowercase();

    // Check patterns in order of specificity

    // DRM protection (most important - permanent restriction)
    if lower.contains("drm")
        || lower.contains("widevine")
        || lower.contains("playready")
        || lower.contains("fairplay")
        || lower.contains("encrypted media")
        || lower.contains("content is protected")
        || lower.contains("youtube premium")
        || lower.contains("youtube music premium")
        || lower.contains("requires purchase")
        || lower.contains("rental")
        || lower.contains("pay to watch")
        || lower.contains("this video requires payment")
    {
        return Some(BlockingReason::DrmProtected);
    }

    // Members-only content
    if lower.contains("members only")
        || lower.contains("members-only")
        || lower.contains("join this channel")
        || lower.contains("membership required")
        || lower.contains("available to members")
    {
        return Some(BlockingReason::MembersOnly);
    }

    // SABR streaming (most specific YouTube protection)
    if lower.contains("sabr") || lower.contains("forcing sabr streaming") {
        return Some(BlockingReason::SabrStreaming);
    }

    // PO Token (new anti-bot measure)
    if lower.contains("po token") || lower.contains("gvs po token") || lower.contains("proof of origin") {
        return Some(BlockingReason::PoTokenRequired);
    }

    // Age restriction
    if lower.contains("age-restricted")
        || lower.contains("sign in to confirm your age")
        || lower.contains("age_verification")
    {
        return Some(BlockingReason::AgeRestricted);
    }

    // Private video
    if lower.contains("private video")
        || lower.contains("video is private")
        || lower.contains("sign in if you've been granted access")
    {
        return Some(BlockingReason::PrivateVideo);
    }

    // Video unavailable
    if lower.contains("video unavailable")
        || lower.contains("video has been removed")
        || lower.contains("this video is no longer available")
        || lower.contains("video is unavailable")
    {
        return Some(BlockingReason::VideoUnavailable);
    }

    // Geographic restriction
    if lower.contains("not available in your country")
        || lower.contains("geo")
        || lower.contains("blocked in your country")
        || lower.contains("geographic restriction")
    {
        return Some(BlockingReason::GeoBlocked);
    }

    // Rate limiting
    if lower.contains("429") || lower.contains("rate limit") || lower.contains("too many requests") {
        return Some(BlockingReason::RateLimited);
    }

    // Bot detection
    if lower.contains("bot")
        || lower.contains("captcha")
        || lower.contains("unusual traffic")
        || lower.contains("automated")
    {
        return Some(BlockingReason::BotDetection);
    }

    // HTTP 403 (general)
    if lower.contains("403") || lower.contains("forbidden") {
        return Some(BlockingReason::Http403Forbidden);
    }

    // Network timeout
    if lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("connection refused")
        || lower.contains("network unreachable")
    {
        return Some(BlockingReason::NetworkTimeout);
    }

    // Unknown
    if !error.is_empty() {
        return Some(BlockingReason::Unknown);
    }

    None
}

/// Full diagnostic analysis of an error
pub fn analyze_error(error: &str) -> BlockingDiagnostics {
    let reason = diagnose_error(error).unwrap_or(BlockingReason::Unknown);

    // Extract relevant patterns
    let patterns = extract_patterns(error);

    // Extract context (first useful line)
    let context = error
        .lines()
        .find(|line| {
            let l = line.trim().to_lowercase();
            l.starts_with("error:")
                || l.contains("forbidden")
                || l.contains("unavailable")
                || l.contains("sabr")
                || l.contains("token")
        })
        .map(|s| s.trim().to_string());

    BlockingDiagnostics::new(reason, context).with_patterns(patterns)
}

/// Extract matched patterns from error message
fn extract_patterns(error: &str) -> Vec<String> {
    let patterns = [
        "403",
        "forbidden",
        "sabr",
        "po token",
        "age-restricted",
        "private",
        "unavailable",
        "timeout",
        "429",
        "rate limit",
        "captcha",
        "bot",
        "geo",
        // DRM patterns
        "drm",
        "widevine",
        "playready",
        "fairplay",
        "encrypted",
        "premium",
        "purchase",
        "rental",
        "members only",
        "membership",
    ];

    let lower = error.to_lowercase();

    patterns
        .iter()
        .filter(|p| lower.contains(*p))
        .map(|p| p.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_403_detection() {
        let error = "ERROR: HTTP Error 403: Forbidden";
        assert_eq!(diagnose_error(error), Some(BlockingReason::Http403Forbidden));
    }

    #[test]
    fn test_sabr_detection() {
        let error = "YouTube is forcing SABR streaming for this client";
        assert_eq!(diagnose_error(error), Some(BlockingReason::SabrStreaming));
    }

    #[test]
    fn test_po_token_detection() {
        let error = "mweb client https formats require a GVS PO Token";
        assert_eq!(diagnose_error(error), Some(BlockingReason::PoTokenRequired));
    }

    #[test]
    fn test_age_restricted_detection() {
        let error = "Sign in to confirm your age";
        assert_eq!(diagnose_error(error), Some(BlockingReason::AgeRestricted));
    }

    #[test]
    fn test_timeout_detection() {
        let error = "Timed out after 30s";
        assert_eq!(diagnose_error(error), Some(BlockingReason::NetworkTimeout));
    }

    #[test]
    fn test_geo_detection() {
        let error = "Video not available in your country";
        assert_eq!(diagnose_error(error), Some(BlockingReason::GeoBlocked));
    }

    #[test]
    fn test_drm_detection() {
        let error = "This video is DRM protected";
        assert_eq!(diagnose_error(error), Some(BlockingReason::DrmProtected));
    }

    #[test]
    fn test_drm_widevine_detection() {
        let error = "Widevine encrypted content cannot be downloaded";
        assert_eq!(diagnose_error(error), Some(BlockingReason::DrmProtected));
    }

    #[test]
    fn test_drm_premium_detection() {
        let error = "This video requires YouTube Premium";
        assert_eq!(diagnose_error(error), Some(BlockingReason::DrmProtected));
    }

    #[test]
    fn test_drm_purchase_detection() {
        let error = "This video requires purchase to watch";
        assert_eq!(diagnose_error(error), Some(BlockingReason::DrmProtected));
    }

    #[test]
    fn test_members_only_detection() {
        let error = "This video is available to members only";
        assert_eq!(diagnose_error(error), Some(BlockingReason::MembersOnly));
    }

    #[test]
    fn test_drm_is_permanent() {
        assert!(BlockingReason::DrmProtected.is_permanent());
        assert!(BlockingReason::VideoUnavailable.is_permanent());
        assert!(!BlockingReason::Http403Forbidden.is_permanent());
    }

    #[test]
    fn test_drm_has_explanation() {
        assert!(BlockingReason::DrmProtected.user_explanation().is_some());
        assert!(BlockingReason::MembersOnly.user_explanation().is_some());
        assert!(BlockingReason::Http403Forbidden.user_explanation().is_none());
    }
}

