# Rubirinth App

<p align="center">
  <strong>Современный, быстрый и удобный лаунчер Minecraft на базе Modrinth App с глубокой интеграцией CurseForge, поддержкой скинов Ely.by и полной русской локализацией.</strong>
</p>

<p align="center">
  <a href="https://github.com/ggwpsayori/rubirinth/releases/latest">
    <img src="https://img.shields.io/github/v/release/ggwpsayori/rubirinth?color=2ea043&label=%D0%A0%D0%B5%D0%BB%D0%B8%D0%B7&style=for-the-badge" alt="Latest Release">
  </a>
  <img src="https://img.shields.io/badge/%D0%9F%D0%BB%D0%B0%D1%82%D1%84%D0%BE%D1%80%D0%BC%D0%B0-Windows%20x64-0078d7?style=for-the-badge&logo=windows" alt="Platform">
  <a href="LICENSE">
    <img src="https://img.shields.io/github/license/ggwpsayori/rubirinth?style=for-the-badge" alt="License">
  </a>
</p>

---

## 🌟 Ключевые возможности

### 🌐 Двойной каталог: Modrinth + CurseForge
* **Поиск и установка в один клик:** ищите и скачивайте моды, модпаки, наборы ресурсов и шейдеры сразу из двух крупнейших платформ через единый интерфейс.
* **Переключатель каталогов:** удобный фильтр в поисковой выдаче с сохранением выбранного источника.
* **Свободное комбинирование:** вы можете скачивать моды с CurseForge в сборки Modrinth и наоборот — метаданные не перезаписываются и не конфликтуют.
* **Цветная индикация площадок:** легко определяйте источник проекта по значкам в каталоге, карточках и списке модов сборки (зелёный значок Modrinth / оранжевый огонь CurseForge).

### 🇷🇺 Качественная русская локализация
* Полный перевод интерфейса лаунчера.
* Переведены все категории контента, теги, описания фильтров и системные сообщения.

### 👕 Система скинов Ely.by
* Удобный просмотр и смена скинов игрового профиля прямо в окне приложения.

### ⚡ Производительность и надёжность
* Нативное ядро на **Rust** (Tauri 2) и ультрабыстрый фронтенд на **Vue 3 + Vite**.
* Локальное SQLite-кэширование метаданных и пакетные сетевые запросы исключают лаги и подвисания интерфейса даже на массивных сборках с сотнями модов.
* Автоматическая проверка совместимости версий игры и загрузчиков (Fabric, Forge, NeoForge, Quilt).

---

## 📥 Установка

1. Перейдите на страницу **[Последнего релиза](https://github.com/ggwpsayori/rubirinth/releases/latest)**.
2. Скачайте файл установщика **`Rubirinth_x.x.x_x64-setup.exe`**.
3. Запустите установщик и следуйте подсказкам мастера установки.

---

## 🛠️ Сборка из исходников

Для сборки проекта на вашем компьютере требуются:
* **Node.js** версии 20+
* **pnpm** версии 10+
* **Rust** (stable toolchain)
* **Visual Studio Build Tools** с компонентами C++ (для Windows)

### Инструкция:

```bash
# 1. Клонируйте репозиторий
git clone https://github.com/ggwpsayori/rubirinth.git
cd rubirinth

# 2. Установите зависимости
pnpm install

# 3. Запуск в режиме разработки
pnpm app:dev

# 4. Сборка готового релизного установщика
pnpm app:build
```

---

## 🤝 Благодарности и лицензия

* Проект создан на основе кодовой базы **[Modrinth App (Theseus)](https://github.com/modrinth/theseus)** от команды Modrinth.
* Распространяется под лицензией **GNU General Public License v3.0 (GPL-3.0)**. Подробнее см. в файле [LICENSE](LICENSE).
