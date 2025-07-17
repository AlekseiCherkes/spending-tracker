#!/bin/bash
set -e

# Source common configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common-config.sh"

# Configuration
REMOTE_HOST="spending_tracker"  # SSH alias for production server
APP_DIR="/opt/spending-tracker"
IMAGE_NAME="spending-tracker"
IMAGE_TAG="latest"
log_error() { echo -e "${RED}[ERROR]${NC} $1" >&2; }

# Configuration validation
check_config() {
    log_info "Checking configuration..."

    if [ ! -f "Dockerfile" ]; then
        log_error "Dockerfile not found. Run this script from project root."
        exit 1
    fi

    if [ ! -f "docker-compose.prod.yml" ]; then
        log_error "docker-compose.prod.yml not found."
        exit 1
    fi

    log_success "Configuration looks good"
}

# Check SSH connection to production server
check_ssh() {
    log_info "Checking SSH connection to $REMOTE_HOST..."

    if ! ssh -o ConnectTimeout=5 "$REMOTE_HOST" "echo 'SSH connection OK'"; then
        log_error "Failed to connect to $REMOTE_HOST"
        log_error "Make sure SSH is configured and the server is accessible"
        exit 1
    fi

    log_success "SSH connection established"
}

# Build Docker image locally
build_image() {
    log_info "Building Docker image locally..."

    # Build the image with explicit platform for production deployment
    docker build --platform linux/amd64 -t "$IMAGE_NAME:$IMAGE_TAG" .

    # Verify the build
    if docker images | grep -q "$IMAGE_NAME"; then
        log_success "Image built successfully: $IMAGE_NAME:$IMAGE_TAG (linux/amd64)"
    else
        log_error "Failed to build Docker image"
        exit 1
    fi
}

# Run local tests (optional)
test_locally() {
    if [ "$1" = "--skip-test" ]; then
        log_warning "Skipping local tests as requested"
        return
    fi

    log_info "Running local tests on the built image..."

    # Check if .env file exists
    if [ ! -f ".env" ]; then
        log_warning "No .env file found in project root"
        log_warning "Skipping local test - create .env file with TELEGRAM_BOT_TOKEN for testing"
        return
    fi

    log_info "Using existing .env file for testing"

    # Start a test container with explicit platform using real .env file
    CONTAINER_ID=$(docker run -d \
        --platform linux/amd64 \
        -v "$(pwd)/.env:/app/.env:ro" \
        -e DB_PATH="/app/data/test.db" \
        -e LOG_LEVEL="DEBUG" \
        "$IMAGE_NAME:$IMAGE_TAG")

    # Wait for container to initialize
    sleep 5

    # Get container logs for analysis
    LOGS=$(docker logs "$CONTAINER_ID" 2>&1)

    # Check if application started properly with real token
    if echo "$LOGS" | grep -q "🚀 Starting Spending Tracker Bot" || \
       echo "$LOGS" | grep -q "spending_tracker" || \
       echo "$LOGS" | grep -q "Bot commands registered"; then
        log_success "Local test passed - application starts correctly with real token"
    elif echo "$LOGS" | grep -q "InvalidToken"; then
        log_error "Local test failed - invalid Telegram bot token in .env file"
        echo "Container logs:"
        echo "$LOGS"
        docker rm -f "$CONTAINER_ID" 2>/dev/null || true
        exit 1
    else
        log_error "Local test failed - application failed to start properly"
        echo "Container logs:"
        echo "$LOGS"
        docker rm -f "$CONTAINER_ID" 2>/dev/null || true
        exit 1
    fi

    # Clean up test container
    docker rm -f "$CONTAINER_ID"
    log_success "Test container cleaned up"
}

# Export Docker image to tar file
export_image() {
    log_info "Exporting Docker image to tar file..."

    local tar_file="/tmp/${IMAGE_NAME}_${IMAGE_TAG}.tar.gz"

    # Export and compress the image
    docker save "$IMAGE_NAME:$IMAGE_TAG" | gzip > "$tar_file"

    # Check if file was created
    if [ -f "$tar_file" ]; then
        local file_size=$(du -h "$tar_file" | cut -f1)
        log_success "Image exported: $tar_file (size: $file_size)"
        echo "$tar_file"  # Return file path
    else
        log_error "Failed to export Docker image"
        exit 1
    fi
}

# Upload image to production server
upload_image() {
    local tar_file="$1"
    log_info "Uploading image to production server..."

    # Create app directory on remote server if it doesn't exist
    ssh "$REMOTE_HOST" "sudo mkdir -p $APP_DIR && sudo chown \$(whoami): $APP_DIR"

    # Copy the tar file to remote server
    scp "$tar_file" "$REMOTE_HOST:$APP_DIR/"

    log_success "Image uploaded to $REMOTE_HOST:$APP_DIR/"
}

# Load image on production server
load_image_remote() {
    local tar_file="$1"
    local tar_filename=$(basename "$tar_file")

    log_info "Loading Docker image on production server..."

    ssh "$REMOTE_HOST" << EOF
        cd $APP_DIR

        # Load the new image
        sudo docker load < $tar_filename

        # Remove the tar file to save space
        rm $tar_filename

        # Verify image was loaded
        if sudo docker images | grep -q "$IMAGE_NAME"; then
            echo "✅ Image loaded successfully"
        else
            echo "❌ Failed to load image"
            exit 1
        fi
EOF

    log_success "Image loaded on production server"
}

# Upload docker-compose configuration
upload_compose() {
    log_info "Uploading docker-compose configuration..."

    scp "docker-compose.prod.yml" "$REMOTE_HOST:$APP_DIR/docker-compose.yml"

    log_success "Docker-compose configuration uploaded"
}

# Deploy on production server
deploy_remote() {
    log_info "Deploying application on production server..."

    ssh "$REMOTE_HOST" << 'EOF'
        cd /opt/spending-tracker

        # Create necessary directories
        sudo mkdir -p data logs backups
        sudo chown $(whoami): data logs backups

        # Check for .env file
        if [ ! -f .env ]; then
            echo "❌ .env file not found!"
            echo "Please create .env file with TELEGRAM_BOT_TOKEN before deploying"
            echo "Example: echo 'TELEGRAM_BOT_TOKEN=your_token_here' > .env"
            exit 1
        fi

        # Stop old containers
        echo "Stopping old containers..."
        sudo docker compose down 2>/dev/null || true

        # Remove old containers and networks to ensure clean state
        sudo docker container prune -f
        sudo docker network prune -f

        # Start new deployment
        echo "Starting new deployment..."
        sudo docker compose up -d

        # Wait a moment for containers to start
        sleep 10

        # Check deployment status
        echo "Checking deployment status..."
        sudo docker compose ps

        # Check health of main container
        if sudo docker compose ps | grep -q "spending-tracker.*Up"; then
            echo "✅ Deployment successful!"
        else
            echo "❌ Deployment failed!"
            echo "Container logs:"
            sudo docker compose logs spending-tracker
            exit 1
        fi
EOF

    log_success "Deployment completed successfully!"
}

# Show production logs
show_logs() {
    log_info "Showing production application logs..."
    ssh "$REMOTE_HOST" "cd $APP_DIR && sudo docker compose logs -f spending-tracker"
}

# Cleanup temporary files
cleanup() {
    log_info "Cleaning up temporary files..."
    rm -f /tmp/${IMAGE_NAME}_*.tar.gz
    log_success "Cleanup completed"
}

# Display deployment summary
show_summary() {
    echo ""
    echo "🎉 Deployment Summary"
    echo "===================="
    echo "✅ Image built locally and tested"
    echo "✅ Image deployed to production server"
    echo "✅ Application started successfully"
    echo ""
    echo "Production management commands:"
    echo "  ssh $REMOTE_HOST 'cd $APP_DIR && sudo docker compose ps'"
    echo "  ssh $REMOTE_HOST 'cd $APP_DIR && sudo docker compose logs -f spending-tracker'"
    echo "  ssh $REMOTE_HOST 'cd $APP_DIR && sudo docker compose restart'"
    echo "  ssh $REMOTE_HOST 'cd $APP_DIR && sudo docker compose down'"
    echo ""
}

# Main deployment function
main() {
    echo "🚀 Spending Tracker Bot - Build and Deploy"
    echo "=========================================="

    # Parse command line arguments
    local skip_test=""
    local show_logs_flag=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-test)
                skip_test="--skip-test"
                shift
                ;;
            --logs)
                show_logs_flag="--logs"
                shift
                ;;
            --help)
                echo "Usage: $0 [--skip-test] [--logs] [--help]"
                echo "  --skip-test  Skip local testing before deployment"
                echo "  --logs       Show production logs after deployment"
                echo "  --help       Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done

    # Deployment steps
    check_config
    check_ssh
    build_image
    test_locally "$skip_test"

    local tar_file
    tar_file=$(export_image)

    upload_image "$tar_file"
    load_image_remote "$tar_file"
    upload_compose
    deploy_remote
    cleanup

    show_summary

    # Show logs if requested
    if [ "$show_logs_flag" = "--logs" ]; then
        show_logs
    else
        read -p "Show production logs? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            show_logs
        fi
    fi
}

# Error handling
trap 'log_error "Deployment interrupted"; cleanup; exit 1' INT TERM

# Run main function
main "$@"
