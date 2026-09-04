use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

use crate::downloader::platform;
use crate::downloader::utils::run_output_with_timeout;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ToolType {
    YtDlp,
    Ffmpeg,
}

impl ToolType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolType::YtDlp => "yt-dlp",
            ToolType::Ffmpeg => "ffmpeg",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        match name {
            "yt-dlp" => Some(ToolType::YtDlp),
            "ffmpeg" => Some(ToolType::Ffmpeg),
            _ => None,
        }
    }

    /// yt-dlp uses `--version`, ffmpeg uses a single dash.
    fn version_arg(&self) -> &'static str {
        match self {
            ToolType::YtDlp => "--version",
            ToolType::Ffmpeg => "-version",
        }
    }

    /// Why the user should care that this tool is missing.
    fn purpose(&self) -> &'static str {
        match self {
            ToolType::YtDlp => "downloading",
            ToolType::Ffmpeg => "merging video and audio above 720p",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub tool_type: ToolType,
    pub version: Option<String>,
    pub path: Option<String>,
    pub is_available: bool,
    pub last_updated: Option<String>, // ISO date string
}

pub struct ToolManager;

impl ToolManager {
    pub fn new() -> Self {
        Self
    }

    pub fn get_tool_info(&self, tool_type: ToolType) -> ToolInfo {
        let name = tool_type.as_str().to_string();
        let path = platform::resolve_tool(&name);
        let version = path.as_deref().and_then(|p| self.get_version(p, &tool_type));

        ToolInfo {
            name,
            tool_type,
            version,
            is_available: path.is_some(),
            path,
            last_updated: None, // TODO: Store/retrieve this from persistent config
        }
    }

    pub fn get_all_tools(&self) -> Vec<ToolInfo> {
        vec![
            self.get_tool_info(ToolType::YtDlp),
            self.get_tool_info(ToolType::Ffmpeg),
        ]
    }

    fn get_version(&self, path: &str, tool_type: &ToolType) -> Option<String> {
        let output = Command::new(path).arg(tool_type.version_arg()).output().ok()?;
        if !output.status.success() {
            return None;
        }
        let out = String::from_utf8_lossy(&output.stdout);
        match tool_type {
            ToolType::YtDlp => Some(out.trim().to_string()),
            // "ffmpeg version 7.1-full_build-www.gyan.dev Copyright ..." -> "7.1-full_build-..."
            ToolType::Ffmpeg => out
                .split_whitespace()
                .nth(2)
                .map(|v| v.to_string())
                .or_else(|| out.lines().next().map(|l| l.trim().to_string())),
        }
    }
}

#[tauri::command]
pub async fn get_tools_status() -> Result<Vec<ToolInfo>, String> {
    let manager = ToolManager::new();
    Ok(manager.get_all_tools())
}

// ============ Downloaded installs (Windows) ============

/// Upstream artifact for a tool we install ourselves.
#[cfg(windows)]
fn download_url(tool: ToolType) -> &'static str {
    match tool {
        ToolType::YtDlp => "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe",
        // BtbN publishes static win64 builds containing ffmpeg.exe and ffprobe.exe.
        ToolType::Ffmpeg => {
            "https://github.com/BtbN/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-win64-gpl.zip"
        }
    }
}

/// Progress event for a tool install, mirroring `download-progress` for videos.
#[derive(Debug, Clone, Serialize)]
pub struct ToolInstallProgress {
    pub tool: String,
    pub downloaded_mb: f64,
    pub total_mb: Option<f64>,
    pub percent: Option<f64>,
    pub status: String,
}

#[cfg(windows)]
fn emit_progress(app: Option<&tauri::AppHandle>, payload: ToolInstallProgress) {
    use tauri::Emitter;
    if let Some(handle) = app {
        let _ = handle.emit("tool-install-progress", payload);
    }
}

/// Stream a download, reporting progress as it goes.
///
/// The ffmpeg archive is ~160 MB; buffering it silently made the Install button
/// look frozen for minutes, so the body is streamed and progress emitted.
#[cfg(windows)]
async fn fetch_bytes(
    url: &str,
    tool: ToolType,
    app: Option<&tauri::AppHandle>,
) -> Result<Vec<u8>, String> {
    use futures_util::StreamExt;

    let client = reqwest::Client::builder()
        // No overall timeout: a slow link must not kill a large download
        // mid-way. The per-read timeout below still catches a dead connection.
        .connect_timeout(std::time::Duration::from_secs(30))
        .user_agent("youtube-downloader-tauri")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {}\nURL: {}", e, url))?;

    if !resp.status().is_success() {
        return Err(format!("Download failed: HTTP {}\nURL: {}", resp.status(), url));
    }

    let total = resp.content_length();
    let total_mb = total.map(|t| t as f64 / 1_048_576.0);
    let mut buf: Vec<u8> = Vec::with_capacity(total.unwrap_or(0) as usize);
    let mut stream = resp.bytes_stream();
    let mut last_emit = std::time::Instant::now();

    while let Some(chunk) = tokio::time::timeout(std::time::Duration::from_secs(120), stream.next())
        .await
        .map_err(|_| "Download stalled: no data for 120s.".to_string())?
    {
        let chunk = chunk.map_err(|e| format!("Download interrupted: {}", e))?;
        buf.extend_from_slice(&chunk);

        if last_emit.elapsed() >= std::time::Duration::from_millis(300) {
            let downloaded_mb = buf.len() as f64 / 1_048_576.0;
            let percent = total_mb.map(|t| (downloaded_mb / t * 100.0).min(100.0));
            emit_progress(
                app,
                ToolInstallProgress {
                    tool: tool.as_str().to_string(),
                    downloaded_mb,
                    total_mb,
                    percent,
                    status: match (percent, total_mb) {
                        (Some(p), Some(t)) => {
                            format!("⬇️ {} {:.0}% ({:.1}/{:.1} MB)", tool.as_str(), p, downloaded_mb, t)
                        }
                        _ => format!("⬇️ {} {:.1} MB", tool.as_str(), downloaded_mb),
                    },
                },
            );
            last_emit = std::time::Instant::now();
        }
    }

    Ok(buf)
}

#[cfg(windows)]
fn write_managed_binary(file_name: &str, bytes: &[u8]) -> Result<std::path::PathBuf, String> {
    let dir = platform::managed_bin_dir()
        .ok_or_else(|| "Could not determine the local application data directory.".to_string())?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create {}: {}", dir.display(), e))?;

    let dest = dir.join(file_name);
    // Windows refuses to overwrite a running executable; replacing via a temp
    // file keeps a failed write from destroying a working binary.
    let tmp = dir.join(format!("{}.download", file_name));
    std::fs::write(&tmp, bytes).map_err(|e| format!("Failed to write {}: {}", tmp.display(), e))?;
    if dest.exists() {
        let _ = std::fs::remove_file(&dest);
    }
    std::fs::rename(&tmp, &dest)
        .map_err(|e| format!("Failed to install {}: {}", dest.display(), e))?;

    Ok(dest)
}

/// Pull ffmpeg.exe / ffprobe.exe out of the release zip into our bin dir.
#[cfg(windows)]
fn extract_ffmpeg_zip(bytes: Vec<u8>) -> Result<Vec<std::path::PathBuf>, String> {
    let reader = std::io::Cursor::new(bytes);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| format!("Failed to open ffmpeg archive: {}", e))?;

    let wanted = ["ffmpeg.exe", "ffprobe.exe"];
    let mut installed = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;
        if !entry.is_file() {
            continue;
        }
        let entry_name = entry.name().replace('\\', "/");
        let base = match entry_name.rsplit('/').next() {
            Some(b) => b.to_string(),
            None => continue,
        };
        if !wanted.contains(&base.as_str()) {
            continue;
        }

        let mut buf = Vec::new();
        std::io::copy(&mut entry, &mut buf)
            .map_err(|e| format!("Failed to extract {}: {}", base, e))?;
        installed.push(write_managed_binary(&base, &buf)?);
    }

    if installed.is_empty() {
        return Err("The ffmpeg archive did not contain ffmpeg.exe.".to_string());
    }
    Ok(installed)
}

/// Install a tool by downloading it — no package manager required.
#[cfg(windows)]
async fn install_by_download(
    tool: ToolType,
    app: Option<&tauri::AppHandle>,
) -> Result<String, String> {
    let url = download_url(tool);
    let mut log = String::new();
    log.push_str(&format!("Downloading {} from:\n{}\n", tool.as_str(), url));

    let bytes = fetch_bytes(url, tool, app).await?;
    let size_mb = bytes.len() as f64 / 1_048_576.0;
    log.push_str(&format!("Downloaded {:.1} MB.\n", size_mb));

    emit_progress(
        app,
        ToolInstallProgress {
            tool: tool.as_str().to_string(),
            downloaded_mb: size_mb,
            total_mb: Some(size_mb),
            percent: Some(100.0),
            status: format!("📦 Unpacking {}...", tool.as_str()),
        },
    );

    let installed = match tool {
        ToolType::YtDlp => vec![write_managed_binary(&platform::exe_name("yt-dlp"), &bytes)?],
        ToolType::Ffmpeg => extract_ffmpeg_zip(bytes)?,
    };

    for path in &installed {
        log.push_str(&format!("Installed: {}\n", path.display()));
    }
    Ok(log)
}

// ============ Package-manager installs (macOS / Linux) ============

#[cfg(not(windows))]
fn brew_exists() -> bool {
    Path::new("/opt/homebrew/bin/brew").exists()
        || Path::new("/usr/local/bin/brew").exists()
        || platform::find_in_path("brew").is_some()
}

fn join_output(prefix: &str, output: &std::process::Output) -> String {
    let mut s = String::new();
    s.push_str(prefix);
    s.push('\n');
    let out = String::from_utf8_lossy(&output.stdout);
    let err = String::from_utf8_lossy(&output.stderr);
    if !out.trim().is_empty() {
        s.push_str(&out);
        if !out.ends_with('\n') {
            s.push('\n');
        }
    }
    if !err.trim().is_empty() {
        s.push_str(&err);
        if !err.ends_with('\n') {
            s.push('\n');
        }
    }
    s
}

#[cfg(not(windows))]
async fn install_via_brew(tool: ToolType) -> Result<String, String> {
    if !brew_exists() {
        return Err(format!(
            "Homebrew (brew) was not found.\n\
Install Homebrew first, then retry.\n\
Hint: see https://brew.sh/\n\
Or install {} manually and restart the app.\n",
            tool.as_str()
        ));
    }

    let name = tool.as_str().to_string();
    let out = match run_output_with_timeout("brew", vec!["install".into(), name.clone()], 900).await
    {
        Ok(o) => o,
        Err(_) => {
            // If already installed, brew install may fail; try upgrade.
            run_output_with_timeout("brew", vec!["upgrade".into(), name.clone()], 900)
                .await
                .map_err(|e| format!("brew failed: {}", e))?
        }
    };

    Ok(join_output(&format!("brew install/upgrade {}:", name), &out))
}

/// Refuse to update a copy we did not install, explaining who owns it.
///
/// Downloading over a Scoop/choco/winget install would silently shadow it with
/// a ~170 MB copy the user never asked for. Returns `None` when the tool is
/// ours to update.
#[cfg(windows)]
fn foreign_install_refusal(tool: ToolType, current_path: &str) -> Option<String> {
    let source = platform::classify(current_path);
    if source == platform::ToolSource::Managed {
        return None;
    }

    let mut msg = format!(
        "{} was installed by {} ({}).\n",
        tool.as_str(),
        source.label(),
        current_path
    );
    match source.update_hint(tool.as_str()) {
        Some(cmd) => msg.push_str(&format!("Update it with:\n    {}\n", cmd)),
        None => msg.push_str("Update it the same way it was installed.\n"),
    }
    msg.push_str(
        "\nThe app will not download over a copy it did not install.\n\
To let the app manage this tool instead, remove that copy and click Install.\n",
    );
    Some(msg)
}

// ============ Commands ============

#[tauri::command]
pub async fn install_tool(
    tool_type: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let _ = &app;
    let tool = ToolType::from_name(&tool_type)
        .ok_or_else(|| "Unknown or unsupported tool type".to_string())?;

    let manager = ToolManager::new();
    if manager.get_tool_info(tool).is_available {
        return Ok(format!("{} is already installed.", tool.as_str()));
    }

    #[cfg(windows)]
    let mut log = install_by_download(tool, Some(&app)).await?;
    #[cfg(not(windows))]
    let mut log = install_via_brew(tool).await?;

    let refreshed = ToolManager::new().get_tool_info(tool);
    if refreshed.is_available {
        log.push_str(&format!(
            "\n✅ {} is ready ({}).\n",
            tool.as_str(),
            refreshed.version.as_deref().unwrap_or("version unknown")
        ));
        Ok(log)
    } else {
        log.push_str(
            "\n⚠️ Install finished, but the tool was still not detected.\n\
Try restarting the app, or check PATH / location.\n",
        );
        Ok(log)
    }
}

#[tauri::command]
pub async fn update_tool(tool_type: String, app: tauri::AppHandle) -> Result<String, String> {
    let _ = &app;
    let tool = ToolType::from_name(&tool_type)
        .ok_or_else(|| "Unknown or unsupported tool type".to_string())?;

    let manager = ToolManager::new();
    let info = manager.get_tool_info(tool);
    if !info.is_available {
        return Err(format!(
            "{} is not installed yet — use Install first (needed for {}).",
            tool.as_str(),
            tool.purpose()
        ));
    }

    // Windows: only ever update our own copy. Downloading over a Scoop/choco/
    // winget install would silently shadow it — a ~170 MB surprise the user
    // never asked for — so hand back the right command instead.
    #[cfg(windows)]
    {
        if let Some(refusal) = foreign_install_refusal(tool, info.path.as_deref().unwrap_or_default())
        {
            return Err(refusal);
        }

        let log = install_by_download(tool, Some(&app)).await?;
        let refreshed = ToolManager::new().get_tool_info(tool);
        return Ok(format!(
            "{}\n✅ {} updated ({}).\n",
            log,
            tool.as_str(),
            refreshed.version.as_deref().unwrap_or("version unknown")
        ));
    }

    #[cfg(not(windows))]
    {
        let name = tool.as_str().to_string();
        let (program, args): (&str, Vec<String>) = if brew_exists() {
            ("brew", vec!["upgrade".into(), name.clone()])
        } else if tool == ToolType::YtDlp {
            let pip = if platform::find_in_path("pip3").is_some() {
                "pip3"
            } else {
                "pip"
            };
            (pip, vec!["install".into(), "-U".into(), name.clone()])
        } else {
            return Err(
                "Homebrew (brew) was not found — install ffmpeg with your package manager."
                    .to_string(),
            );
        };

        let output = run_output_with_timeout(program, args, 900)
            .await
            .map_err(|e| format!("Failed to run update command: {}", e))?;

        if output.status.success() {
            Ok(join_output(&format!("{} update {}:", program, name), &output))
        } else {
            Err(format!(
                "Update failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_tools_are_reported() {
        let tools = ToolManager::new().get_all_tools();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"yt-dlp"), "got {:?}", names);
        assert!(names.contains(&"ffmpeg"), "got {:?}", names);
    }

    #[test]
    fn unknown_tool_is_rejected() {
        assert!(ToolType::from_name("lux").is_none());
        assert!(ToolType::from_name("yt-dlp").is_some());
        assert!(ToolType::from_name("ffmpeg").is_some());
    }

    /// The reported bug: clicking Install on Windows demanded Homebrew, which
    /// cannot exist there, so the button could never succeed.
    #[cfg(windows)]
    #[test]
    fn windows_install_never_asks_for_homebrew() {
        for tool in [ToolType::YtDlp, ToolType::Ffmpeg] {
            let url = download_url(tool);
            assert!(
                url.starts_with("https://github.com/"),
                "{} should install from a direct download, got {}",
                tool.as_str(),
                url
            );
        }
    }

    /// End-to-end install against the real network. Ignored by default; run with:
    ///   cargo test --lib real_install -- --ignored --nocapture --test-threads=1
    #[cfg(windows)]
    #[tokio::test]
    #[ignore]
    async fn real_install_makes_tools_available() {
        // Goes through install_by_download rather than the command wrapper: the
        // wrapper short-circuits on an already-present system copy, which would
        // leave the download path untested on a machine that has the tool.
        for tool in [ToolType::YtDlp, ToolType::Ffmpeg] {
            let log = install_by_download(tool, None)
                .await
                .unwrap_or_else(|e| panic!("install {} failed: {}", tool.as_str(), e));
            println!("--- {} ---\n{}", tool.as_str(), log);

            let info = ToolManager::new().get_tool_info(tool);
            assert!(info.is_available, "{} still not detected", tool.as_str());
            assert!(info.version.is_some(), "{} reported no version", tool.as_str());
        }
    }

    /// Exercises the zip path specifically — `install_tool` short-circuits when
    /// a system ffmpeg is already present, which would leave this untested.
    #[cfg(windows)]
    #[tokio::test]
    #[ignore]
    async fn real_ffmpeg_archive_yields_binaries() {
        let log = install_by_download(ToolType::Ffmpeg, None)
            .await
            .unwrap_or_else(|e| panic!("ffmpeg download failed: {}", e));
        println!("{}", log);

        let dir = platform::managed_bin_dir().expect("data dir");
        for name in ["ffmpeg.exe", "ffprobe.exe"] {
            assert!(dir.join(name).is_file(), "{} was not extracted", name);
        }

        let out = Command::new(dir.join("ffmpeg.exe"))
            .arg("-version")
            .output()
            .expect("extracted ffmpeg should run");
        assert!(out.status.success(), "extracted ffmpeg did not execute cleanly");
    }

    /// Update must leave a Scoop/choco/winget install alone and say so.
    #[cfg(windows)]
    #[test]
    fn update_refuses_to_shadow_a_system_copy() {
        let scoop = r"C:\Users\me\scoop\shims\ffmpeg.exe";
        let refusal = foreign_install_refusal(ToolType::Ffmpeg, scoop)
            .expect("a Scoop copy must not be overwritten");
        assert!(refusal.contains("Scoop"), "should name the owner: {}", refusal);
        assert!(refusal.contains("scoop update ffmpeg"), "should give the command: {}", refusal);
        assert!(refusal.contains(scoop), "should name the path: {}", refusal);

        let choco = r"C:\ProgramData\chocolatey\bin\yt-dlp.exe";
        let refusal = foreign_install_refusal(ToolType::YtDlp, choco).expect("choco copy");
        assert!(refusal.contains("choco upgrade yt-dlp"), "{}", refusal);
    }

    /// ...but our own copy stays updatable.
    #[cfg(windows)]
    #[test]
    fn update_proceeds_for_our_own_copy() {
        let ours = platform::managed_bin_dir()
            .expect("data dir")
            .join(platform::exe_name("yt-dlp"));
        assert!(
            foreign_install_refusal(ToolType::YtDlp, &ours.to_string_lossy()).is_none(),
            "the app must still update what it installed"
        );
    }

    #[cfg(windows)]
    #[test]
    fn managed_install_dir_is_writable() {
        let dir = platform::managed_bin_dir().expect("data dir");
        std::fs::create_dir_all(&dir).expect("bin dir should be creatable without admin rights");
        assert!(dir.is_dir());
    }
}


