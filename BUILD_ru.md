# 🔨 YouTube Downloader — сборка и разработка

Язык: [English](BUILD.md) · **Русский**

**Version:** 1.6.2

---

## 📑 Оглавление

- [Требования](#-требования)
- [Установка на macOS](#-macos-установка-и-первая-сборка)
- [Установка на Windows](#-windows-установка-и-первая-сборка)
- [Режим разработки (dev)](#-режим-разработки-dev)
- [Релизная сборка](#-релизная-сборка)
- [Управление версиями](#-управление-версиями)
- [Структура проекта](#-структура-проекта)
- [Разработка Frontend](#-разработка-frontend)
- [Разработка Backend (Rust)](#-разработка-backend-rust)
- [Тестирование и проверка кода](#-тестирование-и-проверка-кода)
- [Конфигурация](#-конфигурация)
- [Оптимизация](#-оптимизация)
- [Кастомизация](#-кастомизация)
- [Частые проблемы](#-частые-проблемы)

---

## 🛠️ Требования

| Инструмент | Зачем | Версия |
|---|---|---|
| **Node.js** + npm | Frontend (Vite) и Tauri CLI | 18+ / 8+ |
| **Rust** + Cargo | Бэкенд (нативное приложение) | 1.70+ |
| **yt-dlp** | Скачивание видео | последняя |
| **ffmpeg** | Склейка видео + аудио | любая |
| **Python** | Только для `scripts/version.py` | 3.10+ |
| **Chrome** | Cookies для приватных видео (опционально) | любая |

**Платформо-специфичное:**

| | macOS | Windows |
|---|---|---|
| Компилятор | Xcode CLT (`xcode-select --install`) | VS Build Tools → «Desktop development with C++» |
| Пакетный менеджер | Homebrew | Chocolatey (необязательно) |

> 👉 Пошаговые чеклисты с нуля:
> - [docs/MACOS_SETUP_ru.md](docs/MACOS_SETUP_ru.md)
> - [docs/WINDOWS_SETUP_ru.md](docs/WINDOWS_SETUP_ru.md)

---

## 🍎 macOS: установка и первая сборка

```bash
# 1. Установить инструменты (если ещё не стоят)
brew install node yt-dlp ffmpeg
# Rust: https://rustup.rs/

# 2. Клонировать и поставить зависимости
git clone https://github.com/kureinmaxim/ProjectYouTube.git
cd ProjectYouTube/youtube-downloader
npm install

# 3. Первая сборка (проверка что всё на месте)
npm run tauri build

# Результат:
# src-tauri/target/release/bundle/macos/youtube-downloader.app
# src-tauri/target/release/bundle/dmg/*.dmg
```

Установить в `/Applications` и закрепить в Dock:

```bash
cd ..          # вернуться в корень ProjectYouTube
make install-app
```

> ⚠️ Не закрепляйте `.app` из `target/` — он удаляется при каждой пересборке.

---

## 🪟 Windows: установка и первая сборка

### Что установить

1. **Rust** — [rustup-init.exe](https://rustup.rs/), default options → **перезапустить PowerShell**
2. **Node.js LTS** — [nodejs.org](https://nodejs.org/), галочка «Add to PATH» → **перезапустить PowerShell**
3. **Visual Studio Build Tools** — [visualstudio.microsoft.com](https://visualstudio.microsoft.com/downloads/) → выбрать **«Desktop development with C++»**
4. **yt-dlp** — `choco install yt-dlp` или скачать `yt-dlp.exe` и добавить в PATH
5. **ffmpeg** — `choco install ffmpeg` или добавить `ffmpeg.exe` в PATH
6. **Python 3.10+** — [python.org](https://www.python.org/downloads/), «Add to PATH» (нужен только для `scripts/version.py`)
7. **Chrome** (опционально) — для cookies

### Проверка

```powershell
rustc --version    # 1.70+
node --version     # v18+
npm --version      # 8+
yt-dlp --version
python --version   # 3.10+
```

### Первая сборка

```powershell
git clone https://github.com/kureinmaxim/ProjectYouTube.git
cd ProjectYouTube\youtube-downloader
npm install
npm run tauri build
```

Артефакты:

```
src-tauri\target\release\youtube-downloader.exe
src-tauri\target\release\bundle\msi\youtube-downloader_*_x64_en-US.msi
```

---

## 🚀 Режим разработки (dev)

Hot-reload: изменения в TypeScript/CSS применяются мгновенно, Rust перекомпилируется автоматически.

### macOS

```bash
# из корня ProjectYouTube
make dev
```

### Windows

```powershell
cd youtube-downloader
npm run tauri dev
```

Приложение откроется автоматически. Frontend: `http://localhost:1420/`.

Только frontend (без окна Tauri):

```bash
cd youtube-downloader
npm run dev
```

---

## 📦 Релизная сборка

### macOS

```bash
make build
# → youtube-downloader/src-tauri/target/release/bundle/macos/youtube-downloader.app
# → youtube-downloader/src-tauri/target/release/bundle/dmg/*.dmg

make install-app   # скопировать в /Applications
make run           # запустить
make run-verbose   # запустить с логами (диагностика пустого окна)
```

### Windows

```powershell
cd youtube-downloader
npm run tauri build
# → src-tauri\target\release\youtube-downloader.exe
# → src-tauri\target\release\bundle\msi\*.msi
```

---

## 🔢 Управление версиями

Источник истины: `youtube-downloader/package.json`. Скрипт синхронизирует `Cargo.toml` и `tauri.conf.json`.

### macOS (Make)

```bash
make version-status          # текущая версия
make version-bump-patch      # 1.6.0 → 1.6.1
make version-bump-minor      # 1.6.0 → 1.7.0
make version-set v=2.0.0     # конкретная версия
```

### Windows / без Make

```powershell
python scripts\version.py status
python scripts\version.py bump patch
python scripts\version.py set 2.0.0
```

После бампа: обновить [CHANGELOG.md](CHANGELOG.md), собрать, затегировать.

Подробнее: [VERSION_MANAGEMENT_ru.md](VERSION_MANAGEMENT_ru.md)

---

## 📂 Структура проекта

```
ProjectYouTube/
├── youtube-downloader/           # Tauri-приложение
│   ├── index.html               # HTML интерфейс
│   ├── package.json             # NPM зависимости
│   ├── vite.config.ts           # Vite конфигурация
│   ├── src/                     # Frontend
│   │   ├── main.ts             # TypeScript логика
│   │   └── styles.css          # CSS стили
│   └── src-tauri/               # Rust бэкенд
│       ├── Cargo.toml           # Rust зависимости
│       ├── tauri.conf.json      # Tauri конфигурация
│       └── src/
│           ├── lib.rs           # Главный модуль
│           ├── ytdlp.rs         # Интеграция с yt-dlp + fallback
│           └── downloader/      # Модуль скачивания
│               ├── utils.rs     # Network detection (TUN/SOCKS5/IP)
│               ├── tools.rs     # Управление yt-dlp
│               ├── commands.rs  # Tauri команды
│               └── backends/    # Download backends
├── scripts/
│   └── version.py               # Управление версиями
├── Makefile                     # macOS-команды (dev, build, version-*)
└── docs/                        # Документация (русский)
```

---

## 🎨 Разработка Frontend

**Стек:** HTML/CSS + TypeScript + Vite + Tauri API

### Файлы

| Файл | Что делает |
|---|---|
| `index.html` | Разметка интерфейса |
| `src/main.ts` | Вся логика: URL → info → download → progress |
| `src/styles.css` | Все стили (CSS-переменные, dark mode) |

### CSS-переменные (тема)

```css
:root {
  --color-primary: #8b5cf6;     /* Фиолетовый */
  --color-secondary: #ec4899;   /* Розовый */
  --bg-primary: #0a0a0f;        /* Тёмный фон */
}
```

Изменения применяются мгновенно в `npm run tauri dev`.

---

## 🦀 Разработка Backend (Rust)

### Главные файлы

**`lib.rs`** — точка входа, регистрация команд:

```rust
mod ytdlp;
use ytdlp::{get_video_info, download_video, get_formats};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_video_info,
            download_video,
            get_formats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**`ytdlp.rs`** — интеграция с yt-dlp:

| Функция | Что делает |
|---|---|
| `get_video_info()` | Получение метаданных видео |
| `download_video()` | Скачивание с прогрессом |
| `get_formats()` | Доступные форматы |

### Добавление новой команды

```rust
// 1. Новая функция в ytdlp.rs
#[tauri::command]
pub async fn new_command(param: String) -> Result<String, String> {
    Ok("Result".to_string())
}

// 2. Регистрация в lib.rs
.invoke_handler(tauri::generate_handler![
    get_video_info, download_video, get_formats,
    new_command,  // ← добавить
])

// 3. Вызов из frontend (main.ts)
const result = await invoke("new_command", { param: "value" });
```

---

## 🧪 Тестирование и проверка кода

### Unit-тесты (Rust)

```bash
cd youtube-downloader/src-tauri
cargo test
cargo test -- --nocapture   # с подробным выводом
```

### Lint и форматирование

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

### Ручное тестирование

1. `npm run tauri dev` (или `make dev`)
2. Вставить YouTube URL → **Get Info**
3. Выбрать качество и папку → **Download**
4. Проверить прогресс-бар и скачанный файл

### Проверка офлайн-сборки (macOS)

```bash
make check-assets   # убедиться, что UI не грузит ничего из сети
```

---

## 🔧 Конфигурация

### tauri.conf.json

```json
{
  "productName": "youtube-downloader",
  "version": "1.6.0",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.icns", "icons/icon.ico"]
  }
}
```

### Cargo.toml (основные зависимости)

```toml
[dependencies]
tauri = { version = "2", features = ["devtools"] }
tauri-plugin-dialog = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
```

---

## 🚀 Оптимизация

### Уменьшение размера

```toml
# Cargo.toml
[profile.release]
strip = true          # убрать debug-символы
lto = true            # Link Time Optimization
codegen-units = 1     # лучшая оптимизация
opt-level = "s"       # оптимизация по размеру
```

### DevTools

В dev-режиме доступны Chrome DevTools: ПКМ → Inspect Element или F12.

### Логирование

```rust
// Rust
println!("Debug: {:?}", value);
eprintln!("Error: {}", error);
```

```typescript
// TypeScript
console.log("Info:", info);
console.error("Error:", error);
```

---

## 🎨 Кастомизация

### Цветовая схема

В `src/styles.css`:

```css
:root {
  --color-primary: #8b5cf6;      /* ваш цвет */
  --color-secondary: #ec4899;    /* ваш цвет */
  --bg-primary: #0a0a0f;         /* ваш цвет */
}
```

### Новое качество видео

В `src-tauri/src/ytdlp.rs`:

```rust
let format_arg = match quality.as_str() {
    "best" => "bestvideo+bestaudio/best",
    "1080p" => "bestvideo[height<=1080]+bestaudio/best[height<=1080]",
    "custom" => "YOUR_FORMAT_HERE",  // ← добавьте
    _ => "best",
};
```

В `index.html`:

```html
<option value="custom">🎬 Custom Quality</option>
```

---

## 🐛 Частые проблемы

### Общие

| Проблема | Решение |
|---|---|
| `yt-dlp not found` | Установить и проверить: `yt-dlp --version` |
| Chrome cookies не работают | Chrome установлен и авторизован на YouTube |
| Ошибка компиляции Rust | `cd src-tauri && cargo clean`, затем пересобрать |
| Frontend не обновляется | Удалить `node_modules/.vite`, перезапустить `npm run tauri dev` |
| Permission denied при скачивании | Выбрать другую папку с правами на запись |

### macOS

| Проблема | Решение |
|---|---|
| Пустое белое окно | См. [docs/MACOS_SETUP_ru.md](docs/MACOS_SETUP_ru.md) → «Пустое белое окно» |
| `xcrun: error` | `xcode-select --install` |
| `command not found: rustc` | Перезапустить терминал после установки Rust |
| `IP: N/A` / таймауты | Сломан DNS — [docs/NETWORK_SETUP_ru.md](docs/NETWORK_SETUP_ru.md) |

### Windows

| Проблема | Решение |
|---|---|
| `rustc не найден` | Перезапустить PowerShell после установки rustup |
| `npm не найден` | Перезапустить PowerShell после установки Node.js |
| MSVC / линкер не найден | Установить VS Build Tools → «Desktop development with C++» |
| `python не найден` | Попробовать `py` вместо `python` |

---

**Разработчик:** Куреин М.Н.
