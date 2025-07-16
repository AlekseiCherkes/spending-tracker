#!/bin/bash

##############################################################################
# Spending Tracker Bot - Monitoring Script
#
# Monitors system resources and service health for e2-micro instances
##############################################################################

set -euo pipefail

# Configuration
APP_NAME="spending-tracker"
SERVICE_NAME="spending-tracker"
APP_DIR="/opt/spending-tracker"
LOG_FILE="/var/log/spending-tracker-monitor.log"

# Thresholds for e2-micro (adjust as needed)
MEMORY_THRESHOLD=80  # Percent
DISK_THRESHOLD=85    # Percent
CPU_THRESHOLD=70     # Percent

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

log_to_file() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
}

log_info() {
    local msg="$1"
    echo -e "${BLUE}[INFO]${NC} $msg"
    log_to_file "INFO: $msg"
}

log_success() {
    local msg="$1"
    echo -e "${GREEN}[OK]${NC} $msg"
    log_to_file "OK: $msg"
}

log_warning() {
    local msg="$1"
    echo -e "${YELLOW}[WARNING]${NC} $msg"
    log_to_file "WARNING: $msg"
}

log_error() {
    local msg="$1"
    echo -e "${RED}[ERROR]${NC} $msg"
    log_to_file "ERROR: $msg"
}

check_service_status() {
    local status

    if systemctl is-active --quiet "$SERVICE_NAME"; then
        status="running"
        local uptime=$(systemctl show "$SERVICE_NAME" --property=ActiveEnterTimestamp --value)
        log_success "Service is running (started: $uptime)"
    else
        status="stopped"
        log_error "Service is NOT running"

        # Show last few log entries to help diagnose
        echo "Recent service logs:"
        journalctl -u "$SERVICE_NAME" --since "10 minutes ago" --no-pager | tail -5
    fi

    echo "$status"
}

check_memory_usage() {
    local mem_info
    mem_info=$(free | grep Mem)

    local total=$(echo "$mem_info" | awk '{print $2}')
    local used=$(echo "$mem_info" | awk '{print $3}')
    local available=$(echo "$mem_info" | awk '{print $7}')

    local used_percent=$((used * 100 / total))
    local available_mb=$((available / 1024))

    if [[ $used_percent -gt $MEMORY_THRESHOLD ]]; then
        log_warning "High memory usage: ${used_percent}% (${available_mb}MB available)"
    else
        log_success "Memory usage: ${used_percent}% (${available_mb}MB available)"
    fi

    # Show top memory consumers
    if [[ $used_percent -gt $MEMORY_THRESHOLD ]]; then
        echo "Top memory consumers:"
        ps aux --sort=-%mem | head -6
    fi

    echo "$used_percent"
}

check_disk_usage() {
    local disk_info
    disk_info=$(df / | tail -1)

    local used_percent=$(echo "$disk_info" | awk '{print $5}' | sed 's/%//')
    local available=$(echo "$disk_info" | awk '{print $4}')
    local available_gb=$((available / 1024 / 1024))

    if [[ $used_percent -gt $DISK_THRESHOLD ]]; then
        log_warning "High disk usage: ${used_percent}% (${available_gb}GB available)"
    else
        log_success "Disk usage: ${used_percent}% (${available_gb}GB available)"
    fi

    # Check application directory size
    if [[ -d "$APP_DIR" ]]; then
        local app_size=$(du -sh "$APP_DIR" | cut -f1)
        log_info "Application directory size: $app_size"
    fi

    echo "$used_percent"
}

check_cpu_usage() {
    # Get 1-minute average CPU usage
    local cpu_usage
    cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | awk -F'%' '{print $1}')

    # Remove any floating point
    local cpu_percent=${cpu_usage%.*}

    if [[ $cpu_percent -gt $CPU_THRESHOLD ]]; then
        log_warning "High CPU usage: ${cpu_percent}%"

        # Show top CPU consumers
        echo "Top CPU consumers:"
        ps aux --sort=-%cpu | head -6
    else
        log_success "CPU usage: ${cpu_percent}%"
    fi

    echo "$cpu_percent"
}

check_database() {
    local db_path="$APP_DIR/data/spending_tracker.db"

    if [[ -f "$db_path" ]]; then
        local db_size=$(du -h "$db_path" | cut -f1)
        log_info "Database size: $db_size"

        # Check database integrity (if service is not running or in maintenance)
        if ! systemctl is-active --quiet "$SERVICE_NAME"; then
            if sqlite3 "$db_path" "PRAGMA integrity_check;" | grep -q "ok"; then
                log_success "Database integrity: OK"
            else
                log_error "Database integrity: FAILED"
            fi
        fi
    else
        log_warning "Database file not found: $db_path"
    fi
}

check_network() {
    # Check if we can reach Telegram API
    if curl -s --max-time 10 https://api.telegram.org >/dev/null; then
        log_success "Network connectivity: OK (Telegram API reachable)"
    else
        log_error "Network connectivity: FAILED (Cannot reach Telegram API)"
    fi
}

check_logs() {
    # Check for recent errors in service logs
    local error_count
    error_count=$(journalctl -u "$SERVICE_NAME" --since "10 minutes ago" | grep -i "error\|exception\|failed" | wc -l)

    if [[ $error_count -eq 0 ]]; then
        log_success "No recent errors in service logs"
    else
        log_warning "Found $error_count error(s) in recent logs"
        echo "Recent errors:"
        journalctl -u "$SERVICE_NAME" --since "10 minutes ago" | grep -i "error\|exception\|failed" | tail -3
    fi
}

check_backup_status() {
    local backup_dir="$APP_DIR/backups"

    if [[ -d "$backup_dir" ]]; then
        local backup_count=$(find "$backup_dir" -name "*.db" -mtime -1 | wc -l)
        local latest_backup=$(find "$backup_dir" -name "*.db" -type f -printf '%T@ %p\n' | sort -nr | head -1 | cut -d' ' -f2-)

        if [[ $backup_count -gt 0 ]]; then
            local backup_age=$(stat -c %Y "$latest_backup")
            local current_time=$(date +%s)
            local age_hours=$(( (current_time - backup_age) / 3600 ))

            if [[ $age_hours -lt 25 ]]; then
                log_success "Recent backup found (${age_hours}h ago)"
            else
                log_warning "Latest backup is ${age_hours}h old"
            fi
        else
            log_warning "No recent backups found"
        fi
    else
        log_warning "Backup directory not found"
    fi
}

generate_alert() {
    local alert_type="$1"
    local message="$2"

    # Log alert
    log_to_file "ALERT [$alert_type]: $message"

    # In production, you might want to send notifications here
    # Examples:
    # - Send email
    # - Post to Slack/Discord webhook
    # - Send Telegram message to admin

    echo "🚨 ALERT [$alert_type]: $message"
}

show_summary() {
    echo
    echo "=== System Health Summary ==="
    echo "Timestamp: $(date)"
    echo "Hostname: $(hostname)"
    echo "Uptime: $(uptime -p)"
    echo

    # Service status
    local service_status
    service_status=$(check_service_status)

    # Resource usage
    local mem_usage
    local disk_usage
    local cpu_usage

    mem_usage=$(check_memory_usage)
    disk_usage=$(check_disk_usage)
    cpu_usage=$(check_cpu_usage)

    echo
    echo "=== Resource Usage ==="
    echo "Memory: ${mem_usage}%"
    echo "Disk: ${disk_usage}%"
    echo "CPU: ${cpu_usage}%"
    echo

    # Generate alerts if needed
    if [[ $mem_usage -gt $MEMORY_THRESHOLD ]]; then
        generate_alert "HIGH_MEMORY" "Memory usage is ${mem_usage}% (threshold: ${MEMORY_THRESHOLD}%)"
    fi

    if [[ $disk_usage -gt $DISK_THRESHOLD ]]; then
        generate_alert "HIGH_DISK" "Disk usage is ${disk_usage}% (threshold: ${DISK_THRESHOLD}%)"
    fi

    if [[ $cpu_usage -gt $CPU_THRESHOLD ]]; then
        generate_alert "HIGH_CPU" "CPU usage is ${cpu_usage}% (threshold: ${CPU_THRESHOLD}%)"
    fi

    if [[ "$service_status" != "running" ]]; then
        generate_alert "SERVICE_DOWN" "Spending Tracker service is not running"
    fi
}

# Main monitoring function
main() {
    local mode="${1:-summary}"

    case "$mode" in
        "summary"|"")
            show_summary
            ;;
        "service")
            check_service_status >/dev/null
            ;;
        "memory")
            check_memory_usage >/dev/null
            ;;
        "disk")
            check_disk_usage >/dev/null
            ;;
        "cpu")
            check_cpu_usage >/dev/null
            ;;
        "database")
            check_database
            ;;
        "network")
            check_network
            ;;
        "logs")
            check_logs
            ;;
        "backup")
            check_backup_status
            ;;
        "full")
            show_summary
            echo
            check_database
            check_network
            check_logs
            check_backup_status
            ;;
        *)
            echo "Usage: $0 [summary|service|memory|disk|cpu|database|network|logs|backup|full]"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
