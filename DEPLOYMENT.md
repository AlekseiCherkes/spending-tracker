# Docker-Based Deployment Guide

This guide covers the new Docker-based deployment approach for the Spending Tracker Telegram Bot. This approach emphasizes simplicity, testability, and isolation.

## Overview

The new deployment system is built on the following principles:
- **Build locally, deploy remotely** - No source code on production
- **Full testability** - Every component can be tested locally
- **Minimal complexity** - ~70 lines vs 1300+ lines of the old approach
- **No system dependencies** - Only Docker required on production
- **Immutable deployments** - Same image from dev to prod

## Architecture

```
Local Development          Production Server
─────────────────          ─────────────────
Source Code                Docker Image Only
↓                         ↓
Build & Test              Load & Run
↓                         ↓
Export Image              Deploy
↓
Upload via SSH ──────────→ Production Ready
```

## System Requirements

### Production Server
- **Any Linux distribution** with Docker support
- **Docker** and **Docker Compose** installed
- **SSH access** with sudo privileges
- **Minimum resources**: 512MB RAM, 1GB disk space

### Development Machine
- **Docker** and **Docker Compose**
- **SSH access** to production server
- **Git** for source control

## Quick Start

### 1. Local Development Setup

```bash
# Clone repository
git clone YOUR_REPOSITORY_URL spending-tracker
cd spending-tracker

# Test locally
./scripts/local-test.sh
```

### 2. Production Server Setup (One-time)

```bash
# SSH to your server
ssh your-server

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Logout and login again for group changes
exit
ssh your-server

# Create application directory
sudo mkdir -p /opt/spending-tracker/{data,logs,backups}
sudo chown $(whoami): /opt/spending-tracker

# Create environment file
echo "TELEGRAM_BOT_TOKEN=your_bot_token_here" > /opt/spending-tracker/.env

exit
```

### 3. Configure SSH Access

Add to your `~/.ssh/config`:

```
Host spending_tracker
    HostName your-server-ip
    User your-username
    IdentityFile ~/.ssh/your-key
    ForwardAgent yes
```

### 4. Deploy

```bash
# From your local development machine
./scripts/build-and-deploy.sh
```

## File Structure

```
spending-tracker/
├── Dockerfile                    # Application container definition
├── docker-compose.local.yml      # Local development environment
├── docker-compose.prod.yml       # Production environment template
├── scripts/
│   ├── local-test.sh             # Local testing workflow
│   └── build-and-deploy.sh       # Main deployment script
└── [source code...]
```

## Development Workflow

### Local Testing

```bash
# Quick local test
./scripts/local-test.sh

# Manual testing
docker-compose -f docker-compose.local.yml up -d
docker-compose -f docker-compose.local.yml logs -f
docker-compose -f docker-compose.local.yml down
```

### Deployment

```bash
# Full deployment with tests
./scripts/build-and-deploy.sh

# Skip local tests (faster)
./scripts/build-and-deploy.sh --skip-test

# Deploy and show logs immediately
./scripts/build-and-deploy.sh --logs
```

## Production Management

### Service Status

```bash
ssh spending_tracker 'cd /opt/spending-tracker && sudo docker-compose ps'
```

### View Logs

```bash
ssh spending_tracker 'cd /opt/spending-tracker && sudo docker-compose logs -f spending-tracker'
```

### Restart Service

```bash
ssh spending_tracker 'cd /opt/spending-tracker && sudo docker-compose restart'
```

### Stop Service

```bash
ssh spending_tracker 'cd /opt/spending-tracker && sudo docker-compose down'
```

### Check Health

```bash
ssh spending_tracker 'sudo docker-compose -f /opt/spending-tracker/docker-compose.yml ps'
ssh spending_tracker 'sudo docker stats spending-tracker --no-stream'
```

## Backup and Recovery

### Automatic Backups

The system includes automatic daily backups:
- **Schedule**: Daily at midnight
- **Retention**: 7 days
- **Location**: `/opt/spending-tracker/backups/`

### Manual Backup

```bash
ssh spending_tracker 'sudo docker-compose -f /opt/spending-tracker/docker-compose.yml exec spending-tracker cp /app/data/spending_tracker.db /app/backups/manual_backup_$(date +%Y%m%d_%H%M%S).db'
```

### Restore from Backup

```bash
# Stop the service
ssh spending_tracker 'cd /opt/spending-tracker && sudo docker-compose down'

# Copy backup to data directory
ssh spending_tracker 'sudo cp /opt/spending-tracker/backups/backup_YYYYMMDD_HHMMSS.db /opt/spending-tracker/data/spending_tracker.db'

# Restart the service
ssh spending_tracker 'cd /opt/spending-tracker && sudo docker-compose up -d'
```

## Monitoring

### Health Checks

The application includes built-in health checks:
- **Interval**: Every 30 seconds
- **Timeout**: 10 seconds
- **Retries**: 3 attempts

### Resource Monitoring

```bash
# Check container resource usage
ssh spending_tracker 'sudo docker stats --no-stream'

# Check disk usage
ssh spending_tracker 'df -h /opt/spending-tracker'

# Check memory usage
ssh spending_tracker 'free -h'
```

### Log Analysis

```bash
# Application logs
ssh spending_tracker 'cd /opt/spending-tracker && sudo docker-compose logs spending-tracker'

# Backup container logs
ssh spending_tracker 'cd /opt/spending-tracker && sudo docker-compose logs backup'

# System journal
ssh spending_tracker 'sudo journalctl -u docker'
```

## Troubleshooting

### Common Issues

**Deployment fails with SSH error**:
```bash
# Test SSH connection
ssh spending_tracker 'echo "SSH OK"'

# Check SSH config
cat ~/.ssh/config | grep -A5 spending_tracker
```

**Container fails to start**:
```bash
# Check container logs
ssh spending_tracker 'cd /opt/spending-tracker && sudo docker-compose logs spending-tracker'

# Check if .env file exists
ssh spending_tracker 'ls -la /opt/spending-tracker/.env'
```

**Image build fails locally**:
```bash
# Check Docker daemon
docker version

# Check Dockerfile syntax
docker build --no-cache .
```

### Recovery Procedures

**Complete system recovery**:
```bash
# 1. Save backups
ssh spending_tracker 'sudo cp -r /opt/spending-tracker/backups /tmp/'

# 2. Clean install
ssh spending_tracker 'sudo rm -rf /opt/spending-tracker'
ssh spending_tracker 'sudo mkdir -p /opt/spending-tracker/{data,logs,backups}'
ssh spending_tracker 'sudo chown $(whoami): /opt/spending-tracker'

# 3. Restore configuration
ssh spending_tracker 'echo "TELEGRAM_BOT_TOKEN=your_token" > /opt/spending-tracker/.env'

# 4. Restore backups
ssh spending_tracker 'sudo cp -r /tmp/backups /opt/spending-tracker/'

# 5. Redeploy
./scripts/build-and-deploy.sh
```

## Performance Optimization

### Resource Limits

The Docker containers are configured with appropriate resource limits:
- **Memory**: Unlimited (will use only what's needed)
- **CPU**: Unlimited (burstable on cloud instances)
- **Disk**: Automatic cleanup of old backups

### Network Optimization

- Uses bridge networking for isolation
- No exposed ports (only outbound connections to Telegram API)
- Minimal network overhead

## Security Considerations

### Container Security
- **Non-root user**: Application runs as user ID 1000
- **Read-only filesystem**: Except for data directories
- **Minimal image**: Based on Python slim image
- **No shell access**: Production containers don't include shell

### Network Security
- **No inbound ports**: Only outbound HTTPS to Telegram API
- **Isolated network**: Containers run in isolated Docker network
- **SSH-only access**: No web interfaces or APIs exposed

### Data Security
- **Environment variables**: Stored in protected .env file
- **Database**: SQLite file with filesystem permissions
- **Backups**: Local filesystem (consider encryption for sensitive data)

## Migration from Old System

### From Legacy systemd Deployment

```bash
# 1. Stop old service
ssh spending_tracker 'sudo systemctl stop spending-tracker'
ssh spending_tracker 'sudo systemctl disable spending-tracker'

# 2. Backup existing data
ssh spending_tracker 'sudo cp /opt/spending-tracker/data/spending_tracker.db /tmp/backup.db'

# 3. Clean old installation
ssh spending_tracker 'sudo rm -rf /opt/spending-tracker'

# 4. Follow production setup above

# 5. Restore data
ssh spending_tracker 'cp /tmp/backup.db /opt/spending-tracker/data/spending_tracker.db'

# 6. Deploy new system
./scripts/build-and-deploy.sh
```

## Scaling Considerations

### Single Server Scaling
- **Vertical scaling**: Increase server resources
- **Current limits**: ~1000 concurrent users per GB RAM

### Multi-Server Scaling
- **Horizontal scaling**: Deploy to multiple servers
- **Database**: Consider PostgreSQL for shared database
- **Load balancing**: Use Telegram webhook with load balancer

## Cost Optimization

### Cloud Provider Recommendations
- **GCP e2-micro**: $5-7/month (always free tier eligible)
- **AWS t3.nano**: $3-5/month
- **DigitalOcean Basic**: $5/month

### Resource Usage
- **Memory**: ~50-100MB baseline
- **CPU**: <5% on micro instances
- **Disk**: <1GB including backups
- **Network**: Minimal (only Telegram API calls)

---

**Last Updated**: January 2024
**Version**: 2.0 (Docker-based)
**Migration**: From systemd-based deployment
