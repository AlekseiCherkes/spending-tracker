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

#### Next Priority Tasks 🎯
1. **Add expense tracking**: `/add` command for recording spending
2. **Auto user registration**: Create users on first bot interaction
3. **View commands**: `/spending`, `/stats`, `/month`
4. **Category management**: `/categories` command

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
