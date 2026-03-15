# Architecture

## Module Diagram

```
main → bot → domain → dal → SQLite
```

## Modules

### `src/main.rs`
Entry point. Loads `.env`, initializes logger, opens DB, creates draft store, starts bot.

### `src/bot/`
Telegram bot layer using teloxide.
- **`mod.rs`** — dispatcher setup with `dptree`
- **`handlers.rs`** — message and callback query handlers; whitelist enforcement
- **`keyboards.rs`** — inline keyboard builders (summary, categories, accounts)

### `src/domain/`
Business logic, independent of Telegram.
- **`spending.rs`** — `SpendingDraft`, `EditState` enum, `DraftStore` (in-memory per-user draft management)

### `src/dal/`
Data access layer wrapping rusqlite.
- **`mod.rs`** — `Db` struct (Arc<Mutex<Connection>>), all query methods
- **`models.rs`** — data structs: User, Account, Category, Currency, Spending
- **`queries.rs`** — SQL schema DDL
- **`seed.rs`** — default data seeding (currencies, users, categories, accounts)

## Data Model

- **currencies** — EUR, USD, BYN
- **users** — whitelisted Telegram users (Alex admin, Hanna)
- **accounts** — bank accounts with currency and optional owner
- **categories** — 20 spending categories with sort order
- **spendings** — individual expense records linked to account, category, reporter

## Key Decisions

- Single SQLite connection behind `Arc<Mutex<>>` — sufficient for low-traffic family bot
- `REAL` for amounts — no decimal crate needed for personal finance
- Simple version-based migrations (no external framework)
- `INSERT OR IGNORE` for idempotent seeding
- Telegram user IDs from env vars (with fallback defaults for tests)
