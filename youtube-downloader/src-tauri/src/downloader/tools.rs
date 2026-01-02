use serde::{Deserialize, Serialize};
use std::process::Command;

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
        let common_paths = [
            format!("/opt/homebrew/bin/{}", binary_name),
            format!("/usr/local/bin/{}", binary_name),
            format!("/usr/bin/{}", binary_name),
        ];

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
