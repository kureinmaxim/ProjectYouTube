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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
