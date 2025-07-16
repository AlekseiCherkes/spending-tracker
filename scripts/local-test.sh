#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

echo "🧪 Local Testing Workflow for Spending Tracker Bot"
echo "=================================================="

# Check if we're in the right directory
if [ ! -f "docker-compose.local.yml" ]; then
    log_error "docker-compose.local.yml not found. Run this script from project root."
    exit 1
fi

# Set test token if not provided
if [ -z "$TELEGRAM_BOT_TOKEN" ]; then
    log_warning "TELEGRAM_BOT_TOKEN not set, using test token"
    export TELEGRAM_BOT_TOKEN="test_token_for_local_testing"
fi

# Create local directories
log_info "Creating local data directories..."
mkdir -p local-data local-logs

# Stop any existing containers
log_info "Stopping existing containers..."
docker compose -f docker-compose.local.yml down 2>/dev/null || true

# Build the image
log_info "Building Docker image..."
docker compose -f docker-compose.local.yml build

# Run basic tests
log_info "Running basic image tests..."
CONTAINER_ID=$(docker run -d \
    --platform linux/amd64 \
    -e TELEGRAM_BOT_TOKEN="test_token" \
    -e DB_PATH="/app/data/test.db" \
    spending-tracker:latest)

# Wait for container to start and attempt initialization
sleep 5

# Check container logs for successful application startup
LOGS=$(docker logs "$CONTAINER_ID" 2>&1)
if echo "$LOGS" | grep -q "Starting Spending Tracker Bot"; then
    log_success "Application starts correctly (expected token error is normal for testing)"
else
    log_error "Application failed to start properly"
    echo "Container logs:"
    echo "$LOGS"
    docker rm -f "$CONTAINER_ID" 2>/dev/null || true
    exit 1
fi

# Clean up test container
docker rm -f "$CONTAINER_ID"

# Start local environment
log_info "Starting local development environment..."
docker compose -f docker-compose.local.yml up -d

# Check status
log_info "Checking container status..."
sleep 5
docker compose -f docker-compose.local.yml ps

# Show logs
echo ""
log_success "🎉 Local environment is ready!"
echo "=================================================="
echo "Useful commands:"
echo "  View logs:           docker compose -f docker-compose.local.yml logs -f"
echo "  Stop environment:    docker compose -f docker-compose.local.yml down"
echo "  Restart:             docker compose -f docker-compose.local.yml restart"
echo "  Shell access:        docker compose -f docker-compose.local.yml exec spending-tracker bash"
echo ""

# Ask if user wants to see logs
read -p "Show application logs? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Showing logs (Ctrl+C to exit):"
    docker compose -f docker-compose.local.yml logs -f spending-tracker
fi
