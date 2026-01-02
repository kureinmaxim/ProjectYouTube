# 🔨 YouTube Downloader - Build & Development Guide

Руководство по разработке, сборке и запуску приложения YouTube Downloader.

---

## 🚀 Для разработчиков (Quick Start)

### macOS - Dev Mode (режим разработки)

```bash
# Dev режим - быстрая пересборка с hot-reload
cd youtube-downloader
npm run tauri dev

# Приложение запустится автоматически
# Frontend: http://localhost:1420/
# Backend: Rust с hot-reload
```

### macOS - Build Mode (релизная сборка)

```bash
# Полная сборка - создание .app и .dmg
cd youtube-downloader
npm run tauri build

# Результаты:
# src-tauri/target/release/bundle/macos/youtube-downloader.app
# src-tauri/target/release/bundle/dmg/youtube-downloader_X.X.X_aarch64.dmg
```

### Полезные команды разработки

```bash
# Установка зависимостей
cd youtube-downloader
npm install

# Проверка Rust кода
cd src-tauri
cargo check
cargo clippy -- -D warnings
cargo fmt

# Тесты
cargo test

# Очистка
cargo clean
```

---

## 📂 Структура проекта

```
youtube-downloader/
├── index.html              # HTML интерфейс
├── package.json            # NPM зависимости
├── src/                    # Frontend код
│   ├── main.ts            # TypeScript логика
│   └── styles.css         # CSS стили
├── src-tauri/              # Rust Backend
│   ├── Cargo.toml         # Rust зависимости
│   ├── tauri.conf.json    # Tauri конфигурация
│   └── src/
│       ├── lib.rs         # Главный модуль
│       └── ytdlp.rs       # Интеграция с yt-dlp
└── vite.config.ts         # Vite конфигурация
```

---

## 🛠️ Требования

### macOS

```bash
# Проверка необходимых инструментов
rustc --version    # Rust 1.70+
cargo --version    # Cargo
node --version     # Node.js 18+
npm --version      # npm 8+
yt-dlp --version   # yt-dlp (для скачивания видео)
```

### Установка отсутствующих инструментов

> **👉 Первый раз настраиваете проект?** Используйте платформо-специфичные гайды:
> 
> - **macOS:** [MACOS_SETUP.md](MACOS_SETUP.md) - Пошаговая установка для macOS
> - **Windows:** [WINDOWS_SETUP.md](WINDOWS_SETUP.md) - Пошаговая установка для Windows

Эти гайды содержат детальные инструкции по установке всех необходимых инструментов и первой сборке проекта.

#### Быстрая установка (macOS)

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js (через Homebrew)
brew install node

# yt-dlp
brew install yt-dlp
# или
curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o ~/bin/yt-dlp
chmod +x ~/bin/yt-dlp
```

---

## 📦 Первоначальная настройка

### Клонирование и установка

```bash
# 1. Перейти в проект
cd /Users/olgazaharova/Project/ProjectYouTube/youtube-downloader

# 2. Установить npm зависимости
npm install

# 3. Первая сборка (проверка что все работает)
npm run tauri build
```

---

## 🎨 Разработка Frontend

### Технологии
- **HTML/CSS** - Структура и стили
- **TypeScript** - Логика приложения
- **Vite** - Dev server с hot-reload
- **Tauri API** - Интеграция с backend

### Запуск dev сервера

```bash
cd youtube-downloader
npm run dev  # Только frontend без Tauri

# или

npm run tauri dev  # Frontend + Tauri backend
```

### Редактирование стилей

Файл `src/styles.css` содержит все стили. Изменения применяются автоматически при сохранении.

```css
/* Основные CSS переменные */
:root {
  --color-primary: #8b5cf6;     /* Фиолетовый */
  --color-secondary: #ec4899;   /* Розовый */
  --bg-primary: #0a0a0f;        /* Темный фон */
  /* ... */
}
```

---

## 🦀 Разработка Backend (Rust)

### Основные файлы

**lib.rs** - Главная точка входа
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

**ytdlp.rs** - Интеграция с yt-dlp
- `get_video_info()` - Получение информации о видео
- `download_video()` - Скачивание с прогрессом
- `get_formats()` - Доступные форматы

### Добавление новых команд

```rust
// 1. Добавьте функцию в ytdlp.rs
#[tauri::command]
pub async fn new_command(param: String) -> Result<String, String> {
    Ok("Result".to_string())
}

// 2. Зарегистрируйте в lib.rs
.invoke_handler(tauri::generate_handler![
    get_video_info,
    download_video,
    get_formats,
    new_command,  // ← добавить
])

// 3. Вызовите из frontend (main.ts)
const result = await invoke("new_command", { param: "value" });
```

---

## 🧪 Тестирование

### Тестирование в dev режиме

```bash
cd youtube-downloader
npm run tauri dev

# Тестируйте вручную в открывшемся приложении:
# 1. Вставьте YouTube URL
# 2. Нажмите "Получить информацию"
# 3. Проверьте отображение видео
# 4. Выберите качество и папку
# 5. Скачайте видео
```

### Unit тесты (Rust)

```bash
cd src-tauri
cargo test

# С подробным выводом
cargo test -- --nocapture
```

### Проверка кода

```bash
# Линтинг
cargo clippy -- -D warnings

# Форматирование
cargo fmt --check
```

---

## 📦 Сборка для релиза

### macOS

```bash
cd youtube-downloader
npm run tauri build

# Результаты в:
# src-tauri/target/release/bundle/macos/youtube-downloader.app
# src-tauri/target/release/bundle/dmg/youtube-downloader_0.1.0_aarch64.dmg
```

### Тестирование релизной сборки

```bash
# Запустить .app файл
open src-tauri/target/release/bundle/macos/youtube-downloader.app

# Или установить .dmg
open src-tauri/target/release/bundle/dmg/youtube-downloader_0.1.0_aarch64.dmg
```

---

## 🔧 Конфигурация

### tauri.conf.json

Основные настройки приложения:

```json
{
  "productName": "youtube-downloader",
  "version": "0.1.0",
  "identifier": "com.olgazaharova.youtube-downloader",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

### Cargo.toml

Rust зависимости:

```toml
[dependencies]
tauri = { version = "2", features = ["devtools"] }
tauri-plugin-dialog = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
```

---

## 🐛 Частые проблемы

### "yt-dlp not found"

```bash
# Проверьте установку
yt-dlp --version

# Установите если отсутствует
brew install yt-dlp

# Или укажите полный путь в ytdlp.rs
Command::new("/usr/local/bin/yt-dlp")
```

### "Chrome cookies не работают"

```bash
# Убедитесь что Chrome установлен и вы авторизованы на YouTube
# yt-dlp автоматически найдет cookies в:
# ~/Library/Application Support/Google/Chrome/Default/Cookies (macOS)
```

### Ошибка компиляции Rust

```bash
# Очистите и пересоберите
cd src-tauri
cargo clean
cd ..
npm run tauri build
```

### Frontend не обновляется

```bash
# Очистите кеш Vite
rm -rf node_modules/.vite
npm run tauri dev
```

### Permission denied при скачивании

```bash
# Проверьте права на папку Downloads
ls -la ~/Downloads

# Выберите другую папку с правами на запись
```

---

## 📊 Мониторинг производительности

### Dev Tools

В dev режиме доступны Chrome DevTools:
- Правый клик → Inspect Element
- Или F12

### Логирование

```rust
// В Rust коде
println!("Debug: {:?}", value);
eprintln!("Error: {}", error);

// В TypeScript
console.log("Info:", info);
console.error("Error:", error);
```

---

## 🚀 Оптимизация

### Размер приложения

```bash
# Проверить размер bundle
du -sh src-tauri/target/release/bundle/macos/youtube-downloader.app

# Для уменьшения размера:
# 1. Используйте strip в Cargo.toml
# 2. Включите LTO (Link Time Optimization)
```

### Cargo.toml оптимизации

```toml
[profile.release]
strip = true          # Убрать debug символы
lto = true           # Link Time Optimization
codegen-units = 1    # Лучшая оптимизация
opt-level = "s"      # Оптимизация размера
```

---

## 📞 Зависимости проекта

### NPM Packages

```json
{
  "@tauri-apps/api": "^2.x",
  "@tauri-apps/plugin-dialog": "^2.x"
}
```

### Rust Crates

```toml
tauri = "2"
tauri-plugin-dialog = "2"
serde = "1"
serde_json = "1"
tokio = "1"
```

### Внешние инструменты

- **yt-dlp** - Скачивание видео
- **Google Chrome** - Для cookies (опционально)

---

## 🎯 Рабочий процесс

### Ежедневная разработка

```bash
# 1. Запустить dev режим
cd youtube-downloader
npm run tauri dev

# 2. Редактировать код
# - main.ts для логики
# - styles.css для стилей  
# - ytdlp.rs для backend

# 3. Тестировать изменения (hot-reload)

# 4. Коммит
git add -A
git commit -m "feat: добавил новую функцию"
git push
```

### Подготовка релиза

```bash
# 1. Обновить версию
# - package.json
# - src-tauri/Cargo.toml
# - src-tauri/tauri.conf.json

# 2. Собрать
npm run tauri build

# 3. Протестировать .app файл

# 4. Создать release
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

---

## 📞 Поддержка

При проблемах:
1. ✅ Проверьте что yt-dlp установлен: `yt-dlp --version`
2. ✅ Проверьте что Chrome установлен (для cookies)
3. ✅ Очистите кеш: `cargo clean`
4. ✅ Пересоберите: `npm run tauri build`
5. ✅ Проверьте логи в терминале

**Разработчик:** Kurein Maxim  
**Дата создания:** 02.01.2026

---

## 🎨 Кастомизация

### Изменить цветовую схему

В `src/styles.css`:

```css
:root {
  --color-primary: #8b5cf6;      /* Фиолетовый → Ваш цвет */
  --color-secondary: #ec4899;    /* Розовый → Ваш цвет */
  --bg-primary: #0a0a0f;         /* Темный фон → Ваш цвет */
}
```

### Добавить новое качество видео

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
<select id="quality-select">
  <option value="custom">🎬 Custom Quality</option>
</select>
```

---

**Приятной разработки! 🚀**
