#!/bin/bash
# Format and check code quality using standard Python tools

set -e

echo "🔧 Running code formatting and quality checks..."

# Check if we're in a virtual environment
if [[ -z "$VIRTUAL_ENV" ]]; then
    echo "⚠️  Warning: Not in a virtual environment. Make sure you have the dev dependencies installed."
fi

# Run isort for import sorting
echo "📦 Sorting imports with isort..."
python -m isort spending_tracker/ tests/ --check-only --diff

# Run black for code formatting
echo "🖤 Formatting code with black..."
python -m black spending_tracker/ tests/ --check --diff

# Run flake8 for linting
echo "🔍 Running flake8 linting..."
python -m flake8 spending_tracker/ tests/

# Run mypy for type checking
echo "🎯 Running mypy type checking..."
python -m mypy spending_tracker/

# Run tests
echo "🧪 Running tests..."
python -m pytest tests/ -v

echo "✅ All checks passed!"
