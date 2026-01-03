// InfoExtractor Orchestrator - automatic mode selection and fallback
//
// Strategy:
// 1. For YouTube: Python mode preferred (better anti-bot bypass)
// 2. For other sites: CLI mode preferred (faster)
// 3. Auto-fallback on failure

use super::cli::CliInfoExtractor;
use super::diagnostics::{diagnose_error, BlockingReason};
use super::python::PythonInfoExtractor;
use super::traits::{ExtendedFormat, ExtendedVideoInfo, ExtractorConfig, ExtractorMode, InfoExtractor};
use crate::downloader::errors::DownloadError;

/// Orchestrator that manages Python and CLI extractors
pub struct InfoExtractorOrchestrator {
    python: PythonInfoExtractor,
    cli: CliInfoExtractor,
}

impl InfoExtractorOrchestrator {
    pub fn new() -> Self {
        Self {
            python: PythonInfoExtractor::new(),
            cli: CliInfoExtractor::new(),
        }
    }

    /// Determine optimal mode for URL
    pub fn recommend_mode(&self, url: &str) -> ExtractorMode {
        let is_youtube = url.to_lowercase().contains("youtube.com")
            || url.to_lowercase().contains("youtu.be");

        if is_youtube {
            // YouTube aggressive blocking → Python better
            if self.python.is_available() {
                ExtractorMode::Python
            } else {
                ExtractorMode::Cli
            }
        } else {
            // Other sites → CLI faster
            if self.cli.is_available() {
                ExtractorMode::Cli
            } else {
                ExtractorMode::Python
            }
        }
    }

    /// Get availability status
    pub fn get_status(&self) -> OrchestratorStatus {
        OrchestratorStatus {
            python_available: self.python.is_available(),
            cli_available: self.cli.is_available(),
            recommended_mode: self.recommend_mode("https://youtube.com"),
        }
    }

    /// Extract video info with automatic mode selection
    pub async fn extract(
        &self,
        url: &str,
        config: ExtractorConfig,
    ) -> Result<ExtendedVideoInfo, ExtractorResult> {
        match config.mode {
            ExtractorMode::Python => self.extract_python(url, &config).await,
            ExtractorMode::Cli => self.extract_cli(url, &config).await,
            ExtractorMode::Auto => self.extract_auto(url, &config).await,
        }
    }

    /// Extract using Python mode only
    async fn extract_python(
        &self,
        url: &str,
        config: &ExtractorConfig,
    ) -> Result<ExtendedVideoInfo, ExtractorResult> {
        if !self.python.is_available() {
            return Err(ExtractorResult {
                error: DownloadError::ToolNotFound("Python yt_dlp module not installed".to_string()),
                blocking_reason: None,
                used_mode: ExtractorMode::Python,
                tried_fallback: false,
                suggestion: Some("Install yt-dlp: pip3 install yt-dlp".to_string()),
            });
        }

        match self.python.extract(url, config).await {
            Ok(info) => Ok(info),
            Err(e) => {
                let reason = diagnose_error(&e.to_string());
                Err(ExtractorResult {
                    error: e,
                    blocking_reason: reason,
                    used_mode: ExtractorMode::Python,
                    tried_fallback: false,
                    suggestion: self.suggest_for_reason(&reason),
                })
            }
        }
    }

    /// Extract using CLI mode only
    async fn extract_cli(
        &self,
        url: &str,
        config: &ExtractorConfig,
    ) -> Result<ExtendedVideoInfo, ExtractorResult> {
        if !self.cli.is_available() {
            return Err(ExtractorResult {
                error: DownloadError::ToolNotFound("yt-dlp binary not found".to_string()),
                blocking_reason: None,
                used_mode: ExtractorMode::Cli,
                tried_fallback: false,
                suggestion: Some("Install yt-dlp: brew install yt-dlp".to_string()),
            });
        }

        match self.cli.extract(url, config).await {
            Ok(info) => Ok(info),
            Err(e) => {
                let reason = diagnose_error(&e.to_string());
                Err(ExtractorResult {
                    error: e,
                    blocking_reason: reason,
                    used_mode: ExtractorMode::Cli,
                    tried_fallback: false,
                    suggestion: self.suggest_for_reason(&reason),
                })
            }
        }
    }

    /// Extract with automatic mode selection and fallback
    async fn extract_auto(
        &self,
        url: &str,
        config: &ExtractorConfig,
    ) -> Result<ExtendedVideoInfo, ExtractorResult> {
        let is_youtube = url.to_lowercase().contains("youtube.com")
            || url.to_lowercase().contains("youtu.be");

        // Strategy based on URL type
        let (primary, fallback): (ExtractorMode, ExtractorMode) = if is_youtube {
            // YouTube: Python first (better anti-bot)
            (ExtractorMode::Python, ExtractorMode::Cli)
        } else {
            // Other: CLI first (faster)
            (ExtractorMode::Cli, ExtractorMode::Python)
        };

        // Try primary
        let primary_available = match primary {
            ExtractorMode::Python => self.python.is_available(),
            ExtractorMode::Cli => self.cli.is_available(),
            _ => false,
        };

        if primary_available {
            eprintln!(
                "[Orchestrator] Trying primary mode: {} for {}",
                primary,
                if is_youtube { "YouTube" } else { "other site" }
            );

            let result = match primary {
                ExtractorMode::Python => self.python.extract(url, config).await,
                ExtractorMode::Cli => self.cli.extract(url, config).await,
                _ => unreachable!(),
            };

            if let Ok(info) = result {
                eprintln!("[Orchestrator] Primary mode {} succeeded", primary);
                return Ok(info);
            } else if let Err(ref e) = result {
                eprintln!("[Orchestrator] Primary mode {} failed: {}", primary, e);
            }
        }

        // Try fallback
        let fallback_available = match fallback {
            ExtractorMode::Python => self.python.is_available(),
            ExtractorMode::Cli => self.cli.is_available(),
            _ => false,
        };

        if fallback_available {
            eprintln!("[Orchestrator] Trying fallback mode: {}", fallback);

            let result = match fallback {
                ExtractorMode::Python => self.python.extract(url, config).await,
                ExtractorMode::Cli => self.cli.extract(url, config).await,
                _ => unreachable!(),
            };

            match result {
                Ok(info) => {
                    eprintln!("[Orchestrator] Fallback mode {} succeeded", fallback);
                    return Ok(info);
                }
                Err(e) => {
                    let reason = diagnose_error(&e.to_string());
                    return Err(ExtractorResult {
                        error: e,
                        blocking_reason: reason,
                        used_mode: fallback,
                        tried_fallback: true,
                        suggestion: self.suggest_for_reason(&reason),
                    });
                }
            }
        }

        // Neither available
        Err(ExtractorResult {
            error: DownloadError::ToolNotFound("Neither Python yt_dlp nor yt-dlp binary available".to_string()),
            blocking_reason: None,
            used_mode: ExtractorMode::Auto,
            tried_fallback: false,
            suggestion: Some("Install yt-dlp: brew install yt-dlp OR pip3 install yt-dlp".to_string()),
        })
    }

    /// Generate suggestion based on blocking reason
    fn suggest_for_reason(&self, reason: &Option<BlockingReason>) -> Option<String> {
        match reason {
            Some(BlockingReason::Http403Forbidden) => Some(
                "YouTube returned 403 Forbidden. Try:\n\
                 1) Use a VPN/Proxy\n\
                 2) Update cookies (re-login to YouTube)\n\
                 3) Wait and try again later"
                    .to_string(),
            ),
            Some(BlockingReason::SabrStreaming) => Some(
                "YouTube is using SABR streaming protection. Try:\n\
                 1) Use Python mode with cookies\n\
                 2) Try audio-only download\n\
                 3) Use a proxy/VPN"
                    .to_string(),
            ),
            Some(BlockingReason::PoTokenRequired) => Some(
                "YouTube requires PO Token. Try:\n\
                 1) Use cookies from logged-in browser\n\
                 2) See: https://github.com/yt-dlp/yt-dlp/wiki/PO-Token-Guide"
                    .to_string(),
            ),
            Some(BlockingReason::AgeRestricted) => Some(
                "Video is age-restricted. Try:\n\
                 1) Use cookies from a logged-in account\n\
                 2) Enable 'Chrome (logged-in)' in Tools → Cookies"
                    .to_string(),
            ),
            Some(BlockingReason::GeoBlocked) => Some(
                "Video is not available in your country. Try:\n\
                 1) Use a VPN with a different country\n\
                 2) Use a proxy server in allowed region"
                    .to_string(),
            ),
            Some(BlockingReason::NetworkTimeout) => Some(
                "Network timeout. Try:\n\
                 1) Check your internet connection\n\
                 2) Use a proxy/VPN\n\
                 3) Try again later"
                    .to_string(),
            ),
            Some(BlockingReason::RateLimited) => Some(
                "YouTube is rate-limiting requests. Try:\n\
                 1) Wait 10-15 minutes\n\
                 2) Use a different IP (VPN/proxy)"
                    .to_string(),
            ),
            Some(BlockingReason::BotDetection) => Some(
                "YouTube detected automated access. Try:\n\
                 1) Use Python mode with cookies\n\
                 2) Use a fresh proxy/VPN"
                    .to_string(),
            ),
            Some(BlockingReason::PrivateVideo) => Some(
                "Video is private. You need:\n\
                 1) Cookies from an authorized account\n\
                 2) Access permission from the uploader"
                    .to_string(),
            ),
            Some(BlockingReason::VideoUnavailable) => Some(
                "Video is unavailable. It may have been:\n\
                 1) Deleted by the uploader\n\
                 2) Removed for copyright violation\n\
                 3) Made private"
                    .to_string(),
            ),
            Some(BlockingReason::Unknown) => Some(
                "Unknown error occurred. Try:\n\
                 1) Check the video URL\n\
                 2) Try again later\n\
                 3) Use a VPN/proxy"
                    .to_string(),
            ),
            None => None,
        }
    }
}

impl Default for InfoExtractorOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Status of the orchestrator
#[derive(Debug, Clone)]
pub struct OrchestratorStatus {
    pub python_available: bool,
    pub cli_available: bool,
    pub recommended_mode: ExtractorMode,
}

/// Result with diagnostic information
#[derive(Debug)]
pub struct ExtractorResult {
    pub error: DownloadError,
    pub blocking_reason: Option<BlockingReason>,
    pub used_mode: ExtractorMode,
    pub tried_fallback: bool,
    pub suggestion: Option<String>,
}

impl std::fmt::Display for ExtractorResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)?;

        if let Some(reason) = &self.blocking_reason {
            write!(f, "\n\nBlocking reason: {:?}", reason)?;
        }

        if let Some(suggestion) = &self.suggestion {
            write!(f, "\n\n{}", suggestion)?;
        }

        Ok(())
    }
}

