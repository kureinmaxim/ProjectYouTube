# 🪟 Windows: Быстрый старт

Язык: [English](../WINDOWS_SETUP.md) · **Русский**

**Version:** 1.6.4 | **Updated:** 2026-09-04

Краткое руководство для первой сборки YouTube Downloader на Windows.

## ✅ Чеклист установки

### 1. Rust
- [ ] Скачать [rustup-init.exe](https://rustup.rs/)
- [ ] Установить (выбрать default options)
- [ ] Перезапустить PowerShell
- [ ] Проверить: `rustc --version` (должно быть 1.70+)

### 2. Node.js (и для сборки, и для скачивания с YouTube)
- [ ] Скачать LTS с [nodejs.org](https://nodejs.org/)
- [ ] Установить с опцией "Add to PATH"
- [ ] Перезапустить PowerShell
- [ ] Проверить: `node --version` (должно быть v18+)
- [ ] Проверить: `npm --version` (должно быть 8+)

yt-dlp 2026 для YouTube нужен JavaScript runtime (n-challenge). По умолчанию
включается только Deno. Приложение само передаёт `--js-runtimes` для Node / Deno /
Bun, если находит. Без этого Get Info может пройти, а все стратегии скачивания
падают с `Requested format is not available` — это не блокировка YouTube.
Альтернатива — Deno (`scoop install deno`).

### 3. Python (для скриптов версионирования)
- [ ] Скачать 3.10+ с [python.org](https://www.python.org/downloads/)
- [ ] Установить с опцией "Add Python to PATH"
- [ ] Перезапустить PowerShell
- [ ] Проверить: `python --version` или `py --version`

### 4. yt-dlp и ffmpeg — их ставит само приложение

Перед первым запуском делать ничего не нужно. Запустите приложение, откройте панель
**Tools** и нажмите **Install** напротив каждого инструмента. Бинари скачиваются из
GitHub releases в `%LOCALAPPDATA%\youtube-downloader\bin` — без пакетного менеджера, без
прав администратора, без правки PATH и без перезапуска. ffmpeg весит ~170 МБ, поэтому
его прогресс виден в логе.

ffmpeg склеивает видео и аудио; без него качество выше 720p не скачается.

Хотите поставить сами? Подойдёт любой способ, приложение их найдёт.
Шимы Scoop (`scoop\shims\ffmpeg.exe`) разворачиваются до настоящего бинарника —
не указывайте `--ffmpeg-location` на папку шимов сами.

- [ ] `winget install yt-dlp.yt-dlp` и `winget install Gyan.FFmpeg`
- [ ] `choco install yt-dlp ffmpeg`
- [ ] `scoop install yt-dlp ffmpeg`
- [ ] Или положить `.exe` в любую папку из PATH
- [ ] Проверить: `yt-dlp --version` и `ffmpeg -version`

**Update** в панели Tools заменяет только ту копию, которую приложение поставило само.
Если инструмент пришёл из winget / Chocolatey / Scoop, приложение назовёт владельца и
покажет команду обновления, но качать поверх не станет.

### 5. Google Chrome (опционально, для cookies)
- [ ] Скачать и установить с [google.com/chrome](https://www.google.com/chrome/)
- [ ] Авторизоваться на YouTube для доступа к приватным видео

### 6. Visual Studio Build Tools (для компиляции Rust)
- [ ] Скачать [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
- [ ] Установить "Desktop development with C++"
- [ ] Или установить полную Visual Studio (Community Edition)

## 🚀 Первая сборка

Откройте PowerShell в папке проекта:

```powershell
# 1. Перейдите в корень репозитория
cd путь\к\ProjectYouTube

# 2. Проверьте установку всех инструментов
rustc --version    # Должно быть 1.70+
node --version     # Должно быть v18+
npm --version      # Должно быть 8+
python --version   # Или: py --version
# yt-dlp и ffmpeg здесь необязательны — приложение поставит их само

# 3. Установите npm зависимости
cd youtube-downloader
npm install

# 4. Первая сборка Rust (может занять несколько минут)
npm run tauri build

# Результат будет в:
# src-tauri\target\release\youtube-downloader.exe
# src-tauri\target\release\bundle\msi\youtube-downloader_*_x64_en-US.msi
```

## 🔍 Проверка

После успешной сборки:

```powershell
# Проверьте наличие артефактов
dir src-tauri\target\release\youtube-downloader.exe
dir src-tauri\target\release\bundle\msi\*.msi

# Запустите приложение
.\src-tauri\target\release\youtube-downloader.exe
```

## ⚡ Быстрые команды

```powershell
# Dev режим (горячая перезагрузка) - для разработки
cd youtube-downloader
npm run tauri dev

# Production build - для релиза
npm run tauri build

# Проверка версии (через Python скрипт)
python scripts\version.py status

# Очистка артефактов
cd src-tauri
cargo clean
```

## 🎨 Режим разработки

Для ежедневной работы используйте dev режим:

```powershell
cd C:\Project\ProjectYouTube\youtube-downloader
npm run tauri dev
```

**Что происходит:**
- ✅ Vite dev server с hot-reload на http://localhost:1420/
- ✅ Rust backend компилируется автоматически
- ✅ Изменения в HTML/CSS/JS применяются мгновенно
- ✅ Приложение открывается автоматически

## ❗ Частые проблемы

| Проблема | Решение |
|----------|---------|
| `rustc не найден` | Перезапустите PowerShell после установки rustup |
| `npm не найден` | Установите Node.js и перезапустите PowerShell |
| `yt-dlp не найден` | Откройте **Tools** и нажмите **Install** (или добавьте папку с yt-dlp.exe в PATH) |
| Качество выше 720p не скачивается | Нет ffmpeg — поставьте его в панели **Tools** |
| `ffmpeg is not installed`, но `ffmpeg -version` работает | Шим Scoop. 1.6.3+ разворачивает его. Или поставьте ffmpeg в **Tools** |
| Get Info работает, все стратегии падают (`format is not available` / `[Errno 22]`) | Это не блокировка YouTube. Нужны Node.js или Deno в runtime и сборка 1.6.3+. См. [YOUTUBE_BLOCKING.md](../YOUTUBE_BLOCKING.md) |
| `Python не найден` | Используйте `py` вместо `python` |
| `MSVC не найден` | Установите Visual Studio Build Tools |
| `Permission denied` | Запустите PowerShell от администратора |
| `Chrome cookies не работают` | Убедитесь что Chrome установлен и вы авторизованы на YouTube |
| `Failed to compile` | Очистите кеш: `cargo clean` и попробуйте снова |

## 🔧 Настройка PATH (если нужно)

### Для yt-dlp

```powershell
# Добавить папку с yt-dlp в PATH
$env:Path += ";C:\путь\к\папке\с\yt-dlp"

# Или глобально через System Properties > Environment Variables
```

### Для Python

```powershell
# Если установлен без "Add to PATH"
# System Properties > Environment Variables > Path > Add:
# C:\Users\YOUR_USERNAME\AppData\Local\Programs\Python\Python311\
# C:\Users\YOUR_USERNAME\AppData\Local\Programs\Python\Python311\Scripts\
```

## 🧪 Тестирование приложения

После установки протестируйте основной функционал:

1. **Запустите приложение** (dev или build версию)
2. **Проверьте Network Status Bar** вверху:
   - Mode: `direct` / `proxy` / `vpn`
   - External IP: ваш внешний IP
   - yt-dlp: версия и свежесть
3. **Вставьте YouTube URL**, например: `https://youtu.be/dQw4w9WgXcQ`
4. **Нажмите "Get Info"** — должна появиться информация о видео
5. **Выберите качество** (720p по умолчанию)
6. **Выберите папку** для сохранения
7. **Нажмите "Download"**
8. **Проверьте прогресс-бар** и скачанный файл

### 🛡️ При блокировках YouTube
- Включите **Auto fallback** в Tools → Mode
- Приложение автоматически пробует разные стратегии (android/tv/web клиенты)

## 📦 Создание установщика (Inno Setup)

Для создания .exe установщика:

1. **Установите Inno Setup:**
   ```powershell
   choco install innosetup
   ```

2. **Создайте .iss файл** (пример будет добавлен позже)

3. **Скомпилируйте:**
   ```powershell
   iscc installer\youtube-downloader.iss
   ```

## 📚 Подробности

- Полное руководство по сборке: [../BUILD_ru.md](../BUILD_ru.md)
- Управление версиями: [../VERSION_MANAGEMENT_ru.md](../VERSION_MANAGEMENT_ru.md)
- Основная документация: [../README_ru.md](../README_ru.md)

## 💡 Советы

- **Для разработки** всегда используйте `npm run tauri dev` - это быстрее
- **Для релиза** используйте `npm run tauri build` - создаст .exe и .msi
- **Для обновления версии** используйте `python scripts\version.py bump patch`
- **При проблемах** сначала попробуйте `cargo clean`, потом пересоберите

## 🎉 Готово!

Теперь у вас работает YouTube Downloader на Windows! Приятного использования! 🚀
