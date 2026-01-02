#youtube
## Как скачивать и сохранять видео
  ==Использовать утилиту yt-dlp:==

# 0 Установить Google Chrome Browser  

---
# 1 Установка `yt-dlp` (если ещё не установлено)

a. Скачайте последнюю версию `yt-dlp` с GitHub:

Откройте терминал и выполните:
```bash
curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o ~/bin/yt-dlp
chmod +x ~/bin/yt-dlp
```

b. Убедитесь, что `yt-dlp` доступен из терминала:
```bash
yt-dlp --version
```
Вы должны увидеть последнюю версию (например, `2025.03.31`).

---

**2 Скачивание видео с YouTube

a. ==Получите cookies из браузера (Google Chrome):==

1. Убедитесь, что ваш ==браузер (Chrome) открыт и вы авторизованы на YouTube.

2. Введите следующую команду, чтобы получить cookies для `yt-dlp`:
```bash
yt-dlp --cookies-from-browser chrome 'https://youtu.be/URL_ВАШЕГО_ВИДЕО'
```
Замените `https://youtu.be/URL_ВАШЕГО_ВИДЕО` на URL видео, которое хотите скачать.
Пример old:
```bash
yt-dlp --cookies-from-browser chrome 'https://youtu.be/EH3yeiZ5JRo?si=mg8cPZf5j_IkiKfn'
```


https://youtu.be/SEH4T6W_TTg?si=HUbzZ8A-1MCDbPA_
https://youtu.be/S4SL9-k1qOw?si=HjhcGoW-SjLukTyz
https://youtu.be/VFSilOufJc8?si=YBvDTemIbl5kE0f3
https://youtu.be/qHsMV5LhOEc?si=FbrLEPoHN903JkOc
https://youtu.be/wIfYyHyXv5c?si=CnYMTDkZmcQ9vCyv
https://youtu.be/4Id81jV_aRY?si=LBvnDoyz3uFelZ1S

NB!!! new   Попробуйте запустить напрямую через Python:
python3 -m yt_dlp --cookies-from-browser chrome 'https://youtu.be/SEH4T6W_TTg'
python3 -m yt_dlp --cookies-from-browser chrome 'https://youtu.be/4Id81jV_aRY'
python3 -m yt_dlp --cookies-from-browser chrome 'https://youtu.be/0b6x5eJK4WM'
https://youtu.be/VFSilOufJc8

!!!new для https://youtu.be/o05TcqpYo5I?si=8ynG8ZdySUf8-ROa
yt-dlp --extractor-args "youtube:player_client=android" 'https://youtu.be/qHsMV5LhOEc?si=FbrLEPoHN903JkOc'
yt-dlp --extractor-args "youtube:player_client=android" 'https://youtu.be/SEH4T6W_TTg?si=HUbzZ8A-1MCDbPA_'


yt-dlp -f "bv*+ba/best" --merge-output-format mp4 'yt-dlp --cookies-from-browser chrome 'https://youtu.be/Kf0OmKJb4ck?si=8bDYwisXKfaqAtL6''

OR

yt-dlp --cookies-from-browser chrome --extractor-args "youtube:player_client=ios" --list-formats 'https://youtu.be/o05TcqpYo5I?si=3Yj2lEuGS4d8FavI'
AND 
yt-dlp --extractor-args "youtube:player_client=ios" --list-formats 'https://youtu.be/o05TcqpYo5I?si=3Yj2lEuGS4d8FavI'

yt-dlp --cookies-from-browser chrome --extractor-args "youtube:player_client=web" --list-formats 'https://youtu.be/o05TcqpYo5I?si=QBzSMqYakOUzhp1P'

==и для наихудшего качества==

yt-dlp -f worst --cookies-from-browser chrome 'https://youtu.be/xBuKrMQ5sd8?si=2Hw-NAnW_aipIdtA'

==какие форматы есть вообще:==

yt-dlp -F 'https://youtu.be/BH5uFF1nJok'

Скачаем **720p видео + звук**:

`yt-dlp -f "136+140" 'https://youtu.be/BH5uFF1nJok'`

- `136` — видео 720p (без звука)
- `140` — аудио AAC  
    `yt-dlp` потом автоматически их склеит в mp4.

Если хочешь упростить (чтобы `yt-dlp` сам брал лучшее доступное ≤720p):
```bash
yt-dlp -f "bestvideo[height<=720]+bestaudio/best[height<=720]" 'https://youtu.be/BH5uFF1nJok'
```

b. Видео будет скачан ==в текущую рабочую папку.==

Если хотите изменить папку для сохранения:
```bash
yt-dlp -P /путь/к/папке/для/скачивания --cookies-from-browser chrome 'https://youtu.be/URL_ВАШЕГО_ВИДЕО'
```
   Пример:
```bash
 yt-dlp -P ~/Videos/yt-downloads --cookies-from-browser chrome 'https://youtu.be/RbmFqLcfUwI?si=9BQn2JIkTizKNqr9'  
```

c. ==Только аудио==
```Shell
yt-dlp -f 249 --cookies-from-browser chrome 'https://www.youtube.com/watch?v=o2o3yCItcwk&authuser=0'
```

  **3 Настройка имени файла (опционально)

Чтобы имя файла было удобным (например, по названию видео), можно использовать следующий флаг:
 ```bash
yt-dlp -o '~/Downloads/%(title)s.%(ext)s' --cookies-from-browser chrome 'https://youtu.be/URL_ВАШЕГО_ВИДЕО'
```
 
В этом случае файл будет сохранён в папке `~/Downloads` с названием, совпадающим с заголовком видео.

### Примечания

- Если `yt-dlp` не обновляется через pip или brew, используйте [собственные скачивания бинарников](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp).
    
- **Для получения cookies**: убедитесь, что вы авторизованы в YouTube в браузере, иначе скачивание может не работать.
    
- В случае ошибок с видеоформатами используйте команду `yt-dlp --list-formats 'https://youtu.be/VIDEO_ID'` для получения доступных форматов.

Пример полной команды:
```bash
yt-dlp -o '~/Downloads/%(title)s.%(ext)s' --cookies-from-browser chrome 'https://youtu.be/RbmFqLcfUwI?si=9BQn2JIkTizKNqr9'
```

**4 Конвертировать в mp4 можно через **
 MKV2MP4 программу:
[https://www.corecode.io/mkv2mp4/](https://www.corecode.io/tableedit/)