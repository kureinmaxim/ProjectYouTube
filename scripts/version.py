#!/usr/bin/env python3
"""
Version management script for YouTube Downloader.
Synchronizes versions across all version files.

Usage:
    python scripts/version.py status          # Show current versions
    python scripts/version.py sync            # Sync all versions from package.json
    python scripts/version.py bump patch      # 0.1.0 → 0.1.1
    python scripts/version.py bump minor      # 0.1.0 → 0.2.0
    python scripts/version.py bump major      # 0.1.0 → 1.0.0
    python scripts/version.py set 1.0.0       # Set specific version
"""

import argparse
import json
import re
import sys
from pathlib import Path

# File paths relative to project root
VERSION_FILES = {
    "npm": "youtube-downloader/package.json",
    "tauri_cargo": "youtube-downloader/src-tauri/Cargo.toml",
    "tauri_conf": "youtube-downloader/src-tauri/tauri.conf.json",
}


def get_project_root() -> Path:
    """Find project root by looking for youtube-downloader directory."""
    current = Path(__file__).resolve().parent
    while current != current.parent:
        if (current / "youtube-downloader").exists():
            return current
        current = current.parent
    # Fallback: assume script is in scripts/
    return Path(__file__).resolve().parent.parent


def read_cargo_version(path: Path) -> str:
    """Read version from Cargo.toml."""
    content = path.read_text(encoding="utf-8")
    match = re.search(r'^version\s*=\s*"([^"]+)"', content, re.MULTILINE)
    if match:
        return match.group(1)
    raise ValueError(f"Version not found in {path}")


def write_cargo_version(path: Path, version: str) -> None:
    """Write version to Cargo.toml."""
    content = path.read_text(encoding="utf-8")
    new_content = re.sub(
        r'^(version\s*=\s*")[^"]+(")' ,
        rf'\g<1>{version}\g<2>',
        content,
        count=1,
        flags=re.MULTILINE,
    )
    path.write_text(new_content, encoding="utf-8")


def read_package_json_version(path: Path) -> str:
    """Read version from package.json."""
    data = json.loads(path.read_text(encoding="utf-8"))
    return data.get("version", "0.0.0")


def write_package_json_version(path: Path, version: str) -> None:
    """Write version to package.json."""
    data = json.loads(path.read_text(encoding="utf-8"))
    data["version"] = version
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def read_tauri_conf_version(path: Path) -> str:
    """Read version from tauri.conf.json."""
    data = json.loads(path.read_text(encoding="utf-8"))
    return data.get("version", "0.0.0")


def write_tauri_conf_version(path: Path, version: str) -> None:
    """Write version to tauri.conf.json."""
    data = json.loads(path.read_text(encoding="utf-8"))
    data["version"] = version
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def get_all_versions(root: Path) -> dict[str, str]:
    """Get versions from all files."""
    versions = {}
    
    # npm package.json (main source)
    npm_path = root / VERSION_FILES["npm"]
    if npm_path.exists():
        versions["npm"] = read_package_json_version(npm_path)
    
    # Tauri Cargo.toml
    tauri_cargo_path = root / VERSION_FILES["tauri_cargo"]
    if tauri_cargo_path.exists():
        versions["tauri_cargo"] = read_cargo_version(tauri_cargo_path)
    
    # Tauri config
    tauri_conf_path = root / VERSION_FILES["tauri_conf"]
    if tauri_conf_path.exists():
        versions["tauri_conf"] = read_tauri_conf_version(tauri_conf_path)
    
    return versions


def set_all_versions(root: Path, version: str) -> None:
    """Set version in all files."""
    # npm package.json
    npm_path = root / VERSION_FILES["npm"]
    if npm_path.exists():
        write_package_json_version(npm_path, version)
        print(f"  ✓ {VERSION_FILES['npm']}: {version}")
    
    # Tauri Cargo.toml
    tauri_cargo_path = root / VERSION_FILES["tauri_cargo"]
    if tauri_cargo_path.exists():
        write_cargo_version(tauri_cargo_path, version)
        print(f"  ✓ {VERSION_FILES['tauri_cargo']}: {version}")
    
    # Tauri config
    tauri_conf_path = root / VERSION_FILES["tauri_conf"]
    if tauri_conf_path.exists():
        write_tauri_conf_version(tauri_conf_path, version)
        print(f"  ✓ {VERSION_FILES['tauri_conf']}: {version}")


def bump_version(version: str, bump_type: str) -> str:
    """Bump version according to semver."""
    parts = version.split(".")
    if len(parts) != 3:
        raise ValueError(f"Invalid version format: {version}")
    
    major, minor, patch = map(int, parts)
    
    if bump_type == "major":
        major += 1
        minor = 0
        patch = 0
    elif bump_type == "minor":
        minor += 1
        patch = 0
    elif bump_type == "patch":
        patch += 1
    else:
        raise ValueError(f"Unknown bump type: {bump_type}")
    
    return f"{major}.{minor}.{patch}"


def cmd_status(root: Path) -> None:
    """Show current version status."""
    print("📦 YouTube Downloader Version Status")
    print("━" * 50)
    
    versions = get_all_versions(root)
    unique_versions = set(versions.values())
    
    # Display with aligned columns
    max_path_len = max(len(VERSION_FILES.get(name, name)) for name in versions)
    
    for name, version in versions.items():
        file_path = VERSION_FILES.get(name, name)
        print(f"  {file_path.ljust(max_path_len)} : {version}")
    
    print()
    if len(unique_versions) == 1:
        print("✓ All versions synchronized")
    else:
        print("⚠ Versions out of sync!")
        print()
        print("To fix, run:")
        print("  macOS:   make version-sync")
        print("  Or:      python3 scripts/version.py sync")


def cmd_sync(root: Path, target_version: str | None = None) -> None:
    """Synchronize all versions."""
    if target_version:
        version = target_version
    else:
        # Use package.json version as source of truth
        npm_path = root / VERSION_FILES["npm"]
        version = read_package_json_version(npm_path)
    
    print(f"🔄 Syncing all files to version {version}")
    set_all_versions(root, version)
    print("✓ Done!")


def cmd_bump(root: Path, bump_type: str) -> None:
    """Bump version."""
    npm_path = root / VERSION_FILES["npm"]
    current = read_package_json_version(npm_path)
    new_version = bump_version(current, bump_type)
    
    print(f"⬆️  Bumping version: {current} → {new_version}")
    set_all_versions(root, new_version)
    print("✓ Done!")


def cmd_set(root: Path, version: str) -> None:
    """Set specific version."""
    # Validate version format
    if not re.match(r"^\d+\.\d+\.\d+$", version):
        print(f"❌ Invalid version format: {version}")
        print("   Expected: X.Y.Z (e.g., 1.0.0)")
        sys.exit(1)
    
    print(f"📝 Setting version to {version}")
    set_all_versions(root, version)
    print("✓ Done!")


def main() -> None:
    parser = argparse.ArgumentParser(description="YouTube Downloader version management")
    subparsers = parser.add_subparsers(dest="command", required=True)
    
    # status
    subparsers.add_parser("status", help="Show version status")
    
    # sync
    sync_parser = subparsers.add_parser("sync", help="Sync all versions")
    sync_parser.add_argument("version", nargs="?", help="Target version (optional)")
    
    # bump
    bump_parser = subparsers.add_parser("bump", help="Bump version")
    bump_parser.add_argument("type", choices=["major", "minor", "patch"])
    
    # set
    set_parser = subparsers.add_parser("set", help="Set specific version")
    set_parser.add_argument("version", help="Version to set (X.Y.Z)")
    
    args = parser.parse_args()
    root = get_project_root()
    
    if args.command == "status":
        cmd_status(root)
    elif args.command == "sync":
        cmd_sync(root, args.version)
    elif args.command == "bump":
        cmd_bump(root, args.type)
    elif args.command == "set":
        cmd_set(root, args.version)


if __name__ == "__main__":
    main()
