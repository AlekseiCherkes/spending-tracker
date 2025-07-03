# Spending Tracker Telegram Bot

A Telegram bot for personal spending tracking.

## Getting Started

### Prerequisites
- Python 3.13+
- Telegram Bot Token (get from [@BotFather](https://t.me/BotFather))

### Installation
1. Clone the repository
2. Create and activate a virtual environment:
   ```bash
   # Create virtual environment
   python3 -m venv venv
   
   # Activate virtual environment
   # On macOS/Linux:
   source venv/bin/activate
   
   # On Windows:
   venv\Scripts\activate
   ```
3. Install dependencies:
   ```bash
   # Install all dependencies (base + telegram)
   pip install -r requirements.txt
   pip install -r requirements-telegram.txt
   
   # For development work
   pip install -r requirements-dev.txt
   ```
4. Install the package in development mode (optional):
   ```bash
   pip install -e .
   ```

### Setting up the Bot
1. Create a `.env` file in the project root:
   ```bash
   TELEGRAM_BOT_TOKEN=your_bot_token_here
   ```
2. Get your bot token from [@BotFather](https://t.me/BotFather) on Telegram

### Running the Bot
```bash
# Make sure your virtual environment is activated first!
# You should see (venv) in your prompt

# Run as a module
python -m spending_tracker

# Or if you installed it as a package:
spending-tracker
```

### Virtual Environment Management
```bash
# Activate virtual environment
source venv/bin/activate  # macOS/Linux
venv\Scripts\activate     # Windows

# Deactivate virtual environment
deactivate

# Check if virtual environment is active
# Look for (venv) in your prompt
```

## 🤖 Bot Commands

Current bot commands:
- `/start` - Welcome message and introduction
- `/help` - Show available commands
- `/status` - Check bot status
- `/about` - Information about the bot

## 🔒 Dependency Management

This project uses **pip-tools** for dependency locking to ensure reproducible builds:

### Understanding the Files
- `pyproject.toml` - Source of truth for dependencies with version ranges
- `requirements.txt` - Locked base dependencies with exact versions
- `requirements-dev.txt` - Locked development dependencies
- `requirements-telegram.txt` - Locked telegram bot dependencies

### Managing Dependencies

#### Adding New Dependencies
1. Edit `pyproject.toml` to add new dependencies:
   ```toml
   dependencies = [
       "click>=8.0",
       "python-dotenv>=1.0.0",
       "your-new-package>=1.0.0",  # Add here
   ]
   ```

2. Update lock files:
   ```bash
   # Option 1: Use the script
   ./scripts/update-deps.sh
   
   # Option 2: Manual update
   pip-compile pyproject.toml
   pip-compile --extra dev pyproject.toml --output-file requirements-dev.txt
   pip-compile --extra telegram pyproject.toml --output-file requirements-telegram.txt
   ```

3. Install the new dependencies:
   ```bash
   pip install -r requirements.txt
   # or use pip-sync for exact matching
   pip-sync requirements.txt
   ```

#### Why We Lock Dependencies
- ✅ **Reproducible builds** - Same versions everywhere
- ✅ **Security** - Know exactly what versions you're using
- ✅ **Debugging** - Consistent environment for troubleshooting
- ✅ **CI/CD** - Reliable automated builds

## Current Status
✅ Basic Python module structure  
✅ Python 3.13+ requirement  
✅ Modern project configuration (pyproject.toml)  
✅ Virtual environment setup  
✅ Dependency locking with pip-tools  
✅ Basic Telegram bot functionality  
🔄 Coming soon: Database integration  
🔄 Coming soon: Expense tracking features  
🔄 Coming soon: Spending categories  
🔄 Coming soon: Monthly reports  

## Project Structure
```
spending-tracker/
├── spending_tracker/         # Main package directory
│   ├── __init__.py          # Package initialization
│   ├── __main__.py          # Main entry point (starts bot)
│   └── bot.py               # Telegram bot implementation
├── tests/                   # Test suite
│   ├── __init__.py
│   └── test_main.py        # Core functionality tests
├── scripts/                 # Development scripts
│   └── update-deps.sh       # Update dependency lock files
├── venv/                    # Virtual environment (excluded from git)
├── pyproject.toml           # Modern Python project configuration
├── requirements.txt         # Locked base dependencies
├── requirements-dev.txt     # Locked development dependencies
├── requirements-telegram.txt # Locked telegram dependencies
├── ARCHITECTURE.md          # Architecture decisions and rationale
├── README.md               # This file
└── .gitignore              # Git ignore rules
```

## Architecture

For detailed information about architectural decisions, technology choices, and future considerations, see [ARCHITECTURE.md](ARCHITECTURE.md).

## Development

### Running Tests
```bash
pytest tests/ -v
```

### Code Quality
```bash
# Format code
black spending_tracker/

# Lint code
flake8 spending_tracker/

# Type checking
mypy spending_tracker/
```

### Development Plan
We're building this step by step:
1. ✅ Basic Python module structure
2. ✅ Python 3.13+ requirement with modern configuration
3. ✅ Virtual environment setup
4. ✅ Dependency locking with pip-tools
5. ✅ Basic Telegram bot interface
6. ✅ Focused testing strategy
7. 🔄 Add database functionality (SQLite + SQLAlchemy)
8. 🔄 Add expense tracking features
9. 🔄 Add spending categories
10. 🔄 Add reporting and analytics

## Contributing

1. Review [ARCHITECTURE.md](ARCHITECTURE.md) for context
2. Create a feature branch
3. Add tests first (TDD approach)
4. Implement feature
5. Run full test suite
6. Update documentation if needed
