# 📘 YouTube Downloader - Обзор Проекта

**Версия:** 1.1.0  
**Дата:** 03.01.2026

---

## 📝 Оглавление

1. [Что это за проект](#-что-это-за-проект)
2. [Зачем нужны эти технологии](#-зачем-нужны-эти-технологии)
3. [Технологический стек](#-технологический-стек)
4. [Архитектура системы](#-архитектура-системы)
5. [Компоненты и их взаимодействие](#-компоненты-и-их-взаимодействие)
6. [Файлы и их назначение](#-файлы-и-их-назначение)
7. [Как все работает вместе](#-как-все-работает-вместе)
8. [Текущее состояние проекта](#-текущее-состояние-проекта)

---

## 🎯 Что это за проект

**YouTube Downloader** — это современное десктопное приложение для скачивания видео с YouTube на macOS. Приложение построено на **Tauri** (Rust + Web) и предоставляет простой, красивый интерфейс для загрузки видео в различных качествах.

### Основные возможности

- 📥 **Простое скачивание** — вставьте ссылку YouTube и скачайте
- 🎨 **Современный UI** — dark mode с градиентами и анимациями
- 🎬 **Выбор качества** — Best, 1080p, 720p, 480p, только аудио (MP3)
- 📊 **Прогресс в реальном времени** — визуальный прогресс-бар
- 🌐 **Network Status** — мониторинг IP, Proxy и VPN режима
- 🛠 **Tools Management** — управление версиями зависимостей (yt-dlp, FFmpeg)
- 🔐 **Chrome cookies** — автоматическая поддержка для приватных видео
- 📁 **Выбор папки** — сохранение в любую директорию
- ⚡ **Быстрая работа** — нативная производительность благодар Rust
- 🌍 **English Interface** — полностью локализованный интерфейс

---

## 🤔 Зачем нужны эти технологии

### Зачем Tauri?

**Tauri** — это фреймворк для создания десктопных приложений с веб-технологиями.

**Преимущества:**
- ✅ Малый размер приложения (~10MB вместо ~150MB у Electron)
- ✅ Низкое потребление памяти (использует системный WebView)
- ✅ Безопасность (Rust backend изолирован от frontend)
- ✅ Нативная производительность
- ✅ Кроссплатформенность (один код для macOS и Windows)

**Роль в проекте:**
- Создает нативное окно приложения
- Обеспечивает безопасный доступ к файловой системе
- Запускает процессы (yt-dlp)
- Предоставляет API для системных операций

### Зачем Rust?

**Rust** — системный язык программирования с фокусом на безопасность и производительность.

**Преимущества:**
- ✅ **Безопасность памяти** без garbage collector
- ✅ **Производительность** на уровне C/C++
- ✅ **Надежность** — большинство ошибок обнаруживается на этапе компиляции
- ✅ **Современный ecosystem** с Cargo

**Роль в проекте:**
- Backend логика (интеграция с yt-dlp)
- Обработка файловых операций
- Прогресс мониторинг скачивания
- События и state management

### Зачем yt-dlp?

**yt-dlp** — мощный инструмент командной строки для скачивания видео с YouTube и других платформ.

**Преимущества:**
- ✅ Поддержка 1000+ сайтов
- ✅ Обход ограничений YouTube
- ✅ Множество форматов и качеств
- ✅ Поддержка cookies для приватного контента
- ✅ Активная разработка и обновления

**Роль в проекте:**
- Ядро скачивания видео
- Извлечение метаданных (название, длительность, превью)
- Конвертация форматов

### Зачем TypeScript?

**TypeScript** используется для frontend логики.

**Где используется:**
- `main.ts` — основная логика UI
- Tauri API клиент
- Обработка событий и состояния

**Роль в проекте:**
- Типобезопасный frontend код
- Интеграция с Tauri командами
- Управление UI состоянием

---

## 🛠 Технологический стек

### Frontend (UI)

| Технология | Версия | Назначение |
|-----------|--------|------------|
| **HTML/CSS** | HTML5 | Структура и стили интерфейса |
| **TypeScript** | 5.x | Логика приложения |
| **Vite** | 6.x | Dev server и сборка |
| **Google Fonts** | - | Современная типографика (Inter) |

### Backend (Логика)

| Технология | Версия | Назначение |
|-----------|--------|------------|
| **Rust** | 1.70+ | Основной язык backend |
| **Tauri** | 2.x | Desktop framework |
| **Tokio** | 1.x | Асинхронная runtime |
| **Serde** | 1.x | Сериализация данных |

### Инструменты

| Технология | Версия | Назначение |
|-----------|--------|------------|
| **yt-dlp** | latest | Скачивание видео |
| **Node.js** | 18+ | Сборка frontend |
| **Python** | 3.10+ | Скрипты версионирования |
| **Make** | latest | Автоматизация команд |

---

## 🏗 Архитектура системы

### Общая схема

```
┌─────────────────────────────────────────────────────────────────┐
│                   TAURI DESKTOP APP                             │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │              FRONTEND (HTML/CSS/TypeScript)               │  │
│  │                                                           │  │
│  │  ┌──────────────────┐      ┌────────────────────────┐    │  │
│  │  │  index.html      │      │  main.ts               │    │  │
│  │  │  (UI компоненты) │─────▶│  (Логика приложения)  │    │  │
│  │  │                  │      │                        │    │  │
│  │  │  • URL input     │      │  • Event handlers      │    │  │
│  │  │  • Video preview │      │  • Tauri invoke()      │    │  │
│  │  │  • Quality select│      │  • Progress updates    │    │  │
│  │  │  • Progress bar  │      │  • Error handling      │    │  │
│  │  └──────────────────┘      └───────┬────────────────┘    │  │
│  │                                    │                      │  │
│  │                                    │ Tauri Commands       │  │
│  └────────────────────────────────────┼──────────────────────┘  │
│                                       │                         │
│                                       ▼                         │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              RUST BACKEND (lib.rs)                       │   │
│  │                                                           │   │
│  │  ┌──────────────────┐                                    │   │
│  │  │  Tauri Commands  │                                    │   │
│  │  │  (downloader/*)  │                                    │   │
│  │  │                  │                                    │   │
│  │  │  • get_video_info()    ──────────┐                   │   │
│  │  │  • download_video()    ───┐      │                   │   │
│  │  │  • get_network_status()  ─┼──┐   │                   │   │
│  │  └──────────────────┘          │  │   │                   │   │
│  │                                │  │   │                   │   │
│  │                                ▼  ▼   ▼                   │   │
│  │  ┌──────────────────────────────────────────────┐        │   │
│  │  │  downloader/ytdlp.rs (yt-dlp Integration)    │        │   │
│  │  │                                              │        │   │
│  │  │  • Execute yt-dlp процесс                   │        │   │
│  │  │  • Parse JSON output                        │        │   │
│  │  │  • Monitor progress                         │        │   │
│  │  │  • Emit events                              │        │   │
│  │  └────────────────┬─────────────────────────────┘        │   │
│  └────────────────────┼──────────────────────────────────────┘   │
│                       │                                          │
│                       ▼                                          │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │         SYSTEM INTEGRATION                               │   │
│  │                                                           │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌────────────────┐ │   │
│  │  │ yt-dlp CLI   │  │ File System  │  │ Chrome Cookies │ │   │
│  │  │              │  │              │  │                │ │   │
│  │  │ • Download   │  │ • Save files │  │ • Extract      │ │   │
│  │  │ • Extract    │  │ • Pick dir   │  │ • Authenticate │ │   │
│  │  │ • Convert    │  │              │  │                │ │   │
│  │  └──────────────┘  └──────────────┘  └────────────────┘ │   │
│  └──────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
                          │
                          ▼
          ┌───────────────────────────────┐
          │    EXTERNAL SERVICES          │
          │  • YouTube API                │
          │  • CDN для thumbnail          │
          └───────────────────────────────┘
```

---

## 🔌 Компоненты и их взаимодействие

### 1. Frontend (TypeScript)

**Файлы:**
- `index.html` — главная страница UI
- `src/main.ts` — логика приложения
- `src/styles.css` — modern dark UI стили

**Взаимодействие:**
```typescript
// Пользователь вводит URL
const url = urlInput.value;

// Вызов Tauri команды для получения информации
const info = await invoke("get_video_info", { url });

// Отображение результата
displayVideoInfo(info);

// Скачивание
await invoke("download_video", {
  url,
  quality: "720p",
  outputPath: selectedPath
});
```

### 2. Backend Commands (Rust)

**Файлы:**
- `src-tauri/src/lib.rs` — инициализация плагинов и модуля downloader
- `src-tauri/src/downloader/mod.rs` — публичный интерфейс модуля
- `src-tauri/src/downloader/commands.rs` — реализация Tauri команд
- `src-tauri/src/downloader/ytdlp.rs` — интеграция с yt-dlp

**Взаимодействие:**
```rust
#[tauri::command]
pub async fn get_video_info(url: String) -> Result<VideoInfo, String> {
    // Запускаем yt-dlp с --dump-json
    let output = Command::new("yt-dlp")
        .args(["--dump-json", "--cookies-from-browser", "chrome", &url])
        .output()
        .map_err(|e| format!("Failed: {}", e))?;
    
    // Парсим JSON
    let json: serde_json::Value = serde_json::from_str(&output.stdout)?;
    
    Ok(VideoInfo {
        title: json["title"].as_str().unwrap().to_string(),
        duration: format_duration(json["duration"].as_f64()),
        thumbnail: json["thumbnail"].as_str().unwrap().to_string(),
        uploader: json["uploader"].as_str().unwrap().to_string(),
    })
}
```

### 3. yt-dlp Integration

**Как работает:**

```
Tauri Command
    │
    ├─▶ Собирает аргументы для yt-dlp
    │   • URL
    │   • --format (quality)
    │   • --cookies-from-browser chrome
    │   • --output path
    │
    ├─▶ Запускает процесс
    │   Command::new("yt-dlp")
    │
    ├─▶ Мониторит вывод
    │   • stdout для прогресса
    │   • stderr для ошибок
    │
    └─▶ Эмитит события
        app_handle.emit("download-progress", ...)
```

### 4. Event System

**Прогресс скачивания:**
```rust
// Backend эмитит события
app_handle.emit("download-progress", DownloadProgress {
    percent: 45.0,
    status: "Downloading...".to_string(),
})?;

// Frontend слушает события
listen("download-progress", (event) => {
    updateProgress(event.payload.percent);
});
```

---

## 📂 Файлы и их назначение

### Корневая директория

| Файл/Папка | Назначение |
|------------|------------|
| `youtube-downloader/` | Главное приложение (Tauri проект) |
| `scripts/` | Python скрипты (version.py) |
| `Makefile` | Команды автоматизации |
| `README.md` | Главная документация |
| `BUILD.md` | Руководство по сборке |
| `VERSION_MANAGEMENT.md` | Управление версиями |
| `MACOS_SETUP.md` | Установка для macOS |
| `WINDOWS_SETUP.md` | Установка для Windows |

### `youtube-downloader/` (Tauri App)

| Файл | Назначение |
|------|------------|
| `index.html` | Главная страница приложения |
| `package.json` | NPM зависимости и скрипты |
| `vite.config.ts` | Конфигурация Vite |
| `src/main.ts` | Основная логика TypeScript |
| `src/styles.css` | CSS стили (dark mode, анимации) |
| `src-tauri/` | Rust backend |

### `youtube-downloader/src-tauri/` (Rust Backend)

| Файл | Назначение |
|------|------------|
| `Cargo.toml` | Rust зависимости |
| `tauri.conf.json` | Конфигурация Tauri |
| `src/lib.rs` | Entry point, регистрация модулей |
| `src/main.rs` | Точка входа приложения |
| `src/downloader/` | Модуль логики скачивания |
| `src/downloader/commands.rs` | Обработчики команд фронтенда |
| `src/downloader/ytdlp.rs` | Обертка над yt-dlp |
| `src/downloader/network.rs` | Определение IP и прокси |

---

## 🔄 Как все работает вместе

### Сценарий 1: Получение информации о видео

```
┌──────────────┐
│ 1. Пользователь вставляет YouTube URL
└──────────────┘
       │
       │ (index.html → main.ts)
       ▼
┌──────────────────────────────────────────────────┐
│ 2. TypeScript вызывает invoke("get_video_info")  │
└──────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────┐
│ 3. Rust получает команду (lib.rs)                │
└──────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────┐
│ 4. ytdlp.rs запускает yt-dlp процесс              │
│    yt-dlp --dump-json --cookies-from-browser ...  │
└──────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────┐
│ 5. yt-dlp обращается к YouTube API                │
│    Получает метаданные видео                      │
└──────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────┐
│ 6. Парсинг JSON output                            │
│    Извлекаем: title, duration, thumbnail          │
└──────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────┐
│ 7. Возврат VideoInfo в frontend                   │
└──────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────┐
│ 8. Отображение в UI                               │
│    • Превью видео                                 │
│    • Название                                     │
│    • Автор • Длительность                         │
└──────────────────────────────────────────────────┘
```

### Сценарий 2: Скачивание видео

```
Пользователь выбирает качество и папку
    │
    │ Нажимает "Скачать видео"
    ▼
invoke("download_video", {
  url: "https://youtu.be/...",
  quality: "720p",
  outputPath: "/Users/.../Downloads"
})
    │
    ▼
ytdlp.rs строит команду:
    yt-dlp -f "bestvideo[height<=720]+bestaudio" \
           --cookies-from-browser chrome \
           -P /Users/.../Downloads \
           https://youtu.be/...
    │
    ▼
Запускает процесс асинхронно
    │
    ├─▶ Читает stdout построчно
    │   Эмитит события прогресса:
    │   emit("download-progress", {
    │     percent: 45.0,
    │     status: "Downloading..."
    │   })
    │
    └─▶ Frontend обновляет прогресс-бар
        в реальном времени
    │
    ▼
Процесс завершается
    │
    ├─▶ Success: файл сохранен
    │   emit("download-progress", {
    │     percent: 100.0,
    │     status: "Complete!"
    │   })
    │
    └─▶ Error: показать сообщение
```

### Сценарий 3: Chrome Cookies

```
yt-dlp --cookies-from-browser chrome
    │
    ▼
yt-dlp ищет Chrome cookies:
    macOS: ~/Library/Application Support/Google/Chrome/Default/Cookies
    │
    ▼
Извлекает cookies для youtube.com
    │
    ▼
Использует для аутентификации
    │
    └─▶ Позволяет скачивать:
        • Приватные видео
        • Возрастной контент
        • Видео с ограничениями
```

---

## 📊 Текущее состояние проекта

### ✅ Готовые компоненты

| Компонент | Статус | Описание |
|-----------|--------|----------|
| **Frontend UI** | ✅ Готово | Modern dark mode с градиентами |
| **Backend интеграция** | ✅ Готово | Rust + yt-dlp работает |
| **Получение инфо** | ✅ Готово | Название, превью, автор, длительность |
| **Скачивание** | ✅ Готово | Все качества + MP3 |
| **Прогресс-бар** | ✅ Готово | Real-time обновления |
| **Chrome cookies** | ✅ Готово | Автоматическое извлечение |
| **File picker** | ✅ Готово | Выбор папки сохранения |
| **Версионирование** | ✅ Готово | Python скрипт + Makefile |
| **Документация** | ✅ Готово | Полный набор .md файлов |

### 🎯 Возможности для улучшения

| Фича | Приоритет | Описание |
|------|-----------|----------|
| **Batch скачивание** | 🟡 Средний | Скачивание нескольких видео |
| **Плейлисты** | 🟡 Средний | Скачивание целиx плейлистов |
| **История** | 🟢 Низкий | Сохранение истории скачивания |
| **Темы** | 🟢 Низкий | Light mode опция |
| **Другие платформы** | 🟢 Низкий | Vimeo, TikTok поддержка |
| **Настройки** | 🟡 Средний | Формат по умолчанию, язык |

### 📋 Возможности

- [x] Базовое скачивание видео
- [x] Выбор качества
- [x] Прогресс-бар
- [x] Chrome cookies
- [x] Только аудио (MP3)
- [x] Выбор папки
- [x] Modern UI
- [x] Network Status Bar
- [x] English Localization
- [ ] Batch download
- [ ] Плейлисты
- [ ] История
- [ ] Настройки
- [ ] Subtitles download

---

## 🎓 Полезные команды

### Разработка

```bash
# Dev режим (hot-reload)
make dev

# Или напрямую
cd youtube-downloader
npm run tauri dev

# Проверка версии
make version-status

# Сборка release
make build
```

### Версионирование

```bash
# Увеличить patch версию (0.1.0 → 0.1.1)
make version-bump-patch

# Увеличить minor версию (0.1.0 → 0.2.0)
make version-bump-minor

# Установить конкретную версию
make version-set v=1.0.0

# Синхронизировать все файлы версий
make version-sync
```

### Тестирование

```bash
# Запустить Rust тесты
cd youtube-downloader/src-tauri
cargo test

# Проверить код
cargo clippy

# Форматировать
cargo fmt
```

---

## 🔧 Настройка и конфигурация

### tauri.conf.json

Основные настройки приложения:

```json
{
  "productName": "youtube-downloader",
  "version": "0.1.0",
  "identifier": "com.olgazaharova.youtube-downloader",
  "bundle": {
    "active": true,
    "targets": "all",
    "macOS": {
      "minimumSystemVersion": "11.0"
    }
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

### package.json

Frontend зависимости и скрипты:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build"
  }
}
```

---

## 📱 Платформы

### macOS (Основная)

- ✅ Полностью поддерживается
- ✅ Native .app bundle
- ✅ .dmg installer
- ✅ Apple Silicon (M1/M2) + Intel

### Windows (Будущее)

- 🔄 В планах
- Требует тестирования yt-dlp на Windows
- Поддержка .exe и .msi установщиков

---

## 📚 Дополнительные ресурсы

### Документация проекта

- [README.md](README.md) — Основная информация
- [BUILD.md](BUILD.md) — Руководство по сборке
- [VERSION_MANAGEMENT.md](VERSION_MANAGEMENT.md) — Управление версиями
- [MACOS_SETUP.md](MACOS_SETUP.md) — Установка для macOS
- [WINDOWS_SETUP.md](WINDOWS_SETUP.md) — Установка для Windows

### Внешние ресурсы

- [Tauri Documentation](https://tauri.app)
- [yt-dlp GitHub](https://github.com/yt-dlp/yt-dlp)
- [Rust Book](https://doc.rust-lang.org/book/)

---

**Дата обновления:** 02.01.2026  
**Автор:** Kurein Maxim
