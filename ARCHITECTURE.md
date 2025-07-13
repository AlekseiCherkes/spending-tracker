# Architecture Documentation

## Overview
This document captures key architectural decisions made during the development of the Spending Tracker Telegram Bot.

## Core Architecture Decisions

### 1. **Telegram Bot as Primary Interface**
- **Decision**: Use Telegram Bot API as the main user interface
- **Rationale**:
  - Provides instant messaging experience
  - No need for separate frontend development
  - Built-in user authentication via Telegram
  - Cross-platform accessibility (mobile, desktop, web)
- **Alternative Considered**: CLI interface (removed for simplicity)

### 2. **Python 3.12+ Minimum Version**
- **Decision**: Require Python 3.12+
- **Rationale**:
  - Available out-of-the-box in Ubuntu 24.04 LTS (default system Python)
  - Stable modern Python version with performance improvements
  - Excellent async/await support for Telegram bot
  - Robust type hints and error handling
  - Wide compatibility with dependencies

### 3. **Package-Based Structure**
- **Decision**: Use `spending_tracker/` package with `__main__.py` entry point
- **Rationale**:
  - Allows both `python -m spending_tracker` and console script execution
  - Clean separation of concerns
  - Follows Python packaging best practices
  - Easy to extend with additional modules

### 4. **Single Entry Point Philosophy**
- **Decision**: Always start as bot (no CLI mode switching)
- **Rationale**:
  - Simplifies user experience
  - Reduces complexity in main module
  - Focuses development effort on one interface
  - CLI can be added later if needed

### 5. **Modern Build System**
- **Decision**: Use `pyproject.toml` with setuptools backend
- **Rationale**:
  - Modern Python packaging standard
  - Consolidates all configuration in one file
  - Better dependency management
  - Tool configuration (pytest, black, mypy) in same file

### 6. **Dependency Management Strategy**
- **Decision**: Use pip-tools for dependency locking
- **Rationale**:
  - Reproducible builds across environments
  - Separate development and production dependencies
  - Easy to update and audit dependencies
  - Telegram-specific dependencies in separate file

### 7. **Database Access Strategy**
- **Decision**: Use built-in `sqlite3` module with direct SQL
- **Rationale**:
  - No external dependencies (part of Python standard library)
  - Complete transparency - can see exactly what SQL is executed
  - Simple and well-established
  - No ORM complexity or abstraction layers
  - Easy to understand and debug
- **Alternative Considered**: SQLAlchemy ORM (rejected for complexity)

### 8. **Single DAL Class Architecture**
- **Decision**: Use a single DAL class instead of separate DAL classes per table
- **Rationale**:
  - Simplified connection management and sharing
  - Easier handling of complex queries across multiple tables
  - Cleaner transaction management for multi-table operations
  - Reduced code complexity and coupling
  - Single point of database configuration and connection pooling
- **Alternative Considered**: Multiple DAL classes (rejected for complexity in joins)

## Current Project Structure

```
spending-tracker/
├── spending_tracker/           # Main package
│   ├── __init__.py            # Package metadata
│   ├── __main__.py            # Entry point (starts bot)
│   ├── bot.py                 # Telegram bot implementation
│   └── dal.py                 # Data Access Layer (SQLite)
├── tests/                     # Test suite
│   ├── __init__.py
│   ├── test_main.py          # Core functionality tests
│   └── test_dal.py           # Database tests
├── scripts/                   # Utility scripts
│   └── update-deps.sh        # Dependency update script
├── requirements*.txt          # Locked dependencies
├── pyproject.toml            # Project configuration
└── venv/                     # Virtual environment
```

## Technology Stack

### Core Dependencies
- **python-telegram-bot**: Official Telegram Bot API wrapper
- **python-dotenv**: Environment variable management
- **click**: Command-line interface utilities (future CLI use)

### Development Dependencies
- **pytest**: Testing framework
- **pytest-asyncio**: Async testing support
- **black**: Code formatting
- **flake8**: Linting
- **mypy**: Type checking
- **pip-tools**: Dependency management

### Database Stack
- **sqlite3**: Built-in Python SQLite driver (no external dependencies)
- **Direct SQL**: Raw SQL queries for transparency and control
- **Single DAL**: Unified Data Access Layer with all table operations

## Database Schema

### Tables
1. **users**: User information and Telegram integration
2. **currencies**: Currency definitions (EUR, USD, BYN, etc.)
3. **accounts**: Financial accounts with currency and optional IBAN
4. **categories**: Global spending categories with sort ordering
5. **spendings**: Individual spending records with full tracking

### Key Relationships
- **accounts** → **currencies** (many-to-one)
- **accounts** ↔ **users** (many-to-many via account_users)
- **spendings** → **accounts** (many-to-one)
- **spendings** → **categories** (many-to-one)
- **spendings** → **users** (many-to-one, reporter)

## Key Design Patterns

### 1. **Async-First Design**
- Bot operations are async by default
- Prepares for database operations
- Better performance for I/O operations

### 2. **Environment-Based Configuration**
- Secrets (bot token) via environment variables
- Easy deployment across different environments
- Secure credential management

### 3. **Focused Testing Strategy**
- Tests verify structure, not execution
- Avoids hanging tests that start the bot
- Fast feedback loop for development
- Database tests use temporary files for isolation

### 4. **Simple Data Access Pattern**
- Direct SQL queries with sqlite3
- Transaction handling with context managers
- Dict-based data representation
- Comprehensive error handling

### 5. **Comprehensive Testing Strategy**
- **158 total tests**: 99 DAL tests + 59 new bot/state tests
- **Async testing**: pytest-asyncio for bot handlers
- **Telegram mocking**: MagicMock for Telegram objects (avoiding attribute restrictions)
- **State isolation**: Separate test instances for state management
- **Coverage areas**: Command handlers, text parsing, callbacks, user management, draft expenses

## Future Considerations

### Planned Features
1. ✅ **User Management**: SQLite database for user storage
2. ✅ **Bot-Database Integration**: `/users` command lists all registered users
3. ✅ **Currency Management**: Currency definitions and operations
4. ✅ **Account Management**: Financial accounts with currency support
5. **Category Management**: Spending categories with sort ordering
6. **Expense Storage**: Expense tracking with full relationships
7. **Reporting**: Monthly/weekly spending summaries
8. **Export Features**: CSV/Excel export capabilities

### Potential Architectural Changes
1. **Multi-user Support**: User session management
2. **Advanced Analytics**: Spending trends and insights
3. **Integration Options**: Bank account linking (future)
4. **CLI Mode**: Administrative commands (if needed)

## Development Workflow

### Adding New Features
1. Create feature branch
2. Add tests first (TDD approach)
3. Implement feature in bot.py
4. Update dependencies if needed
5. Run full test suite
6. Update this documentation if architecture changes

### Dependency Updates
```bash
./scripts/update-deps.sh
```

### Code Quality
```bash
# Run all quality checks (format, lint, type check, tests)
./scripts/dev.sh check

# Auto-fix formatting issues
./scripts/dev.sh fix

# Run tests only
./scripts/dev.sh test
```

### Running the Bot
```bash
python -m spending_tracker
# or
spending-tracker
```

## Development Tooling

### Code Quality Stack
- **black**: Code formatting (88-character line length)
- **isort**: Import sorting (compatible with black)
- **flake8**: Linting and code style enforcement
- **mypy**: Static type checking
- **pre-commit**: Automatic quality checks on git commits

### Development Scripts
- `./scripts/dev.sh`: Unified development utility with colored output
- `./scripts/format-code.sh`: Run all quality checks
- `./scripts/fix-code.sh`: Auto-fix formatting issues

### Pre-commit Integration
Automatically runs quality checks on every git commit:
- Trailing whitespace removal
- End-of-file fixing
- YAML validation
- Code formatting (black)
- Import sorting (isort)
- Linting (flake8)
- Type checking (mypy)

### Configuration Files
- `pyproject.toml`: Tool configuration (black, isort, mypy, pytest)
- `.flake8`: Flake8-specific configuration (doesn't support pyproject.toml)
- `.pre-commit-config.yaml`: Pre-commit hooks configuration
- `requirements-dev.txt`: Development dependencies with version locking

### Quality Standards
- 100% type coverage with mypy
- PEP 8 compliance via flake8
- Consistent formatting via black
- Sorted imports via isort
- Clean git history via pre-commit hooks

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2024-01 | Telegram Bot as primary interface | User experience and development simplicity |
| 2024-01 | Python 3.12+ minimum | Available in Ubuntu 24.04 LTS, stable modern version |
| 2024-01 | Remove CLI functionality | Focus on single interface |
| 2024-01 | pip-tools for dependency locking | Reproducible builds |
| 2024-01 | Simplified main entry point | Reduced complexity |
| 2024-01 | sqlite3 with direct SQL | Simplicity, transparency, no external deps |
| 2024-01 | Bot-DAL integration | First database command `/users` implemented |
| 2024-01 | Single DAL class architecture | Simplified connection management and complex queries |
| 2024-01 | Standard Python tooling setup | Replace custom scripts with black, isort, flake8, mypy |
| 2024-01 | Pre-commit hooks integration | Automatic quality checks and consistent codebase |
| 2024-01 | Comprehensive test suite | 158 total tests with async support and Telegram mocking |

---

*Last updated: January 2024*
*Next review: When adding Category table functionality*
