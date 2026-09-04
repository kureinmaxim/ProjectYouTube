mod downloader;
mod ytdlp;

use ytdlp::{get_video_info, download_video, get_formats};
use downloader::tools::{get_tools_status, update_tool, install_tool};
use downloader::utils::{NetworkStatus, get_network_status_info};

/// Get network status (proxy, mode, external IP) for UI display
#[tauri::command]
async fn get_network_status(user_proxy: Option<String>) -> Result<NetworkStatus, String> {
    Ok(get_network_status_info(user_proxy).await)
}

/// Default download folder for this machine.
#[tauri::command]
async fn default_output_dir() -> Result<String, String> {
    downloader::platform::default_output_dir()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "Could not determine a downloads folder".to_string())
}

/// Verify a folder exists and can be written to before a download starts.
#[tauri::command]
async fn check_output_dir(path: String) -> Result<(), String> {
    downloader::platform::check_writable_dir(&path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_video_info,
            download_video,
            get_formats,
            get_tools_status,
            update_tool,
            install_tool,
            get_network_status,
            default_output_dir,
            check_output_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
