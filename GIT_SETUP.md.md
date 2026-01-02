# Git Setup - Следующие шаги

## ✅ Выполнено

1. ✅ Создан `.gitignore`
2. ✅ Инициализирован Git репозиторий
3. ✅ Настроен author: Kurein Maxim  
4. ✅ Создан initial commit (48 файлов, 5118 строк)
5. ✅ Переименована ветка в `main`

---

## 🚀 Следующие шаги

### Вариант 1: Через GitHub CLI (gh)

```bash
# Если gh установлен
cd /Users/olgazaharova/Project/ProjectYouTube

# Создать репозиторий на GitHub
gh repo create youtube-downloader --public --source=. --remote=origin

# Push кода
git push -u origin main
```

### Вариант 2: Вручную через GitHub Web

1. **Создайте репозиторий на GitHub:**
   - Откройте https://github.com/new
   - Repository name: `youtube-downloader`
   - Description: "Modern desktop app for downloading YouTube videos"
   - Public или Private (на ваш выбор)
   - **НЕ** создавайте README, .gitignore или LICENSE (у нас уже есть)
   - Нажмите "Create repository"

2. **Добавьте remote и push:**
   ```bash
   cd /Users/olgazaharova/Project/ProjectYouTube
   
   # Добавьте remote (замените USERNAME на ваш GitHub username)
   git remote add origin https://github.com/USERNAME/youtube-downloader.git
   
   # Или через SSH (если настроены SSH ключи):
   git remote add origin git@github.com:USERNAME/youtube-downloader.git
   
   # Push кода
   git push -u origin main
   ```

3. **Проверьте:**
   - Откройте https://github.com/USERNAME/youtube-downloader
   - Должны увидеть весь код и документацию

---

## 📝 Полезные команды Git

```bash
# Проверить статус
git status

# Посмотреть коммиты
git log --oneline

# Посмотреть изменения
git diff

# Создать новый коммит
git add -A
git commit -m "feat: добавил новую функцию"

# Push изменений
git push

# Создать и переключиться на новую ветку
git checkout -b feature/new-feature

# Посмотреть все ветки
git branch -a
```

---

## 🏷️ Создание первого релиза

После push на GitHub:

```bash
# Создать тег для релиза
git tag -a v0.1.0 -m "Release v0.1.0: Initial release"
git push origin v0.1.0

# Или через GitHub CLI
gh release create v0.1.0 \
  --title "YouTube Downloader v0.1.0" \
  --notes "Initial release with basic download functionality"
```

---

## 🔧 Workflow для разработки

```bash
# 1. Создать feature ветку
git checkout -b feature/batch-download

# 2. Внести изменения
# ... редактируйте код ...

# 3. Commit
git add -A
git commit -m "feat: add batch download support"

# 4. Push ветку
git push -u origin feature/batch-download

# 5. Создать Pull Request на GitHub
gh pr create --title "Add batch download" --body "Добавляет поддержку batch скачивания"

# 6. После merge удалить ветку
git checkout main
git pull
git branch -d feature/batch-download
```

---

## 📋 .gitignore уже настроен для:

- ✅ macOS системные файлы (.DS_Store)
- ✅ Node.js (node_modules, dist)
- ✅ Rust (target/, Cargo.lock)
- ✅ Tauri build артефакты
- ✅ IDE файлы (.vscode, .idea)
- ✅ Python (.venv, __pycache__)
- ✅ Build outputs (.exe, .dmg, .msi)
- ✅ Logs и temporary files

---

## 🎯 Рекомендации

1. **Используйте semantic commits:**
   - `feat:` - новая функция
   - `fix:` - исправление бага
   - `docs:` - изменения в документации
   - `chore:` - рутинные задачи (обновление версии)
   - `refactor:` - рефакторинг кода

2. **Делайте частые коммиты** - лучше много маленьких, чем один большой

3. **Используйте branches** для новых функций

4. **Делайте Pull Requests** для review (даже если работаете один)

---

**Дата создания:** 02.01.2026  
**Автор:** Kurein Maxim
