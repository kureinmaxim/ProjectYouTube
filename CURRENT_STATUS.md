# YouTube Downloader - Текущее состояние и проблемы

**Дата:** 02.01.2026  
**Автор:** Kurein Maxim

---

## ✅ Что успешно реализовано

### 1. Базовая архитектура приложения
- ✅ **Tauri 2.0** приложение с Rust backend
- ✅ **TypeScript** frontend с modern UI
- ✅ **Dark mode** интерфейс с градиентами и анимациями
- ✅ Интеграция с **yt-dlp** через системные команды

### 2. Функциональность
- ✅ Получение информации о видео (название, автор, длительность, превью)
- ✅ Выбор качества (Best, 1080p, 720p, 480p, MP3)
- ✅ Выбор папки сохранения
- ✅ Прогресс-бар скачивания
- ✅ **Терминал-лог** с real-time отображением процесса
- ✅ Chrome cookies поддержка

### 3. UI компоненты
- ✅ URL input с валидацией
- ✅ Video preview card (thumbnail + metadata)
- ✅ Quality selector
- ✅ Folder picker (Tauri dialog)
- ✅ Progress bar с процентами
- ✅ Status messages (success/error)
- ✅ **Collapsible terminal log** с цветовой кодировкой

### 4. Техническая реализация
- ✅ Автоматический поиск `yt-dlp` в стандартных путях:
  - `/opt/homebrew/bin/yt-dlp` (Apple Silicon)
  - `/usr/local/bin/yt-dlp` (Intel Mac)
  - `/usr/bin/yt-dlp`
- ✅ Rust функции: `get_video_info()`, `download_video()`, `get_formats()`
- ✅ Event system для прогресса
- ✅ Логирование всех операций в терминал

### 5. Документация
- ✅ `README.md` - основная документация
- ✅ `BUILD.md` - руководство по сборке
- ✅ `VERSION_MANAGEMENT.md` - управление версиями
- ✅ `MACOS_SETUP.md` - установка для macOS
- ✅ `WINDOWS_SETUP.md` - установка для Windows
- ✅ `PROJECT_OVERVIEW.md` - полный обзор архитектуры
- ✅ `GIT_SETUP.md` - Git инструкции

### 6. Инфраструктура
- ✅ Git репозиторий инициализирован
- ✅ GitHub репозиторий: https://github.com/kureinmaxim/ProjectYouTube
- ✅ Release v0.1.0 создан
- ✅ Python скрипт для версионирования (`scripts/version.py`)
- ✅ Makefile с командами автоматизации

---

## ❌ Текущая проблема: YouTube API таймауты

### Симптомы
При попытке получить информацию о видео приложение зависает на 20+ секунд, затем возвращает ошибку:

```
yt-dlp error: ERROR: [youtube] oDQFh40rsBI: Unable to download API page:
HTTPSConnectionPool(host='www.youtube.com', port=443): Read timed out. 
(read timeout=20.0) (caused by TransportError("HTTPSConnectionPool
(host='www.youtube.com', port=443): Read timed out. (read timeout=20.0)"))
```

### Что уже попробовали

#### 1. Убрали Chrome cookies из get_video_info
```rust
// Было:
.args(["--dump-json", "--no-playlist", "--cookies-from-browser", "chrome", &url])

// Стало:
.args(["--dump-json", "--no-playlist", "--no-warnings", &url])
```
**Результат:** Не помогло, все равно таймаут

#### 2. Добавили Android player client (из рабочих команд YouTube.md)
```rust
.args([
    "--dump-json",
    "--no-playlist", 
    "--no-warnings",
    "--extractor-args", "youtube:player_client=android",
    &url,
])
```
**Результат:** Не помогло, таймаут остался

#### 3. Увеличили socket timeout
```rust
.args([
    "--dump-json",
    "--no-playlist",
    "--no-warnings",
    "--socket-timeout", "30",
    "--extractor-args", "youtube:player_client=android",
    &url,
])
```
**Результат:** Таймаут увеличился до 30 секунд, но все равно происходит

#### 4. Обновили yt-dlp
```bash
brew upgrade yt-dlp
# 2025.9.23 → 2025.12.8
```
**Результат:** Не помогло

### Рабочие команды из YouTube.md

Эти команды работали раньше (из файла `/Users/olgazaharova/Project/ProjectYouTube/YouTube.md`):

```bash
# Android client
yt-dlp --extractor-args "youtube:player_client=android" 'URL'

# iOS client  
yt-dlp --cookies-from-browser chrome --extractor-args "youtube:player_client=ios" 'URL'

# Через Python module
python3 -m yt_dlp --cookies-from-browser chrome 'URL'
```

### Текущий код (ytdlp.rs)

```rust
#[tauri::command]
pub async fn get_video_info(url: String) -> Result<VideoInfo, String> {
    let ytdlp_path = find_ytdlp();
    
    let output = Command::new(&ytdlp_path)
        .args([
            "--dump-json",
            "--no-playlist",
            "--no-warnings",
            "--socket-timeout", "30",
            "--extractor-args", "youtube:player_client=android",
            &url,
        ])
        .output()
        .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp error: {}", error));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // ... parse and return VideoInfo
}
```

---

## 🔍 Гипотезы о причине проблемы

### 1. YouTube блокирует IP адрес
- Возможно слишком много запросов с одного IP
- YouTube определяет автоматизацию

### 2. Нужны дополнительные headers
- User-Agent
- Cookies обязательны даже для get_info?

### 3. Проблема с сетью/DNS
- Медленный DNS lookup
- Проблемы с HTTPS соединением

### 4. YouTube изменил API
- Новые ограничения в 2025
- Android/iOS client больше не работают

---

## 💡 Возможные решения для проверки

### Option 1: Использовать youtube-dl вместо yt-dlp
```bash
youtube-dl --dump-json "URL"
```

### Option 2: Добавить User-Agent
```rust
"--user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
```

### Option 3: Использовать yt-dlp через Python module
```rust
Command::new("python3")
    .args(["-m", "yt_dlp", "--dump-json", &url])
```

### Option 4: Добавить retry логику с экспоненциальной задержкой
```rust
for attempt in 1..=3 {
    match try_get_video_info(&url) {
        Ok(info) => return Ok(info),
        Err(e) if attempt < 3 => {
            std::thread::sleep(Duration::from_secs(2_u64.pow(attempt)));
            continue;
        }
        Err(e) => return Err(e),
    }
}
```

### Option 5: Использовать YouTube Data API v3 напрямую
- Требует API key
- Лимит: 10,000 запросов/день
- Не требует yt-dlp

### Option 6: Proxy/VPN
```rust
"--proxy", "socks5://127.0.0.1:1080"
```

### Option 7: Кеширование результатов
- Сохранять VideoInfo в локальную базу
- Избежать повторных запросов к YouTube

### Option 8: Попробовать другие extractor args
```bash
--extractor-args "youtube:player_client=web"
--extractor-args "youtube:player_client=mweb"  # mobile web
--extractor-args "youtube:player_client=tv"
```

---

## 🧪 Команды для тестирования

### Прямой тест yt-dlp (в терминале)
```bash
# Test 1: Android client
time yt-dlp --extractor-args "youtube:player_client=android" --dump-json --no-warnings "https://youtu.be/dQw4w9WgXcQ" | jq -r '.title'

# Test 2: iOS client
time yt-dlp --extractor-args "youtube:player_client=ios" --dump-json --no-warnings "https://youtu.be/dQw4w9WgXcQ" | jq -r '.title'

# Test 3: Web client
time yt-dlp --extractor-args "youtube:player_client=web" --dump-json --no-warnings "https://youtu.be/dQw4w9WgXcQ" | jq -r '.title'

# Test 4: С cookies
time yt-dlp --cookies-from-browser chrome --dump-json --no-warnings "https://youtu.be/dQw4w9WgXcQ" | jq -r '.title'

# Test 5: Через Python
time python3 -m yt_dlp --dump-json --no-warnings "https://youtu.be/dQw4w9WgXcQ" | jq -r '.title'
```

### Проверка сетевого соединения
```bash
# Test DNS lookup
time nslookup www.youtube.com

# Test HTTPS connection
time curl -I https://www.youtube.com

# Check if YouTube accessible
curl -w "%{time_total}\n" -o /dev/null -s "https://www.youtube.com"
```

---

## 📊 Системная информация

**OS:** macOS (iMac mini)  
**yt-dlp version:** 2025.12.8  
**yt-dlp path:** `/opt/homebrew/bin/yt-dlp`  
**Python:** 3.x  
**Node.js:** 18+  
**Rust:** 1.70+

---

## 📝 Логи ошибок

### Из терминала приложения
```
[7:06:01 PM] Получение информации о видео: https://youtu.be/oDQFh40rsBI?si=ZtZq6nJrl_xoEChT
[7:06:01 PM] Выполняется команда yt-dlp...
[7:08:58 PM] Ошибка: yt-dlp error: ERROR: [youtube] oDQFh40rsBI: Unable to download API page:
HTTPSConnectionPool(host='www.youtube.com', port=443): Read timed out. (read timeout=20.0)
(caused by TransportError("HTTPSConnectionPool(host='www.youtube.com', port=443): 
Read timed out. (read timeout=20.0)"))
```

**Время выполнения:** ~2 минуты 57 секунд до таймаута

---

## 🎯 Следующие шаги для исследования

1. **Протестировать все варианты player_client** напрямую в терминале
2. **Проверить работает ли YouTube API вообще** с этого IP
3. **Попробовать добавить verbose режим** `--verbose` для диагностики
4. **Проверить работу через VPN/Proxy**
5. **Рассмотреть альтернативы yt-dlp** (pytube, youtube-dl, YouTube Data API v3)

---

## 🔗 Полезные ссылки

- yt-dlp GitHub: https://github.com/yt-dlp/yt-dlp
- yt-dlp issues: https://github.com/yt-dlp/yt-dlp/issues
- YouTube Data API: https://developers.google.com/youtube/v3
- Проект на GitHub: https://github.com/kureinmaxim/ProjectYouTube

---

**Создано:** 02.01.2026 19:12  
**Последнее обновление:** 02.01.2026 19:12
