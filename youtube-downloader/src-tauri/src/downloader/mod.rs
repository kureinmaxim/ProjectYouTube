// Downloader module - production-grade abstraction layer

#[allow(dead_code)]
pub mod errors;
#[allow(dead_code)]
pub mod models;
#[allow(dead_code)]
pub mod traits;
#[allow(dead_code)]
pub mod backends;
#[allow(dead_code)]
pub mod orchestrator;
#[allow(dead_code)]
pub mod utils;

pub use errors::DownloadError;
pub use models::{VideoInfo, VideoFormat, DownloadOptions, DownloadProgress, NetworkConfig};
pub use traits::{DownloaderBackend, ProgressEmitter};
pub use orchestrator::Downloader;
