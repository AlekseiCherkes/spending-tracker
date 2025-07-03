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
3. Install the package in development mode (optional):
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

## Current Status
✅ Basic Python module structure  
✅ Python 3.13+ requirement  
✅ Modern project configuration (pyproject.toml)  
✅ Virtual environment setup  
🔄 Coming soon: Telegram bot functionality  
🔄 Coming soon: Database integration  
🔄 Coming soon: Expense tracking features  

## Project Structure
```
spending-tracker/
├── spending_tracker/     # Main package directory
│   ├── __init__.py      # Package initialization
│   └── __main__.py      # Main entry point
├── venv/                # Virtual environment (excluded from git)
├── pyproject.toml       # Modern Python project configuration
├── requirements.txt     # Python dependencies (empty for now)
├── README.md           # This file
└── .gitignore          # Git ignore rules
```

## Development Plan
We're building this step by step:
1. ✅ Basic Python module structure
2. ✅ Python 3.13+ requirement with modern configuration
3. ✅ Virtual environment setup
4. 🔄 Add basic CLI interface
5. 🔄 Add data models
6. 🔄 Add Telegram bot integration
7. 🔄 Add database functionality
8. 🔄 Add expense tracking features
