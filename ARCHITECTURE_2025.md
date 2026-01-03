# Production-Grade Architecture 2025

**Last Updated:** 2026-01-03

Документ описывает production-grade архитектуру для обхода блокировок YouTube в Tauri-приложении.

---

## 📊 Текущее состояние

### ✅ Что уже реализовано:

1. **Dual backend для get_video_info:**
   - Python module (`python3 -m yt_dlp`) — первичный
   - Native binary (`yt-dlp`) — fallback

2. **Multi-tool fallback для download:**
   - yt-dlp → lux → you-get

3. **Модульная архитектура:**
   - `DownloaderBackend` trait
   - Отдельные бэкенды (Python, Lux, YouGet)
   - Общие модели

4. **Cookies & Proxy поддержка:**
   - `--cookies-from-browser chrome`
   - `--cookies /path/to/cookies.txt`
   - Auto-detect SOCKS5 proxy

---

## 🎯 Рекомендуемая архитектура

### Концепция: Разделение InfoExtractor и Downloader

```
┌─────────────────────────────────────────────────────────────────┐
│                     FRONTEND (TypeScript)                       │
├─────────────────────────────────────────────────────────────────┤
│                    FormatSelector UI                            │
│                         ↓                                       │
│              Unified Format Model                               │
└─────────────────────────────────────────────────────────────────┘
                          ↓ invoke()
┌─────────────────────────────────────────────────────────────────┐
│                     BACKEND (Rust)                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              InfoExtractor Orchestrator                  │   │
│  │                                                         │   │
│  │   ┌─────────────┐         ┌─────────────┐              │   │
│  │   │ Python Mode │ ←auto→  │  CLI Mode   │              │   │
│  │   │  (yt_dlp)   │ switch  │  (yt-dlp)   │              │   │
│  │   └─────────────┘         └─────────────┘              │   │
│  │          ↓                       ↓                      │   │
│  │   ┌─────────────────────────────────────────────────┐  │   │
│  │   │          Unified VideoInfo + Formats             │  │   │
│  │   └─────────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                          ↓                                      │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Download Orchestrator                       │   │
│  │                                                         │   │
│  │   ┌─────────┐   ┌─────────┐   ┌─────────┐             │   │
│  │   │ yt-dlp  │ → │   lux   │ → │ you-get │             │   │
│  │   └─────────┘   └─────────┘   └─────────┘             │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🔧 Компоненты

### 1. InfoExtractor Trait (новый)

```rust
// src-tauri/src/downloader/info_extractor.rs

use async_trait::async_trait;
use crate::downloader::models::{VideoInfo, ExtendedFormat};

/// Режим извлечения информации
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExtractorMode {
    /// Python модуль yt_dlp (надёжнее для YouTube)
    Python,
    /// CLI бинарник yt-dlp (быстрее, не требует Python)
    Cli,
    /// Автовыбор: Python → CLI fallback
    Auto,
}

/// Конфигурация извлечения
#[derive(Debug, Clone)]
pub struct ExtractorConfig {
    pub mode: ExtractorMode,
    pub proxy: Option<String>,
    pub cookies_path: Option<String>,
    pub cookies_from_browser: bool,
    pub timeout_seconds: u32,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            mode: ExtractorMode::Auto,
            proxy: None,
            cookies_path: None,
            cookies_from_browser: true,
            timeout_seconds: 30,
        }
    }
}

/// Расширенная информация о формате
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedFormat {
    pub format_id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f32>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub filesize: Option<u64>,
    pub tbr: Option<f32>,  // Total bitrate
    pub format_note: Option<String>,
}

/// Trait для извлечения информации о видео
#[async_trait]
pub trait InfoExtractor: Send + Sync {
    /// Название экстрактора
    fn name(&self) -> &'static str;
    
    /// Получить информацию о видео
    async fn extract_info(
        &self,
        url: &str,
        config: &ExtractorConfig,
    ) -> Result<VideoInfo, ExtractorError>;
    
    /// Получить все форматы (расширенные)
    async fn extract_formats(
        &self,
        url: &str,
        config: &ExtractorConfig,
    ) -> Result<Vec<ExtendedFormat>, ExtractorError>;
}
```

### 2. Python InfoExtractor

```rust
// src-tauri/src/downloader/extractors/python.rs

pub struct PythonInfoExtractor;

#[async_trait]
impl InfoExtractor for PythonInfoExtractor {
    fn name(&self) -> &'static str { "python-yt-dlp" }
    
    async fn extract_info(
        &self,
        url: &str,
        config: &ExtractorConfig,
    ) -> Result<VideoInfo, ExtractorError> {
        // Использует: python3 -m yt_dlp --dump-json
        // Преимущества:
        // - Лучше обходит bot-protection
        // - Работает с cookies авторизации
        // - Меньше триггерит блокировки YouTube
        todo!()
    }
    
    async fn extract_formats(
        &self,
        url: &str,
        config: &ExtractorConfig,
    ) -> Result<Vec<ExtendedFormat>, ExtractorError> {
        // Парсит все форматы из JSON
        todo!()
    }
}
```

### 3. CLI InfoExtractor

```rust
// src-tauri/src/downloader/extractors/cli.rs

pub struct CliInfoExtractor;

#[async_trait]
impl InfoExtractor for CliInfoExtractor {
    fn name(&self) -> &'static str { "cli-yt-dlp" }
    
    async fn extract_info(
        &self,
        url: &str,
        config: &ExtractorConfig,
    ) -> Result<VideoInfo, ExtractorError> {
        // Использует: /opt/homebrew/bin/yt-dlp --dump-json
        // Преимущества:
        // - Быстрее (нативный бинарник)
        // - Не требует Python
        // - Проще для CI/CD
        todo!()
    }
    
    async fn extract_formats(
        &self,
        url: &str,
        config: &ExtractorConfig,
    ) -> Result<Vec<ExtendedFormat>, ExtractorError> {
        todo!()
    }
}
```

### 4. InfoExtractor Orchestrator

```rust
// src-tauri/src/downloader/extractors/orchestrator.rs

pub struct InfoExtractorOrchestrator {
    python: PythonInfoExtractor,
    cli: CliInfoExtractor,
}

impl InfoExtractorOrchestrator {
    pub fn new() -> Self {
        Self {
            python: PythonInfoExtractor,
            cli: CliInfoExtractor,
        }
    }
    
    /// Извлечь информацию с автоматическим fallback
    pub async fn extract(
        &self,
        url: &str,
        config: ExtractorConfig,
    ) -> Result<VideoInfo, ExtractorError> {
        match config.mode {
            ExtractorMode::Python => {
                self.python.extract_info(url, &config).await
            }
            ExtractorMode::Cli => {
                self.cli.extract_info(url, &config).await
            }
            ExtractorMode::Auto => {
                // Стратегия Auto:
                // 1. Проверить наличие Python + yt_dlp модуля
                // 2. Если есть → Python mode
                // 3. Если Python fail → CLI fallback
                
                if python_available() {
                    match self.python.extract_info(url, &config).await {
                        Ok(info) => return Ok(info),
                        Err(e) => {
                            eprintln!("[Orchestrator] Python failed: {}, trying CLI...", e);
                        }
                    }
                }
                
                self.cli.extract_info(url, &config).await
            }
        }
    }
    
    /// Определить оптимальный режим для данного URL
    pub fn recommend_mode(&self, url: &str) -> ExtractorMode {
        let is_youtube = url.contains("youtube.com") || url.contains("youtu.be");
        
        if is_youtube {
            // YouTube агрессивно блокирует CLI → Python лучше
            ExtractorMode::Python
        } else {
            // Для других сайтов CLI быстрее
            ExtractorMode::Cli
        }
    }
}
```

### 5. Unified Format Selector

```rust
// src-tauri/src/downloader/format_selector.rs

/// Качество видео для UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityOption {
    pub label: String,           // "1080p (1920x1080)"
    pub value: String,           // "1080p"
    pub format_spec: String,     // "bv*[height<=1080]+ba/b[height<=1080]"
    pub estimated_size: Option<String>,
    pub codec_info: Option<String>,
}

pub struct FormatSelector;

impl FormatSelector {
    /// Конвертировать raw форматы в опции для UI
    pub fn build_quality_options(formats: &[ExtendedFormat]) -> Vec<QualityOption> {
        let mut options = Vec::new();
        
        // Best Quality
        if let Some(best) = Self::find_best_video(formats) {
            options.push(QualityOption {
                label: format!("Best Quality ({}x{})", 
                    best.width.unwrap_or(0), 
                    best.height.unwrap_or(0)),
                value: "best".to_string(),
                format_spec: "bv*+ba/best".to_string(),
                estimated_size: Self::format_size(best.filesize),
                codec_info: best.vcodec.clone(),
            });
        }
        
        // Standard resolutions
        for (label, height) in &[("1080p", 1080), ("720p", 720), ("480p", 480), ("360p", 360)] {
            if let Some(fmt) = Self::find_by_height(formats, *height) {
                options.push(QualityOption {
                    label: format!("{} ({}x{})", label, 
                        fmt.width.unwrap_or(0), 
                        fmt.height.unwrap_or(0)),
                    value: label.to_string(),
                    format_spec: format!("bv*[height<={}]+ba/b[height<={}]", height, height),
                    estimated_size: Self::format_size(fmt.filesize),
                    codec_info: fmt.vcodec.clone(),
                });
            }
        }
        
        // Audio only
        if let Some(audio) = Self::find_best_audio(formats) {
            options.push(QualityOption {
                label: "Audio Only (MP3)".to_string(),
                value: "audio".to_string(),
                format_spec: "ba/b".to_string(),
                estimated_size: Self::format_size(audio.filesize),
                codec_info: audio.acodec.clone(),
            });
        }
        
        options
    }
    
    fn find_best_video(formats: &[ExtendedFormat]) -> Option<&ExtendedFormat> {
        formats.iter()
            .filter(|f| f.vcodec.as_ref().map_or(false, |v| v != "none"))
            .max_by_key(|f| f.height.unwrap_or(0))
    }
    
    fn find_by_height(formats: &[ExtendedFormat], target: u32) -> Option<&ExtendedFormat> {
        formats.iter()
            .filter(|f| {
                f.height.map_or(false, |h| {
                    h >= target * 9 / 10 && h <= target * 11 / 10
                })
            })
            .max_by_key(|f| f.filesize.unwrap_or(0))
    }
    
    fn find_best_audio(formats: &[ExtendedFormat]) -> Option<&ExtendedFormat> {
        formats.iter()
            .filter(|f| {
                f.vcodec.as_ref().map_or(false, |v| v == "none") &&
                f.acodec.as_ref().map_or(false, |a| a != "none")
            })
            .max_by_key(|f| f.tbr.map(|b| (b * 100.0) as u32).unwrap_or(0))
    }
    
    fn format_size(bytes: Option<u64>) -> Option<String> {
        bytes.map(|b| {
            let mb = b as f64 / 1_048_576.0;
            if mb >= 1024.0 {
                format!("{:.1} GB", mb / 1024.0)
            } else {
                format!("{:.0} MB", mb)
            }
        })
    }
}
```

---

## 🔄 Когда использовать какой режим

| Ситуация | Режим | Почему |
|----------|-------|--------|
| YouTube (публичное видео) | Python + cookies | Лучше обходит bot-protection |
| YouTube (с авторизацией) | Python + cookies | Обязательны cookies для приватных видео |
| Instagram/TikTok/X | CLI | Быстрее, не требует Python |
| Возраст-ограниченное | Python + cookies | Нужна авторизация |
| За прокси/VPN | Python | Меньше триггерит блокировки |
| CI/CD/Server | CLI | Проще деплой, не зависит от Python |

---

## 📁 Структура файлов (план)

```
src-tauri/src/
├── lib.rs
├── main.rs
├── ytdlp.rs                    # Текущий код (legacy)
└── downloader/
    ├── mod.rs
    ├── errors.rs
    ├── models.rs               # VideoInfo, ExtendedFormat, QualityOption
    ├── traits.rs               # DownloaderBackend
    │
    ├── extractors/             # ← НОВОЕ
    │   ├── mod.rs
    │   ├── traits.rs           # InfoExtractor trait
    │   ├── python.rs           # Python mode
    │   ├── cli.rs              # CLI mode
    │   └── orchestrator.rs     # Auto-switch логика
    │
    ├── format_selector.rs      # ← НОВОЕ: Unified format logic
    │
    ├── backends/
    │   ├── mod.rs
    │   ├── python.rs           # Python yt-dlp downloader
    │   ├── lux.rs
    │   └── youget.rs
    │
    ├── orchestrator.rs         # Download orchestrator
    ├── tools.rs
    └── utils.rs
```

---

## 🚀 План миграции

### Phase 1: Рефакторинг InfoExtractor ✅ (частично сделано)
- [x] Dual backend в `get_video_info` (Python → CLI fallback)
- [ ] Вынести в отдельный trait `InfoExtractor`
- [ ] Создать `InfoExtractorOrchestrator`

### Phase 2: Unified Format Selector
- [ ] Создать `ExtendedFormat` модель
- [ ] Создать `FormatSelector` с умной логикой
- [ ] Интегрировать в UI

### Phase 3: Улучшенная диагностика
- [ ] Определение причины блокировки (403/SABR/PO Token)
- [ ] Авто-рекомендация режима для пользователя
- [ ] Logging для отладки

### Phase 4: Server Mode (опционально)
- [ ] Возможность использовать remote server для download
- [ ] API endpoint на сервере
- [ ] Desktop ↔ Server коммуникация

---

## 🔒 Почему это работает

1. **Python mode = "легитимный пользователь"**
   - YouTube видит тот же user-agent, что и браузер
   - Cookies = авторизованный аккаунт
   - Меньше bot-fingerprinting триггеров

2. **CLI mode = скорость**
   - Нативный бинарник, не нужен Python runtime
   - Проще CI/CD пайплайны
   - Меньше зависимостей

3. **Auto-switch = надёжность**
   - Если Python fail → CLI fallback
   - Если YouTube блокирует → альтернативный tool (lux, you-get)
   - Graceful degradation

---

## 📚 References

- [yt-dlp GitHub](https://github.com/yt-dlp/yt-dlp)
- [PO Token Guide](https://github.com/yt-dlp/yt-dlp/wiki/PO-Token-Guide)
- [SABR Streaming](https://github.com/nichobi/yt-dlp/issues/10046)
- Текущая документация: `YOUTUBE_BLOCKING.md`

