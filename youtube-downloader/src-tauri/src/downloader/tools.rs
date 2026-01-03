use serde::{Deserialize, Serialize};
use std::process::Command;
use tokio::io::AsyncReadExt;
use tokio::process::Command as TokioCommand;
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolType {
    YtDlp,
    Lux,
    YouGet,
}

impl ToolType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolType::YtDlp => "yt-dlp",
            ToolType::Lux => "lux",
            ToolType::YouGet => "you-get",
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
        let (path, version) = self.detect_tool(&tool_type);
        
        ToolInfo {
            name,
            tool_type,
            version: version.clone(),
            path: path.clone(),
            is_available: path.is_some(),
            last_updated: None, // TODO: Store/retrieve this from persistent config
        }
    }

    pub fn get_all_tools(&self) -> Vec<ToolInfo> {
        vec![
            self.get_tool_info(ToolType::YtDlp),
            self.get_tool_info(ToolType::Lux),
            self.get_tool_info(ToolType::YouGet),
        ]
    }

    fn detect_tool(&self, tool_type: &ToolType) -> (Option<String>, Option<String>) {
        let binary_name = match tool_type {
            ToolType::YtDlp => "yt-dlp",
            ToolType::Lux => "lux",
            ToolType::YouGet => "you-get",
        };

        // 1. Try common paths first
        let home = dirs::home_dir().unwrap_or_default();
        let mut common_paths = vec![
            format!("/opt/homebrew/bin/{}", binary_name),
            format!("/usr/local/bin/{}", binary_name),
            format!("/usr/bin/{}", binary_name),
        ];
        // User-level install locations (pipx, cargo, etc.)
        if let Some(home_str) = home.to_str() {
            common_paths.push(format!("{}/.local/bin/{}", home_str, binary_name)); // pipx default
            common_paths.push(format!("{}/.cargo/bin/{}", home_str, binary_name)); // cargo install
        }

        for path in common_paths {
            if std::path::Path::new(&path).exists() {
                let version = self.get_version(&path, tool_type);
                return (Some(path), version);
            }
        }

        // 2. Try PATH
        if let Ok(output) = Command::new("which").arg(binary_name).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let version = self.get_version(&path, tool_type);
                return (Some(path), version);
            }
        }

        (None, None)
    }

    fn get_version(&self, path: &str, tool_type: &ToolType) -> Option<String> {
        let arg = match tool_type {
            ToolType::YtDlp => "--version",
            ToolType::Lux => "-v", // lux uses -v
            ToolType::YouGet => "--version",
        };

        match Command::new(path).arg(arg).output() {
            Ok(output) if output.status.success() => {
                let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
                // Clean up version string if needed (some tools output extra text)
                Some(out)
            },
            _ => None,
        }
    }
}

#[tauri::command]
pub async fn get_tools_status() -> Result<Vec<ToolInfo>, String> {
    let manager = ToolManager::new();
    Ok(manager.get_all_tools())
}

async fn run_output_with_timeout(
    program: &str,
    args: Vec<String>,
    timeout_secs: u64,
) -> Result<std::process::Output, String> {
    let mut child = TokioCommand::new(program)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start {}: {}", program, e))?;

    let mut stdout_pipe = child
        .stdout
        .take()
        .ok_or_else(|| format!("Failed to capture stdout from {}", program))?;
    let mut stderr_pipe = child
        .stderr
        .take()
        .ok_or_else(|| format!("Failed to capture stderr from {}", program))?;

    let stdout_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        stdout_pipe
            .read_to_end(&mut buf)
            .await
            .map_err(|e| format!("Failed to read stdout: {}", e))?;
        Ok::<Vec<u8>, String>(buf)
    });
    let stderr_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        stderr_pipe
            .read_to_end(&mut buf)
            .await
            .map_err(|e| format!("Failed to read stderr: {}", e))?;
        Ok::<Vec<u8>, String>(buf)
    });

    let waited = timeout(Duration::from_secs(timeout_secs), child.wait()).await;
    match waited {
        Ok(status_res) => {
            let status = status_res.map_err(|e| format!("Failed to wait for {}: {}", program, e))?;
            let stdout = stdout_task
                .await
                .map_err(|e| format!("stdout task failed: {}", e))??;
            let stderr = stderr_task
                .await
                .map_err(|e| format!("stderr task failed: {}", e))??;
            Ok(std::process::Output { status, stdout, stderr })
        }
        Err(_) => {
            let _ = child.kill().await;
            stdout_task.abort();
            stderr_task.abort();
            Err(format!("Timed out after {}s", timeout_secs))
        }
    }
}

fn brew_exists() -> bool {
    std::path::Path::new("/opt/homebrew/bin/brew").exists()
        || std::path::Path::new("/usr/local/bin/brew").exists()
        || Command::new("which").arg("brew").output().map(|o| o.status.success()).unwrap_or(false)
}

fn pipx_exists() -> bool {
    std::path::Path::new("/opt/homebrew/bin/pipx").exists()
        || std::path::Path::new("/usr/local/bin/pipx").exists()
        || Command::new("which").arg("pipx").output().map(|o| o.status.success()).unwrap_or(false)
}

fn join_output(prefix: &str, output: &std::process::Output) -> String {
    let mut s = String::new();
    s.push_str(prefix);
    s.push('\n');
    let out = String::from_utf8_lossy(&output.stdout);
    let err = String::from_utf8_lossy(&output.stderr);
    if !out.trim().is_empty() {
        s.push_str(&out);
        if !out.ends_with('\n') { s.push('\n'); }
    }
    if !err.trim().is_empty() {
        s.push_str(&err);
        if !err.ends_with('\n') { s.push('\n'); }
    }
    s
}

#[tauri::command]
pub async fn update_tool(tool_type: String) -> Result<String, String> {
    // Basic update implementation (can be expanded)
    let tool_enum = match tool_type.as_str() {
        "yt-dlp" => ToolType::YtDlp,
        "lux" => ToolType::Lux,
        "you-get" => ToolType::YouGet,
        _ => return Err("Unknown tool type".to_string()),
    };

    let manager = ToolManager::new();
    let info = manager.get_tool_info(tool_enum.clone());

    if !info.is_available {
         return Err("Tool not installed. Please install it manually first.".to_string());
    }

    // Try to determine update command based on system
    // This is a naive implementation; improved logic would detect how it was installed (brew, pip, etc.)
    let update_cmd = match tool_enum {
        ToolType::YtDlp => {
             if std::path::Path::new("/opt/homebrew/bin/brew").exists() {
                 ("brew", vec!["upgrade", "yt-dlp"])
             } else {
                 ("pip3", vec!["install", "-U", "yt-dlp"])
             }
        },
        ToolType::Lux => {
            if std::path::Path::new("/opt/homebrew/bin/brew").exists() {
                 ("brew", vec!["upgrade", "annie"]) // lux is often 'annie' in brew
             } else {
                 ("go", vec!["get", "-u", "github.com/iawia002/lux"])
             }
        },
        ToolType::YouGet => {
            ("pip3", vec!["install", "-U", "you-get"])
        },
    };

    let output = Command::new(update_cmd.0)
        .args(update_cmd.1)
        .output()
        .map_err(|e| format!("Failed to run update command: {}", e))?;

    if output.status.success() {
        Ok("Tool updated successfully".to_string())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("Update failed: {}", error))
    }
}

#[tauri::command]
pub async fn install_tool(tool_type: String) -> Result<String, String> {
    let tool_enum = match tool_type.as_str() {
        "yt-dlp" => ToolType::YtDlp,
        "lux" => ToolType::Lux,
        "you-get" => ToolType::YouGet,
        _ => return Err("Unknown tool type".to_string()),
    };

    let manager = ToolManager::new();
    let info = manager.get_tool_info(tool_enum.clone());
    if info.is_available {
        return Ok(format!("{} is already installed.", tool_type));
    }

    // Currently we support macOS best-effort installs via brew/pipx.
    // If brew is missing, we provide a clear hint.
    if !brew_exists() {
        return Err(
            "Homebrew (brew) was not found.\n\
Install Homebrew first, then retry.\n\
Hint: see https://brew.sh/\n"
                .to_string(),
        );
    }

    let mut log = String::new();

    match tool_enum {
        ToolType::YtDlp => {
            let out = match run_output_with_timeout("brew", vec!["install".into(), "yt-dlp".into()], 600).await {
                Ok(o) => o,
                Err(_) => {
                    // If already installed, brew install may fail; try upgrade.
                    run_output_with_timeout("brew", vec!["upgrade".into(), "yt-dlp".into()], 600)
                        .await
                        .map_err(|e| format!("brew failed: {}", e))?
                }
            };
            log.push_str(&join_output("brew install/upgrade yt-dlp:", &out));
        }
        ToolType::Lux => {
            // Try modern name first, then legacy brew formula name.
            let out1 = run_output_with_timeout("brew", vec!["install".into(), "lux".into()], 600).await;
            match out1 {
                Ok(out) => log.push_str(&join_output("brew install lux:", &out)),
                Err(_) => {
                    let out = match run_output_with_timeout("brew", vec!["install".into(), "annie".into()], 600).await {
                        Ok(o) => o,
                        Err(_) => run_output_with_timeout("brew", vec!["upgrade".into(), "annie".into()], 600)
                            .await
                            .map_err(|e| format!("brew failed: {}", e))?,
                    };
                    log.push_str(&join_output("brew install/upgrade annie (lux):", &out));
                }
            }
        }
        ToolType::YouGet => {
            // Use pipx to avoid macOS/Homebrew system-python restrictions.
            if !pipx_exists() {
                let out = run_output_with_timeout("brew", vec!["install".into(), "pipx".into()], 600).await
                    .map_err(|e| format!("Failed to install pipx via brew: {}", e))?;
                log.push_str(&join_output("brew install pipx:", &out));
            }

            // Ensure pipx paths are set (may require user re-login / terminal restart)
            let _ = run_output_with_timeout("pipx", vec!["ensurepath".into()], 120).await;

            let out = match run_output_with_timeout("pipx", vec!["install".into(), "you-get".into()], 600).await {
                Ok(o) => o,
                Err(_) => run_output_with_timeout("pipx", vec!["upgrade".into(), "you-get".into()], 600)
                    .await
                    .map_err(|e| format!("pipx failed: {}", e))?,
            };
            log.push_str(&join_output("pipx install/upgrade you-get:", &out));
        }
    }

    // Re-check and return friendly result
    let refreshed = ToolManager::new().get_tool_info(tool_enum);
    if refreshed.is_available {
        log.push_str("\n✅ Installed successfully. If the app still shows 'Not found', restart the app.\n");
        Ok(log)
    } else {
        log.push_str("\n⚠️ Install command finished, but tool was not detected.\n\
Try restarting the app, or check PATH / location.\n");
        Ok(log)
    }
}
