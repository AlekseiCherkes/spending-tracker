#!/usr/bin/env bash
set -euo pipefail

# Pulls a consistent snapshot of the production SQLite database into the
# project root so local development can run against real data.
#
# Uses `sqlite3 .backup` on the server (safe while the bot is running, unlike
# a raw scp of an active WAL-mode file), then transfers the dump and cleans
# up the temporary file on the server.

REMOTE="spending_tracker"
REMOTE_DB="/opt/spending-tracker/data/spending_tracker.db"
REMOTE_TMP="/tmp/spending-tracker-snapshot.db"
LOCAL_DB="spending_tracker.db"

echo "==> Creating consistent snapshot on remote..."
ssh "$REMOTE" "sudo sqlite3 $REMOTE_DB \".backup $REMOTE_TMP\" && sudo chown \$(id -u):\$(id -g) $REMOTE_TMP"

echo "==> Downloading snapshot to $LOCAL_DB..."
if [[ -f "$LOCAL_DB" ]]; then
    cp "$LOCAL_DB" "${LOCAL_DB}.bak"
    echo "    (existing $LOCAL_DB backed up to ${LOCAL_DB}.bak)"
fi
scp "$REMOTE:$REMOTE_TMP" "$LOCAL_DB"

echo "==> Cleaning up remote snapshot..."
ssh "$REMOTE" "rm -f $REMOTE_TMP"

echo "==> Done. Local DB ready at $LOCAL_DB"
