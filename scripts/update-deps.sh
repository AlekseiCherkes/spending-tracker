#!/bin/bash
# Update all dependency lock files

echo "🔄 Updating dependency lock files..."

echo "📦 Updating base requirements..."
pip-compile pyproject.toml

echo "🛠️  Updating dev requirements..."
pip-compile --extra dev pyproject.toml --output-file requirements-dev.txt

echo "🤖 Updating telegram requirements..."
pip-compile --extra telegram pyproject.toml --output-file requirements-telegram.txt

echo "✅ All dependency lock files updated!"
echo ""
echo "To install dependencies:"
echo "  pip install -r requirements.txt              # Base dependencies"
echo "  pip install -r requirements-dev.txt          # Development dependencies"
echo "  pip install -r requirements-telegram.txt     # Telegram bot dependencies"
echo ""
echo "Or use pip-sync for exact environment matching:"
echo "  pip-sync requirements.txt                     # Sync to exact base deps"
echo "  pip-sync requirements-dev.txt                 # Sync to exact dev deps"
