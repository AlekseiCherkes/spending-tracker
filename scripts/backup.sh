#!/usr/bin/env bash
# Runs on the server via cron (daily at 3 AM)
set -euo pipefail

DB="/opt/spending-tracker/data/spending_tracker.db"
BACKUP="/tmp/spending_tracker_backup_$(date +%Y%m%d).sql"

echo "Dumping database..."
sqlite3 "$DB" .dump > "$BACKUP"

echo "Uploading to Google Drive..."
gdrive files upload "$BACKUP"

rm -f "$BACKUP"
echo "Backup complete."
