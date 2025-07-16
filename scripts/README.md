# Deployment Scripts

Automated deployment scripts for the Spending Tracker Telegram Bot optimized for GCP e2-micro instances.

## Quick Deploy

```bash
# Clone your repository
git clone YOUR_REPO_URL spending-tracker
cd spending-tracker

# Run automated deployment
sudo ./scripts/deploy.sh
```

## Available Scripts

| Script | Purpose | Usage |
|--------|---------|-------|
| `deploy.sh` | Full deployment setup | `sudo ./scripts/deploy.sh` |
| `backup.sh` | Database backup with rotation | `sudo -u spending-tracker ./scripts/backup.sh` |
| `update.sh` | Safe application updates | `sudo ./scripts/update.sh` |
| `monitor.sh` | System monitoring | `sudo ./scripts/monitor.sh [mode]` |
| `cron-setup.sh` | Setup automated tasks | `sudo ./scripts/cron-setup.sh` |

## Environment Setup

1. Copy `.env.template` to `.env`
2. Fill in your Telegram Bot Token
3. Adjust other settings as needed

## Post-Deployment

```bash
# Check service status
sudo systemctl status spending-tracker

# View logs
sudo journalctl -u spending-tracker -f

# Monitor resources
sudo /opt/spending-tracker/scripts/monitor.sh

# Setup automated tasks
sudo /opt/spending-tracker/scripts/cron-setup.sh
```

## Documentation

See `DEPLOYMENT.md` for complete deployment guide with troubleshooting and operational procedures.

## Architecture

- **Target**: GCP e2-micro (0.25 vCPU, 1GB RAM)
- **OS**: Ubuntu 24.04 LTS
- **Database**: SQLite with automated backups
- **Service**: systemd with security hardening
- **Monitoring**: Resource usage and health checks
