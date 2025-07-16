# Deployment Guide for GCP e2-micro

This guide covers deploying the Spending Tracker Telegram Bot on a Google Cloud Platform e2-micro instance running Ubuntu 24.04.

## Overview

The deployment package includes:
- **Automated deployment script** - Sets up the entire environment
- **Backup system** - Automated SQLite database backups
- **Update mechanism** - Safe updates with rollback capability
- **Monitoring** - Resource monitoring optimized for e2-micro
- **Systemd service** - Reliable service management

## System Requirements

### GCP e2-micro Instance
- **CPU**: 0.25 vCPU (burstable to 0.5 vCPU)
- **Memory**: 1 GB RAM
- **Storage**: 10 GB persistent disk (standard)
- **OS**: Ubuntu 24.04 LTS
- **Network**: Allow HTTP/HTTPS outbound (for Telegram API)

### Prerequisites
- Telegram Bot Token from [@BotFather](https://t.me/BotFather)
- Git repository with your bot code
- SSH access to your GCP instance

## Quick Start

### 1. Prepare Your Instance

```bash
# Connect to your GCP instance
gcloud compute ssh your-instance-name

# Update system packages
sudo apt update && sudo apt upgrade -y
```

### 2. Clone and Deploy

```bash
# Clone your repository
git clone YOUR_REPOSITORY_URL spending-tracker
cd spending-tracker

# Make deployment script executable
chmod +x scripts/deploy.sh

# Run deployment (requires sudo)
sudo ./scripts/deploy.sh
```

The script will prompt you for:
- Telegram Bot Token
- Git repository URL (if not set via environment)

### 3. Verify Deployment

```bash
# Check service status
sudo systemctl status spending-tracker

# View logs
sudo journalctl -u spending-tracker -f

# Run health check
sudo /opt/spending-tracker/scripts/monitor.sh
```

## Deployment Scripts Reference

### Main Deployment Script

**Location**: `scripts/deploy.sh`

**Purpose**: Automates the complete deployment process

**Usage**:
```bash
# Interactive deployment
sudo ./scripts/deploy.sh

# Non-interactive with environment variables
sudo TELEGRAM_BOT_TOKEN="your_token" REPO_URL="your_repo" ./scripts/deploy.sh
```

**What it does**:
1. Installs system dependencies (Python 3.12, git, sqlite3, etc.)
2. Creates dedicated `spending-tracker` user
3. Clones repository and sets up Python virtual environment
4. Creates `.env` configuration file
5. Sets up systemd service with security hardening
6. Configures firewall (UFW)
7. Sets up log rotation
8. Runs pre-deployment tests
9. Starts the service

### Backup Script

**Location**: `scripts/backup.sh`

**Purpose**: Creates consistent SQLite database backups with rotation

**Usage**:
```bash
# Manual backup
sudo -u spending-tracker /opt/spending-tracker/scripts/backup.sh

# Automated via cron (set up separately)
```

**Features**:
- Uses SQLite's `.backup` command for consistency
- Verifies backup integrity
- Rotates old backups (7 days retention by default)
- Returns backup file path for use by other scripts

### Update Script

**Location**: `scripts/update.sh`

**Purpose**: Safely updates the application with backup and rollback

**Usage**:
```bash
# Update to latest version
sudo /opt/spending-tracker/scripts/update.sh
```

**Process**:
1. Creates pre-update backup
2. Stops the service gracefully
3. Updates code from git repository
4. Updates Python dependencies
5. Runs quality checks
6. Starts service and verifies health
7. Rolls back automatically if any step fails

### Monitoring Script

**Location**: `scripts/monitor.sh`

**Purpose**: Monitors system resources and service health

**Usage**:
```bash
# Basic health summary
sudo /opt/spending-tracker/scripts/monitor.sh

# Specific checks
sudo /opt/spending-tracker/scripts/monitor.sh memory
sudo /opt/spending-tracker/scripts/monitor.sh service
sudo /opt/spending-tracker/scripts/monitor.sh database

# Full monitoring report
sudo /opt/spending-tracker/scripts/monitor.sh full
```

**Monitoring Areas**:
- Service status and uptime
- Memory usage (threshold: 80%)
- Disk usage (threshold: 85%)
- CPU usage (threshold: 70%)
- Database integrity
- Network connectivity (Telegram API)
- Recent error logs
- Backup status

### Cron Setup Script

**Location**: `scripts/cron-setup.sh`

**Purpose**: Sets up automated tasks

**Usage**:
```bash
sudo /opt/spending-tracker/scripts/cron-setup.sh
```

**Scheduled Tasks**:
- **Daily backup**: 2:00 AM
- **Monitoring**: Every 5 minutes
- **Weekly report**: Sundays 3:00 AM
- **Log rotation**: Daily 1:00 AM
- **Log cleanup**: Sundays 4:00 AM

## Configuration

### Environment Variables

The deployment creates `/opt/spending-tracker/.env` with these variables:

```bash
# Required
TELEGRAM_BOT_TOKEN=your_bot_token_here

# Application settings
LOG_LEVEL=INFO
DB_PATH=/opt/spending-tracker/data/spending_tracker.db
ENVIRONMENT=production

# Optional tuning
BACKUP_RETENTION_DAYS=7
MEMORY_THRESHOLD=80
DISK_THRESHOLD=85
CPU_THRESHOLD=70
```

### Service Configuration

**Systemd Unit**: `/etc/systemd/system/spending-tracker.service`

**Key Features**:
- Automatic restart on failure
- Security hardening (NoNewPrivileges, PrivateTmp, etc.)
- Proper working directory and environment
- Journal logging

## Operational Tasks

### Daily Operations

```bash
# Check service health
sudo systemctl status spending-tracker

# View recent logs
sudo journalctl -u spending-tracker --since "1 hour ago"

# Check resource usage
sudo /opt/spending-tracker/scripts/monitor.sh

# Manual backup
sudo -u spending-tracker /opt/spending-tracker/scripts/backup.sh
```

### Updates

```bash
# Check for updates (doesn't apply them)
sudo -u spending-tracker git -C /opt/spending-tracker fetch

# Apply updates safely
sudo /opt/spending-tracker/scripts/update.sh
```

### Troubleshooting

```bash
# Service not starting
sudo systemctl status spending-tracker
sudo journalctl -u spending-tracker -n 50

# High resource usage
sudo /opt/spending-tracker/scripts/monitor.sh full

# Database issues
sudo -u spending-tracker sqlite3 /opt/spending-tracker/data/spending_tracker.db "PRAGMA integrity_check;"

# Check bot connectivity
curl -s https://api.telegram.org/botYOUR_TOKEN/getMe
```

## Security Considerations

### System Security
- Dedicated non-privileged user for the application
- Systemd security hardening enabled
- UFW firewall configured (SSH only)
- Application files owned by `spending-tracker` user

### Application Security
- Environment variables stored in protected `.env` file (600 permissions)
- Database file in restricted directory
- No sudo privileges for application user

### Data Security
- Automatic encrypted backups (if using encrypted storage)
- Database integrity checks
- Retention policy for sensitive logs

## Monitoring and Alerting

### Built-in Monitoring
The monitoring script tracks:
- **Service health**: Running/stopped status
- **Resource usage**: Memory, CPU, disk
- **Database status**: Size, integrity
- **Network**: Telegram API connectivity
- **Logs**: Recent errors and exceptions

### Setting Up Alerts
For production use, consider integrating with:
- **Email notifications**: Modify `scripts/monitor.sh` to send emails
- **Telegram alerts**: Send messages to admin chat
- **External monitoring**: Uptime Robot, DataDog, etc.

### Log Analysis
```bash
# Application logs
sudo journalctl -u spending-tracker

# Monitor logs
tail -f /opt/spending-tracker/logs/monitor.log

# Backup logs
tail -f /opt/spending-tracker/logs/backup.log
```

## Performance Optimization for e2-micro

### Memory Management
- **Current baseline**: ~50-100MB for the bot
- **SQLite**: Minimal memory footprint
- **Python**: Virtual environment keeps dependencies isolated

### CPU Optimization
- **Async operations**: All I/O is non-blocking
- **Efficient polling**: Uses Telegram's webhook mode if configured
- **Minimal processing**: Direct SQL queries, no ORM overhead

### Storage Management
- **Database**: SQLite with automatic backup rotation
- **Logs**: Automatic rotation and cleanup
- **Backups**: 7-day retention by default

## Scaling Considerations

### When to Upgrade from e2-micro
Consider upgrading when:
- Memory usage consistently >80%
- Response time >2 seconds
- More than 100 active users
- Need for multiple bot instances

### Migration Path
1. **e2-small**: 2 vCPU, 2GB RAM - handles 200+ users
2. **Container Engine**: For multiple bots
3. **Cloud SQL**: For high availability database

## Backup and Recovery

### Backup Strategy
- **Frequency**: Daily automatic backups
- **Retention**: 7 days (configurable)
- **Storage**: Local disk (consider Cloud Storage for critical data)
- **Verification**: Automatic integrity checks

### Recovery Procedures

```bash
# List available backups
ls -la /opt/spending-tracker/backups/

# Restore from backup
sudo systemctl stop spending-tracker
sudo -u spending-tracker cp /opt/spending-tracker/backups/spending_tracker_YYYYMMDD_HHMMSS.db /opt/spending-tracker/data/spending_tracker.db
sudo systemctl start spending-tracker

# Verify restoration
sudo /opt/spending-tracker/scripts/monitor.sh database
```

## Maintenance Schedule

### Daily
- Automatic backups (2:00 AM)
- Log rotation (1:00 AM)
- Resource monitoring (every 5 minutes)

### Weekly
- Full system report (Sundays 3:00 AM)
- Old log cleanup (Sundays 4:00 AM)

### Monthly
- Review backup retention policy
- Check for system updates
- Review monitoring thresholds

### Quarterly
- Security updates review
- Performance optimization review
- Disaster recovery testing

## Support and Troubleshooting

### Common Issues

**Service won't start**:
```bash
# Check configuration
sudo -u spending-tracker /opt/spending-tracker/venv/bin/python -m spending_tracker --help

# Check bot token
curl -s https://api.telegram.org/botYOUR_TOKEN/getMe
```

**High memory usage**:
```bash
# Check memory consumers
ps aux --sort=-%mem | head -10

# Restart service
sudo systemctl restart spending-tracker
```

**Database corruption**:
```bash
# Check integrity
sudo -u spending-tracker sqlite3 /opt/spending-tracker/data/spending_tracker.db "PRAGMA integrity_check;"

# Restore from backup
sudo /opt/spending-tracker/scripts/update.sh  # Uses latest backup for rollback
```

### Getting Help

1. Check service logs: `sudo journalctl -u spending-tracker`
2. Run full monitoring: `sudo /opt/spending-tracker/scripts/monitor.sh full`
3. Review application logs in `/opt/spending-tracker/logs/`
4. Test bot manually: `sudo -u spending-tracker /opt/spending-tracker/venv/bin/python -m spending_tracker`

---

**Last Updated**: January 2024
**Version**: 1.0
**Tested On**: Ubuntu 24.04 LTS, GCP e2-micro
