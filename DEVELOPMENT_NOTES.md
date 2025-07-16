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

#### 2024-01-XX: Docker-Based Deployment Migration
- **Decision**: Migrated from complex systemd-based deployment (1300+ lines) to Docker-based approach (~125 lines)
- **Rationale**: Original deployment was untestable, overly complex, and required production server access for any testing. Docker approach enables local testing, immutable deployments, and massive simplification
- **Key Principles**: Testability first, minimal complexity, production isolation, zero-dependency production
- **Implementation**:
  - Dockerfile for application containerization
  - docker-compose.local.yml for local development
  - docker-compose.prod.yml for production deployment
  - scripts/local-test.sh for local testing workflow
  - scripts/build-and-deploy.sh for production deployment
- **Benefits**: 90%+ code reduction, local testability, no source code on production, single dependency (Docker)
- **Status**: ✅ Implemented and documented in DEPLOYMENT.md and DEPLOYMENT_PRINCIPLES.md

## Development Commands

```bash
# Run all checks (required after every task)
./scripts/dev.sh check

# Run tests only
pytest tests/ -v

# Update dependencies
./scripts/update-deps.sh

# Start bot locally (traditional way, requires .env with TELEGRAM_BOT_TOKEN)
python -m spending_tracker

# Docker-based development workflow (recommended)
./scripts/local-test.sh              # Test with Docker locally
./scripts/build-and-deploy.sh        # Deploy to production
./scripts/build-and-deploy.sh --skip-test  # Deploy without local testing
```

## Docker Deployment Workflow

### Local Development
1. **Test changes**: `./scripts/local-test.sh`
2. **Review logs**: Check that everything works as expected
3. **Stop local env**: `docker compose -f docker-compose.local.yml down`

### Production Deployment
1. **Deploy**: `./scripts/build-and-deploy.sh`
2. **Monitor**: Script will show deployment status and offer to show logs
3. **Verify**: Check production logs and health

### Quick Commands
```bash
# Local testing
./scripts/local-test.sh

# Production deployment
./scripts/build-and-deploy.sh

# Check production status
ssh spending_tracker 'cd /opt/spending-tracker && sudo docker compose ps'

# View production logs
ssh spending_tracker 'cd /opt/spending-tracker && sudo docker compose logs -f spending-tracker'
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
