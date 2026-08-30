# 🍎 macOS: Быстрый старт

**Version:** 1.5.1 | **Updated:** 2026-08-30

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

# Установка собранного .app в /Applications (именно его закрепляйте в Dock)
make install-app

# Запуск установленного приложения
make run

# Запуск с логами в терминале (если окно открывается пустым)
make run-verbose

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
| `Could not resolve host` / `IP: N/A` в приложении | Сломан системный DNS — [NETWORK_SETUP.md](NETWORK_SETUP.md) |
| Пустое белое окно при запуске | См. раздел «Приложение открывается пустым белым окном» |

## ⬜ Приложение открывается пустым белым окном

Окно с заголовком «Downloader» появляется, но внутри пусто и бело. Rust-часть
запустилась, а веб-интерфейс не отрисовался. Интерфейс тёмный, поэтому
**белый фон = страница не нарисовалась вообще** (при сломанных стилях был бы
виден чёрный текст на белом).

### Причина 1: сеть блокирует внешний шрифт (исправлено в v1.5.1)

До версии 1.5.1 `index.html` грузил шрифт Inter с `fonts.googleapis.com`.
Такой `<link rel="stylesheet">` **блокирует отрисовку**: пока он не загрузится
или не отвалится с ошибкой, браузерный движок не рисует ничего.

Дальше всё зависит от того, *как* сеть отказывает:

| Поведение сети | Что видно |
|----------------|-----------|
| Google доступен | интерфейс за ~0,15 с |
| DNS сразу отвечает «нет такого хоста» | интерфейс за ~0,15 с (шрифт системный) |
| Запрос **виснет** (DPI-фильтрация, мёртвый прокси, VPN-туннель без выхода) | **белое окно, пока не истечёт таймаут** |

Третий случай — типичная ситуация у российских провайдеров и при
полуподнятом VPN. Отсюда и «на одном Wi-Fi работает, на другом нет»:
меняется не приложение, а то, как сеть отвечает на запрос к Google.

**Исправление:** шрифт Inter теперь лежит внутри приложения
(`@fontsource-variable/inter`), интерфейсу больше не нужна сеть, чтобы
нарисоваться. Обновитесь и пересоберите:

```bash
cd /Users/olgazaharova/Project/ProjectYouTube
git pull
make build
make install-app
```

**Срочный обходной путь для уже установленной старой сборки** (пересборка не
нужна) — заставить запрос падать сразу вместо зависания:

```bash
printf "0.0.0.0 fonts.googleapis.com\n0.0.0.0 fonts.gstatic.com\n" | sudo tee -a /etc/hosts
sudo dscacheutil -flushcache; sudo killall -HUP mDNSResponder
```

Интерфейс появится сразу, шрифт будет системный. После обновления на 1.5.1+
эти строки из `/etc/hosts` можно убрать.

### Причина 2: в Dock закреплена dev-сборка

`make dev` запускает бинарник из `target/debug/`, который берёт интерфейс с
Vite dev-сервера `http://localhost:1420`. Закрепите такой бинарник в Dock,
запустите без dev-сервера — грузить нечего, снова пустое окно.

Проверьте, что именно запущено (пока пустое окно открыто):

```bash
ps -Ao pid,command | grep -i youtube-downloader | grep -v grep
```

- путь содержит `/target/debug/` → dev-сборка, нужен запущенный `make dev`;
- путь содержит `/Applications/` или `/target/release/` → релиз, смотрите причины 1 и 3.

### Причина 3: иконка в Dock указывает внутрь `target/`

`.app` внутри `src-tauri/target/release/bundle/macos/` удаляется при каждой
пересборке и при `make clean` — ссылка в Dock ведёт в никуда или на
недособранный бандл.

**Решение:** `make install-app` и закреплять копию из `/Applications`,
она переживает пересборки.

### Что смотреть, если не помогло

```bash
# запуск в терминале — ошибки старта видны сразу
make run-verbose

# в сборке не должно остаться внешних ресурсов
make check-assets

# цел ли бандл
ls -l /Applications/youtube-downloader.app/Contents/MacOS/
```

Если интерфейс загрузился, но не запустился JS, приложение само пишет об этом
в окне («Interface did not start») вместо пустого экрана.

## 🌐 Системный DNS не резолвит (VPN / Tailscale exit node)

Одна поломка, три разных симптома — легко принять их за три разные проблемы:

| Где проявляется | Что видно |
|-----------------|-----------|
| Приложение | `Network timeout (possible IP throttling)`, в статус-баре `IP: N/A` |
| Сборка | `Could not resolve host: static.crates.io`, `make build` висит на crates |
| До версии 1.5.1 | пустое белое окно (запрос шрифта уходил в тот же мёртвый DNS) |

Проверка за 30 секунд:

```bash
dig +time=3 +tries=1 @1.1.1.1 www.youtube.com +short
```
```bash
curl -sS -o /dev/null -w "%{http_code}\n" --max-time 10 https://www.youtube.com
```

**Явный резолвер отдаёт адреса, а `curl` пишет `Resolving timed out`** — сеть в
порядке, сломан системный резолвер. Смотрим, кто его подменил:

```bash
scutil --dns | head -8
```

Строка `if_index : NN (utunN)` означает, что DNS навязан VPN-туннелем, и адреса
из `networksetup -setdnsservers Wi-Fi …` игнорируются — у туннельного резолвера
приоритет выше.

Разбор причин, где какая настройка живёт и как починить насовсем:
**[NETWORK_SETUP.md](NETWORK_SETUP.md)**.

Быстрый обходной путь, если нужно собрать проект прямо сейчас:

```bash
sudo tailscale set --exit-node=
```
```bash
sudo dscacheutil -flushcache; sudo killall -HUP mDNSResponder
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

## 📚 Подробности

- Сеть, VPN и DNS: [NETWORK_SETUP.md](NETWORK_SETUP.md)
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
