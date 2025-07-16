#!/bin/bash

##############################################################################
# Spending Tracker Bot - Cron Jobs Setup Script
#
# Sets up automated tasks for backup and monitoring
##############################################################################

set -euo pipefail

# Configuration
APP_USER="spending-tracker"
APP_DIR="/opt/spending-tracker"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

check_cron_available() {
    log_info "Checking if cron is available..."

    if ! command -v crontab >/dev/null 2>&1; then
        log_error "Cron is not installed!"
        log_error "Please run the deployment script first: sudo ./deploy/deploy.sh"
        exit 1
    fi

    # Verify cron service is running
    if ! systemctl is-active --quiet cron; then
        log_error "Cron service is not running!"
        log_error "Please start cron service: sudo systemctl start cron"
        exit 1
    fi

    log_success "Cron is available and running"
}

setup_app_user_cron() {
    log_info "Setting up cron jobs for $APP_USER..."

    # Create temporary cron file
    local temp_cron=$(mktemp)

    # Get existing cron jobs
    sudo -u "$APP_USER" crontab -l > "$temp_cron" 2>/dev/null || true

        # Add backup job (daily at 2:00 AM)
    if ! grep -q "backup.sh" "$temp_cron"; then
        echo "# Daily backup at 2:00 AM" >> "$temp_cron"
        echo "0 2 * * * $APP_DIR/deploy/backup.sh >> $APP_DIR/logs/backup.log 2>&1" >> "$temp_cron"
        log_info "Added daily backup job"
    else
        log_warning "Backup job already exists"
    fi

    # Add monitoring job (every 5 minutes)
    if ! grep -q "monitor.sh" "$temp_cron"; then
        echo "# System monitoring every 5 minutes" >> "$temp_cron"
        echo "*/5 * * * * $APP_DIR/deploy/monitor.sh summary >> $APP_DIR/logs/monitor.log 2>&1" >> "$temp_cron"
        log_info "Added monitoring job"
    else
        log_warning "Monitoring job already exists"
    fi

    # Add weekly full monitoring (Sundays at 3:00 AM)
    if ! grep -q "monitor.sh full" "$temp_cron"; then
        echo "# Weekly full monitoring report" >> "$temp_cron"
        echo "0 3 * * 0 $APP_DIR/deploy/monitor.sh full >> $APP_DIR/logs/weekly-report.log 2>&1" >> "$temp_cron"
        log_info "Added weekly monitoring report"
    else
        log_warning "Weekly monitoring job already exists"
    fi

    # Install cron jobs
    sudo -u "$APP_USER" crontab "$temp_cron"
    rm -f "$temp_cron"

    log_success "Cron jobs installed for $APP_USER"
}

setup_root_cron() {
    log_info "Setting up system-level cron jobs..."

    # Create temporary cron file
    local temp_cron=$(mktemp)

    # Get existing root cron jobs
    crontab -l > "$temp_cron" 2>/dev/null || true

    # Add log rotation trigger (daily at 1:00 AM)
    if ! grep -q "logrotate.*spending-tracker" "$temp_cron"; then
        echo "# Force log rotation for spending-tracker" >> "$temp_cron"
        echo "0 1 * * * /usr/sbin/logrotate -f /etc/logrotate.d/spending-tracker >/dev/null 2>&1" >> "$temp_cron"
        log_info "Added log rotation job"
    else
        log_warning "Log rotation job already exists"
    fi

    # Add cleanup job for old monitor logs (weekly)
    if ! grep -q "cleanup.*monitor.log" "$temp_cron"; then
        echo "# Clean old monitor logs weekly" >> "$temp_cron"
        echo "0 4 * * 0 find $APP_DIR/logs -name 'monitor.log.*' -mtime +30 -delete >/dev/null 2>&1" >> "$temp_cron"
        log_info "Added log cleanup job"
    else
        log_warning "Log cleanup job already exists"
    fi

    # Install root cron jobs
    crontab "$temp_cron"
    rm -f "$temp_cron"

    log_success "System cron jobs installed"
}

create_log_directory() {
    log_info "Creating log directory structure..."

    # Create logs directory if it doesn't exist
    sudo -u "$APP_USER" mkdir -p "$APP_DIR/logs"

    # Create initial log files with proper permissions
    local log_files=(
        "backup.log"
        "monitor.log"
        "weekly-report.log"
    )

    for log_file in "${log_files[@]}"; do
        local full_path="$APP_DIR/logs/$log_file"
        if [[ ! -f "$full_path" ]]; then
            sudo -u "$APP_USER" touch "$full_path"
            log_info "Created log file: $log_file"
        fi
    done

    log_success "Log directory structure created"
}

show_cron_status() {
    echo
    echo "=== Cron Jobs Status ==="
    echo
    echo "Application user ($APP_USER) cron jobs:"
    sudo -u "$APP_USER" crontab -l | grep -v "^#" || echo "No cron jobs found"
    echo
    echo "System (root) cron jobs:"
    crontab -l | grep spending-tracker || echo "No spending-tracker related cron jobs found"
    echo
    echo "Next scheduled runs:"
    echo "  - Backup: Daily at 2:00 AM"
    echo "  - Monitoring: Every 5 minutes"
    echo "  - Weekly report: Sundays at 3:00 AM"
    echo "  - Log rotation: Daily at 1:00 AM"
    echo "  - Log cleanup: Sundays at 4:00 AM"
    echo
}

# Main setup process
main() {
    echo "================================================================"
    log_info "Setting up automated tasks for Spending Tracker Bot"
    echo "================================================================"

    check_root
    check_cron_available
    create_log_directory
    setup_app_user_cron
    setup_root_cron
    show_cron_status

    log_success "Cron setup completed successfully!"
    echo
    echo "Log files location: $APP_DIR/logs/"
    echo "To view cron logs: sudo journalctl -u cron"
}

# Handle script interruption
trap 'log_error "Cron setup interrupted"; exit 1' INT TERM

# Run main function
main "$@"
