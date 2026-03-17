#!/usr/bin/env bash
# Runs on the server via cron (every 4 hours)
set -euo pipefail

DB="/opt/spending-tracker/data/spending_tracker.db"

LOGS=$(journalctl -u spending-tracker --since "4 hours ago" -p warning --no-pager 2>/dev/null | grep -v "^-- No entries --$" || true)

if [ -z "$LOGS" ]; then
    exit 0
fi

ADMIN_TG_ID=$(sqlite3 "$DB" "SELECT telegram_id FROM users WHERE is_admin=1 LIMIT 1")
BOT_TOKEN=$(systemctl show spending-tracker -p Environment --value | grep -oP 'TELOXIDE_TOKEN=\K[^ ]+')

if [ -n "$ADMIN_TG_ID" ] && [ -n "$BOT_TOKEN" ]; then
    MESSAGE="⚠️ Warning logs (last 4h):%0A${LOGS:0:4000}"
    curl -s "https://api.telegram.org/bot${BOT_TOKEN}/sendMessage" \
        -d "chat_id=${ADMIN_TG_ID}" \
        -d "text=${MESSAGE}" > /dev/null
fi
