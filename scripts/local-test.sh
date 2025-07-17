#!/bin/bash
set -e

# Source common configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common-config.sh"

echo "🧪 Local Testing Workflow for Spending Tracker Bot"
echo "=================================================="

# Check if we're in the right directory
if [ ! -f "docker-compose.local.yml" ]; then
    log_error "docker-compose.local.yml not found. Run this script from project root."
    exit 1
fi

# Check if .env file exists for local testing
if [ ! -f ".env" ]; then
    log_warning ".env file not found - creating test .env file"
    log_info "For production, copy .env.example and set real token"
    echo "TELEGRAM_BOT_TOKEN=test_token_for_local_testing" > .env
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
    -v "$(pwd)/.env:/app/.env:ro" \
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
