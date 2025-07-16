#!/bin/bash

##############################################################################
# Spending Tracker Bot - Backup Script
#
# Automatically backs up SQLite database with rotation
##############################################################################

set -euo pipefail

# Configuration
APP_DIR="${APP_DIR:-/opt/spending-tracker}"
DB_PATH="${DB_PATH:-$APP_DIR/data/spending_tracker.db}"
BACKUP_DIR="$APP_DIR/backups"
RETENTION_DAYS="${BACKUP_RETENTION_DAYS:-7}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() {
    echo -e "[$(date +'%Y-%m-%d %H:%M:%S')] $1"
}

log_success() {
    echo -e "[$(date +'%Y-%m-%d %H:%M:%S')] ${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "[$(date +'%Y-%m-%d %H:%M:%S')] ${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "[$(date +'%Y-%m-%d %H:%M:%S')] ${RED}❌ $1${NC}"
}

create_backup_dir() {
    if [[ ! -d "$BACKUP_DIR" ]]; then
        mkdir -p "$BACKUP_DIR"
        log_info "Created backup directory: $BACKUP_DIR"
    fi
}

check_database() {
    if [[ ! -f "$DB_PATH" ]]; then
        log_error "Database file not found: $DB_PATH"
        exit 1
    fi

    # Check if database is accessible
    if ! sqlite3 "$DB_PATH" "SELECT 1;" >/dev/null 2>&1; then
        log_error "Database is corrupted or inaccessible: $DB_PATH"
        exit 1
    fi

    log_info "Database check passed: $DB_PATH"
}

create_backup() {
    local backup_file="$BACKUP_DIR/spending_tracker_$TIMESTAMP.db"

    log_info "Creating backup: $backup_file"

    # Use SQLite's .backup command for consistent backup
    if sqlite3 "$DB_PATH" ".backup $backup_file"; then
        log_success "Backup created successfully: $backup_file"

        # Verify backup integrity
        if sqlite3 "$backup_file" "PRAGMA integrity_check;" | grep -q "ok"; then
            log_success "Backup integrity verified"
        else
            log_error "Backup integrity check failed"
            rm -f "$backup_file"
            exit 1
        fi

        # Show backup size
        local backup_size=$(du -h "$backup_file" | cut -f1)
        log_info "Backup size: $backup_size"

        echo "$backup_file"
    else
        log_error "Failed to create backup"
        exit 1
    fi
}

rotate_old_backups() {
    log_info "Rotating old backups (keeping last $RETENTION_DAYS days)..."

    local deleted_count=0
    while IFS= read -r -d '' file; do
        rm -f "$file"
        ((deleted_count++))
        log_info "Deleted old backup: $(basename "$file")"
    done < <(find "$BACKUP_DIR" -name "spending_tracker_*.db" -mtime +$RETENTION_DAYS -print0 2>/dev/null)

    if [[ $deleted_count -eq 0 ]]; then
        log_info "No old backups to delete"
    else
        log_success "Deleted $deleted_count old backup(s)"
    fi
}

show_backup_summary() {
    local backup_count=$(find "$BACKUP_DIR" -name "spending_tracker_*.db" 2>/dev/null | wc -l)
    local total_size=$(du -sh "$BACKUP_DIR" 2>/dev/null | cut -f1)

    echo
    echo "=== Backup Summary ==="
    echo "Total backups: $backup_count"
    echo "Total size: $total_size"
    echo "Backup directory: $BACKUP_DIR"
    echo

    if [[ $backup_count -gt 0 ]]; then
        echo "Recent backups:"
        find "$BACKUP_DIR" -name "spending_tracker_*.db" -printf "%T@ %p\n" 2>/dev/null | \
            sort -nr | head -5 | while read -r timestamp file; do
            local date_str=$(date -d "@$timestamp" '+%Y-%m-%d %H:%M:%S')
            local size=$(du -h "$file" | cut -f1)
            echo "  $date_str - $(basename "$file") ($size)"
        done
    fi
}

# Handle script interruption
trap 'log_error "Backup interrupted"; exit 1' INT TERM

# Main backup process
main() {
    log_info "Starting database backup process..."

    create_backup_dir
    check_database

    local backup_file
    backup_file=$(create_backup)

    rotate_old_backups
    show_backup_summary

    log_success "Backup process completed successfully"

    # Return backup file path for use by other scripts
    echo "BACKUP_FILE=$backup_file"
}

# Allow script to be sourced or run directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
