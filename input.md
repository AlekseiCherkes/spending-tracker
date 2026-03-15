Хочу сделать приложение для учета личных финансовых расходов в семье.
Приложение будет использовать только в личных целях, поэтому большой нагрузки испытывать не будет.
В техническом дизайне приложения я придерживаюсь максимально простых решений, следую принципу KISS (делать всё как можно
более проще).
Всё должно быть максимально простым и поддерживаемым.

# Технический стек

- Rust
- sqlite.
- telegram bot как главный интерфейс к приложению
- Разработка ведётся на macOS/Ubuntu. Под windows разработка не ведётся, поэтому эту систему можно не учитывать в
  скриптах и документации.
- нужно использовать классические linters и утилиты форматирования кода для Rust

# Deploy

- Ubuntu 24.04 на VPS
- К VPS есть доступ по ssh
- systemd сервис для запуска процесса
- переменные окружения для настройки (api tokens, db path, log path, log level)
- переменные окружения указываются в systemd файле
- нужно выбрать хорошие пути по которым лежат логи и файл с БД
- бинарный файл приложения собирается на локальной машине и заливается на удалённый VPS по ssh. Удалённой машине не нужн
- банарь нужно собирать на macOS для кроссплатформенного linux (muse)
- нужна простейшая cron job, которая раз в четыре часа делает grep по логам приложения и отправляет сообщение в
  telegram, если там есть warnings или errors
- приложение пишут логи в stdout/stderr (не знаю, что толком лучше)
- приложение должно использовать какую-то протейшую библиотеку для логиравания

Нужно сделать файл docs/deploy.md, который описывает необходимую настройку сервера (руками) с состояния чистого Ubuntu
24.04.
Нужно сделать скрипт scripts/deploy.sh, для сборки и деплоя проекта из текущей папки.

Нужно подумать над способом бэкапирования sqlite БД. Наверно можно делать dump всей БД в csv или другой текстовый формат
и заливать его куда-то на GDrive или другое облако.

# Структура проекта:

Должен быть разбит на отдедльные модули, которые в будущем, возможно, будут добавляться.
Набор модулей:

- слой доступа к данным (DAL). Чтение sqlite БД и реализация всех необходимых функций доступа к данным в виде rust api (
  т.е инкаписуляция sqlite запросов и инициализация slite БД).
    - нужно предусмотреть миграцию данных во время инициализации
- слой бизнес логики
    - Пока этот слой будет маленьким, но в будущем могут появиться модули импорта/экспорта данных и модель генерации
      месячных отчётов.
    - При внесении траты (spending) в бизнес логике есть состояние, когда трата внесена, но начинает редактироваться (
      пользователь может изменять её параметры перед финальной записью траты в БД). Этот состояние можно хранить в
      памяти RAM и это может быть чатью бизнес логими в этом слое.
- слой UI и интерфейса
    - telegram bot. Использует остальные модули (в первую очередь бизнес логику) и предоставляет интерфейс пользователя
      в виде telegram бота.
    - В будущем может добавиться web UI для генерации каких-то более сложных отчётов

Каждый из модулей должен быть покрыт своими тестами.

Можно написать документацию в docs/architecture.md с общим и очень коротким описанием этих модулей и схемой из
зависимостей (в начале документа).

# Структура данных

Даю сниппеты из своих черновиков:

## Core Tables

- **users**: User information and Telegram integration
- **currencies**: Currency definitions (EUR, USD, BYN, etc.)
- **accounts**: Financial accounts with currency and optional IBAN
- **categories**: Global spending categories with sort ordering
- **spendings**: Individual spending records with full tracking

## Tables

1. **users**: User information and Telegram integration
2. **currencies**: Currency definitions (EUR, USD, BYN, etc.)
3. **accounts**: Financial accounts with currency and optional IBAN
4. **categories**: Global spending categories with sort ordering
5. **spendings**: Individual spending records with full tracking

## Key Relationships

- **accounts** → **currencies** (many-to-one)
- **accounts** ↔ **users** (many-to-many via account_users)
- **spendings** → **accounts** (many-to-one)
- **spendings** → **categories** (many-to-one)
- **spendings** → **users** (many-to-one, reporter)

Идею можно взять из такого сниппета:

    def _create_tables(self) -> None:
        """Create all tables if they don't exist."""
        with self._get_connection() as conn:
            # Users table
            conn.execute(
                """
                CREATE TABLE IF NOT EXISTS users (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    telegram_id INTEGER NOT NULL UNIQUE
                )
            """
            )

            # Currencies table
            conn.execute(
                """
                CREATE TABLE IF NOT EXISTS currencies (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    currency_code TEXT NOT NULL UNIQUE
                )
            """
            )

            # Accounts table
            conn.execute(
                """
                CREATE TABLE IF NOT EXISTS accounts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    currency_id INTEGER NOT NULL,
                    iban TEXT,
                    name TEXT NOT NULL,
                    FOREIGN KEY (currency_id) REFERENCES currencies (id)
                )
            """
            )

            # Categories table
            conn.execute(
                """
                CREATE TABLE IF NOT EXISTS categories (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    sort_order INTEGER NOT NULL DEFAULT 0
                )
            """
            )

            # Spending table
            conn.execute(
                """
                CREATE TABLE IF NOT EXISTS spending (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    account_id INTEGER NOT NULL,
                    amount DECIMAL(10, 2) NOT NULL,
                    category_id INTEGER NOT NULL,
                    reporter_id INTEGER NOT NULL,
                    notes TEXT,
                    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (account_id) REFERENCES accounts (id),
                    FOREIGN KEY (category_id) REFERENCES categories (id),
                    FOREIGN KEY (reporter_id) REFERENCES users (id)
                )
            """
            )

Где-то нужно продумать скрипт, который заполняет БД начальными данными.
Этот скрипт можно использовать для локальной разработки (отладки) и в тестах.

# Структура команд для Telegram bot

Тут нужно продумать. Для начала нужна команда, которая создаёт трату (spending).
