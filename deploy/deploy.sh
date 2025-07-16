#!/bin/bash

##############################################################################
# Spending Tracker Bot - Production Deployment Script for GCP e2-micro
#
# This script automates the full deployment process on Ubuntu 24.04
# Designed for GCP e2-micro instances (0.25 vCPU, 1GB RAM)
##############################################################################

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
APP_NAME="spending-tracker"
APP_USER="spending-tracker"
APP_DIR="/opt/spending-tracker"
REPO_URL="${REPO_URL:-}"
TELEGRAM_BOT_TOKEN="${TELEGRAM_BOT_TOKEN:-}"

# Functions
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

check_ubuntu() {
    if ! grep -q "Ubuntu 24.04" /etc/os-release; then
        log_warning "This script is designed for Ubuntu 24.04, but will continue..."
    fi
}

get_telegram_token() {
    if [[ -z "$TELEGRAM_BOT_TOKEN" ]]; then
        echo
        log_info "Please enter your Telegram Bot Token:"
        read -r -s TELEGRAM_BOT_TOKEN
        echo
        if [[ -z "$TELEGRAM_BOT_TOKEN" ]]; then
            log_error "Telegram Bot Token is required!"
            exit 1
        fi
    fi
}

get_repo_url() {
    if [[ -z "$REPO_URL" ]]; then
        echo
        log_info "Please enter your Git repository URL:"
        read -r REPO_URL
        if [[ -z "$REPO_URL" ]]; then
            log_error "Repository URL is required!"
            exit 1
        fi
    fi
}

install_dependencies() {
    log_info "Installing system dependencies..."

    apt update
    apt install -y \
        python3.12 \
        python3.12-venv \
        python3.12-dev \
        git \
        sqlite3 \
        ufw \
        curl \
        htop \
        logrotate

    log_success "System dependencies installed"
}

create_app_user() {
    log_info "Creating application user..."

    if id "$APP_USER" &>/dev/null; then
        log_warning "User $APP_USER already exists"
    else
        useradd -m -s /bin/bash "$APP_USER"
        log_success "User $APP_USER created"
    fi

    # Create application directory
    mkdir -p "$APP_DIR"
    chown "$APP_USER:$APP_USER" "$APP_DIR"
}

setup_application() {
    log_info "Setting up application..."

    # Switch to app user for git operations
    sudo -u "$APP_USER" bash << EOF
        cd "$APP_DIR"

        # Clone or update repository
        if [[ -d .git ]]; then
            log_info "Updating existing repository..."
            git pull origin main
        else
            log_info "Cloning repository..."
            git clone "$REPO_URL" .
        fi

        # Create Python virtual environment
        python3.12 -m venv venv
        source venv/bin/activate

        # Install Python dependencies
        pip install --upgrade pip
        pip install -r requirements-telegram.txt

        # Create data directory
        mkdir -p data
        mkdir -p logs
        mkdir -p backups
EOF

    log_success "Application setup completed"
}

create_env_file() {
    log_info "Creating environment configuration..."

    cat > "$APP_DIR/.env" << EOF
# Telegram Bot Configuration
TELEGRAM_BOT_TOKEN=$TELEGRAM_BOT_TOKEN

# Application Configuration
LOG_LEVEL=INFO
DB_PATH=$APP_DIR/data/spending_tracker.db

# Deployment Configuration
ENVIRONMENT=production
EOF

    chown "$APP_USER:$APP_USER" "$APP_DIR/.env"
    chmod 600 "$APP_DIR/.env"

    log_success "Environment file created"
}

setup_systemd_service() {
    log_info "Setting up systemd service..."

    cat > /etc/systemd/system/"$APP_NAME".service << EOF
[Unit]
Description=Spending Tracker Telegram Bot
After=network.target
StartLimitIntervalSec=0

[Service]
Type=simple
User=$APP_USER
Group=$APP_USER
WorkingDirectory=$APP_DIR
EnvironmentFile=$APP_DIR/.env
ExecStart=$APP_DIR/venv/bin/python -m spending_tracker
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$APP_DIR

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable "$APP_NAME"

    log_success "Systemd service configured"
}

setup_firewall() {
    log_info "Configuring firewall..."

    # Enable UFW if not already enabled
    if ! ufw status | grep -q "Status: active"; then
        ufw --force enable
    fi

    # Allow SSH
    ufw allow ssh

    log_success "Firewall configured"
}

setup_log_rotation() {
    log_info "Setting up log rotation..."

    cat > /etc/logrotate.d/"$APP_NAME" << EOF
$APP_DIR/logs/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    copytruncate
    su $APP_USER $APP_USER
}
EOF

    log_success "Log rotation configured"
}

run_tests() {
    log_info "Running pre-deployment tests..."

    sudo -u "$APP_USER" bash << EOF
        cd "$APP_DIR"
        source venv/bin/activate

        # Check if all dependencies are installed
        python -c "import spending_tracker; print('✅ Package import successful')"

        # Run quality checks if available
        if [[ -f scripts/dev.sh ]]; then
            echo "Running quality checks..."
            if ./scripts/dev.sh check; then
                echo "✅ All quality checks passed"
            else
                echo "❌ Quality checks failed - deployment aborted"
                exit 1
            fi
        else
            echo "⚠️  Quality check script not found, skipping checks"
        fi
EOF

    log_success "Tests completed"
}

start_service() {
    log_info "Starting the service..."

    systemctl start "$APP_NAME"
    sleep 3

    if systemctl is-active --quiet "$APP_NAME"; then
        log_success "Service started successfully"
    else
        log_error "Service failed to start"
        systemctl status "$APP_NAME"
        exit 1
    fi
}

show_status() {
    echo
    echo "================================================================"
    log_success "Deployment completed successfully!"
    echo "================================================================"
    echo
    echo "Service Status:"
    systemctl status "$APP_NAME" --no-pager -l
    echo
    echo "Useful Commands:"
    echo "  - View logs:           sudo journalctl -u $APP_NAME -f"
    echo "  - Restart service:     sudo systemctl restart $APP_NAME"
    echo "  - Stop service:        sudo systemctl stop $APP_NAME"
    echo "  - Check status:        sudo systemctl status $APP_NAME"
    echo "  - Run backup:          sudo -u $APP_USER $APP_DIR/deploy/backup.sh"
    echo "  - Update application:  sudo $APP_DIR/deploy/update.sh"
    echo
    echo "Application Directory: $APP_DIR"
    echo "Configuration File:    $APP_DIR/.env"
    echo "Database Location:     $APP_DIR/data/spending_tracker.db"
    echo
}

# Main deployment process
main() {
    echo "================================================================"
    log_info "Starting Spending Tracker Bot Deployment"
    log_info "Target: GCP e2-micro instance with Ubuntu 24.04"
    echo "================================================================"

    check_root
    check_ubuntu
    get_telegram_token
    get_repo_url

    install_dependencies
    create_app_user
    setup_application
    create_env_file
    setup_systemd_service
    setup_firewall
    setup_log_rotation
    run_tests
    start_service
    show_status
}

# Handle script interruption
trap 'log_error "Deployment interrupted"; exit 1' INT TERM

# Run main function
main "$@"
