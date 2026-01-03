// Downloader module - production-grade abstraction layer
//
// Architecture (2025):
// - extractors/: InfoExtractor trait with Python and CLI modes
// - format_selector: Unified format selection logic
// - backends/: Download backends (yt-dlp, lux, you-get)
// - orchestrator: Download orchestration with fallback

#![allow(dead_code)]
#![allow(unused_imports)]

// Core modules
pub mod errors;
pub mod models;
pub mod traits;
pub mod utils;
pub mod tools;

// New architecture (2025) - will be integrated in future versions
pub mod extractors;
pub mod format_selector;

// Legacy modules (still used, will be refactored)
pub mod backends;
pub mod orchestrator;

// ============ Public API ============

// Errors
pub use errors::DownloadError;

// Models
pub use models::{DownloadOptions, DownloadProgress, NetworkConfig, VideoFormat, VideoInfo};

// Traits
pub use traits::{DownloaderBackend, ProgressEmitter};

// InfoExtractor (new - v1.2.0)
pub use extractors::{
    BlockingDiagnostics, BlockingReason, CliInfoExtractor, ExtendedFormat, ExtractorConfig,
    ExtractorMode, InfoExtractor, InfoExtractorOrchestrator, PythonInfoExtractor,
};

// FormatSelector (new - v1.2.0)
pub use format_selector::{FormatSelector, QualityOption};

// Legacy (will be deprecated)
pub use orchestrator::Downloader;
