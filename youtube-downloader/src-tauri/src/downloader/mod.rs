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
#[allow(dead_code)]
pub mod tools; // Added tools module

// Public API (will be used when new architecture is fully integrated)
#[allow(unused_imports)]
pub use errors::DownloadError;
#[allow(unused_imports)]
pub use models::{VideoInfo, VideoFormat, DownloadOptions, DownloadProgress, NetworkConfig};
#[allow(unused_imports)]
pub use traits::{DownloaderBackend, ProgressEmitter};
#[allow(unused_imports)]
pub use orchestrator::Downloader;
