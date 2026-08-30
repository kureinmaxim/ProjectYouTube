# YouTube Downloader

Современное десктопное приложение для скачивания видео с YouTube на macOS.

![Version](https://img.shields.io/badge/version-1.5.1-blue.svg)
![Platform](https://img.shields.io/badge/platform-macOS-lightgrey.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

## ✨ Возможности

- 🎨 **Современный UI** — Dark mode с градиентами и анимациями
- 📥 **Простое скачивание** — Вставьте ссылку и скачайте
- 🎬 **Выбор качества** — Best, 1080p, 720p, 480p, MP3
- 📊 **Прогресс в реальном времени** — Визуальный прогресс-бар
- 🔐 **Chrome cookies** — Автоматическая поддержка для приватных видео
- 📁 **Выбор папки** — Сохраняйте куда удобно
- 🛡️ **Auto Fallback** — Обход блокировок YouTube (android/tv/web клиенты)
- 🌐 **Network Status** — Автоопределение режима (TUN/SOCKS5/Direct)
- 🔍 **Умная диагностика** — Внешний IP, проверка прокси, свежесть yt-dlp
- 🖥️ **System Proxy** — Автоопределение HTTP/SOCKS через macOS
- 📴 **Offline UI** — Интерфейс рисуется без сети: шрифты и стили внутри приложения

## 🚀 Быстрый старт

### Установка зависимостей

```bash
cd youtube-downloader
npm install
```

### Запуск в dev режиме

```bash
# Из корня проекта
make dev

# Или из папки youtube-downloader
cd youtube-downloader
npm run tauri dev
```

### Сборка для релиза

```bash
# Из корня проекта
make build

# Результат:
# youtube-downloader/src-tauri/target/release/bundle/macos/youtube-downloader.app
# youtube-downloader/src-tauri/target/release/bundle/dmg/*.dmg

# Установить в /Applications
make install-app
```

> ⚠️ В Dock закрепляйте копию из `/Applications` (`make install-app`), а не
> `.app` из `target/` и не dev-сборку: первая удаляется при каждой пересборке,
> второй нужен запущенный dev-сервер.
> Если приложение открывается пустым белым окном — разбор причин в
> [MACOS_SETUP.md](MACOS_SETUP.md#-приложение-открывается-пустым-белым-окном).

## 📋 Требования

- **macOS** 11.0+
- **Node.js** 18+
- **Rust** 1.70+
- **yt-dlp** (для скачивания)
- **ffmpeg** (для склейки видео+аудио)
- **Google Chrome** (опционально, для cookies)

### Установка yt-dlp

```bash
brew install yt-dlp
```

### Установка ffmpeg

```bash
brew install ffmpeg
```

## 🛠️ Команды разработки

| Команда | Описание |
|---------|----------|
| `make help` | Показать все доступные команды |
| `make dev` | Запустить в dev режиме |
| `make build` | Собрать релизную версию |
| `make install-app` | Установить собранный `.app` в `/Applications` |
| `make run` | Запустить установленное приложение |
| `make run-verbose` | Запустить с логами в терминале (диагностика пустого окна) |
| `make clean` | Очистить артефакты сборки |
| `make test` | Запустить тесты |
| `make lint` | Проверить код |

## 📦 Управление версиями

```bash
# Проверить текущую версию
make version-status

# Синхронизировать версии во всех файлах
make version-sync

# Увеличить версию
make version-bump-patch    # 0.1.0 → 0.1.1
make version-bump-minor    # 0.1.0 → 0.2.0
make version-bump-major    # 0.1.0 → 1.0.0

# Установить конкретную версию
make version-set v=1.0.0
```

Подробнее: [VERSION_MANAGEMENT.md](VERSION_MANAGEMENT.md)

## 📚 Документация

- [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md) - Полный обзор проекта и архитектуры
- [ARCHITECTURE_2026.md](ARCHITECTURE_2026.md) - Production-grade архитектура
- [YOUTUBE_BLOCKING.md](YOUTUBE_BLOCKING.md) - Решение проблем с блокировками YouTube
- [MACOS_SETUP.md](MACOS_SETUP.md) - Быстрый старт для macOS
- [WINDOWS_SETUP.md](WINDOWS_SETUP.md) - Быстрый старт для Windows
- [BUILD.md](BUILD.md) - Руководство по сборке и разработке
- [VERSION_MANAGEMENT.md](VERSION_MANAGEMENT.md) - Управление версиями

## 🏗️ Структура проекта

```
ProjectYouTube/
├── youtube-downloader/       # Главное приложение
│   ├── src/                  # Frontend (TypeScript, CSS)
│   │   ├── main.ts          # Логика приложения
│   │   └── styles.css       # Стили
│   ├── src-tauri/           # Backend (Rust)
│   │   └── src/
│   │       ├── lib.rs       # Главный модуль
│   │       ├── ytdlp.rs     # Интеграция с yt-dlp + fallback
│   │       └── downloader/  # Модуль скачивания
│   │           ├── utils.rs       # Network detection (TUN/SOCKS5/IP)
│   │           ├── tools.rs       # Управление yt-dlp
│   │           ├── commands.rs    # Tauri команды
│   │           └── backends/      # Download backends
│   └── index.html           # HTML интерфейс
├── scripts/                  # Утилиты
│   └── version.py           # Управление версиями
├── Makefile                 # Команды разработки
└── *.md                     # Документация
```

## 🎯 Использование

1. **Запустите приложение**
2. **Вставьте YouTube URL** в поле ввода
3. **Нажмите "Получить информацию"** - увидите превью видео
4. **Выберите качество** (по умолчанию 720p)
5. **Выберите папку** для сохранения
6. **Нажмите "Скачать видео"**
7. **Наблюдайте прогресс** скачивания
8. **Готово!** Видео в выбранной папке

## 🧩 PO Token и выбор клиента

В блоке **Tools** доступны расширенные настройки YouTube (если загрузка «висит» из-за SABR/блокировок):

- **Player client** — принудительно выбрать клиент yt-dlp.
  - `Auto` — рекомендуемый режим, включает встроенные стратегии и fallback.
  - `all` — пробует все клиенты (часто помогает при блокировках).
- **PO Token** — вставьте PO Token (если YouTube требует его для GVS).
- **PO Token client** — для какого клиента использовать токен (обычно `mweb`).

![PO Token UI](youtube-downloader/src/assets/po-token-ui.png)

### Как применять

1. Откройте **Tools** и выберите **Player client** (`Auto` или `all`).
2. При наличии PO Token вставьте его и выберите **PO Token client** (`mweb`).
3. Запустите скачивание — выбранные параметры будут использованы автоматически.

> Рекомендация: если видите сообщения про SABR/403, попробуйте `all` и/или `PO Token (mweb)`.
>
> Гайд по PO Token: https://github.com/yt-dlp/yt-dlp/wiki/PO-Token-Guide

### Как получить PO Token (mweb)

Кратко по официальному гайду yt-dlp:

1. Откройте **YouTube Music** в браузере и войдите в аккаунт.
2. Откройте DevTools → **Network**.
3. В фильтре запросов выберите `v1/player`.
4. Воспроизведите любое видео, появится запрос `player`.
5. В теле запроса найдите поле `serviceIntegrityDimensions.poToken` и скопируйте значение.
6. В приложении вставьте токен в поле **PO Token**, выберите **PO Token client = mweb**.

### Памятка по типовым ошибкам

- **SABR / 403 / Forbidden** — чаще всего нужен другой клиент (`all`) и/или PO Token (`mweb`).
- **Network timeout / timed out** — попробуйте VPN/Proxy или смените IP.
- **Requested format is not available** — выберите `Best` или `audio`.
- **Private / age-restricted** — включите cookies (Chrome) или используйте cookies.txt.

## 🔧 Технологии

- **Tauri** 2.0 - Desktop framework
- **Rust** - Backend
- **TypeScript** - Frontend логика
- **Vite** - Dev server
- **yt-dlp** - Скачивание видео

## 📝 Лицензия

MIT

## 👤 Автор

Kurein Maxim

---

**Приятного использования! 🚀**
