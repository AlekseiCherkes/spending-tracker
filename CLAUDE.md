# Spending Tracker

Personal family spending tracker: Rust + SQLite + Telegram bot.

## Commands

- `cargo run` — run locally (needs `.env` with TELOXIDE_TOKEN)
- `./scripts/check.sh` — fmt + clippy + test
- `./scripts/deploy.sh` — check, cross-compile (musl), upload to VPS, restart service
- `./scripts/download-prod-db.sh` — pull a snapshot of the prod SQLite DB into the project root for local dev

## Env Vars

- `TELOXIDE_TOKEN` — Telegram bot token
- `DATABASE_PATH` — SQLite path (default: `spending_tracker.db`)
- `RUST_LOG` — log level

## Docs

- [docs/architecture.md](docs/architecture.md) — modules, data model, key technical decisions
- [docs/deploy.md](docs/deploy.md) — server setup, systemd, cron, gdrive
- [docs/local-dev.md](docs/local-dev.md) — local development setup
