#!/usr/bin/env bash
set -euo pipefail

REMOTE="spending_tracker"
REMOTE_DIR="/opt/spending-tracker"

echo "==> Running checks..."
./scripts/check.sh

echo "==> Cross-compiling for x86_64-unknown-linux-musl..."
cross build --release --target x86_64-unknown-linux-musl

echo "==> Uploading binary..."
scp target/x86_64-unknown-linux-musl/release/spending-tracker "$REMOTE:$REMOTE_DIR/"

echo "==> Uploading server scripts..."
scp scripts/backup.sh scripts/check_logs.sh "$REMOTE:$REMOTE_DIR/scripts/"

echo "==> Restarting service..."
ssh "$REMOTE" sudo systemctl restart spending-tracker

echo "==> Verifying..."
ssh "$REMOTE" systemctl is-active spending-tracker

echo "==> Deploy complete!"
