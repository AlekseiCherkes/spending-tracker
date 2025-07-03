# spending-tracker
A simple Python module for personal spending tracking.

## Getting Started

### Prerequisites
- Python 3.13+

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
   # Install base dependencies
   pip install -r requirements.txt
   
   # For development work
   pip install -r requirements-dev.txt
   
   # For telegram bot features
   pip install -r requirements-telegram.txt
   ```
4. Install the package in development mode (optional):
   ```bash
   pip install -e .
   ```

### Running the Module
```bash
# Make sure your virtual environment is activated first!
# You should see (venv) in your prompt

# Run as a module
python -m spending_tracker

# Or run the main file directly
python spending_tracker/__main__.py

# If you installed it as a package:
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
🔄 Coming soon: Telegram bot functionality  
🔄 Coming soon: Database integration  
🔄 Coming soon: Expense tracking features  

## Project Structure
```
spending-tracker/
├── spending_tracker/         # Main package directory
│   ├── __init__.py          # Package initialization
│   └── __main__.py          # Main entry point
├── scripts/                 # Development scripts
│   └── update-deps.sh       # Update dependency lock files
├── venv/                    # Virtual environment (excluded from git)
├── pyproject.toml           # Modern Python project configuration
├── requirements.txt         # Locked base dependencies
├── requirements-dev.txt     # Locked development dependencies
├── requirements-telegram.txt # Locked telegram dependencies
├── README.md               # This file
└── .gitignore              # Git ignore rules
```

## Development Plan
We're building this step by step:
1. ✅ Basic Python module structure
2. ✅ Python 3.13+ requirement with modern configuration
3. ✅ Virtual environment setup
4. ✅ Dependency locking with pip-tools
5. 🔄 Add basic CLI interface
6. 🔄 Add data models
7. 🔄 Add Telegram bot integration
8. 🔄 Add database functionality
9. 🔄 Add expense tracking features
