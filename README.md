# Spending Tracker Telegram Bot

A Telegram bot for personal spending tracking.

## Getting Started

### Prerequisites
- Python 3.12+ (available out-of-the-box in Ubuntu 24.04 LTS)
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

#### For Development
1. Create a `.env` file in the project root:
   ```bash
   TELEGRAM_BOT_TOKEN=your_bot_token_here
   ```
2. Get your bot token from [@BotFather](https://t.me/BotFather) on Telegram

#### For Production Deployment
For production deployment on GCP e2-micro or other servers, use the automated deployment script:
```bash
# Clone and deploy
git clone YOUR_REPOSITORY_URL spending-tracker
cd spending-tracker
sudo ./deploy/deploy.sh
```

The deployment script will:
- Prompt you for your Telegram Bot Token interactively (secure input)
- Set up the entire production environment automatically
- Configure systemd service, monitoring, and backups

See `DEPLOYMENT.md` for complete deployment documentation.

### Running the Bot (Development)
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
- `/status` - Check bot status (includes user count)
- `/about` - Information about the bot
- `/users` - List all registered users

### 💰 Expense Tracking
- Send any message containing a number to start adding an expense
- Use the inline keyboard to select category and account
- Click "Save" to store the expense in the database

#### How to Add Expenses
1. Send a message with an amount (e.g., "15.50", "купил кофе 3.20", "обед 12")
2. The bot will automatically detect the number and create a draft expense
3. Use the inline keyboard to:
   - Change the category (defaults to first category by priority)
   - Change the account (defaults to first available account)
   - Save the expense or cancel
4. The expense will be saved with timestamp and user information

#### Features
- 🔄 **Auto-detection**: Numbers in messages are automatically recognized as expenses
- 👤 **Auto-registration**: New users are automatically registered on first interaction
- 🎯 **Smart defaults**: Uses first category and account by priority
- 📊 **Interactive interface**: Inline keyboards for easy expense management
- ✅ **Validation**: Ensures all required fields are filled before saving

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

## 🧪 Test Data

### Populating Test Data
To populate the database with test data (currencies, users, categories, and accounts), run:

```bash
# Make sure your virtual environment is activated
source venv/bin/activate

# Run the test data script
python scripts/populate_test_data.py
```

The script includes:
- **Currencies**: EUR, USD, BYN
- **Users**: Alex and Hanna with Telegram IDs
- **Categories**: 20 spending categories in Russian (prioritized order)
- **Accounts**: Sample bank accounts with IBAN numbers

The script is safe to run multiple times - it won't create duplicates.

### Test Data Details
- **Alex** (Telegram ID: 5033919666): Revolut (Joint), Nordea (Spending)
- **Hanna** (Random Telegram ID): S-pankki
- **Categories**: From "Продукты и хозтовары" to "Другое" with proper sort order
- **All accounts**: EUR currency by default

## Current Status
✅ Basic Python module structure
✅ Python 3.12+ requirement
✅ Modern project configuration (pyproject.toml)
✅ Virtual environment setup
✅ Dependency locking with pip-tools
✅ Basic Telegram bot functionality
✅ Database integration with SQLite
✅ Test data population script
✅ Expense tracking with inline keyboards
✅ Auto-registration of users
✅ In-memory state management
🔄 Coming soon: Expense viewing and reporting
🔄 Coming soon: Account management
🔄 Coming soon: Category management
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
2. ✅ Python 3.12+ requirement with modern configuration
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
