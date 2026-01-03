// InfoExtractor module - production-grade video info extraction
//
// Provides two modes:
// - Python mode: Uses `python3 -m yt_dlp` (better for YouTube, avoids bot detection)
// - CLI mode: Uses native `yt-dlp` binary (faster, no Python dependency)
//
// The Orchestrator automatically switches between modes based on:
// - Service type (YouTube prefers Python)
// - Availability of Python/yt_dlp module
// - Previous failure (auto-fallback)
//
// NOTE: This module is prepared for future integration.
// Currently the app uses ytdlp.rs directly.

#![allow(dead_code)]
#![allow(unused_imports)]

mod traits;
mod python;
mod cli;
mod orchestrator;
mod diagnostics;

pub use traits::{InfoExtractor, ExtractorConfig, ExtractorMode, ExtendedFormat};
pub use python::PythonInfoExtractor;
pub use cli::CliInfoExtractor;
pub use orchestrator::InfoExtractorOrchestrator;
pub use diagnostics::{BlockingReason, BlockingDiagnostics, diagnose_error};

