#!/usr/bin/env bash
set -euo pipefail

REMOTE="spending_tracker"
REMOTE_DIR="/opt/spending-tracker"
TMP_DIR="/tmp/spending-tracker-deploy"

echo "==> Running checks..."
./scripts/check.sh

echo "==> Cross-compiling for x86_64-unknown-linux-musl..."
cross build --release --target x86_64-unknown-linux-musl

echo "==> Uploading files to staging area..."
ssh "$REMOTE" "mkdir -p $TMP_DIR/scripts"
scp target/x86_64-unknown-linux-musl/release/spending-tracker "$REMOTE:$TMP_DIR/"
scp scripts/backup.sh scripts/check_logs.sh "$REMOTE:$TMP_DIR/scripts/"

echo "==> Installing files..."
ssh "$REMOTE" "sudo mv $TMP_DIR/spending-tracker $REMOTE_DIR/ && \
    sudo mv $TMP_DIR/scripts/* $REMOTE_DIR/scripts/ && \
    sudo chmod +x $REMOTE_DIR/spending-tracker $REMOTE_DIR/scripts/*.sh && \
    rm -rf $TMP_DIR"

echo "==> Restarting service..."
ssh "$REMOTE" sudo systemctl restart spending-tracker

echo "==> Verifying..."
ssh "$REMOTE" systemctl is-active spending-tracker

echo "==> Deploy complete!"
