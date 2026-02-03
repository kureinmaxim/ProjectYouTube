# 🍎 macOS: Быстрый старт

**Version:** 1.4.1 | **Updated:** 2026-01-07

Краткое руководство для первой сборки YouTube Downloader на macOS.

## ✅ Чеклист установки

### 1. Homebrew
- [ ] Установить Homebrew: `/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"`
- [ ] Проверить: `brew --version`

### 2. Rust
- [ ] Установить rustup: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- [ ] Выбрать default installation
- [ ] Перезапустить терминал
- [ ] Проверить: `rustc --version` (должно быть 1.70+)

### 3. Node.js
- [ ] Установить через Homebrew: `brew install node`
- [ ] Проверить: `node --version` (должно быть v18+)
- [ ] Проверить: `npm --version` (должно быть 8+)

### 4. Python (для скриптов версионирования)
- [ ] Установить через Homebrew: `brew install python@3.11`
- [ ] Проверить: `python3 --version` (должно быть 3.10+)

### 5. yt-dlp (единственный инструмент для скачивания)
- [ ] Установить через Homebrew: `brew install yt-dlp`
- [ ] Или вручную: 
  ```bash
  curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o ~/bin/yt-dlp
  chmod +x ~/bin/yt-dlp
  ```
- [ ] Проверить: `yt-dlp --version`
- [ ] **Важно:** Держите yt-dlp актуальным (`brew upgrade yt-dlp`) — приложение показывает свежесть версии

### 6. ffmpeg (склейка видео+аудио)
- [ ] Установить через Homebrew: `brew install ffmpeg`
- [ ] Проверить: `ffmpeg -version`

### 7. Google Chrome (опционально, для cookies)
- [ ] Скачать и установить с [google.com/chrome](https://www.google.com/chrome/)
- [ ] Авторизоваться на YouTube для доступа к приватным видео

## 🚀 Первая сборка

Откройте Terminal в папке проекта:

```bash
# 1. Перейдите в проект
cd /Users/olgazaharova/Project/ProjectYouTube

# 2. Проверьте установку всех инструментов
rustc --version    # Должно быть 1.70+
node --version     # Должно быть v18+
npm --version      # Должно быть 8+
python3 --version  # Должно быть 3.10+
yt-dlp --version   # Должна показаться версия
ffmpeg -version    # Должна показаться версия

# 3. Установите npm зависимости
cd youtube-downloader
npm install

# 4. Первая сборка Rust (может занять несколько минут)
npm run tauri build

# Результат будет в:
# src-tauri/target/release/bundle/macos/youtube-downloader.app
# src-tauri/target/release/bundle/dmg/*.dmg
```

## 🔍 Проверка

После успешной сборки:

```bash
# Проверьте наличие артефактов
ls -lh src-tauri/target/release/bundle/macos/youtube-downloader.app
ls -lh src-tauri/target/release/bundle/dmg/*.dmg

# Запустите приложение
open src-tauri/target/release/bundle/macos/youtube-downloader.app
```

## ⚡ Быстрые команды (через Makefile)

```bash
# Dev режим (горячая перезагрузка) - для разработки
cd /Users/olgazaharova/Project/ProjectYouTube
make dev

# Production build - для релиза
make build

# Проверка версии
make version-status

# Очистка артефактов
make clean
```

## 🎨 Режим разработки

Для ежедневной работы используйте dev режим:

```bash
cd /Users/olgazaharova/Project/ProjectYouTube
make dev

# Или напрямую:
cd youtube-downloader
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
| `command not found: rustc` | Перезапустите терминал после установки rustup |
| `command not found: npm` | Установите Node.js: `brew install node` |
| `command not found: yt-dlp` | Установите: `brew install yt-dlp` |
| `Permission denied` | Проверьте права доступа или используйте `chmod +x` |
| `xcrun: error` | Установите Xcode Command Line Tools: `xcode-select --install` |
| `Chrome cookies не работают` | Убедитесь что Chrome установлен и вы авторизованы на YouTube |
| `Failed to compile` | Очистите кеш: `cd youtube-downloader && cargo clean` |

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

## 📚 Подробности

- Полное руководство по сборке: [BUILD.md](BUILD.md)
- Управление версиями: [VERSION_MANAGEMENT.md](VERSION_MANAGEMENT.md)
- Основная документация: [README.md](README.md)

## 💡 Советы

- **Для разработки** всегда используйте `make dev` - это быстрее
- **Для релиза** используйте `make build` - создаст .app и .dmg
- **Для обновления версии** используйте `make version-bump-*`
- **При проблемах** сначала попробуйте `make clean`, потом пересоберите

## 🎉 Готово!

Теперь у вас работает YouTube Downloader! Приятного использования! 🚀
