#!/bin/bash
# Automatically fix code formatting issues

set -e

echo "🔧 Auto-fixing code formatting..."

# Check if we're in a virtual environment
if [[ -z "$VIRTUAL_ENV" ]]; then
    echo "⚠️  Warning: Not in a virtual environment. Make sure you have the dev dependencies installed."
fi

# Run isort to fix import sorting
echo "📦 Fixing imports with isort..."
python -m isort spending_tracker/ tests/

# Run black to fix code formatting
echo "🖤 Fixing code formatting with black..."
python -m black spending_tracker/ tests/

echo "✅ Code formatting fixed!"
echo "💡 You can now run './scripts/format-code.sh' to check quality." 