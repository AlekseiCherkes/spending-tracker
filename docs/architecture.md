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
- **`seed.rs`** — `#[cfg(test)]` only: fixture data (Alice/Bob, abstract accounts, sample 2025 spendings) used by `Db::open_in_memory`

## Data Model

- **currencies** — codes like EUR, USD, BYN
- **users** — whitelisted Telegram users
- **accounts** — bank accounts with currency and optional owner
- **categories** — spending categories with sort order
- **spendings** — individual expense records linked to account, category, reporter

## Key Decisions

- `rusqlite` with `bundled` feature — statically links SQLite for musl cross-compilation
- `teloxide` with `rustls` (not native-tls) — avoids OpenSSL dependency in cross-compilation
- Single SQLite connection behind `Arc<Mutex<>>` — sufficient for low-traffic family bot
- `REAL` for amounts — no decimal crate needed for personal finance
- Simple version-based migrations (no external framework)
- No preseeding in production: the binary creates the empty schema and the DB is populated once by hand via `sqlite3`. Local development uses a snapshot of the prod DB (`scripts/download-prod-db.sh`). Tests get fixture data from a `#[cfg(test)]`-only `seed.rs` (Alice/Bob, abstract accounts, sample 2025 spendings)
