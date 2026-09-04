# 🌐 Сеть: VPN, DNS и настройки Tailscale

Язык: [English](../NETWORK_SETUP.md) · **Русский**

**Version:** 1.6.2 | **Updated:** 2026-08-30

Приложение целиком зависит от разрешения имён: yt-dlp резолвит YouTube,
статус-бар — сервис определения внешнего IP, сборка — npm и crates.io. Когда
системный DNS не отвечает, это выглядит как блокировка YouTube, хотя дело в
настройках самой машины.

Документ описывает, где какая настройка живёт и какая конфигурация рабочая.

---

## Где какая настройка живёт

Главный источник путаницы: настройки Tailscale разнесены по трём разным местам,
и самая нужная — выбор exit node — находится не там, где её ищут.

| Настройка | Где | На что влияет |
|-----------|-----|---------------|
| Global nameservers, Override DNS servers | веб-админка → **DNS** | все устройства тейлнета |
| MagicDNS, Search domains | веб-админка → **DNS** | все устройства тейлнета |
| Use Tailscale DNS settings | приложение → Settings → General | только этот компьютер |
| **Выбор** exit node | **иконка в строке меню** → Exit Node | только этот компьютер |
| Run as exit node | приложение → Settings → Exit Nodes | раздача трафика **с** этого Mac |
| Allow local network access | приложение → Settings → Exit Nodes | доступ в локальную сеть при активном exit node |
| Use Tailscale subnets | приложение → Settings → General | маршруты в анонсированные подсети |

⚠️ **Выбора exit node в окне Settings нет.** Пункт «Run as exit node» означает
противоположное — раздавать свой канал другим устройствам. Нужный переключатель
только в меню строки состояния (иконка Tailscale рядом с часами) → **Exit Node**.

⚠️ Кнопка **Manage…** рядом с «Use Tailscale DNS settings» открывает диалог с
одним тумблером. Сами nameservers оттуда не правятся — они в веб-админке,
о чём диалог и пишет: «DNS can be configured in the admin console».

---

## Рабочая конфигурация

### 1. Веб-админка → DNS

1. **Nameservers** → **Add nameserver** → **Custom** → `1.1.1.1`
2. Вторым (по желанию) — `9.9.9.9`
3. Включить тумблер **Override DNS servers**

Зачем: если глобальный резолвер не задан, при активном exit node Tailscale
подставляет DNS локальной сети — обычно приватный адрес роутера вида
`192.168.x.1`. Через exit node такой адрес недостижим, и каждый запрос висит до
таймаута. Публичный резолвер работает из любой точки туннеля.

Не ставьте приватный адрес (`192.168.x.x`, `10.x.x.x`) в Global nameservers:
он отвечает, только когда вы физически в той же сети.

### 2. Приложение на Mac

Settings → General → **Use Tailscale DNS settings** — включить.

Эквивалент в терминале:

```bash
sudo tailscale set --accept-dns=true
```

### 3. Exit node

Иконка в строке меню → **Exit Node** → нужный узел, либо **None**.

```bash
sudo tailscale set --exit-node=<имя-узла>
sudo tailscale set --exit-node=
```

После любой правки сбросьте кеш резолвера:

```bash
sudo dscacheutil -flushcache; sudo killall -HUP mDNSResponder
```

---

## Проверка

```bash
scutil --dns | head -8
```

В `resolver #1` должен стоять `100.100.100.100` (MagicDNS).

```bash
curl -sS -o /dev/null -w "%{http_code}\n" --max-time 10 https://www.youtube.com
```

`200` — настройка верна. Проверять нужно **при включённом exit node**: именно в
этом режиме конфигурация обычно и разваливается.

Текущее состояние клиента:

```bash
tailscale status
tailscale debug prefs | grep -iE "corpdns|exitnode"
```

`CorpDNS` — это и есть «Use Tailscale DNS settings». `ExitNodeID` непустой —
значит exit node активен.

---

## Диагностика: одна поломка, три симптома

| Где проявляется | Что видно |
|-----------------|-----------|
| Приложение | `Network timeout (possible IP throttling)`, в статус-баре `IP: N/A` |
| Сборка | `Could not resolve host: static.crates.io`, `make build` висит на crates |
| До версии 1.5.1 | пустое белое окно — запрос шрифта уходил в тот же мёртвый DNS |

Подсказка приложения «No proxy detected, try enabling XRAY/Clash» в этом случае
уводит не туда: прокси ни при чём, не работает разрешение имён.

### Отличить DNS от реальной блокировки — 30 секунд

```bash
dig +time=3 +tries=1 @1.1.1.1 www.youtube.com +short
curl -sS -o /dev/null -w "%{http_code}\n" --max-time 10 https://www.youtube.com
```

**Явный резолвер отдаёт адреса, а `curl` пишет `Resolving timed out`** — это
DNS, а не YouTube и не провайдер.

Кто подменил резолвер:

```bash
scutil --dns | head -8
```

Строка `if_index : NN (utunN)` означает, что DNS навязан туннелем. Адреса,
прописанные через `networksetup -setdnsservers Wi-Fi …`, в этом случае просто
игнорируются: у резолвера, привязанного к туннелю, приоритет выше.

Чей это туннель:

```bash
ifconfig utun4 | grep inet
```

`100.64.x.x` и `fd7a:115c:a1e0::` — это Tailscale. Другой адрес — сторонний
VPN-клиент, и его настройки правятся в нём самом.

---

## Частые ошибки конфигурации

| Ошибка | Что происходит |
|--------|----------------|
| Global nameservers пуст + активен exit node | DNS локальной сети переезжает на туннель, где недостижим — всё висит |
| Приватный адрес в Global nameservers | Работает только внутри той же сети, в поездке ломается |
| Правка DNS через `networksetup` на Wi-Fi | Не действует: туннельный резолвер приоритетнее |
| Расчёт на `--accept-dns=false` | Не помогает: exit node перевешивает резолвер и без Tailscale DNS |

---

## Временные обходные пути

Пока нет доступа к админке — снять exit node, DNS вернётся к настройкам Wi-Fi:

```bash
sudo tailscale set --exit-node=
```

Пробить конкретные хосты мимо резолвера — выручает, когда нужно собрать проект
прямо сейчас:

```bash
for h in github.com codeload.github.com registry.npmjs.org static.crates.io index.crates.io; do ip=$(dig +short +time=3 +tries=1 @1.1.1.1 "$h" | grep -E '^[0-9.]+$' | head -1); [ -n "$ip" ] && printf "%s %s\n" "$ip" "$h"; done | sudo tee -a /etc/hosts
```

Эти строки обязательно убрать после починки DNS — адреса CDN меняются, и
однажды они начнут ломать доступ вместо того, чтобы его давать:

```bash
sudo sed -i '' -E '/^[0-9.]+[[:space:]]+(github\.com|codeload\.github\.com|registry\.npmjs\.org|static\.crates\.io|index\.crates\.io)$/d' /etc/hosts
```

---

## Смежное

- [MACOS_SETUP_ru.md](MACOS_SETUP_ru.md) — установка, сборка, пустое белое окно
- [../YOUTUBE_BLOCKING.md](../YOUTUBE_BLOCKING.md) — настоящие блокировки YouTube: SABR, 403, PO Token
