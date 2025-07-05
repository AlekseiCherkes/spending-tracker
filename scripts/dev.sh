#!/bin/bash
# Development utility script for spending-tracker

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to print colored output
print_section() {
    echo -e "${BLUE}==== $1 ====${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Show help
show_help() {
    echo -e "${BLUE}🛠️  Development Script for Spending Tracker${NC}"
    echo ""
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  check    - Run all code quality checks"
    echo "  fix      - Auto-fix formatting issues"
    echo "  test     - Run tests only"
    echo "  install  - Install development dependencies"
    echo "  setup    - Full development setup"
    echo "  clean    - Clean build artifacts"
    echo "  help     - Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 check     # Run quality checks"
    echo "  $0 fix       # Auto-format code"
    echo "  $0 test      # Run tests"
}

# Check if virtual environment is active
check_venv() {
    if [[ -z "$VIRTUAL_ENV" ]]; then
        print_warning "Not in a virtual environment"
        echo "Run: source venv/bin/activate"
        return 1
    fi
    return 0
}

# Install development dependencies
install_deps() {
    print_section "Installing Development Dependencies"

    if ! check_venv; then
        exit 1
    fi

    pip install -r requirements-dev.txt
    pip install -r requirements.txt

    print_success "Dependencies installed"
}

# Setup development environment
setup_dev() {
    print_section "Setting up Development Environment"

    # Create virtual environment if it doesn't exist
    if [[ ! -d "venv" ]]; then
        print_section "Creating virtual environment"
        python -m venv venv
        print_success "Virtual environment created"
        echo "Run: source venv/bin/activate"
        echo "Then run: $0 setup"
        exit 0
    fi

    if ! check_venv; then
        exit 1
    fi

    # Install dependencies
    install_deps

    # Install pre-commit hooks
    print_section "Installing pre-commit hooks"
    pre-commit install

    print_success "Development environment ready!"
}

# Clean build artifacts
clean_build() {
    print_section "Cleaning Build Artifacts"

    # Remove Python cache
    find . -type d -name "__pycache__" -exec rm -rf {} +
    find . -type f -name "*.pyc" -delete

    # Remove build directories
    rm -rf build/
    rm -rf dist/
    rm -rf *.egg-info/

    # Remove test cache
    rm -rf .pytest_cache/
    rm -rf .mypy_cache/

    print_success "Build artifacts cleaned"
}

# Main command dispatch
case "${1:-help}" in
    check)
        ./scripts/format-code.sh
        ;;
    fix)
        ./scripts/fix-code.sh
        ;;
    test)
        if ! check_venv; then
            exit 1
        fi
        print_section "Running Tests"
        python -m pytest tests/ -v
        ;;
    install)
        install_deps
        ;;
    setup)
        setup_dev
        ;;
    clean)
        clean_build
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        print_error "Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac
