# Deployment Guide

## Server Setup (Ubuntu 24.04)

### 1. Create directories

```bash
sudo mkdir -p /opt/spending-tracker/scripts
sudo mkdir -p /opt/spending-tracker/data
```

### 2. Systemd unit file

Create `/etc/systemd/system/spending-tracker.service`:

```ini
[Unit]
Description=Spending Tracker Telegram Bot
After=network.target

[Service]
Type=simple
WorkingDirectory=/opt/spending-tracker
ExecStart=/opt/spending-tracker/spending-tracker
Restart=always
RestartSec=5

Environment=TELOXIDE_TOKEN=your-bot-token-here
Environment=DATABASE_PATH=/opt/spending-tracker/data/spending_tracker.db
Environment=RUST_LOG=info
Environment=ALEX_TELEGRAM_ID=5033919666
Environment=HANNA_TELEGRAM_ID=0000000000

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable spending-tracker
sudo systemctl start spending-tracker
```

### 3. Cron jobs

```bash
sudo crontab -e
```

Add:
```
0 3 * * * /opt/spending-tracker/scripts/backup.sh
0 */4 * * * /opt/spending-tracker/scripts/check_logs.sh
```

- **backup.sh** — daily at 3 AM, dumps SQLite and uploads to Google Drive
- **check_logs.sh** — every 4 hours, sends warning-level logs to admin via Telegram

### 4. gdrive CLI setup

Install [gdrive](https://github.com/glotlabs/gdrive) and authenticate:

```bash
gdrive account add
```

## Local Development

1. Create `.env` in project root:
```
TELOXIDE_TOKEN=your-dev-bot-token
ALEX_TELEGRAM_ID=your-telegram-id
HANNA_TELEGRAM_ID=other-telegram-id
RUST_LOG=debug
```

2. Run: `cargo run`

## Deploy from local machine

```bash
./scripts/deploy.sh
```

This runs checks, cross-compiles for Linux musl, uploads binary and scripts, restarts the service.
