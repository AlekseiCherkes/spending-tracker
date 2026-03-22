# Deployment Guide

## Server Setup (Ubuntu 24.04)

### 1. Install dependencies

```bash
sudo apt update
sudo apt install -y cron sqlite3 git
```

Note: Security updates are enabled by default on Ubuntu 24.04 (Google Cloud) via `unattended-upgrades`.
So we don't need to worry about them.

### 2. Create directories

```bash
sudo mkdir -p /opt/spending-tracker/scripts
sudo mkdir -p /opt/spending-tracker/data
```

### 3. Systemd unit file

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

# Only for pre-seeded data
Environment=ALEX_TELEGRAM_ID=your-telegram-id
Environment=HANNA_TELEGRAM_ID=other-telegram-id

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable spending-tracker
sudo systemctl start spending-tracker
```

### 4. Cron jobs

```bash
sudo crontab -e
```

Add:
```
0 3 * * * /opt/spending-tracker/scripts/backup.sh
0 */4 * * * /opt/spending-tracker/scripts/check_logs.sh
```

- **backup.sh** — daily at 3 AM, dumps SQLite and pushes to GitHub
- **check_logs.sh** — every 4 hours, sends warning-level logs to admin via Telegram

### 5. GitHub backup setup

The backup script commits a daily SQLite dump to a private GitHub repository
using an SSH deploy key (write access to one repo only).

#### 5a. Create the GitHub repository

Create a **private** repository on GitHub (e.g. `spending-tracker-backup`).
Initialize it with a README or leave it empty.

#### 5b. Generate a deploy key on the server

```bash
sudo ssh-keygen -t ed25519 -f /root/.ssh/backup_deploy_key -C "spending-tracker-backup" -N ""
sudo cat /root/.ssh/backup_deploy_key.pub
```

Copy the public key output.

#### 5c. Add the deploy key to GitHub

Go to the backup repository → **Settings → Deploy keys → Add deploy key**.
Paste the public key. Check **"Allow write access"**. Save.

#### 5d. Configure SSH on the server

Create `/root/.ssh/config` (or append to it):

```
Host github-backup
    HostName github.com
    User git
    IdentityFile /root/.ssh/backup_deploy_key
    IdentitiesOnly yes
```

Test the connection:

```bash
sudo ssh -T github-backup
```

Expected output: `Hi <user>/<repo>! You've successfully authenticated...`

#### 5e. Clone the backup repository

```bash
sudo git clone git@github-backup:<user name>/spending-tracker-backup.git /opt/spending-tracker/spending-tracker-backup
cd /opt/spending-tracker/spending-tracker-backup
sudo git config user.email "backup@spending-tracker"
sudo git config user.name "Spending Tracker Backup"
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
