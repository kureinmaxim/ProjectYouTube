#!/usr/bin/env python3
"""
YouTube Format Fetcher via yt-dlp + cookies
===========================================
Получает список доступных форматов видео, используя cookies для авторизации.
Обходит блокировки YouTube, которые срабатывают на неавторизованные запросы.

Использование:
1. Экспортируй cookies из браузера (см. ИНСТРУКЦИЯ ниже)
2. Сохрани как cookies.txt в папку проекта
3. Запусти: python3 formats.py

ИНСТРУКЦИЯ по экспорту cookies:
-------------------------------
Chrome / Brave / Edge:
1. Установи расширение "Get cookies.txt (LOCALLY)" или "EditThisCookie"
2. Зайди на youtube.com и убедись, что ты залогинен (видна аватарка)
3. Экспортируй cookies в формате Netscape (cookies.txt)
4. Сохрани файл как cookies.txt в папку проекта

Firefox:
1. Установи расширение "cookies.txt"
2. Аналогично экспортируй для youtube.com

Safari:
- Safari не поддерживает такие расширения напрямую
- Используй Chrome или yt-dlp --cookies-from-browser safari
"""

import sys
import os
from pathlib import Path

# Проверяем наличие yt-dlp
try:
    from yt_dlp import YoutubeDL
except ImportError:
    print("❌ yt-dlp не установлен!")
    print("   Установи: pip3 install yt-dlp")
    print("   Или: source venv/bin/activate && pip install yt-dlp")
    sys.exit(1)

# ============ НАСТРОЙКИ ============
# URL видео для тестирования (можно изменить)
DEFAULT_URL = "https://youtu.be/nt8cBMecQR0"

# Путь к cookies (относительно скрипта или абсолютный)
COOKIES_FILE = Path(__file__).parent / "cookies.txt"

# ===================================


def check_cookies():
    """Проверяет наличие файла cookies"""
    if not COOKIES_FILE.exists():
        print(f"⚠️  Файл cookies не найден: {COOKIES_FILE}")
        print()
        print("📋 Как создать cookies.txt:")
        print("   1. Установи расширение 'Get cookies.txt (LOCALLY)' в Chrome")
        print("   2. Зайди на youtube.com и залогинься")
        print("   3. Нажми на расширение → 'Export' → сохрани как cookies.txt")
        print(f"   4. Положи файл сюда: {COOKIES_FILE}")
        print()
        return False
    
    # Проверяем размер файла
    size = COOKIES_FILE.stat().st_size
    if size < 100:
        print(f"⚠️  Файл cookies слишком маленький ({size} байт)")
        print("   Возможно, он пустой или повреждён")
        return False
    
    print(f"✅ Найден файл cookies: {COOKIES_FILE} ({size} байт)")
    return True


def get_formats(url: str, use_cookies: bool = True):
    """Получает список форматов для видео"""
    
    ydl_opts = {
        "quiet": True,
        "no_warnings": True,
        "skip_download": True,
        "extract_flat": False,
    }
    
    if use_cookies and COOKIES_FILE.exists():
        ydl_opts["cookies"] = str(COOKIES_FILE)
        print(f"🍪 Используем cookies: {COOKIES_FILE.name}")
    else:
        print("🔓 Запрос без cookies")
    
    print(f"🔗 URL: {url}")
    print()
    print("⏳ Получаем информацию о видео...")
    print()
    
    try:
        with YoutubeDL(ydl_opts) as ydl:
            info = ydl.extract_info(url, download=False)
    except Exception as e:
        print(f"❌ Ошибка: {e}")
        print()
        if "403" in str(e):
            print("💡 Подсказка: HTTP 403 = YouTube блокирует запрос")
            print("   Попробуй:")
            print("   - Обновить cookies (залогиниться заново)")
            print("   - Использовать VPN/прокси")
            print("   - Подождать 5-10 минут")
        elif "Sign in" in str(e) or "login" in str(e).lower():
            print("💡 Подсказка: Требуется авторизация")
            print("   Убедись, что cookies экспортированы с залогиненного аккаунта")
        return None
    
    return info


def print_formats(info: dict):
    """Красиво выводит форматы"""
    
    title = info.get("title", "Unknown")
    duration = info.get("duration", 0)
    uploader = info.get("uploader", "Unknown")
    
    print("=" * 70)
    print(f"📹 {title}")
    print(f"👤 {uploader}")
    print(f"⏱️  {duration // 60}:{duration % 60:02d}")
    print("=" * 70)
    print()
    
    formats = info.get("formats", [])
    
    if not formats:
        print("⚠️  Форматы не найдены!")
        return
    
    # Разделяем на видео и аудио
    video_formats = []
    audio_formats = []
    
    for f in formats:
        vcodec = f.get("vcodec", "none")
        acodec = f.get("acodec", "none")
        
        if vcodec != "none" and vcodec is not None:
            video_formats.append(f)
        elif acodec != "none" and acodec is not None:
            audio_formats.append(f)
    
    # Выводим аудио форматы
    if audio_formats:
        print("🎵 АУДИО форматы:")
        print("-" * 70)
        print(f"{'ID':>6} | {'EXT':>5} | {'BITRATE':>10} | {'CODEC':>12} | {'SIZE':>10}")
        print("-" * 70)
        
        for f in sorted(audio_formats, key=lambda x: x.get("abr") or 0, reverse=True):
            fmt_id = f.get("format_id", "?")
            ext = f.get("ext", "?")
            abr = f.get("abr")
            abr_str = f"{abr:.0f}k" if abr else "?"
            acodec = f.get("acodec", "?")
            filesize = f.get("filesize") or f.get("filesize_approx") or 0
            size_str = f"{filesize / 1024 / 1024:.1f}MB" if filesize else "?"
            
            print(f"{fmt_id:>6} | {ext:>5} | {abr_str:>10} | {acodec:>12} | {size_str:>10}")
        print()
    
    # Выводим видео форматы
    if video_formats:
        print("🎬 ВИДЕО форматы:")
        print("-" * 70)
        print(f"{'ID':>6} | {'EXT':>5} | {'RESOLUTION':>12} | {'FPS':>4} | {'CODEC':>10} | {'SIZE':>10}")
        print("-" * 70)
        
        for f in sorted(video_formats, key=lambda x: (x.get("height") or 0, x.get("fps") or 0), reverse=True):
            fmt_id = f.get("format_id", "?")
            ext = f.get("ext", "?")
            resolution = f.get("resolution") or f"{f.get('width', '?')}x{f.get('height', '?')}"
            fps = f.get("fps", "?")
            vcodec = f.get("vcodec", "?")
            # Сокращаем кодек для красоты
            if vcodec and "." in str(vcodec):
                vcodec = vcodec.split(".")[0]
            filesize = f.get("filesize") or f.get("filesize_approx") or 0
            size_str = f"{filesize / 1024 / 1024:.1f}MB" if filesize else "?"
            
            print(f"{fmt_id:>6} | {ext:>5} | {resolution:>12} | {fps:>4} | {vcodec:>10} | {size_str:>10}")
        print()
    
    # Рекомендации
    print("=" * 70)
    print("💡 РЕКОМЕНДАЦИИ по скачиванию:")
    print()
    
    # Лучшее видео + аудио
    best_video = None
    for f in video_formats:
        if f.get("vcodec", "").startswith("avc1"):  # H.264
            if not best_video or (f.get("height", 0) > best_video.get("height", 0)):
                best_video = f
    
    best_audio = None
    for f in audio_formats:
        if f.get("ext") == "m4a":
            if not best_audio or (f.get("abr", 0) > best_audio.get("abr", 0)):
                best_audio = f
    
    if best_video and best_audio:
        print(f"   Лучшее H.264 + AAC (совместимо везде):")
        print(f"   yt-dlp --cookies cookies.txt -f {best_video['format_id']}+{best_audio['format_id']} \"{info.get('webpage_url')}\"")
        print()
    
    print(f"   Авто-выбор лучшего:")
    print(f"   yt-dlp --cookies cookies.txt -f \"bv*[vcodec^=avc1]+ba[ext=m4a]/b\" \"{info.get('webpage_url')}\"")
    print()
    
    print(f"   Только аудио (MP3):")
    print(f"   yt-dlp --cookies cookies.txt -x --audio-format mp3 \"{info.get('webpage_url')}\"")
    print("=" * 70)


def main():
    # Определяем URL
    if len(sys.argv) > 1:
        url = sys.argv[1]
    else:
        url = DEFAULT_URL
        print(f"ℹ️  Используем тестовый URL: {url}")
        print(f"   Можно указать свой: python3 formats.py <URL>")
        print()
    
    # Проверяем cookies
    has_cookies = check_cookies()
    print()
    
    # Получаем форматы
    info = get_formats(url, use_cookies=has_cookies)
    
    if info:
        print_formats(info)
    else:
        print("❌ Не удалось получить информацию о видео")
        sys.exit(1)


if __name__ == "__main__":
    main()

