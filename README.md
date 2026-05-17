# spending-tracker

Personal family spending tracker. A small Telegram bot writes expenses
into a local SQLite file. Built to be the simplest thing that works for
one household — no auth servers, no web UI, no SaaS dependencies.

## Stack

Rust + SQLite (rusqlite, bundled) + teloxide. Single statically-linked
binary, deployed to a small VPS under systemd. Whitelisted users only.

## Bot commands

- `/start` — greeting
- `/recent` — last 25 transactions (tap a number to edit or delete)
- `/export` — export a month to CSV
- `/accounts`, `/categories`, `/currencies`, `/users` — read-only info
- `/default_account` — change your default account

To record an expense, just send a number to the bot.

## Quick start (local)

Put a bot token in `.env`:

```
TELOXIDE_TOKEN=your-dev-bot-token
RUST_LOG=debug
```

Then:

```bash
cargo run
```

See [docs/local-dev.md](docs/local-dev.md) for pulling a prod snapshot
to develop against real data.

## Scripts

| Script                          | What it does                                                |
|---------------------------------|-------------------------------------------------------------|
| `./scripts/check.sh`            | `cargo fmt --check` + clippy (`-D warnings`) + tests        |
| `./scripts/deploy.sh`           | Backup prod DB, cross-compile musl, upload, restart service |
| `./scripts/download-prod-db.sh` | Pull a consistent prod DB snapshot into the project root    |

## Docs

- [Architecture](docs/architecture.md) — modules, data model, key technical decisions
- [Local development](docs/local-dev.md) — local development setup
- [Deployment](docs/deploy.md) — server setup, systemd, cron, backup
