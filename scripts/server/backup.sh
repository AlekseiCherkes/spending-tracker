#!/usr/bin/env bash
# Runs on the server via cron (daily at 3 AM)
# Dumps SQLite database and pushes to GitHub backup repository.
# Sends a Telegram alert to the admin if anything fails.
set -euo pipefail

DB="/opt/spending-tracker/data/spending_tracker.db"
REPO="/opt/spending-tracker/spending-tracker-backup"
DUMP_FILE="spending_tracker.sql"

# --- Telegram failure notification (same approach as check_logs.sh) ---
notify_failure() {
    local error_msg="$1"
    local admin_tg_id bot_token message
    admin_tg_id=$(sqlite3 "$DB" "SELECT telegram_id FROM users WHERE is_admin=1 LIMIT 1" 2>/dev/null || true)
    bot_token=$(systemctl show spending-tracker -p Environment --value | grep -oP 'TELOXIDE_TOKEN=\K[^ ]+' || true)

    if [ -n "$admin_tg_id" ] && [ -n "$bot_token" ]; then
        message="🔴 Backup failed:%0A${error_msg:0:4000}"
        curl -s "https://api.telegram.org/bot${bot_token}/sendMessage" \
            -d "chat_id=${admin_tg_id}" \
            -d "text=${message}" > /dev/null
    fi
}

trap 'notify_failure "backup.sh failed at line $LINENO"' ERR

# --- Dump and push ---
sqlite3 "$DB" .dump > "$REPO/$DUMP_FILE"

cd "$REPO"
git add "$DUMP_FILE"

if git diff --cached --quiet; then
    echo "No changes to backup."
    exit 0
fi

git commit -m "Backup update"
git push origin main
echo "Backup complete."
