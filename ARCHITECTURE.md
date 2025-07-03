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

### 2. **Python 3.13+ Minimum Version**
- **Decision**: Require Python 3.13+ 
- **Rationale**:
  - Latest performance improvements
  - Modern language features
  - Better async/await support for Telegram bot
  - Type hints and error handling improvements

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

## Current Project Structure

```
spending-tracker/
├── spending_tracker/           # Main package
│   ├── __init__.py            # Package metadata
│   ├── __main__.py            # Entry point (starts bot)
│   └── bot.py                 # Telegram bot implementation
├── tests/                     # Test suite
│   ├── __init__.py
│   └── test_main.py          # Core functionality tests
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
- **black**: Code formatting
- **flake8**: Linting
- **mypy**: Type checking
- **pip-tools**: Dependency management

### Future Database Stack
- **SQLAlchemy 2.0+**: ORM for expense data
- **aiosqlite**: Async SQLite driver
- **alembic**: Database migrations (when needed)

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

## Future Considerations

### Planned Features
1. **Expense Storage**: SQLite database for expense tracking
2. **Spending Categories**: Categorization of expenses
3. **Reporting**: Monthly/weekly spending summaries
4. **Export Features**: CSV/Excel export capabilities

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

### Testing
```bash
pytest tests/ -v
```

### Running the Bot
```bash
python -m spending_tracker
# or
spending-tracker
```

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2024-01 | Telegram Bot as primary interface | User experience and development simplicity |
| 2024-01 | Python 3.13+ minimum | Latest features and performance |
| 2024-01 | Remove CLI functionality | Focus on single interface |
| 2024-01 | pip-tools for dependency locking | Reproducible builds |
| 2024-01 | Simplified main entry point | Reduced complexity |

---

*Last updated: January 2024*
*Next review: When adding database functionality* 