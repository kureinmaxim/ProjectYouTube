// Platform abstraction for locating and storing external tools.
//
// Before this module every "find yt-dlp" site hardcoded macOS layout:
// /opt/homebrew/bin, /usr/local/bin, no .exe suffix, and `which` for PATH
// lookup. On Windows `which` does not exist (it only appears if Git for
// Windows happens to be installed), so PATH lookup failed even when the
// binary was present. Everything platform-specific now lives here.

use std::path::{Path, PathBuf};

/// Executable file name for a tool on the current platform.
pub fn exe_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{}.exe", base)
    } else {
        base.to_string()
    }
}

/// Directory holding binaries the app installed for itself.
///
/// Checked before anything else, so a tool we just downloaded is visible
/// immediately — no PATH edit and no app restart.
pub fn managed_bin_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join("youtube-downloader").join("bin"))
}

/// Path to a managed binary, if we have installed it.
pub fn managed_bin(base: &str) -> Option<PathBuf> {
    let path = managed_bin_dir()?.join(exe_name(base));
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// Well-known install locations, per platform.
pub fn common_paths(base: &str) -> Vec<PathBuf> {
    let exe = exe_name(base);
    let mut paths = Vec::new();
    let home = dirs::home_dir();

    if cfg!(windows) {
        // winget shims
        if let Some(local) = dirs::data_local_dir() {
            paths.push(local.join("Microsoft").join("WindowsApps").join(&exe));
        }
        // Chocolatey
        if let Ok(program_data) = std::env::var("ProgramData") {
            paths.push(Path::new(&program_data).join("chocolatey").join("bin").join(&exe));
        }
        // Scoop
        if let Some(ref h) = home {
            paths.push(h.join("scoop").join("shims").join(&exe));
        }
        // WINDOWS_SETUP.md tells users to drop yt-dlp.exe here
        if let Ok(windir) = std::env::var("SystemRoot") {
            paths.push(Path::new(&windir).join(&exe));
        }
        // pip --user install target
        if let Some(local) = dirs::data_local_dir() {
            paths.push(local.join("Programs").join("Python").join("Scripts").join(&exe));
        }
    } else {
        paths.push(PathBuf::from(format!("/opt/homebrew/bin/{}", exe))); // Apple Silicon
        paths.push(PathBuf::from(format!("/usr/local/bin/{}", exe))); // Intel Mac
        paths.push(PathBuf::from(format!("/usr/bin/{}", exe)));
        if let Some(ref h) = home {
            paths.push(h.join(".local").join("bin").join(&exe)); // pipx
            paths.push(h.join(".cargo").join("bin").join(&exe));
        }
    }

    paths
}

/// Search PATH for an executable, in pure Rust.
///
/// Deliberately does not shell out to `which`/`where`: `which` is absent on a
/// clean Windows box, and that was the reason PATH lookup silently failed.
pub fn find_in_path(base: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;

    // On Windows an entry may be listed without its extension; PATHEXT says
    // which suffixes to try. Elsewhere the name is used verbatim.
    let candidates: Vec<String> = if cfg!(windows) {
        let pathext = std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string());
        let mut names = vec![exe_name(base)];
        for ext in pathext.split(';').filter(|e| !e.is_empty()) {
            let candidate = format!("{}{}", base, ext.to_lowercase());
            if !names.contains(&candidate) {
                names.push(candidate);
            }
        }
        names
    } else {
        vec![base.to_string()]
    };

    for dir in std::env::split_paths(&path_var) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        for name in &candidates {
            let full = dir.join(name);
            if full.is_file() {
                return Some(full);
            }
        }
    }

    None
}

/// Where a resolved tool came from — decides who is responsible for updating it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolSource {
    /// Installed by this app, into `managed_bin_dir()`.
    Managed,
    Scoop,
    Chocolatey,
    Winget,
    Pip,
    Homebrew,
    /// Somewhere on PATH we cannot attribute.
    Unknown,
}

impl ToolSource {
    /// Command that updates a tool from this source, if there is an obvious one.
    pub fn update_hint(&self, tool: &str) -> Option<String> {
        match self {
            ToolSource::Managed => None,
            ToolSource::Scoop => Some(format!("scoop update {}", tool)),
            ToolSource::Chocolatey => Some(format!("choco upgrade {}", tool)),
            ToolSource::Winget => Some(format!("winget upgrade {}", tool)),
            ToolSource::Pip => Some(format!("pip install -U {}", tool)),
            ToolSource::Homebrew => Some(format!("brew upgrade {}", tool)),
            ToolSource::Unknown => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ToolSource::Managed => "this app",
            ToolSource::Scoop => "Scoop",
            ToolSource::Chocolatey => "Chocolatey",
            ToolSource::Winget => "winget",
            ToolSource::Pip => "pip",
            ToolSource::Homebrew => "Homebrew",
            ToolSource::Unknown => "an unknown location",
        }
    }
}

/// True when the path lives in the directory this app installs into.
pub fn is_managed(path: &str) -> bool {
    match managed_bin_dir() {
        Some(dir) => Path::new(path).starts_with(dir),
        None => false,
    }
}

/// Attribute a resolved tool path to whatever installed it.
pub fn classify(path: &str) -> ToolSource {
    if is_managed(path) {
        return ToolSource::Managed;
    }

    // Compare case-insensitively with forward slashes: Windows paths vary in
    // both, and a miss here would wrongly claim we may overwrite the file.
    let normalized = path.replace('\\', "/").to_lowercase();
    let has = |needle: &str| normalized.contains(needle);

    if has("/scoop/") {
        ToolSource::Scoop
    } else if has("/chocolatey/") {
        ToolSource::Chocolatey
    } else if has("/windowsapps/") {
        ToolSource::Winget
    } else if has("/site-packages/") || has("/python") || has("/.local/bin/") {
        ToolSource::Pip
    } else if has("/homebrew/") || has("/usr/local/bin/") {
        ToolSource::Homebrew
    } else {
        ToolSource::Unknown
    }
}

/// Resolve a tool: our own install dir, then well-known locations, then PATH.
pub fn resolve_tool(base: &str) -> Option<String> {
    if let Some(p) = managed_bin(base) {
        return Some(p.to_string_lossy().to_string());
    }
    for path in common_paths(base) {
        if path.is_file() {
            return Some(path.to_string_lossy().to_string());
        }
    }
    find_in_path(base).map(|p| p.to_string_lossy().to_string())
}

/// Resolve a tool, falling back to the bare name so the OS can try PATH itself.
pub fn resolve_tool_or_bare(base: &str) -> String {
    resolve_tool(base).unwrap_or_else(|| base.to_string())
}

/// Where downloads should land by default on this machine.
///
/// The frontend used to hardcode a developer's macOS home path, so every
/// Windows install defaulted to a folder that cannot exist there.
pub fn default_output_dir() -> Option<PathBuf> {
    dirs::download_dir().or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
}

/// Check that a directory exists and can actually be written to.
///
/// Existence alone is not enough: on Windows `C:\Users` exists but creating a
/// folder inside it needs elevation, which is how an invalid saved path burned
/// every download strategy before failing with an unrelated diagnosis.
pub fn check_writable_dir(path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("No output folder is set.".to_string());
    }
    let dir = Path::new(path);
    if !dir.exists() {
        return Err(format!("Output folder does not exist: {}", path));
    }
    if !dir.is_dir() {
        return Err(format!("Output path is not a folder: {}", path));
    }

    let probe = dir.join(".youtube-downloader-write-test");
    match std::fs::write(&probe, b"") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            Ok(())
        }
        Err(e) => Err(format!("Output folder is not writable: {} ({})", path, e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exe_name_carries_platform_suffix() {
        if cfg!(windows) {
            assert_eq!(exe_name("yt-dlp"), "yt-dlp.exe");
        } else {
            assert_eq!(exe_name("yt-dlp"), "yt-dlp");
        }
    }

    #[test]
    fn common_paths_do_not_leak_across_platforms() {
        let paths = common_paths("yt-dlp");
        assert!(!paths.is_empty(), "every platform needs candidate locations");

        let joined = paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("|");

        if cfg!(windows) {
            assert!(
                !joined.contains("/opt/homebrew"),
                "Windows must not search macOS Homebrew paths: {}",
                joined
            );
            assert!(
                joined.contains("yt-dlp.exe"),
                "Windows candidates need the .exe suffix: {}",
                joined
            );
        } else {
            assert!(
                joined.contains("/opt/homebrew"),
                "macOS should still search Homebrew: {}",
                joined
            );
        }
    }

    #[test]
    fn find_in_path_works_without_which() {
        // A binary guaranteed to be on PATH on each platform. This is the
        // regression guard: it must resolve through our own PATH walk, with no
        // `which`/`where` process involved.
        let probe = if cfg!(windows) { "cmd" } else { "sh" };
        let found = find_in_path(probe);
        assert!(found.is_some(), "{} should be locatable on PATH", probe);
        assert!(found.unwrap().is_file());
    }

    #[test]
    fn missing_tool_resolves_to_none() {
        assert!(find_in_path("definitely-not-a-real-binary-xyzzy").is_none());
        assert!(resolve_tool("definitely-not-a-real-binary-xyzzy").is_none());
    }

    #[test]
    fn managed_paths_are_recognised() {
        let dir = managed_bin_dir().expect("data dir");
        let ours = dir.join(exe_name("yt-dlp"));
        assert!(is_managed(&ours.to_string_lossy()));
        assert_eq!(classify(&ours.to_string_lossy()), ToolSource::Managed);
    }

    #[test]
    fn foreign_installs_are_attributed_not_claimed() {
        // The regression: a Scoop/choco copy must never be treated as ours,
        // or Update would download ~170 MB over a working system install.
        let cases = [
            (r"C:\Users\me\scoop\shims\ffmpeg.exe", ToolSource::Scoop),
            (r"C:\ProgramData\chocolatey\bin\yt-dlp.exe", ToolSource::Chocolatey),
            (
                r"C:\Users\me\AppData\Local\Microsoft\WindowsApps\yt-dlp.exe",
                ToolSource::Winget,
            ),
            ("/opt/homebrew/bin/yt-dlp", ToolSource::Homebrew),
            ("/home/me/.local/bin/yt-dlp", ToolSource::Pip),
        ];

        for (path, expected) in cases {
            assert!(!is_managed(path), "{} must not count as app-managed", path);
            assert_eq!(classify(path), expected, "misattributed {}", path);
        }
    }

    #[test]
    fn every_known_source_offers_a_command() {
        for source in [
            ToolSource::Scoop,
            ToolSource::Chocolatey,
            ToolSource::Winget,
            ToolSource::Pip,
            ToolSource::Homebrew,
        ] {
            let hint = source.update_hint("yt-dlp").expect("known source needs a hint");
            assert!(hint.contains("yt-dlp"), "hint should name the tool: {}", hint);
        }
        assert!(ToolSource::Managed.update_hint("yt-dlp").is_none());
    }

    #[test]
    fn managed_dir_is_under_local_app_data() {
        let dir = managed_bin_dir().expect("a data dir should exist");
        assert!(dir.ends_with(Path::new("youtube-downloader").join("bin")));
    }

    #[test]
    fn default_output_dir_is_real() {
        let dir = default_output_dir().expect("every platform has a home");
        assert!(dir.is_absolute(), "got {:?}", dir);
        assert!(
            !dir.to_string_lossy().contains("olgazaharova"),
            "must not be a hardcoded developer path: {:?}",
            dir
        );
    }

    #[test]
    fn writable_check_accepts_a_real_dir_and_rejects_junk() {
        let tmp = std::env::temp_dir();
        assert!(check_writable_dir(&tmp.to_string_lossy()).is_ok());
        assert!(check_writable_dir("").is_err());

        // The exact shape of the bug: a macOS path on Windows, or vice versa.
        let foreign = if cfg!(windows) { "/Users/nobody/Downloads" } else { "Z:/nobody/Downloads" };
        assert!(
            check_writable_dir(foreign).is_err(),
            "{} should be rejected on this platform",
            foreign
        );
    }
}
