#!/bin/bash

##############################################################################
# Spending Tracker Bot - Update Script
#
# Safely updates the application with backup and rollback capabilities
##############################################################################

set -euo pipefail

# Configuration
APP_NAME="spending-tracker"
APP_USER="spending-tracker"
APP_DIR="/opt/spending-tracker"
SERVICE_NAME="spending-tracker"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} [$(date +'%Y-%m-%d %H:%M:%S')] $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} [$(date +'%Y-%m-%d %H:%M:%S')] $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} [$(date +'%Y-%m-%d %H:%M:%S')] $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} [$(date +'%Y-%m-%d %H:%M:%S')] $1"
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

check_service_status() {
    if systemctl is-active --quiet "$SERVICE_NAME"; then
        echo "running"
    else
        echo "stopped"
    fi
}

create_pre_update_backup() {
    log_info "Creating pre-update backup..."

    # Run backup script as app user
    local backup_output
    if backup_output=$(sudo -u "$APP_USER" "$APP_DIR/scripts/backup.sh" 2>&1); then
        local backup_file=$(echo "$backup_output" | grep "BACKUP_FILE=" | cut -d'=' -f2)
        log_success "Pre-update backup created: $backup_file"
        echo "$backup_file"
    else
        log_error "Failed to create pre-update backup"
        echo "$backup_output"
        exit 1
    fi
}

stop_service() {
    log_info "Stopping $SERVICE_NAME service..."

    if systemctl is-active --quiet "$SERVICE_NAME"; then
        systemctl stop "$SERVICE_NAME"

        # Wait for service to stop
        local attempts=0
        while systemctl is-active --quiet "$SERVICE_NAME" && [[ $attempts -lt 30 ]]; do
            sleep 1
            ((attempts++))
        done

        if systemctl is-active --quiet "$SERVICE_NAME"; then
            log_error "Service did not stop within 30 seconds"
            exit 1
        fi

        log_success "Service stopped successfully"
    else
        log_info "Service was not running"
    fi
}

update_code() {
    log_info "Updating application code..."

    sudo -u "$APP_USER" bash << 'EOF'
        cd "$APP_DIR"

        # Store current commit hash for potential rollback
        current_commit=$(git rev-parse HEAD)
        echo "$current_commit" > .last_commit

        # Fetch latest changes
        git fetch origin

        # Get commit info
        current_branch=$(git branch --show-current)
        log_info "Current branch: $current_branch"
        log_info "Current commit: $current_commit"

        # Check if there are updates
        if git diff --quiet HEAD origin/$current_branch; then
            log_info "No code updates available"
            exit 0
        fi

        # Show what will be updated
        echo "Changes to be applied:"
        git log --oneline HEAD..origin/$current_branch | head -10

        # Apply updates
        git pull origin $current_branch

        new_commit=$(git rev-parse HEAD)
        log_success "Code updated to commit: $new_commit"
EOF

    log_success "Code update completed"
}

update_dependencies() {
    log_info "Updating Python dependencies..."

    sudo -u "$APP_USER" bash << 'EOF'
        cd "$APP_DIR"
        source venv/bin/activate

        # Update pip first
        pip install --upgrade pip

        # Install/update dependencies
        pip install -r requirements-telegram.txt

        log_success "Dependencies updated"
EOF

    log_success "Dependencies update completed"
}

run_quality_checks() {
    log_info "Running quality checks..."

    sudo -u "$APP_USER" bash << 'EOF'
        cd "$APP_DIR"
        source venv/bin/activate

        # Run pre-commit checks if available
        if [[ -f scripts/dev.sh ]]; then
            if ./scripts/dev.sh check; then
                log_success "All quality checks passed"
            else
                log_error "Quality checks failed"
                exit 1
            fi
        else
            # Basic checks
            python -c "import spending_tracker; print('✅ Package import successful')"
        fi
EOF

    log_success "Quality checks passed"
}

start_service() {
    log_info "Starting $SERVICE_NAME service..."

    systemctl start "$SERVICE_NAME"

    # Wait for service to start and verify it's healthy
    sleep 5

    local attempts=0
    while ! systemctl is-active --quiet "$SERVICE_NAME" && [[ $attempts -lt 30 ]]; do
        sleep 1
        ((attempts++))
    done

    if systemctl is-active --quiet "$SERVICE_NAME"; then
        log_success "Service started successfully"
    else
        log_error "Service failed to start"
        systemctl status "$SERVICE_NAME" --no-pager
        return 1
    fi
}

verify_deployment() {
    log_info "Verifying deployment..."

    # Check service health
    if ! systemctl is-active --quiet "$SERVICE_NAME"; then
        log_error "Service is not running"
        return 1
    fi

    # Check for any critical errors in recent logs
    if journalctl -u "$SERVICE_NAME" --since "1 minute ago" | grep -i "error\|exception\|failed" | grep -v "backup"; then
        log_warning "Found errors in recent logs, check manually"
    fi

    log_success "Deployment verification completed"
}

rollback() {
    local backup_file="$1"

    log_warning "Rolling back to previous version..."

    # Stop service
    systemctl stop "$SERVICE_NAME" || true

    # Restore code
    sudo -u "$APP_USER" bash << 'EOF'
        cd "$APP_DIR"

        if [[ -f .last_commit ]]; then
            last_commit=$(cat .last_commit)
            log_info "Rolling back to commit: $last_commit"
            git reset --hard "$last_commit"
        fi
EOF

    # Restore database if backup file provided
    if [[ -n "$backup_file" && -f "$backup_file" ]]; then
        log_info "Restoring database from backup..."
        sudo -u "$APP_USER" cp "$backup_file" "$APP_DIR/data/spending_tracker.db"
    fi

    # Restart service
    if start_service; then
        log_success "Rollback completed successfully"
    else
        log_error "Rollback failed - manual intervention required"
        exit 1
    fi
}

show_update_summary() {
    echo
    echo "=== Update Summary ==="
    echo "Service Status: $(systemctl is-active "$SERVICE_NAME")"
    echo "Current Commit: $(sudo -u "$APP_USER" git -C "$APP_DIR" rev-parse HEAD)"
    echo "Update Time: $(date)"
    echo
    echo "Recent Logs:"
    journalctl -u "$SERVICE_NAME" --since "5 minutes ago" --no-pager | tail -10
    echo
}

# Main update process
main() {
    log_info "Starting application update process..."

    check_root

    local initial_status
    initial_status=$(check_service_status)

    local backup_file=""

    # Create backup before making any changes
    backup_file=$(create_pre_update_backup)

    # Perform update
    if stop_service && \
       update_code && \
       update_dependencies && \
       run_quality_checks && \
       start_service && \
       verify_deployment; then

        log_success "Update completed successfully!"
        show_update_summary

    else
        log_error "Update failed, initiating rollback..."
        rollback "$backup_file"
        exit 1
    fi
}

# Handle script interruption
trap 'log_error "Update interrupted"; exit 1' INT TERM

# Run main function
main "$@"
