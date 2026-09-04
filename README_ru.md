# YouTube Downloader v1.6.5

Язык: [English](README.md) · **Русский**

<p align="center">
  <img src="docs/images/icon.png" alt="YouTube Downloader" width="96">
</p>

<p align="center">
  <strong>Современное десктопное приложение для скачивания видео с YouTube на macOS и Windows.</strong><br>
  Tauri + Rust + yt-dlp — без облака и аккаунта.
</p>

<p align="center">
  <a href="https://github.com/kureinmaxim/ProjectYouTube/releases"><img src="https://img.shields.io/github/v/release/kureinmaxim/ProjectYouTube?style=flat-square" alt="Latest release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-0F766E?style=flat-square" alt="MIT License"></a>
  <img src="https://img.shields.io/badge/tauri-2-FFC131?style=flat-square" alt="Tauri 2">
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows-1F2937?style=flat-square" alt="macOS and Windows">
</p>

<p align="center">
  <img src="docs/images/01-main.png" alt="Главное окно Downloader — ссылка, Get Info, Tools, журнал" width="720">
</p>

## Возможности

- **Современный UI** — dark mode с градиентами и анимациями
- **Простое скачивание** — вставьте ссылку и скачайте
- **Выбор качества** — Best, 1080p, 720p, 480p, MP3
- **Прогресс в реальном времени** — визуальный прогресс-бар
- **Chrome cookies** — автоматическая поддержка для приватных видео
- **Выбор папки** — сохраняйте куда удобно
- **Auto Fallback** — обход блокировок YouTube (android / tv / web клиенты)
- **Network Status** — автоопределение режима (TUN / SOCKS5 / Direct)
- **Умная диагностика** — внешний IP, проверка прокси, свежесть yt-dlp
- **System Proxy** — автоопределение HTTP/SOCKS через macOS
- **Offline UI** — шрифты и стили внутри приложения; окно рисуется без сети

<p align="center">
  <img src="docs/images/02-tools.png" alt="Tools — прокси, cookies Chrome, player client, PO Token" width="720">
</p>

## Быстрый старт

```bash
git clone https://github.com/kureinmaxim/ProjectYouTube.git
cd ProjectYouTube
```

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

> В Dock закрепляйте копию из `/Applications` (`make install-app`), а не `.app` из `target/` и не dev-сборку: `target/` удаляется при каждой пересборке, dev-бинарнику нужен запущенный Vite.
> Если приложение открывается пустым белым окном — [docs/MACOS_SETUP_ru.md](docs/MACOS_SETUP_ru.md), раздел «Приложение открывается пустым белым окном».

Windows (Makefile не обязателен):

```powershell
cd youtube-downloader
npm install
npm run tauri dev
```

## Требования

- **macOS** 11.0+ или **Windows** 10/11
- **Node.js** 18+
- **Rust** 1.70+
- **Google Chrome** (опционально, для cookies)

yt-dlp и ffmpeg ставить заранее **не нужно** — откройте панель **Tools** и нажмите **Install**.

### Панель Tools

Оба инструмента показаны там со своей версией и статусом.

**Windows** — Install скачивает бинарь из GitHub releases в
`%LOCALAPPDATA%\youtube-downloader\bin`. Без пакетного менеджера, без прав администратора,
без правки PATH и без перезапуска. ffmpeg весит ~170 МБ, поэтому его прогресс пишется в лог.

**macOS** — Install использует Homebrew:

```bash
brew install yt-dlp ffmpeg
```

Уже стоят из Scoop, Chocolatey, winget, pip или Homebrew? Приложение найдёт и будет
использовать эту копию. **Update** её не перезапишет — назовёт владельца и покажет
команду, которой она обновляется.

## Команды разработки

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

## Управление версиями

```bash
# Проверить текущую версию
make version-status

# Синхронизировать версии во всех файлах
make version-sync

# Увеличить версию
make version-bump-patch    # 1.6.0 → 1.6.1
make version-bump-minor    # 1.6.0 → 1.7.0
make version-bump-major    # 1.6.0 → 2.0.0

# Установить конкретную версию
make version-set v=1.0.0
```

Подробнее: [VERSION_MANAGEMENT_ru.md](VERSION_MANAGEMENT_ru.md)

## Документация

- [Индекс](docs/INDEX_ru.md)
- [BUILD_ru.md](BUILD_ru.md) — сборка и разработка · [EN](BUILD.md)
- [docs/MACOS_SETUP_ru.md](docs/MACOS_SETUP_ru.md) — macOS · [EN](MACOS_SETUP.md)
- [docs/WINDOWS_SETUP_ru.md](docs/WINDOWS_SETUP_ru.md) — Windows · [EN](WINDOWS_SETUP.md)
- [docs/NETWORK_SETUP_ru.md](docs/NETWORK_SETUP_ru.md) — сеть, VPN, DNS · [EN](NETWORK_SETUP.md)
- [YOUTUBE_BLOCKING.md](YOUTUBE_BLOCKING.md) — SABR, 403, PO Token
- [docs/PROJECT_OVERVIEW_ru.md](docs/PROJECT_OVERVIEW_ru.md) — обзор архитектуры
- [docs/ARCHITECTURE_2026_ru.md](docs/ARCHITECTURE_2026_ru.md) — архитектура обхода блокировок
- [CHANGELOG.md](CHANGELOG.md)

## Структура проекта

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
│   │           ├── platform.rs    # Поиск бинарей по ОС + папка установки
│   │           ├── tools.rs       # Установка и обновление yt-dlp / ffmpeg
│   │           ├── commands.rs    # Tauri команды
│   │           └── backends/      # Download backends
│   └── index.html           # HTML интерфейс
├── scripts/                  # Утилиты
│   └── version.py           # Управление версиями
├── Makefile                 # Команды разработки
└── docs/                    # Документация и скриншоты
```

## Использование

1. **Запустите приложение**
2. **Вставьте YouTube URL** в поле ввода
3. **Нажмите "Get Info"** — увидите превью видео
4. **Выберите качество** (по умолчанию 720p)
5. **Выберите папку** для сохранения
6. **Нажмите "Download Video"**
7. **Наблюдайте прогресс** скачивания
8. **Готово.** Файл в выбранной папке

## PO Token и выбор клиента

В блоке **Tools** доступны расширенные настройки YouTube (если загрузка «висит» из-за SABR/блокировок):

- **Player client** — принудительно выбрать клиент yt-dlp.
  - `Auto` — рекомендуемый режим, включает встроенные стратегии и fallback.
  - `all` — пробует все клиенты (часто помогает при блокировках).
- **PO Token** — вставьте PO Token (если YouTube требует его для GVS).
- **PO Token client** — для какого клиента использовать токен (обычно `mweb`).

### Как применять

1. Откройте **Tools** и выберите **Player client** (`Auto` или `all`).
2. При наличии PO Token вставьте его и выберите **PO Token client** (`mweb`).
3. Запустите скачивание — выбранные параметры будут использованы автоматически.

> Если видите сообщения про SABR/403, попробуйте `all` и/или **PO Token (mweb)**.
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
- **Network timeout / timed out** — попробуйте VPN/Proxy или смените IP. Если в статус-баре `IP: N/A`, это часто DNS: [docs/NETWORK_SETUP_ru.md](docs/NETWORK_SETUP_ru.md).
- **Requested format is not available** — выберите `Best` или `audio`.
- **Private / age-restricted** — включите cookies (Chrome) или используйте cookies.txt.

## Технологии

- **Tauri** 2.0 — desktop framework
- **Rust** — backend
- **TypeScript** — frontend логика
- **Vite** — dev server
- **yt-dlp** — скачивание видео

## Лицензия

[MIT](LICENSE)

---

Куреин М.Н.
