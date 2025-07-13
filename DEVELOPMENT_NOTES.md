# Development Notes

## Development Workflow Rules

### Code Quality Requirements
- **Pre-commit checks**: Always run `./scripts/dev.sh check` after completing every task
- **All checks must pass**: formatting (black), linting (flake8), type checking (mypy), tests
- **Commit messages**: Short one-line messages in English for easy copy-paste into git

### Current Development Status

#### Completed ✅
- **Database Layer**: Complete DAL implementation with 99 tests
- **Tables**: users, currencies, accounts, categories, spending
- **Bot Structure**: Basic Telegram bot with info commands
- **Infrastructure**: Testing, linting, type checking, dependency management
- **Expense Tracking**: Full expense addition workflow with inline keyboards
- **User Management**: Auto-registration on first interaction
- **State Management**: In-memory draft expense management
- **Comprehensive Test Suite**: 158 total tests covering all bot functionality
  - Bot tests (37 tests): command handlers, text parsing, callbacks, user management
  - State management tests (22 tests): draft expenses, state isolation, validation
  - Async testing setup with pytest-asyncio
  - Telegram object mocking with MagicMock

#### Next Priority Tasks 🎯
1. **Add viewing commands**: `/spending`, `/stats`, `/month`
2. **Account management**: `/accounts` command to manage user accounts
3. **Category management**: `/categories` command
4. **Enhanced expense features**: Edit existing expenses, add notes
5. **Reporting**: Monthly/weekly summaries and analytics

#### Development Preferences
- **Testing**: TDD approach - tests first, then implementation
- **Error handling**: Comprehensive error handling with user-friendly messages
- **Documentation**: Update ARCHITECTURE.md for significant architectural changes
- **Code style**: Follow existing patterns in bot.py and dal.py

### Architectural Decisions Log

#### 2024-01-XX: Database Implementation
- **Decision**: Use direct SQLite with comprehensive DAL
- **Rationale**: Full control, no external dependencies, transparent SQL
- **Status**: ✅ Implemented with 99 tests

#### 2024-01-XX: Pre-commit Workflow
- **Decision**: Mandatory pre-commit checks for every task
- **Rationale**: Maintain code quality, catch issues early
- **Implementation**: `./scripts/dev.sh check` must pass

#### 2024-01-XX: Expense State Management
- **Decision**: Use in-memory state for draft expenses
- **Rationale**: Simple, fast, and sufficient for single-server deployment
- **Implementation**: `ExpenseStateManager` class with global instance
- **Status**: ✅ Implemented with type safety

#### 2024-01-XX: Text Message Expense Detection
- **Decision**: Parse numbers from any text message as potential expenses
- **Rationale**: Natural and intuitive user experience
- **Implementation**: Regex-based parsing with inline keyboards
- **Status**: ✅ Implemented with validation

#### 2024-01-XX: Comprehensive Test Suite Implementation
- **Decision**: Implement full test coverage for bot functionality and state management
- **Rationale**: Ensure reliability, catch regressions, enable confident refactoring
- **Implementation**:
  - 37 bot tests covering all handlers and user interactions
  - 22 state management tests for draft expense lifecycle
  - Async testing with pytest-asyncio
  - Telegram object mocking strategy with MagicMock
- **Status**: ✅ Implemented with 158 total tests (99 DAL + 59 new)

#### 2024-01-XX: Python Version Update
- **Decision**: Changed from Python 3.13+ to Python 3.12+ requirement
- **Rationale**: Primary reason - Python 3.12 is available out-of-the-box in Ubuntu 24.04 LTS as default system Python. Code analysis showed no Python 3.13-specific features used; Python 3.12 provides stable modern features with broader compatibility
- **Status**: ✅ Updated configuration files and regenerated dependencies
- **Implementation**: Updated pyproject.toml, ARCHITECTURE.md, README.md, and bot.py version displays

## Development Commands

```bash
# Run all checks (required after every task)
./scripts/dev.sh check

# Run tests only
pytest tests/ -v

# Update dependencies
./scripts/update-deps.sh

# Start bot (requires .env with TELEGRAM_BOT_TOKEN)
python -m spending_tracker
```

## Next Steps Planning

### Phase 1: Core Functionality
- [ ] `/add` command implementation
- [ ] User auto-registration
- [ ] Basic spending queries

### Phase 2: Enhanced Features
- [ ] Category management
- [ ] Account management
- [ ] Monthly reports

### Phase 3: Analytics
- [ ] Spending trends
- [ ] Data export
- [ ] Advanced reports
