# 📦 Version Management - YouTube Downloader

Управление версиями для Tauri приложения YouTube Downloader.

---

## ⚡ Быстрая справка

### 🍎 macOS

```bash
# Через Make
make version-status
make version-sync
make version-bump-patch    # 0.1.0 → 0.1.1
make version-bump-minor    # 0.1.0 → 0.2.0
make version-bump-major    # 0.1.0 → 1.0.0
make version-set v=1.0.0

# Или напрямую через Python
python3 scripts/version.py status
python3 scripts/version.py sync
python3 scripts/version.py bump patch
python3 scripts/version.py bump minor
python3 scripts/version.py bump major
python3 scripts/version.py set 1.0.0
```

---

## 🎯 Текущая версия: **0.1.0**

**Дата:** 02.01.2026  
**Статус:** Initial release - YouTube video downloader with modern UI

---

## 📁 Файлы версий

| Файл | Описание | Главный |
|------|----------|---------|
| `package.json` | npm package version | ✅ Источник |
| `src-tauri/Cargo.toml` | Rust app version | |
| `src-tauri/tauri.conf.json` | Tauri config version | |

### Главный источник версии

```json
// package.json
{
  "name": "youtube-downloader",
  "version": "0.1.0"  // ← Основной источник версии
}
```

---

## 🍎 macOS: Управление версиями

### Проверить текущую версию

```bash
cd /Users/olgazaharova/Project/ProjectYouTube

# Через Make
make version-status

# Или через Python напрямую
python3 scripts/version.py status
```

**Вывод:**
```
📦 YouTube Downloader Version Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  youtube-downloader/package.json              : 0.1.0
  youtube-downloader/src-tauri/Cargo.toml      : 0.1.0
  youtube-downloader/src-tauri/tauri.conf.json : 0.1.0

✓ All versions synchronized
```

### Синхронизировать файлы

```bash
# Через Make
make version-sync

# Или через Python
python3 scripts/version.py sync
```

Считывает версию из `package.json` и обновляет остальные файлы.

### Увеличить версию

**Patch (0.1.0 → 0.1.1):** Исправления багов
```bash
make version-bump-patch
# или
python3 scripts/version.py bump patch
```

**Minor (0.1.0 → 0.2.0):** Новые функции
```bash
make version-bump-minor
# или
python3 scripts/version.py bump minor
```

**Major (0.1.0 → 1.0.0):** Breaking changes
```bash
make version-bump-major
# или
python3 scripts/version.py bump major
```

### Установить конкретную версию

```bash
# Через Make
make version-set v=1.0.0

# Или через Python
python3 scripts/version.py set 1.0.0
```

---

## 🚀 Процесс релиза

### macOS

```bash
# 1. Увеличить версию
make version-bump-minor
# или для patch/major:
# make version-bump-patch
# make version-bump-major

# 2. Собрать приложение
cd youtube-downloader
npm run tauri build

# 3. Проверить версию
make version-status

# 4. Коммит
git add -A
git commit -m "chore: release v0.2.0"

# 5. Тег
git tag -a v0.2.0 -m "YouTube Downloader v0.2.0"
git push origin v0.2.0

# 6. GitHub Release (опционально)
gh release create v0.2.0 \
  --title "YouTube Downloader v0.2.0" \
  --notes "Release notes here" \
  youtube-downloader/src-tauri/target/release/bundle/dmg/*.dmg
```

---

## 📋 Semantic Versioning

| Тип | Когда использовать | Пример |
|-----|-------------------|--------|
| **Patch** | Исправления багов, мелкие улучшения UI | 0.1.0 → 0.1.1 |
| **Minor** | Новые функции (плейлисты, история) | 0.1.0 → 0.2.0 |
| **Major** | Полная переработка UI/архитектуры | 0.1.0 → 1.0.0 |

---

## 🔄 Ручное обновление версий

### 1. package.json
```json
{
  "name": "youtube-downloader",
  "version": "X.Y.Z"
}
```

### 2. src-tauri/Cargo.toml
```toml
[package]
name = "youtube-downloader"
version = "X.Y.Z"
```

### 3. src-tauri/tauri.conf.json
```json
{
  "version": "X.Y.Z"
}
```

---

## 📝 История версий

### 0.1.0 (02.01.2026)
- ✨ Первый релиз
- 🎨 Современный dark mode UI
- 📥 Скачивание YouTube видео
- 🎬 Выбор качества (Best, 1080p, 720p, 480p, MP3)
- 📊 Прогресс-бар
- 🔐 Поддержка Chrome cookies
- 📁 Выбор папки сохранения

---

## ❓ Решение проблем

### Версии не синхронизированы

```bash
# Посмотреть текущее состояние
make version-status
# или
python3 scripts/version.py status

# Синхронизировать все файлы
make version-sync
# или
python3 scripts/version.py sync
```

### "make не найден"

Установите через Homebrew:
```bash
brew install make
```

### "python3 не найден"

```bash
# Проверьте установку Python
python3 --version

# Установите если отсутствует
brew install python3
```

### После обновления версии ничего не изменилось

```bash
# Очистите кеш и пересоберите
cd youtube-downloader
npm run tauri build
```

---

**Дата обновления:** 02.01.2026
