# Version management

Language: **English** · [Русский](VERSION_MANAGEMENT_ru.md)

How YouTube Downloader stores the app version and how to change it safely.

Related: [BUILD.md](BUILD.md) · [CHANGELOG.md](CHANGELOG.md)

## Current version

**1.5.1** — source of truth is `youtube-downloader/package.json`.

## What stays in sync

| File | Role |
|---|---|
| `youtube-downloader/package.json` | Source of truth |
| `youtube-downloader/src-tauri/Cargo.toml` | Rust crate version |
| `youtube-downloader/src-tauri/tauri.conf.json` | Tauri bundle version |

Do not edit those three by hand. Use the tool:

```text
make version-status
make version-sync
make version-bump-patch    # 1.5.1 → 1.5.2
make version-bump-minor    # 1.5.1 → 1.6.0
make version-bump-major    # 1.5.1 → 2.0.0
make version-set v=1.6.0
```

Without Make (Windows or any OS):

```text
python scripts/version.py status
python scripts/version.py sync
python scripts/version.py bump patch|minor|major
python scripts/version.py set 1.6.0
```

macOS/Linux if `python` is not on `PATH`: `python3 scripts/version.py …`.

`status` should print the same version three times. If it does not, run `sync`.

## Release checklist

1. Bump (`bump patch` / `minor` / `major` or `set`).
2. Add a `[X.Y.Z]` section to `CHANGELOG.md`.
3. Build and smoke-test (`make build` / `npm run tauri build`).
4. Commit, tag `vX.Y.Z`, push the tag.
5. Optional GitHub Release with the `.dmg` / `.msi`.

```bash
git tag -a v1.5.2 -m "YouTube Downloader v1.5.2"
git push origin v1.5.2

gh release create v1.5.2 \
  --title "YouTube Downloader v1.5.2" \
  --notes-file CHANGELOG.md \
  youtube-downloader/src-tauri/target/release/bundle/dmg/*.dmg
```

## Troubleshooting

**Versions drifted** — `python scripts/version.py sync`.

**`make` not found (macOS)** — `brew install make`.

**UI still shows the old version** — rebuild. Dev mode can cache the previous bundle; quit the app and run `make dev` / `npm run tauri dev` again.

**README badge is stale** — `scripts/version.py` does not rewrite Markdown. Update the version string in `README.md` / `README_ru.md` in the same commit as the bump.
