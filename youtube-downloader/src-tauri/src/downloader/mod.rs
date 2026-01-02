// Downloader module - production-grade abstraction layer

pub mod errors;
pub mod models;
pub mod traits;
pub mod backends;
pub mod orchestrator;

pub use errors::DownloadError;
pub use models::{VideoInfo, VideoFormat, DownloadOptions, DownloadProgress};
pub use traits::{DownloaderBackend, ProgressEmitter};
pub use orchestrator::Downloader;
